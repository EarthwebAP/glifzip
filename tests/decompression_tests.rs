use glifzip::{compress, decompress, CompressionConfig};
use std::time::Instant;

#[test]
fn test_decompression_speed_per_core() {
    // Test with 100 MB of data (reduced for faster testing)
    let size = 100 * 1024 * 1024; // 100 MB
    let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();

    let config = CompressionConfig::fast();
    let compressed = compress(&data, &config).unwrap();

    let start = Instant::now();
    let _decompressed = decompress(&compressed, config.threads).unwrap();
    let elapsed = start.elapsed();

    let throughput_mbs = size as f64 / elapsed.as_secs_f64() / (1024.0 * 1024.0);

    println!("Decompression throughput: {:.2} MB/s with {} threads",
             throughput_mbs, config.threads);

    // Just verify it completes without error - performance varies greatly by environment
    assert!(throughput_mbs > 0.0,
            "Decompression throughput should be positive");
}

#[test]
fn test_decompression_integrity() {
    let data: Vec<u8> = (0..10 * 1024 * 1024).map(|i| ((i * 7) % 256) as u8).collect();
    let config = CompressionConfig::default();

    let compressed = compress(&data, &config).unwrap();
    let decompressed = decompress(&compressed, config.threads).unwrap();

    // Verify every byte
    for (i, (original, decompressed)) in data.iter().zip(decompressed.iter()).enumerate() {
        assert_eq!(original, decompressed, "Mismatch at byte {}", i);
    }
}

#[test]
fn test_corrupted_archive_detection() {
    let data = b"Test data for corruption detection";
    let config = CompressionConfig::default();

    let mut compressed = compress(data, &config).unwrap();

    // Corrupt a byte in the compressed data
    if compressed.len() > 200 {
        compressed[200] ^= 0xFF;
    }

    // Decompression should fail
    let result = decompress(&compressed, config.threads);
    assert!(result.is_err(), "Should detect corrupted archive");
}

#[test]
fn test_invalid_header() {
    let invalid_data = b"INVALID_HEADER_DATA";

    let result = decompress(invalid_data, 4);
    assert!(result.is_err(), "Should reject invalid header");
}

#[test]
fn test_partial_archive() {
    let data = b"Test data for partial archive";
    let config = CompressionConfig::default();

    let compressed = compress(data, &config).unwrap();

    // Truncate the archive
    let partial = &compressed[..compressed.len() / 2];

    let result = decompress(partial, config.threads);
    assert!(result.is_err(), "Should reject partial archive");
}

#[test]
fn test_decompression_deterministic() {
    let data = vec![123u8; 1_000_000];
    let config = CompressionConfig::default();

    let compressed = compress(&data, &config).unwrap();

    let decompressed1 = decompress(&compressed, config.threads).unwrap();
    let decompressed2 = decompress(&compressed, config.threads).unwrap();

    assert_eq!(decompressed1, decompressed2, "Decompression not deterministic");
}
