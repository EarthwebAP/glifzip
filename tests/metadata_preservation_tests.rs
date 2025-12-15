use glifzip::{DirectoryCompressor, DirectoryCompressionConfig, CompressionConfig, FileEntry};
use std::fs::{self, File};
use std::io::Write;
use std::os::unix::fs::{PermissionsExt, MetadataExt};
use std::path::PathBuf;
use tempfile::TempDir;
use std::time::SystemTime;

#[test]
fn test_file_permissions_preserved() {
    let temp_dir = TempDir::new().unwrap();
    let source_dir = temp_dir.path().join("source");
    let extract_dir = temp_dir.path().join("extract");
    let archive_path = temp_dir.path().join("test.glif");

    // Create file with specific permissions
    fs::create_dir(&source_dir).unwrap();
    let file_path = source_dir.join("executable.sh");
    let mut file = File::create(&file_path).unwrap();
    file.write_all(b"#!/bin/bash\necho 'test'\n").unwrap();
    drop(file);

    // Set executable permissions (0755)
    let mut perms = fs::metadata(&file_path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&file_path, perms).unwrap();

    let original_mode = fs::metadata(&file_path).unwrap().permissions().mode();

    // Compress
    let compression_config = CompressionConfig::fast();
    let config = DirectoryCompressionConfig::new(compression_config)
        .with_verbose(false)
        .with_progress(false);

    let compressor = DirectoryCompressor::new(config).unwrap();
    compressor.compress_directory(&source_dir, &archive_path).unwrap();

    // Extract
    DirectoryCompressor::extract_directory(&archive_path, &extract_dir, 4, false, false).unwrap();

    // Verify permissions preserved
    let extracted_mode = fs::metadata(extract_dir.join("executable.sh"))
        .unwrap()
        .permissions()
        .mode();

    assert_eq!(original_mode, extracted_mode);
}

#[test]
fn test_directory_permissions_preserved() {
    let temp_dir = TempDir::new().unwrap();
    let source_dir = temp_dir.path().join("source");
    let extract_dir = temp_dir.path().join("extract");
    let archive_path = temp_dir.path().join("test.glif");

    // Create directory with specific permissions
    fs::create_dir(&source_dir).unwrap();
    let subdir_path = source_dir.join("restricted");
    fs::create_dir(&subdir_path).unwrap();

    let mut perms = fs::metadata(&subdir_path).unwrap().permissions();
    perms.set_mode(0o700);
    fs::set_permissions(&subdir_path, perms).unwrap();

    let original_mode = fs::metadata(&subdir_path).unwrap().permissions().mode();

    // Compress
    let compression_config = CompressionConfig::fast();
    let config = DirectoryCompressionConfig::new(compression_config)
        .with_verbose(false)
        .with_progress(false);

    let compressor = DirectoryCompressor::new(config).unwrap();
    compressor.compress_directory(&source_dir, &archive_path).unwrap();

    // Extract
    DirectoryCompressor::extract_directory(&archive_path, &extract_dir, 4, false, false).unwrap();

    // Verify permissions preserved
    let extracted_mode = fs::metadata(extract_dir.join("restricted"))
        .unwrap()
        .permissions()
        .mode();

    assert_eq!(original_mode, extracted_mode);
}

#[test]
fn test_file_timestamps_preserved() {
    let temp_dir = TempDir::new().unwrap();
    let source_dir = temp_dir.path().join("source");
    let extract_dir = temp_dir.path().join("extract");
    let archive_path = temp_dir.path().join("test.glif");

    // Create file
    fs::create_dir(&source_dir).unwrap();
    let file_path = source_dir.join("timestamped.txt");
    File::create(&file_path).unwrap()
        .write_all(b"test content").unwrap();

    // Set specific timestamp
    let test_time = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(946684800); // 2000-01-01
    filetime::set_file_mtime(&file_path, filetime::FileTime::from_system_time(test_time)).unwrap();

    let original_mtime = fs::metadata(&file_path).unwrap().modified().unwrap();

    // Compress
    let compression_config = CompressionConfig::fast();
    let config = DirectoryCompressionConfig::new(compression_config)
        .with_verbose(false)
        .with_progress(false);

    let compressor = DirectoryCompressor::new(config).unwrap();
    compressor.compress_directory(&source_dir, &archive_path).unwrap();

    // Extract
    DirectoryCompressor::extract_directory(&archive_path, &extract_dir, 4, false, false).unwrap();

    // Verify timestamp preserved (within 1 second tolerance for filesystem precision)
    let extracted_mtime = fs::metadata(extract_dir.join("timestamped.txt"))
        .unwrap()
        .modified()
        .unwrap();

    let diff = if extracted_mtime > original_mtime {
        extracted_mtime.duration_since(original_mtime).unwrap()
    } else {
        original_mtime.duration_since(extracted_mtime).unwrap()
    };

    assert!(diff.as_secs() <= 1, "Timestamp difference too large: {:?}", diff);
}

