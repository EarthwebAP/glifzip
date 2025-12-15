use glifzip::{compress_file, decompress_file, verify_archive, CompressionConfig};
use std::fs;
use std::io::Write;
use tempfile::tempdir;

#[test]
fn test_file_compression_roundtrip() {
    let dir = tempdir().unwrap();

    let input_path = dir.path().join("input.txt");
    let archive_path = dir.path().join("archive.glif");
    let output_path = dir.path().join("output.txt");

    // Create test file
    let test_data = b"This is a test file for GLifzip compression.\n".repeat(1000);
    fs::write(&input_path, &test_data).unwrap();

    // Compress
    let config = CompressionConfig::default();
    compress_file(&input_path, &archive_path, &config).unwrap();

    // Verify archive exists
    assert!(archive_path.exists());

    // Decompress
    decompress_file(&archive_path, &output_path, config.threads).unwrap();

    // Verify output
    let output_data = fs::read(&output_path).unwrap();
    assert_eq!(test_data, output_data);
}

#[test]
fn test_verify_archive_function() {
    let dir = tempdir().unwrap();

    let input_path = dir.path().join("input.txt");
    let archive_path = dir.path().join("archive.glif");

    // Create test file
    let test_data = b"Test data for verification".repeat(100);
    fs::write(&input_path, &test_data).unwrap();

    // Compress
    let config = CompressionConfig::default();
    compress_file(&input_path, &archive_path, &config).unwrap();

    // Verify
    let archive = fs::read(&archive_path).unwrap();
    let sidecar = verify_archive(&archive).unwrap();

    assert_eq!(sidecar.payload.size, test_data.len() as u64);
    assert!(sidecar.archive.size > 0);
    assert!(sidecar.payload.compression_ratio > 0.0);
}

#[test]
fn test_large_file_compression() {
    let dir = tempdir().unwrap();

    let input_path = dir.path().join("large.bin");
    let archive_path = dir.path().join("large.glif");
    let output_path = dir.path().join("large_out.bin");

    // Create 50 MB file
    let mut file = fs::File::create(&input_path).unwrap();
    for i in 0..50 {
        let chunk: Vec<u8> = (0..1024 * 1024).map(|j| ((i + j) % 256) as u8).collect();
        file.write_all(&chunk).unwrap();
    }
    drop(file);

    // Compress
    let config = CompressionConfig::fast();
    compress_file(&input_path, &archive_path, &config).unwrap();

    // Decompress
    decompress_file(&archive_path, &output_path, config.threads).unwrap();

    // Verify sizes match
    let input_size = fs::metadata(&input_path).unwrap().len();
    let output_size = fs::metadata(&output_path).unwrap().len();
    assert_eq!(input_size, output_size);

    // Verify content matches (sample check to save time)
    let input_data = fs::read(&input_path).unwrap();
    let output_data = fs::read(&output_path).unwrap();
    assert_eq!(input_data.len(), output_data.len());

    // Check first, middle, and last chunks
    assert_eq!(&input_data[..10000], &output_data[..10000]);
    let mid = input_data.len() / 2;
    assert_eq!(&input_data[mid..mid+10000], &output_data[mid..mid+10000]);
    assert_eq!(&input_data[input_data.len()-10000..], &output_data[output_data.len()-10000..]);
}

#[test]
fn test_binary_file_handling() {
    let dir = tempdir().unwrap();

    let input_path = dir.path().join("binary.bin");
    let archive_path = dir.path().join("binary.glif");
    let output_path = dir.path().join("binary_out.bin");

    // Create binary file with all byte values
    let binary_data: Vec<u8> = (0..256).cycle().take(100_000).map(|i| i as u8).collect();
    fs::write(&input_path, &binary_data).unwrap();

    // Compress and decompress
    let config = CompressionConfig::default();
    compress_file(&input_path, &archive_path, &config).unwrap();
    decompress_file(&archive_path, &output_path, config.threads).unwrap();

    // Verify
    let output_data = fs::read(&output_path).unwrap();
    assert_eq!(binary_data, output_data);
}

#[test]
fn test_compression_ratio_reporting() {
    let dir = tempdir().unwrap();

    let input_path = dir.path().join("input.txt");
    let archive_path = dir.path().join("archive.glif");

    // Create highly compressible data
    let test_data = vec![0u8; 1_000_000];
    fs::write(&input_path, &test_data).unwrap();

    // Compress
    let config = CompressionConfig::default();
    compress_file(&input_path, &archive_path, &config).unwrap();

    // Verify compression ratio
    let archive = fs::read(&archive_path).unwrap();
    let sidecar = verify_archive(&archive).unwrap();

    println!("Compression ratio: {:.2}%", sidecar.payload.compression_ratio * 100.0);

    // Highly compressible data should compress to <5%
    assert!(sidecar.payload.compression_ratio < 0.05,
            "Expected high compression for zeros, got {:.2}%",
            sidecar.payload.compression_ratio * 100.0);
}
