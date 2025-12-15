use serde::{Deserialize, Serialize};
use std::io::{Result, Error, ErrorKind, Write, Read};
use std::path::PathBuf;
use crate::archive::FileEntry;

/// Manifest entry - simplified reference to a file in the archive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestEntry {
    pub path: PathBuf,
    pub offset: u64,
    pub size: u64,
}

/// Archive manifest - TAR-like structure containing file list and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveManifest {
    /// Version of the manifest format
    pub version: u32,

    /// Total number of files in the archive
    pub file_count: usize,

    /// Total uncompressed size of all files
    pub total_size: u64,

    /// List of all file entries with metadata
    pub entries: Vec<FileEntry>,

    /// Archive creation timestamp
    pub created_at: String,

    /// Archive creator (hostname or user)
    pub creator: String,

    /// Base directory that was archived
    pub base_directory: PathBuf,
}

impl ArchiveManifest {
    /// Create a new manifest
    pub fn new(base_directory: PathBuf) -> Self {
        let hostname = std::env::var("HOSTNAME")
            .or_else(|_| std::env::var("COMPUTERNAME"))
            .unwrap_or_else(|_| "unknown".to_string());

        Self {
            version: 1,
            file_count: 0,
            total_size: 0,
            entries: Vec::new(),
            created_at: chrono::Utc::now().to_rfc3339(),
            creator: hostname,
            base_directory,
        }
    }

    /// Add a file entry to the manifest
    pub fn add_entry(&mut self, entry: FileEntry) {
        self.total_size += entry.size;
        self.file_count += 1;
        self.entries.push(entry);
    }

    /// Serialize manifest to JSON bytes
    pub fn to_json(&self) -> Result<Vec<u8>> {
        serde_json::to_vec_pretty(self)
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))
    }

    /// Deserialize manifest from JSON bytes
    pub fn from_json(data: &[u8]) -> Result<Self> {
        serde_json::from_slice(data)
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))
    }

    /// Write manifest to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        let json = self.to_json()?;

        // Write manifest size as 8-byte big-endian
        let size = json.len() as u64;
        writer.write_all(&size.to_be_bytes())?;

        // Write manifest data
        writer.write_all(&json)?;

        Ok(())
    }

    /// Read manifest from a reader
    pub fn read<R: Read>(reader: &mut R) -> Result<Self> {
        // Read manifest size
        let mut size_buf = [0u8; 8];
        reader.read_exact(&mut size_buf)?;
        let size = u64::from_be_bytes(size_buf);

        if size > 100 * 1024 * 1024 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Manifest too large: {} bytes", size)
            ));
        }

        // Read manifest data
        let mut json = vec![0u8; size as usize];
        reader.read_exact(&mut json)?;

        Self::from_json(&json)
    }

    /// Find an entry by path
    pub fn find_entry(&self, path: &PathBuf) -> Option<&FileEntry> {
        self.entries.iter().find(|e| &e.path == path)
    }

    /// Get all entries sorted by path
    pub fn sorted_entries(&self) -> Vec<&FileEntry> {
        let mut entries: Vec<&FileEntry> = self.entries.iter().collect();
        entries.sort_by(|a, b| a.path.cmp(&b.path));
        entries
    }

    /// List all files in the archive (for CLI list command)
    pub fn list_files(&self) -> Vec<String> {
        self.entries.iter()
            .map(|e| format!("{} {:>10} {}",
                match e.file_type {
                    crate::archive::file_entry::FileType::Regular => "f",
                    crate::archive::file_entry::FileType::Directory => "d",
                    crate::archive::file_entry::FileType::Symlink => "l",
                },
                e.size,
                e.path.display()
            ))
            .collect()
    }

    /// Calculate compression ratio
    pub fn compression_ratio(&self, compressed_size: u64) -> f64 {
        if self.total_size == 0 {
            return 0.0;
        }
        (compressed_size as f64 / self.total_size as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use crate::archive::FileEntry;

    #[test]
    fn test_manifest_creation() {
        let manifest = ArchiveManifest::new(PathBuf::from("/test"));
        assert_eq!(manifest.version, 1);
        assert_eq!(manifest.file_count, 0);
        assert_eq!(manifest.total_size, 0);
    }

    #[test]
    fn test_manifest_add_entry() {
        let mut manifest = ArchiveManifest::new(PathBuf::from("/test"));
        let entry = FileEntry::directory(PathBuf::from("test"), 0o755, 1000, 1000);

        manifest.add_entry(entry);
        assert_eq!(manifest.file_count, 1);
        assert_eq!(manifest.entries.len(), 1);
    }

    #[test]
    fn test_manifest_serialization() {
        let mut manifest = ArchiveManifest::new(PathBuf::from("/test"));
        let entry = FileEntry::directory(PathBuf::from("test"), 0o755, 1000, 1000);
        manifest.add_entry(entry);

        let json = manifest.to_json().unwrap();
        let deserialized = ArchiveManifest::from_json(&json).unwrap();

        assert_eq!(deserialized.file_count, manifest.file_count);
        assert_eq!(deserialized.entries.len(), manifest.entries.len());
    }

    #[test]
    fn test_manifest_write_read() {
        let mut manifest = ArchiveManifest::new(PathBuf::from("/test"));
        let entry = FileEntry::directory(PathBuf::from("test"), 0o755, 1000, 1000);
        manifest.add_entry(entry);

        let mut buffer = Vec::new();
        manifest.write(&mut buffer).unwrap();

        let mut cursor = std::io::Cursor::new(buffer);
        let read_manifest = ArchiveManifest::read(&mut cursor).unwrap();

        assert_eq!(read_manifest.file_count, manifest.file_count);
        assert_eq!(read_manifest.base_directory, manifest.base_directory);
    }

    #[test]
    fn test_manifest_find_entry() {
        let mut manifest = ArchiveManifest::new(PathBuf::from("/test"));
        let entry = FileEntry::directory(PathBuf::from("test"), 0o755, 1000, 1000);
        manifest.add_entry(entry);

        let found = manifest.find_entry(&PathBuf::from("test"));
        assert!(found.is_some());

        let not_found = manifest.find_entry(&PathBuf::from("nonexistent"));
        assert!(not_found.is_none());
    }

    #[test]
    fn test_compression_ratio() {
        let mut manifest = ArchiveManifest::new(PathBuf::from("/test"));
        manifest.total_size = 1000;

        let ratio = manifest.compression_ratio(500);
        assert_eq!(ratio, 50.0);
    }
}
