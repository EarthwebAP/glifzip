//! Apple/macOS metadata handling for GLIF archives
//!
//! Preserves macOS-specific file attributes:
//! - Extended attributes (xattr)
//! - Quarantine status
//! - File type and creator codes
//! - Finder tags and labels
//! - Bundle bits

use std::collections::HashMap;
use std::io::{Result, Error, ErrorKind};
use serde::{Deserialize, Serialize};

/// macOS-specific metadata for archived files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleMetadata {
    /// Extended attributes (xattr)
    pub extended_attributes: HashMap<String, Vec<u8>>,

    /// Quarantine attribute status
    pub quarantine_status: bool,

    /// Resource fork data (if present)
    pub resource_fork: Option<Vec<u8>>,

    /// Finder flags
    pub finder_flags: u16,

    /// File type code (4-byte OSType)
    pub file_type: Option<String>,

    /// Creator code (4-byte OSType)
    pub creator_code: Option<String>,

    /// Finder label (0-7)
    pub finder_label: u8,
}

impl AppleMetadata {
    /// Create new empty Apple metadata
    pub fn new() -> Self {
        Self {
            extended_attributes: HashMap::new(),
            quarantine_status: false,
            resource_fork: None,
            finder_flags: 0,
            file_type: None,
            creator_code: None,
            finder_label: 0,
        }
    }

    /// Load Apple metadata from a file
    #[cfg(target_os = "macos")]
    pub fn from_file(path: &std::path::Path) -> Result<Self> {
        use crate::platform::macos;

        let mut metadata = Self::new();

        // Get extended attributes
        if let Ok(attrs) = macos::get_extended_attributes(path) {
            for (name, value) in attrs {
                metadata.extended_attributes.insert(name, value);
            }
        }

        // Get quarantine status
        if let Ok(quarantined) = macos::get_quarantine_status(path) {
            metadata.quarantine_status = quarantined;
        }

        Ok(metadata)
    }

    /// Load Apple metadata from a file (no-op on non-macOS)
    #[cfg(not(target_os = "macos"))]
    pub fn from_file(_path: &std::path::Path) -> Result<Self> {
        Ok(Self::new())
    }

    /// Save Apple metadata to a file
    #[cfg(target_os = "macos")]
    pub fn save_to_file(&self, path: &std::path::Path) -> Result<()> {
        use crate::platform::macos;

        // Restore extended attributes
        let attrs: Vec<_> = self.extended_attributes.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        macos::set_extended_attributes(path, &attrs)?;

        // Restore quarantine status
        macos::set_quarantine_status(path, self.quarantine_status)?;

        Ok(())
    }

    /// Save Apple metadata to a file (no-op on non-macOS)
    #[cfg(not(target_os = "macos"))]
    pub fn save_to_file(&self, _path: &std::path::Path) -> Result<()> {
        Ok(())
    }

    /// Get the size of serialized metadata
    pub fn serialized_size(&self) -> usize {
        // Rough estimate for serialization
        let attrs_size: usize = self.extended_attributes.iter()
            .map(|(k, v)| k.len() + v.len() + 8)
            .sum();

        let resource_fork_size = self.resource_fork.as_ref()
            .map(|rf| rf.len())
            .unwrap_or(0);

        let file_type_size = self.file_type.as_ref()
            .map(|ft| ft.len() + 4)
            .unwrap_or(0);

        let creator_size = self.creator_code.as_ref()
            .map(|cc| cc.len() + 4)
            .unwrap_or(0);

        attrs_size + resource_fork_size + file_type_size + creator_size + 32
    }

    /// Serialize metadata to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self)
            .map_err(|e| Error::new(ErrorKind::Other, e))
    }

    /// Deserialize metadata from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        serde_json::from_slice(bytes)
            .map_err(|e| Error::new(ErrorKind::Other, e))
    }
}

impl Default for AppleMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// Apple file type constants
pub mod filetypes {
    /// Generic document type
    pub const DOCUMENT: &str = "TEXT";

    /// Executable type
    pub const EXECUTABLE: &str = "APPL";

    /// Folder type
    pub const FOLDER: &str = "fold";

    /// Compressed archive
    pub const ARCHIVE: &str = "GLIF";
}

/// Finder label colors
pub mod finder_labels {
    pub const NONE: u8 = 0;
    pub const RED: u8 = 1;
    pub const ORANGE: u8 = 2;
    pub const YELLOW: u8 = 3;
    pub const GREEN: u8 = 4;
    pub const BLUE: u8 = 5;
    pub const PURPLE: u8 = 6;
    pub const GRAY: u8 = 7;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apple_metadata_creation() {
        let metadata = AppleMetadata::new();
        assert_eq!(metadata.finder_label, 0);
        assert!(!metadata.quarantine_status);
    }

    #[test]
    fn test_apple_metadata_serialization() {
        let mut metadata = AppleMetadata::new();
        metadata.extended_attributes.insert("test".to_string(), vec![1, 2, 3]);
        metadata.finder_label = 2;

        let bytes = metadata.to_bytes().unwrap();
        let restored = AppleMetadata::from_bytes(&bytes).unwrap();

        assert_eq!(restored.extended_attributes.get("test"), Some(&vec![1, 2, 3]));
        assert_eq!(restored.finder_label, 2);
    }

    #[test]
    fn test_serialized_size() {
        let metadata = AppleMetadata::new();
        let size = metadata.serialized_size();
        assert!(size > 0);
    }
}
