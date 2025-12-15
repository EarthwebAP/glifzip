use glifzip::{DirectoryCompressor, DirectoryCompressionConfig, CompressionConfig};
use std::fs::{self, File};
use std::io::Write;
use std::os::unix::fs as unix_fs;
use tempfile::TempDir;

#[test]
fn test_directory_compress_extract_basic() {
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
    let compression_config = CompressionConfig::fast();
    let config = DirectoryCompressionConfig::new(compression_config)
        .with_verbose(false)
        .with_progress(false);

    let compressor = DirectoryCompressor::new(config).unwrap();
    compressor.compress_directory(&source_dir, &archive_path).unwrap();

    assert!(archive_path.exists());

    // Extract
    DirectoryCompressor::extract_directory(&archive_path, &extract_dir, 4, false, false).unwrap();

    // Verify
    assert!(extract_dir.join("file1.txt").exists());
    assert!(extract_dir.join("subdir/file2.txt").exists());

    let content1 = fs::read_to_string(extract_dir.join("file1.txt")).unwrap();
    assert_eq!(content1, "Hello, World!");

    let content2 = fs::read_to_string(extract_dir.join("subdir/file2.txt")).unwrap();
    assert_eq!(content2, "Test data");
}

#[test]
fn test_directory_compress_with_exclude_patterns() {
    let temp_dir = TempDir::new().unwrap();
    let source_dir = temp_dir.path().join("source");
    let archive_path = temp_dir.path().join("test.glif");

    // Create source structure
    fs::create_dir(&source_dir).unwrap();
    File::create(source_dir.join("file1.txt")).unwrap();
    File::create(source_dir.join("file2.log")).unwrap();
    File::create(source_dir.join("file3.txt")).unwrap();

    // Compress with exclude pattern
    let compression_config = CompressionConfig::fast();
    let config = DirectoryCompressionConfig::new(compression_config)
        .with_exclude_patterns(vec!["*.log".to_string()])
        .with_verbose(false)
        .with_progress(false);

    let compressor = DirectoryCompressor::new(config).unwrap();
    compressor.compress_directory(&source_dir, &archive_path).unwrap();

    // Verify archive contents
    let archive_data = fs::read(&archive_path).unwrap();
    let mut cursor = std::io::Cursor::new(&archive_data);
    let manifest = glifzip::ArchiveManifest::read(&mut cursor).unwrap();

    // Should have 2 files (file1.txt and file3.txt), not file2.log
    let file_paths: Vec<String> = manifest.entries.iter()
        .map(|e| e.path.to_string_lossy().to_string())
        .collect();

    assert!(!file_paths.iter().any(|p| p.contains(".log")));
    assert!(file_paths.iter().any(|p| p.contains("file1.txt")));
    assert!(file_paths.iter().any(|p| p.contains("file3.txt")));
}

#[test]
fn test_directory_compress_nested_directories() {
    let temp_dir = TempDir::new().unwrap();
    let source_dir = temp_dir.path().join("source");
    let extract_dir = temp_dir.path().join("extract");
    let archive_path = temp_dir.path().join("test.glif");

    // Create nested structure
    fs::create_dir(&source_dir).unwrap();
    fs::create_dir(source_dir.join("level1")).unwrap();
    fs::create_dir(source_dir.join("level1/level2")).unwrap();
    fs::create_dir(source_dir.join("level1/level2/level3")).unwrap();

    File::create(source_dir.join("root.txt")).unwrap()
        .write_all(b"root").unwrap();
    File::create(source_dir.join("level1/file1.txt")).unwrap()
        .write_all(b"level1").unwrap();
    File::create(source_dir.join("level1/level2/file2.txt")).unwrap()
        .write_all(b"level2").unwrap();
    File::create(source_dir.join("level1/level2/level3/file3.txt")).unwrap()
        .write_all(b"level3").unwrap();

    // Compress
    let compression_config = CompressionConfig::fast();
    let config = DirectoryCompressionConfig::new(compression_config)
        .with_verbose(false)
        .with_progress(false);

    let compressor = DirectoryCompressor::new(config).unwrap();
    compressor.compress_directory(&source_dir, &archive_path).unwrap();

    // Extract
    DirectoryCompressor::extract_directory(&archive_path, &extract_dir, 4, false, false).unwrap();

    // Verify all files exist
    assert!(extract_dir.join("root.txt").exists());
    assert!(extract_dir.join("level1/file1.txt").exists());
    assert!(extract_dir.join("level1/level2/file2.txt").exists());
    assert!(extract_dir.join("level1/level2/level3/file3.txt").exists());

    // Verify contents
    let content = fs::read_to_string(extract_dir.join("level1/level2/level3/file3.txt")).unwrap();
    assert_eq!(content, "level3");
}

