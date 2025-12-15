use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Result, Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use chrono::{DateTime, Utc};

/// Represents file type in the archive
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileType {
    Regular,
    Directory,
    Symlink,
}

/// Represents a file entry in the archive with full metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    /// Relative path within the archive
    pub path: PathBuf,

    /// File type
    pub file_type: FileType,

    /// File size in bytes (0 for directories)
    pub size: u64,

    /// Unix file permissions (mode)
    pub mode: u32,

    /// User ID (owner)
    pub uid: u32,

    /// Group ID
    pub gid: u32,

    /// Last modified time
    pub mtime: DateTime<Utc>,

    /// Last accessed time
    pub atime: DateTime<Utc>,

    /// Symlink target (if file_type is Symlink)
    pub symlink_target: Option<PathBuf>,

    /// Offset in the compressed data blob
    pub data_offset: u64,

    /// SHA256 hash of the file contents (empty for directories)
    pub sha256: String,
}

impl FileEntry {
    /// Create a FileEntry from a filesystem path
    pub fn from_path<P: AsRef<Path>>(
        path: P,
        relative_path: PathBuf,
        data_offset: u64,
    ) -> Result<Self> {
        let path_ref = path.as_ref();
        let metadata = fs::symlink_metadata(path_ref)?;

        let file_type = if metadata.is_symlink() {
            FileType::Symlink
        } else if metadata.is_dir() {
            FileType::Directory
        } else {
            FileType::Regular
        };

        let symlink_target = if file_type == FileType::Symlink {
            Some(fs::read_link(path_ref)?)
        } else {
            None
        };

        // Get Unix-specific metadata
        let mode = metadata.permissions().mode();
        let uid = metadata.uid();
        let gid = metadata.gid();

        // Convert modified time
        let mtime = metadata.modified()?;
        let mtime = DateTime::from(mtime);

        // Convert accessed time
        let atime = metadata.accessed()?;
        let atime = DateTime::from(atime);

        // Calculate SHA256 for regular files
        let sha256 = if file_type == FileType::Regular {
            let data = fs::read(path_ref)?;
            let hash = crate::verification::calculate_sha256(&data);
            crate::verification::hex_encode(&hash)
        } else {
            String::new()
        };

        Ok(Self {
            path: relative_path,
            file_type,
            size: metadata.len(),
            mode,
            uid,
            gid,
            mtime,
            atime,
            symlink_target,
            data_offset,
            sha256,
        })
    }

    /// Create a directory entry
    pub fn directory(relative_path: PathBuf, mode: u32, uid: u32, gid: u32) -> Self {
        let now = Utc::now();
        Self {
            path: relative_path,
            file_type: FileType::Directory,
            size: 0,
            mode,
            uid,
            gid,
            mtime: now,
            atime: now,
            symlink_target: None,
            data_offset: 0,
            sha256: String::new(),
        }
    }

    /// Create a symlink entry
    pub fn symlink(
        relative_path: PathBuf,
        target: PathBuf,
        mode: u32,
        uid: u32,
        gid: u32,
    ) -> Self {
        let now = Utc::now();
        Self {
            path: relative_path,
            file_type: FileType::Symlink,
            size: 0,
            mode,
            uid,
            gid,
            mtime: now,
            atime: now,
            symlink_target: Some(target),
            data_offset: 0,
            sha256: String::new(),
        }
    }

    /// Restore file metadata to a filesystem path
    pub fn restore_metadata<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path_ref = path.as_ref();

        // Set permissions
        let permissions = fs::Permissions::from_mode(self.mode);
        fs::set_permissions(path_ref, permissions)?;

        // Set modification and access times
        let mtime = filetime::FileTime::from_unix_time(
            self.mtime.timestamp(),
            self.mtime.timestamp_subsec_nanos(),
        );
        let atime = filetime::FileTime::from_unix_time(
            self.atime.timestamp(),
            self.atime.timestamp_subsec_nanos(),
        );
        filetime::set_file_times(path_ref, atime, mtime)?;

        // Note: Setting uid/gid requires elevated privileges
        // We skip this for now but could add a --preserve-ownership flag

        Ok(())
    }

    /// Validate file integrity by comparing SHA256
    pub fn verify_integrity(&self, data: &[u8]) -> Result<()> {
        if self.file_type != FileType::Regular {
            return Ok(());
        }

        let calculated_hash = crate::verification::calculate_sha256(data);
        let calculated_hex = crate::verification::hex_encode(&calculated_hash);

        if calculated_hex != self.sha256 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "File integrity check failed for {}: expected {}, got {}",
                    self.path.display(),
                    self.sha256,
                    calculated_hex
                )
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_file_entry_from_regular_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"Hello, World!").unwrap();
        drop(file);

        let entry = FileEntry::from_path(
            &file_path,
            PathBuf::from("test.txt"),
            0,
        ).unwrap();

        assert_eq!(entry.file_type, FileType::Regular);
        assert_eq!(entry.size, 13);
        assert_eq!(entry.path, PathBuf::from("test.txt"));
        assert!(!entry.sha256.is_empty());
    }

    #[test]
    fn test_file_entry_from_directory() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().join("subdir");
        fs::create_dir(&dir_path).unwrap();

        let entry = FileEntry::from_path(
            &dir_path,
            PathBuf::from("subdir"),
            0,
        ).unwrap();

        assert_eq!(entry.file_type, FileType::Directory);
        assert_eq!(entry.sha256, "");
    }

    #[test]
    fn test_file_entry_from_symlink() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("target.txt");
        let link_path = temp_dir.path().join("link.txt");

        File::create(&file_path).unwrap();
        std::os::unix::fs::symlink(&file_path, &link_path).unwrap();

        let entry = FileEntry::from_path(
            &link_path,
            PathBuf::from("link.txt"),
            0,
        ).unwrap();

        assert_eq!(entry.file_type, FileType::Symlink);
        assert!(entry.symlink_target.is_some());
    }

    #[test]
    fn test_verify_integrity() {
        let data = b"Test data";
        let hash = crate::verification::calculate_sha256(data);
        let hash_hex = crate::verification::hex_encode(&hash);

        let mut entry = FileEntry::directory(PathBuf::from("test"), 0o755, 1000, 1000);
        entry.file_type = FileType::Regular;
        entry.sha256 = hash_hex;

        assert!(entry.verify_integrity(data).is_ok());
        assert!(entry.verify_integrity(b"Wrong data").is_err());
    }
}