#[test]
fn test_symlink_preservation() {
    let temp_dir = TempDir::new().unwrap();
    let source_dir = temp_dir.path().join("source");
    let extract_dir = temp_dir.path().join("extract");
    let archive_path = temp_dir.path().join("test.glif");

    // Create file and symlink
    fs::create_dir(&source_dir).unwrap();
    let target_file = source_dir.join("target.txt");
    File::create(&target_file).unwrap()
        .write_all(b"target content").unwrap();

    let link_file = source_dir.join("link.txt");
    std::os::unix::fs::symlink(&target_file, &link_file).unwrap();

    // Compress
    let compression_config = CompressionConfig::fast();
    let config = DirectoryCompressionConfig::new(compression_config)
        .with_verbose(false)
        .with_progress(false);

    let compressor = DirectoryCompressor::new(config).unwrap();
    compressor.compress_directory(&source_dir, &archive_path).unwrap();

    // Extract
    DirectoryCompressor::extract_directory(&archive_path, &extract_dir, 4, false, false).unwrap();

    // Verify symlink is preserved
    let extracted_link = extract_dir.join("link.txt");
    assert!(fs::symlink_metadata(&extracted_link).unwrap().is_symlink());

    // Verify symlink target
    let link_target = fs::read_link(&extracted_link).unwrap();
    assert!(link_target.to_string_lossy().contains("target.txt"));
}

#[test]
fn test_relative_symlink_preservation() {
    let temp_dir = TempDir::new().unwrap();
    let source_dir = temp_dir.path().join("source");
    let extract_dir = temp_dir.path().join("extract");
    let archive_path = temp_dir.path().join("test.glif");

    // Create structure with relative symlink
    fs::create_dir(&source_dir).unwrap();
    fs::create_dir(source_dir.join("subdir")).unwrap();

    let target_file = source_dir.join("target.txt");
    File::create(&target_file).unwrap()
        .write_all(b"target").unwrap();

    // Create relative symlink
    let link_file = source_dir.join("subdir/link.txt");
    std::os::unix::fs::symlink("../target.txt", &link_file).unwrap();

    // Compress
    let compression_config = CompressionConfig::fast();
    let config = DirectoryCompressionConfig::new(compression_config)
        .with_verbose(false)
        .with_progress(false);

    let compressor = DirectoryCompressor::new(config).unwrap();
    compressor.compress_directory(&source_dir, &archive_path).unwrap();

    // Extract
    DirectoryCompressor::extract_directory(&archive_path, &extract_dir, 4, false, false).unwrap();

    // Verify relative symlink works
    let extracted_link = extract_dir.join("subdir/link.txt");
    assert!(fs::symlink_metadata(&extracted_link).unwrap().is_symlink());

    // Verify we can read through the symlink
    let content = fs::read_to_string(&extracted_link).unwrap();
    assert_eq!(content, "target");
}

#[test]
fn test_file_entry_metadata_roundtrip() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");

    // Create file with content
    let mut file = File::create(&file_path).unwrap();
    file.write_all(b"test content").unwrap();
    drop(file);

    // Set permissions
    let mut perms = fs::metadata(&file_path).unwrap().permissions();
    perms.set_mode(0o644);
    fs::set_permissions(&file_path, perms).unwrap();

    // Create FileEntry
    let entry = FileEntry::from_path(&file_path, PathBuf::from("test.txt"), 0).unwrap();

    // Verify metadata captured
    assert_eq!(entry.size, 12);
    assert_eq!(entry.mode & 0o777, 0o644);
    assert!(!entry.sha256.is_empty());
}