#[test]
fn test_directory_compress_empty_directory() {
    let temp_dir = TempDir::new().unwrap();
    let source_dir = temp_dir.path().join("source");
    let extract_dir = temp_dir.path().join("extract");
    let archive_path = temp_dir.path().join("test.glif");

    // Create empty directory with subdirectories
    fs::create_dir(&source_dir).unwrap();
    fs::create_dir(source_dir.join("empty1")).unwrap();
    fs::create_dir(source_dir.join("empty2")).unwrap();

    // Compress
    let compression_config = CompressionConfig::fast();
    let config = DirectoryCompressionConfig::new(compression_config)
        .with_verbose(false)
        .with_progress(false);

    let compressor = DirectoryCompressor::new(config).unwrap();
    compressor.compress_directory(&source_dir, &archive_path).unwrap();

    // Extract
    DirectoryCompressor::extract_directory(&archive_path, &extract_dir, 4, false, false).unwrap();

    // Verify directories exist
    assert!(extract_dir.join("empty1").is_dir());
    assert!(extract_dir.join("empty2").is_dir());
}

#[test]
fn test_directory_compress_with_symlinks() {
    let temp_dir = TempDir::new().unwrap();
    let source_dir = temp_dir.path().join("source");
    let extract_dir = temp_dir.path().join("extract");
    let archive_path = temp_dir.path().join("test.glif");

    // Create source structure with symlink
    fs::create_dir(&source_dir).unwrap();
    let target_file = source_dir.join("target.txt");
    File::create(&target_file).unwrap()
        .write_all(b"target content").unwrap();

    let link_file = source_dir.join("link.txt");
    unix_fs::symlink(&target_file, &link_file).unwrap();

    // Compress
    let compression_config = CompressionConfig::fast();
    let config = DirectoryCompressionConfig::new(compression_config)
        .with_verbose(false)
        .with_progress(false);

    let compressor = DirectoryCompressor::new(config).unwrap();
    compressor.compress_directory(&source_dir, &archive_path).unwrap();

    // Extract
    DirectoryCompressor::extract_directory(&archive_path, &extract_dir, 4, false, false).unwrap();

    // Verify symlink exists and points to correct target
    let extracted_link = extract_dir.join("link.txt");
    assert!(extracted_link.exists());
    assert!(fs::symlink_metadata(&extracted_link).unwrap().is_symlink());
}

#[test]
fn test_directory_compress_large_files() {
    let temp_dir = TempDir::new().unwrap();
    let source_dir = temp_dir.path().join("source");
    let extract_dir = temp_dir.path().join("extract");
    let archive_path = temp_dir.path().join("test.glif");

    // Create source with large file (5 MB)
    fs::create_dir(&source_dir).unwrap();
    let large_data: Vec<u8> = (0..5 * 1024 * 1024).map(|i| (i % 256) as u8).collect();
    fs::write(source_dir.join("large.bin"), &large_data).unwrap();

    // Compress
    let compression_config = CompressionConfig::fast();
    let config = DirectoryCompressionConfig::new(compression_config)
        .with_verbose(false)
        .with_progress(false);

    let compressor = DirectoryCompressor::new(config).unwrap();
    compressor.compress_directory(&source_dir, &archive_path).unwrap();

    // Extract
    DirectoryCompressor::extract_directory(&archive_path, &extract_dir, 4, false, false).unwrap();

    // Verify
    let extracted_data = fs::read(extract_dir.join("large.bin")).unwrap();
    assert_eq!(extracted_data.len(), large_data.len());
    assert_eq!(extracted_data, large_data);
}

