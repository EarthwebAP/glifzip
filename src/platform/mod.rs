//! Platform-specific functionality for different operating systems
//!
//! This module provides cross-platform abstractions for OS-specific features:
//! - macOS/Apple: Finder integration, file associations, extended attributes
//! - Linux: Standard file operations
//! - Windows: File association, context menu integration

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(not(target_os = "macos"))]
pub mod macos {
    // Stub for non-macOS platforms
    pub fn register_file_type() -> std::io::Result<()> {
        Ok(())
    }

    pub fn get_extended_attributes(_path: &std::path::Path) -> std::io::Result<Vec<(String, Vec<u8>)>> {
        Ok(Vec::new())
    }

    pub fn set_extended_attributes(_path: &std::path::Path, _attrs: &[(String, Vec<u8>)]) -> std::io::Result<()> {
        Ok(())
    }

    pub fn get_quarantine_status(_path: &std::path::Path) -> std::io::Result<bool> {
        Ok(false)
    }

    pub fn set_quarantine_status(_path: &std::path::Path, _quarantined: bool) -> std::io::Result<()> {
        Ok(())
    }
}

/// Register GLIF file type with the operating system
pub fn register_glif_filetype() -> std::io::Result<()> {
    macos::register_file_type()
}

/// Get extended file attributes (macOS xattr)
pub fn get_file_attributes(path: &std::path::Path) -> std::io::Result<Vec<(String, Vec<u8>)>> {
    macos::get_extended_attributes(path)
}

/// Set extended file attributes (macOS xattr)
pub fn set_file_attributes(path: &std::path::Path, attrs: &[(String, Vec<u8>)]) -> std::io::Result<()> {
    macos::set_extended_attributes(path, attrs)
}

/// Check if file has macOS quarantine attribute
pub fn is_quarantined(path: &std::path::Path) -> std::io::Result<bool> {
    macos::get_quarantine_status(path)
}

/// Set macOS quarantine attribute
pub fn set_quarantine(path: &std::path::Path, quarantined: bool) -> std::io::Result<()> {
    macos::set_quarantine_status(path, quarantined)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_filetype() {
        assert!(register_glif_filetype().is_ok());
    }

    #[test]
    fn test_file_attributes() {
        use std::env::temp_dir;
        use std::fs::File;

        let temp_file = temp_dir().join("test_attrs.glif");
        let _ = File::create(&temp_file);

        // Should not fail even if attributes aren't supported
        let _ = get_file_attributes(&temp_file);
        let _ = set_file_attributes(&temp_file, &[]);

        let _ = std::fs::remove_file(&temp_file);
    }
}
