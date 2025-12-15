use std::fs;
use std::io::{Result, Error, ErrorKind, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use glob::Pattern;
use indicatif::{ProgressBar, ProgressStyle};

use crate::archive::{ArchiveManifest, FileEntry};
use crate::archive::file_entry::FileType;
use crate::compression::CompressionConfig;

/// Configuration for directory compression
#[derive(Debug, Clone)]
pub struct DirectoryCompressionConfig {
    /// Base compression configuration
    pub compression: CompressionConfig,

    /// Exclude patterns (glob style)
    pub exclude_patterns: Vec<String>,

    /// Follow symbolic links
    pub follow_symlinks: bool,

    /// Preserve file metadata
    pub preserve_metadata: bool,

    /// Show verbose output
    pub verbose: bool,

    /// Show progress bars
    pub show_progress: bool,
}

impl Default for DirectoryCompressionConfig {
    fn default() -> Self {
        Self {
            compression: CompressionConfig::default(),
            exclude_patterns: Vec::new(),
            follow_symlinks: false,
            preserve_metadata: true,
            verbose: false,
            show_progress: true,
        }
    }
}

impl DirectoryCompressionConfig {
    pub fn new(compression: CompressionConfig) -> Self {
        Self {
            compression,
            ..Default::default()
        }
    }

    pub fn with_exclude_patterns(mut self, patterns: Vec<String>) -> Self {
        self.exclude_patterns = patterns;
        self
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn with_progress(mut self, show_progress: bool) -> Self {
        self.show_progress = show_progress;
        self
    }

    pub fn with_follow_symlinks(mut self, follow: bool) -> Self {
        self.follow_symlinks = follow;
        self
    }
}

/// DirectoryCompressor handles recursive directory compression
pub struct DirectoryCompressor {
    config: DirectoryCompressionConfig,
    compiled_patterns: Vec<Pattern>,
}

impl DirectoryCompressor {
    /// Create a new DirectoryCompressor
    pub fn new(config: DirectoryCompressionConfig) -> Result<Self> {
        // Compile glob patterns
        let mut compiled_patterns = Vec::new();
        for pattern_str in &config.exclude_patterns {
            let pattern = Pattern::new(pattern_str)
                .map_err(|e| Error::new(ErrorKind::InvalidInput,
                    format!("Invalid exclude pattern '{}': {}", pattern_str, e)))?;
            compiled_patterns.push(pattern);
        }

        Ok(Self {
            config,
            compiled_patterns,
        })
    }

    /// Check if a path should be excluded
    fn should_exclude(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        for pattern in &self.compiled_patterns {
            if pattern.matches(&path_str) {
                return true;
            }
        }
        false
    }

    /// Collect all files in a directory
    pub fn collect_files<P: AsRef<Path>>(&self, directory: P) -> Result<Vec<PathBuf>> {
        let directory = directory.as_ref();
        if !directory.is_dir() {
            return Err(Error::new(
                ErrorKind::NotFound,
                format!("{} is not a directory", directory.display())
            ));
        }

        let mut files = Vec::new();
        let walker = WalkDir::new(directory)
            .follow_links(self.config.follow_symlinks)
            .into_iter()
            .filter_entry(|e| !self.should_exclude(e.path()));

        for entry in walker {
            let entry = entry.map_err(|e| Error::new(ErrorKind::Other, e))?;
            let path = entry.path();

            // Skip the base directory itself
            if path == directory {
                continue;
            }

            if self.config.verbose {
                println!("  Found: {}", path.display());
            }

            files.push(path.to_path_buf());
        }

        // Sort for deterministic ordering
        files.sort();

        Ok(files)
    }

    /// Create a manifest from a directory
    pub fn create_manifest<P: AsRef<Path>>(&self, directory: P) -> Result<(ArchiveManifest, Vec<u8>)> {
        let directory = directory.as_ref();
        let files = self.collect_files(directory)?;

        if self.config.verbose {
            println!("Collected {} files", files.len());
        }

        let mut manifest = ArchiveManifest::new(directory.to_path_buf());
        let mut file_data = Vec::new();
        let mut current_offset = 0u64;

        // Setup progress bar
        let progress = if self.config.show_progress {
            let pb = ProgressBar::new(files.len() as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                    .unwrap()
                    .progress_chars("#>-")
            );
            Some(pb)
        } else {
            None
        };

        for file_path in &files {
            let relative_path = file_path.strip_prefix(directory)
                .map_err(|e| Error::new(ErrorKind::Other, e))?
                .to_path_buf();

            if let Some(ref pb) = progress {
                pb.set_message(format!("{}", relative_path.display()));
            }

            let metadata = fs::symlink_metadata(file_path)?;

            let entry = if metadata.is_symlink() {
                // Handle symlink
                FileEntry::from_path(file_path, relative_path.clone(), current_offset)?
            } else if metadata.is_dir() {
                // Handle directory
                FileEntry::from_path(file_path, relative_path.clone(), current_offset)?
            } else {
                // Handle regular file
                let file_contents = fs::read(file_path)?;
                let entry = FileEntry::from_path(file_path, relative_path.clone(), current_offset)?;

                // Append file data
                file_data.extend_from_slice(&file_contents);
                current_offset += file_contents.len() as u64;

                entry
            };

            if self.config.verbose {
                println!("  Added: {} ({} bytes)", relative_path.display(), entry.size);
            }

            manifest.add_entry(entry);

            if let Some(ref pb) = progress {
                pb.inc(1);
            }
        }

        if let Some(pb) = progress {
            pb.finish_with_message("Done");
        }

        Ok((manifest, file_data))
    }

    /// Compress a directory into a GLIF archive
    pub fn compress_directory<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        directory: P,
        output_path: Q,
    ) -> Result<()> {
        let directory = directory.as_ref();
        let output_path = output_path.as_ref();

        if self.config.verbose {
            println!("Compressing directory: {}", directory.display());
        }

        // Create manifest and collect file data
        let (manifest, file_data) = self.create_manifest(directory)?;

        if self.config.verbose {
            println!("Total files: {}", manifest.file_count);
            println!("Total size: {} bytes", manifest.total_size);
        }

        // Compress the concatenated file data
        let compressed_data = crate::compress(&file_data, &self.config.compression)?;

        if self.config.verbose {
            println!("Compressed size: {} bytes", compressed_data.len());
            println!("Compression ratio: {:.2}%",
                manifest.compression_ratio(compressed_data.len() as u64));
        }

        // Write the final archive
        let mut output = fs::File::create(output_path)?;

        // Write manifest
        manifest.write(&mut output)?;

        // Write compressed data
        output.write_all(&compressed_data)?;

        if self.config.verbose {
            println!("Archive created: {}", output_path.display());
        }

        Ok(())
    }

    /// Extract a directory archive
    pub fn extract_directory<P: AsRef<Path>, Q: AsRef<Path>>(
        input_path: P,
        output_directory: Q,
        threads: usize,
        verbose: bool,
        show_progress: bool,
    ) -> Result<()> {
        let input_path = input_path.as_ref();
        let output_directory = output_directory.as_ref();

        if verbose {
            println!("Extracting archive: {}", input_path.display());
        }

        // Read the archive
        let archive_data = fs::read(input_path)?;
        let mut cursor = std::io::Cursor::new(&archive_data);

        // Read manifest
        let manifest = ArchiveManifest::read(&mut cursor)?;

        if verbose {
            println!("Files in archive: {}", manifest.file_count);
            println!("Total size: {} bytes", manifest.total_size);
        }

        // Read compressed data
        let compressed_data_start = cursor.position() as usize;
        let compressed_data = &archive_data[compressed_data_start..];

        // Decompress
        let decompressed_data = crate::decompress(compressed_data, threads)?;

        if verbose {
            println!("Decompressed {} bytes", decompressed_data.len());
        }

        // Setup progress bar
        let progress = if show_progress {
            let pb = ProgressBar::new(manifest.file_count as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                    .unwrap()
                    .progress_chars("#>-")
            );
            Some(pb)
        } else {
            None
        };

        // Extract files
        for entry in &manifest.entries {
            let target_path = output_directory.join(&entry.path);

            if let Some(ref pb) = progress {
                pb.set_message(format!("{}", entry.path.display()));
            }

            match entry.file_type {
                FileType::Directory => {
                    fs::create_dir_all(&target_path)?;
                    if verbose {
                        println!("  Created directory: {}", target_path.display());
                    }
                }
                FileType::Symlink => {
                    if let Some(ref target) = entry.symlink_target {
                        if let Some(parent) = target_path.parent() {
                            fs::create_dir_all(parent)?;
                        }
                        std::os::unix::fs::symlink(target, &target_path)?;
                        if verbose {
                            println!("  Created symlink: {} -> {}",
                                target_path.display(), target.display());
                        }
                    }
                }
                FileType::Regular => {
                    // Extract file data
                    let start = entry.data_offset as usize;
                    let end = start + entry.size as usize;

                    if end > decompressed_data.len() {
                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            format!("File data out of bounds for {}", entry.path.display())
                        ));
                    }

                    let file_data = &decompressed_data[start..end];

                    // Verify integrity
                    entry.verify_integrity(file_data)?;

                    // Create parent directories
                    if let Some(parent) = target_path.parent() {
                        fs::create_dir_all(parent)?;
                    }

                    // Write file
                    fs::write(&target_path, file_data)?;

                    if verbose {
                        println!("  Extracted: {} ({} bytes)",
                            target_path.display(), entry.size);
                    }
                }
            }

            // Restore metadata
            if entry.file_type != FileType::Symlink {
                entry.restore_metadata(&target_path)?;
            }

            if let Some(ref pb) = progress {
                pb.inc(1);
            }
        }

        if let Some(pb) = progress {
            pb.finish_with_message("Done");
        }

        if verbose {
            println!("Extraction complete: {}", output_directory.display());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_directory_compressor_creation() {
        let config = DirectoryCompressionConfig::default();
        let compressor = DirectoryCompressor::new(config).unwrap();
        assert_eq!(compressor.compiled_patterns.len(), 0);
    }

    #[test]
    fn test_exclude_patterns() {
        let config = DirectoryCompressionConfig::default()
            .with_exclude_patterns(vec!["*.log".to_string(), "temp/*".to_string()]);
        let compressor = DirectoryCompressor::new(config).unwrap();

        assert!(compressor.should_exclude(Path::new("test.log")));
        assert!(compressor.should_exclude(Path::new("temp/file.txt")));
        assert!(!compressor.should_exclude(Path::new("test.txt")));
    }

    #[test]
    fn test_collect_files() {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        // Create test structure
        fs::create_dir(base.join("subdir")).unwrap();
        File::create(base.join("file1.txt")).unwrap();
        File::create(base.join("subdir/file2.txt")).unwrap();

        let config = DirectoryCompressionConfig::default();
        let compressor = DirectoryCompressor::new(config).unwrap();

        let files = compressor.collect_files(base).unwrap();
        assert_eq!(files.len(), 3); // subdir + 2 files
    }

    #[test]
    fn test_collect_files_with_exclude() {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        // Create test structure
        fs::create_dir(base.join("subdir")).unwrap();
        File::create(base.join("file1.txt")).unwrap();
        File::create(base.join("file2.log")).unwrap();
        File::create(base.join("subdir/file3.txt")).unwrap();

        let config = DirectoryCompressionConfig::default()
            .with_exclude_patterns(vec!["*.log".to_string()]);
        let compressor = DirectoryCompressor::new(config).unwrap();

        let files = compressor.collect_files(base).unwrap();
        // Should exclude file2.log
        assert_eq!(files.len(), 3); // subdir + file1.txt + subdir/file3.txt
    }

    #[test]
    fn test_compress_extract_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("source");
        let extract_dir = temp_dir.path().join("extract");
        let archive_path = temp_dir.path().join("test.glif");

        // Create source structure
        fs::create_dir(&source_dir).unwrap();
        fs::create_dir(source_dir.join("subdir")).unwrap();

        let mut file1 = File::create(source_dir.join("file1.txt")).unwrap();
        file1.write_all(b"Hello, World!").unwrap();

        let mut file2 = File::create(source_dir.join("subdir/file2.txt")).unwrap();
        file2.write_all(b"Test data").unwrap();

        // Compress
        let config = DirectoryCompressionConfig::default()
            .with_verbose(false)
            .with_progress(false);
        let compressor = DirectoryCompressor::new(config).unwrap();
        compressor.compress_directory(&source_dir, &archive_path).unwrap();

        assert!(archive_path.exists());

        // Extract
        DirectoryCompressor::extract_directory(
            &archive_path,
            &extract_dir,
            4,
            false,
            false,
        ).unwrap();

        // Verify
        assert!(extract_dir.join("file1.txt").exists());
        assert!(extract_dir.join("subdir/file2.txt").exists());

        let content1 = fs::read_to_string(extract_dir.join("file1.txt")).unwrap();
        assert_eq!(content1, "Hello, World!");

        let content2 = fs::read_to_string(extract_dir.join("subdir/file2.txt")).unwrap();
        assert_eq!(content2, "Test data");
    }
}