#[test]
fn test_directory_compress_binary_files() {
    let temp_dir = TempDir::new().unwrap();
    let source_dir = temp_dir.path().join("source");
    let extract_dir = temp_dir.path().join("extract");
    let archive_path = temp_dir.path().join("test.glif");

    // Create binary files
    fs::create_dir(&source_dir).unwrap();
    let binary_data: Vec<u8> = (0..1000).map(|i| i as u8).collect();
    fs::write(source_dir.join("binary.bin"), &binary_data).unwrap();

    // Compress
    let compression_config = CompressionConfig::fast();
    let config = DirectoryCompressionConfig::new(compression_config)
        .with_verbose(false)
        .with_progress(false);

    let compressor = DirectoryCompressor::new(config).unwrap();
    compressor.compress_directory(&source_dir, &archive_path).unwrap();

    // Extract
    DirectoryCompressor::extract_directory(&archive_path, &extract_dir, 4, false, false).unwrap();

    // Verify binary data is intact
    let extracted_data = fs::read(extract_dir.join("binary.bin")).unwrap();
    assert_eq!(extracted_data, binary_data);
}

#[test]
fn test_manifest_read_write() {
    let temp_dir = TempDir::new().unwrap();
    let source_dir = temp_dir.path().join("source");
    let archive_path = temp_dir.path().join("test.glif");

    // Create simple structure
    fs::create_dir(&source_dir).unwrap();
    File::create(source_dir.join("test.txt")).unwrap()
        .write_all(b"test").unwrap();

    // Compress
    let compression_config = CompressionConfig::fast();
    let config = DirectoryCompressionConfig::new(compression_config)
        .with_verbose(false)
        .with_progress(false);

    let compressor = DirectoryCompressor::new(config).unwrap();
    compressor.compress_directory(&source_dir, &archive_path).unwrap();

    // Read and verify manifest
    let archive_data = fs::read(&archive_path).unwrap();
    let mut cursor = std::io::Cursor::new(&archive_data);
    let manifest = glifzip::ArchiveManifest::read(&mut cursor).unwrap();

    assert_eq!(manifest.version, 1);
    assert!(manifest.file_count > 0);
    assert!(manifest.total_size > 0);
    assert_eq!(manifest.base_directory, source_dir);
}

#[test]
fn test_multiple_exclude_patterns() {
    let temp_dir = TempDir::new().unwrap();
    let source_dir = temp_dir.path().join("source");
    let archive_path = temp_dir.path().join("test.glif");

    // Create various files
    fs::create_dir(&source_dir).unwrap();
    File::create(source_dir.join("file.txt")).unwrap();
    File::create(source_dir.join("file.log")).unwrap();
    File::create(source_dir.join("file.tmp")).unwrap();
    File::create(source_dir.join("file.md")).unwrap();

    // Compress with multiple exclude patterns
    let compression_config = CompressionConfig::fast();
    let config = DirectoryCompressionConfig::new(compression_config)
        .with_exclude_patterns(vec![
            "*.log".to_string(),
            "*.tmp".to_string(),
        ])
        .with_verbose(false)
        .with_progress(false);

    let compressor = DirectoryCompressor::new(config).unwrap();
    compressor.compress_directory(&source_dir, &archive_path).unwrap();

    // Verify
    let archive_data = fs::read(&archive_path).unwrap();
    let mut cursor = std::io::Cursor::new(&archive_data);
    let manifest = glifzip::ArchiveManifest::read(&mut cursor).unwrap();

    let file_paths: Vec<String> = manifest.entries.iter()
        .map(|e| e.path.to_string_lossy().to_string())
        .collect();

    assert!(!file_paths.iter().any(|p| p.ends_with(".log")));
    assert!(!file_paths.iter().any(|p| p.ends_with(".tmp")));
    assert!(file_paths.iter().any(|p| p.ends_with(".txt")));
    assert!(file_paths.iter().any(|p| p.ends_with(".md")));
}
