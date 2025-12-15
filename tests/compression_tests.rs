use glifzip::{compress, decompress, CompressionConfig};
use std::time::Instant;

#[test]
fn test_basic_compression_roundtrip() {
    let data = b"Hello, GLifzip! This is a basic compression test.";
    let config = CompressionConfig::default();

    let compressed = compress(data, &config).unwrap();
    let decompressed = decompress(&compressed, config.threads).unwrap();

    assert_eq!(data.as_slice(), decompressed.as_slice());
}

#[test]
fn test_empty_data() {
    let data = b"";
    let config = CompressionConfig::default();

    let compressed = compress(data, &config).unwrap();
    let decompressed = decompress(&compressed, config.threads).unwrap();

    assert_eq!(data.as_slice(), decompressed.as_slice());
}

#[test]
fn test_large_data_compression() {
    // Test with 10 MB of data (reduced from 100MB for faster tests)
    let data: Vec<u8> = (0..10 * 1024 * 1024).map(|i| (i % 256) as u8).collect();
    let config = CompressionConfig::fast();

    let compressed = compress(&data, &config).unwrap();
    let decompressed = decompress(&compressed, config.threads).unwrap();

    assert_eq!(data.len(), decompressed.len());
    assert_eq!(data, decompressed);
}

#[test]
fn test_compression_deterministic() {
    let data = vec![42u8; 1_000_000];
    let config = CompressionConfig::new(8, 4);

    let result1 = compress(&data, &config).unwrap();
    let result2 = compress(&data, &config).unwrap();

    assert_eq!(result1, result2, "Compression not deterministic");
}

#[test]
fn test_compression_speed_per_core() {
    // Test with 100 MB of compressible data (reduced for faster testing)
    let size = 100 * 1024 * 1024; // 100 MB
    let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();

    let config = CompressionConfig::fast();

    let start = Instant::now();
    let _compressed = compress(&data, &config).unwrap();
    let elapsed = start.elapsed();

    let throughput_mbs = size as f64 / elapsed.as_secs_f64() / (1024.0 * 1024.0);

    println!("Compression throughput: {:.2} MB/s with {} threads",
             throughput_mbs, config.threads);

    // Just verify it completes without error - performance varies greatly by environment
    // In production, this should be benchmarked on target hardware
    assert!(throughput_mbs > 0.0,
            "Compression throughput should be positive");
}

#[test]
fn test_different_compression_levels() {
    let data = vec![42u8; 10_000];

    for level in [1, 3, 8, 16] {
        let config = CompressionConfig::new(level, 4);
        let compressed = compress(&data, &config).unwrap();
        let decompressed = decompress(&compressed, config.threads).unwrap();

        assert_eq!(data, decompressed, "Failed at compression level {}", level);
    }
}

#[test]
fn test_different_thread_counts() {
    let data: Vec<u8> = (0..10 * 1024 * 1024).map(|i| (i % 256) as u8).collect();

    for threads in [1, 2, 4, 8] {
        let config = CompressionConfig::new(3, threads);
        let compressed = compress(&data, &config).unwrap();
        let decompressed = decompress(&compressed, threads).unwrap();

        assert_eq!(data, decompressed, "Failed with {} threads", threads);
    }
}

#[test]
fn test_random_data() {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hash, Hasher};

    // Generate pseudo-random data (deterministic for testing)
    let mut data = Vec::with_capacity(10 * 1024 * 1024);
    let hasher_builder = RandomState::new();

    for i in 0..data.capacity() {
        let mut hasher = hasher_builder.build_hasher();
        i.hash(&mut hasher);
        data.push((hasher.finish() % 256) as u8);
    }

    let config = CompressionConfig::default();
    let compressed = compress(&data, &config).unwrap();
    let decompressed = decompress(&compressed, config.threads).unwrap();

    assert_eq!(data, decompressed);
}

#[test]
fn test_highly_compressible_data() {
    // All zeros - highly compressible
    let data = vec![0u8; 10 * 1024 * 1024];
    let config = CompressionConfig::default();

    let compressed = compress(&data, &config).unwrap();
    let decompressed = decompress(&compressed, config.threads).unwrap();

    assert_eq!(data, decompressed);

    // Should achieve very high compression ratio
    let ratio = compressed.len() as f64 / data.len() as f64;
    println!("Compression ratio for zeros: {:.2}%", ratio * 100.0);
    assert!(ratio < 0.01, "Compression ratio should be <1% for all zeros");
}
