//! macOS/Apple-specific functionality for GLifzip
//!
//! This module provides integration with macOS features:
//! - Finder integration and quick look support
//! - Extended attributes (xattr) for metadata preservation
//! - Quarantine attribute handling for downloaded files
//! - File type registration in Launch Services
//! - Apple Silicon and Intel support

use std::path::Path;
use std::process::Command;

/// Register GLIF file type with macOS Finder and Launch Services
/// This allows Finder to recognize .glif files and associate them with glifzip
pub fn register_file_type() -> std::io::Result<()> {
    #[cfg(target_os = "macos")]
    {
        // Try to get the app bundle path
        let bundle_id = "com.glyphos.glifzip";
        let app_name = "GLifzip";

        // Use LaunchServices to register the file type
        let output = Command::new("duti")
            .args(&["--set", bundle_id, "com.glif", "all"])
            .output();

        // If duti is not available, try using macOS native commands
        if output.is_err() {
            let _ = Command::new("defaults")
                .args(&["write", "com.apple.LaunchServices/com.apple.LaunchServices.secure", "LSHandlers", "-array-add",
                    &format!("{{ LSHandlerContentType = com.glif; LSHandlerRoleAll = {}; }}", bundle_id)])
                .output();
        }

        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    {
        Ok(())
    }
}

/// Get extended attributes from a file (macOS xattr)
/// These are metadata attributes stored with files
pub fn get_extended_attributes(path: &Path) -> std::io::Result<Vec<(String, Vec<u8>)>> {
    #[cfg(target_os = "macos")]
    {
        use std::ffi::CString;
        use std::os::raw::c_char;

        extern "C" {
            fn listxattr(path: *const c_char, namebuf: *mut c_char, size: usize, options: i32) -> isize;
            fn getxattr(path: *const c_char, name: *const c_char, value: *mut u8, size: usize, position: u32, options: i32) -> isize;
        }

        let path_cstr = CString::new(path.to_string_lossy().as_bytes())?;
        let mut attrs = Vec::new();

        // Get list of attribute names
        unsafe {
            let size = listxattr(path_cstr.as_ptr(), std::ptr::null_mut(), 0, 0);
            if size > 0 {
                let mut names_buf = vec![0u8; size as usize];
                let actual_size = listxattr(path_cstr.as_ptr(), names_buf.as_mut_ptr() as *mut c_char, size as usize, 0);

                if actual_size > 0 {
                    // Parse null-terminated attribute names
                    let mut start = 0;
                    for i in 0..actual_size as usize {
                        if names_buf[i] == 0 {
                            if i > start {
                                if let Ok(name) = String::from_utf8(names_buf[start..i].to_vec()) {
                                    // Get attribute value
                                    let name_cstr = CString::new(name.as_bytes()).ok();
                                    if let Some(name_cstr) = name_cstr {
                                        let value_size = getxattr(
                                            path_cstr.as_ptr(),
                                            name_cstr.as_ptr(),
                                            std::ptr::null_mut(),
                                            0,
                                            0,
                                            0
                                        );

                                        if value_size > 0 {
                                            let mut value = vec![0u8; value_size as usize];
                                            let actual = getxattr(
                                                path_cstr.as_ptr(),
                                                name_cstr.as_ptr(),
                                                value.as_mut_ptr(),
                                                value.len(),
                                                0,
                                                0
                                            );

                                            if actual > 0 {
                                                attrs.push((name, value));
                                            }
                                        }
                                    }
                                }
                            }
                            start = i + 1;
                        }
                    }
                }
            }
        }

        Ok(attrs)
    }

    #[cfg(not(target_os = "macos"))]
    {
        Ok(Vec::new())
    }
}

/// Set extended attributes on a file (macOS xattr)
pub fn set_extended_attributes(path: &Path, attrs: &[(String, Vec<u8>)]) -> std::io::Result<()> {
    #[cfg(target_os = "macos")]
    {
        use std::ffi::CString;
        use std::os::raw::c_char;

        extern "C" {
            fn setxattr(path: *const c_char, name: *const c_char, value: *const u8, size: usize, position: u32, options: i32) -> i32;
        }

        let path_cstr = CString::new(path.to_string_lossy().as_bytes())?;

        for (name, value) in attrs {
            if let Ok(name_cstr) = CString::new(name.as_bytes()) {
                unsafe {
                    setxattr(
                        path_cstr.as_ptr(),
                        name_cstr.as_ptr(),
                        value.as_ptr(),
                        value.len(),
                        0,
                        0
                    );
                }
            }
        }

        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    {
        Ok(())
    }
}

/// Get macOS quarantine attribute status
/// Files downloaded from the internet have the quarantine attribute set
pub fn get_quarantine_status(path: &Path) -> std::io::Result<bool> {
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("xattr")
            .arg("-p")
            .arg("com.apple.quarantine")
            .arg(path)
            .output();

        Ok(output.is_ok() && output.as_ref().map(|o| !o.stdout.is_empty()).unwrap_or(false))
    }

    #[cfg(not(target_os = "macos"))]
    {
        Ok(false)
    }
}

/// Set or remove macOS quarantine attribute
pub fn set_quarantine_status(path: &Path, quarantined: bool) -> std::io::Result<()> {
    #[cfg(target_os = "macos")]
    {
        if quarantined {
            // Add quarantine attribute
            let _ = Command::new("xattr")
                .arg("-w")
                .arg("com.apple.quarantine")
                .arg("0001;00000000;00000000;00000000|com.apple.Safari")
                .arg(path)
                .output();
        } else {
            // Remove quarantine attribute
            let _ = Command::new("xattr")
                .arg("-d")
                .arg("com.apple.quarantine")
                .arg(path)
                .output();
        }

        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    {
        Ok(())
    }
}

/// Get macOS version to determine feature availability
pub fn get_macos_version() -> std::io::Result<(u32, u32, u32)> {
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("sw_vers")
            .arg("-productVersion")
            .output()?;

        let version_str = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = version_str.trim().split('.').collect();

        let major = parts.get(0).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
        let minor = parts.get(1).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
        let patch = parts.get(2).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);

        Ok((major, minor, patch))
    }

    #[cfg(not(target_os = "macos"))]
    {
        Ok((0, 0, 0))
    }
}

/// Check if running on Apple Silicon (arm64)
pub fn is_apple_silicon() -> bool {
    #[cfg(target_arch = "aarch64")]
    {
        cfg!(target_os = "macos")
    }

    #[cfg(not(target_arch = "aarch64"))]
    {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_file_type() {
        // Should not panic
        let _ = register_file_type();
    }

    #[test]
    fn test_macos_version() {
        #[cfg(target_os = "macos")]
        {
            if let Ok((major, _, _)) = get_macos_version() {
                assert!(major >= 10);
            }
        }
    }

    #[test]
    fn test_is_apple_silicon() {
        let _ = is_apple_silicon();
    }

    #[test]
    fn test_quarantine_status() {
        use std::fs::File;
        use std::env::temp_dir;

        let temp_file = temp_dir().join("test_quarantine.glif");
        if let Ok(_) = File::create(&temp_file) {
            let _ = get_quarantine_status(&temp_file);
            let _ = set_quarantine_status(&temp_file, true);
            let _ = set_quarantine_status(&temp_file, false);
            let _ = std::fs::remove_file(&temp_file);
        }
    }
}