#[test]
fn test_file_integrity_verification() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");

    // Create file
    let content = b"test content for integrity check";
    fs::write(&file_path, content).unwrap();

    // Create FileEntry
    let entry = FileEntry::from_path(&file_path, PathBuf::from("test.txt"), 0).unwrap();

    // Verify integrity with correct data
    assert!(entry.verify_integrity(content).is_ok());

    // Verify integrity fails with wrong data
    assert!(entry.verify_integrity(b"wrong content").is_err());
}

#[test]
fn test_mixed_permissions() {
    let temp_dir = TempDir::new().unwrap();
    let source_dir = temp_dir.path().join("source");
    let extract_dir = temp_dir.path().join("extract");
    let archive_path = temp_dir.path().join("test.glif");

    // Create files with different permissions
    fs::create_dir(&source_dir).unwrap();

    let file1 = source_dir.join("readonly.txt");
    File::create(&file1).unwrap().write_all(b"readonly").unwrap();
    let mut perms1 = fs::metadata(&file1).unwrap().permissions();
    perms1.set_mode(0o444);
    fs::set_permissions(&file1, perms1).unwrap();

    let file2 = source_dir.join("readwrite.txt");
    File::create(&file2).unwrap().write_all(b"readwrite").unwrap();
    let mut perms2 = fs::metadata(&file2).unwrap().permissions();
    perms2.set_mode(0o666);
    fs::set_permissions(&file2, perms2).unwrap();

    let file3 = source_dir.join("executable.sh");
    File::create(&file3).unwrap().write_all(b"#!/bin/bash").unwrap();
    let mut perms3 = fs::metadata(&file3).unwrap().permissions();
    perms3.set_mode(0o755);
    fs::set_permissions(&file3, perms3).unwrap();

    let mode1 = fs::metadata(&file1).unwrap().permissions().mode();
    let mode2 = fs::metadata(&file2).unwrap().permissions().mode();
    let mode3 = fs::metadata(&file3).unwrap().permissions().mode();

    // Compress
    let compression_config = CompressionConfig::fast();
    let config = DirectoryCompressionConfig::new(compression_config)
        .with_verbose(false)
        .with_progress(false);

    let compressor = DirectoryCompressor::new(config).unwrap();
    compressor.compress_directory(&source_dir, &archive_path).unwrap();

    // Extract
    DirectoryCompressor::extract_directory(&archive_path, &extract_dir, 4, false, false).unwrap();

    // Verify all permissions preserved
    assert_eq!(fs::metadata(extract_dir.join("readonly.txt")).unwrap().permissions().mode(), mode1);
    assert_eq!(fs::metadata(extract_dir.join("readwrite.txt")).unwrap().permissions().mode(), mode2);
    assert_eq!(fs::metadata(extract_dir.join("executable.sh")).unwrap().permissions().mode(), mode3);
}

#[test]
fn test_empty_file_metadata() {
    let temp_dir = TempDir::new().unwrap();
    let source_dir = temp_dir.path().join("source");
    let extract_dir = temp_dir.path().join("extract");
    let archive_path = temp_dir.path().join("test.glif");

    // Create empty file
    fs::create_dir(&source_dir).unwrap();
    let file_path = source_dir.join("empty.txt");
    File::create(&file_path).unwrap();

    let mut perms = fs::metadata(&file_path).unwrap().permissions();
    perms.set_mode(0o600);
    fs::set_permissions(&file_path, perms).unwrap();

    let original_mode = fs::metadata(&file_path).unwrap().permissions().mode();

    // Compress
    let compression_config = CompressionConfig::fast();
    let config = DirectoryCompressionConfig::new(compression_config)
        .with_verbose(false)
        .with_progress(false);

    let compressor = DirectoryCompressor::new(config).unwrap();
    compressor.compress_directory(&source_dir, &archive_path).unwrap();

    // Extract
    DirectoryCompressor::extract_directory(&archive_path, &extract_dir, 4, false, false).unwrap();

    // Verify empty file exists with correct permissions
    let extracted = extract_dir.join("empty.txt");
    assert!(extracted.exists());
    assert_eq!(fs::metadata(&extracted).unwrap().len(), 0);
    assert_eq!(fs::metadata(&extracted).unwrap().permissions().mode(), original_mode);
}
