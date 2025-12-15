use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use glifzip::{compress, decompress, CompressionConfig};
use std::fs::{File, create_dir_all};
use std::io::Write;
use std::time::Instant;

/// Generate random uncompressible data
fn generate_random_data(size: usize) -> Vec<u8> {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hash, Hasher};

    let mut data = Vec::with_capacity(size);
    let state = RandomState::new();

    for i in 0..size {
        let mut hasher = state.build_hasher();
        i.hash(&mut hasher);
        data.push((hasher.finish() % 256) as u8);
    }

    data
}

/// Generate highly compressible text data
fn generate_compressible_text(size: usize) -> Vec<u8> {
    let pattern = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit. ";
    let mut data = Vec::with_capacity(size);

    while data.len() < size {
        data.extend_from_slice(pattern);
    }

    data.truncate(size);
    data
}

/// Generate source code-like data
fn generate_source_code(size: usize) -> Vec<u8> {
    let code_patterns = [
        b"fn main() {\n    println!(\"Hello, world!\");\n}\n",
        b"pub struct MyStruct {\n    field1: u32,\n    field2: String,\n}\n",
        b"impl MyTrait for MyStruct {\n    fn method(&self) -> bool {\n        true\n    }\n}\n",
        b"use std::collections::HashMap;\n",
        b"#[derive(Debug, Clone)]\n",
    ];

    let mut data = Vec::with_capacity(size);
    let mut idx = 0;

    while data.len() < size {
        data.extend_from_slice(code_patterns[idx % code_patterns.len()]);
        idx += 1;
    }

    data.truncate(size);
    data
}

/// Generate repeated zeros (highly compressible)
fn generate_zeros(size: usize) -> Vec<u8> {
    vec![0u8; size]
}

/// Benchmark compression throughput with different data types
fn benchmark_compression_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("compression_throughput");

    // 1 GB datasets
    let size_1gb = 1024 * 1024 * 1024;

    // Test with different data types
    let test_cases = vec![
        ("random_uncompressible_1gb", generate_random_data(size_1gb)),
        ("compressible_text_1gb", generate_compressible_text(size_1gb)),
        ("source_code_1gb", generate_source_code(size_1gb)),
        ("zeros_1gb", generate_zeros(size_1gb)),
    ];

    group.sample_size(10);

    for (name, data) in test_cases {
        group.throughput(Throughput::Bytes(data.len() as u64));

        // Test with default config (auto-detect cores)
        group.bench_with_input(BenchmarkId::new(name, "default"), &data, |b, data| {
            let config = CompressionConfig::balanced();
            b.iter(|| compress(black_box(data), &config).unwrap());
        });
    }

    group.finish();
}

/// Benchmark decompression throughput with different data types
fn benchmark_decompression_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("decompression_throughput");

    // 1 GB datasets
    let size_1gb = 1024 * 1024 * 1024;

    let test_cases = vec![
        ("random_uncompressible_1gb", generate_random_data(size_1gb)),
        ("compressible_text_1gb", generate_compressible_text(size_1gb)),
        ("source_code_1gb", generate_source_code(size_1gb)),
        ("zeros_1gb", generate_zeros(size_1gb)),
    ];

    group.sample_size(10);

    for (name, data) in test_cases {
        // Pre-compress the data
        let config = CompressionConfig::balanced();
        let compressed = compress(&data, &config).unwrap();

        group.throughput(Throughput::Bytes(data.len() as u64));

        group.bench_with_input(BenchmarkId::new(name, "default"), &compressed, |b, compressed| {
            b.iter(|| decompress(black_box(compressed), config.threads).unwrap());
        });
    }

    group.finish();
}

/// Benchmark multi-core scaling for compression
fn benchmark_compression_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("compression_scaling");

    // 1 GB dataset
    let size = 1024 * 1024 * 1024;
    let data = generate_compressible_text(size);

    group.throughput(Throughput::Bytes(size as u64));
    group.sample_size(10);

    // Test with 1, 2, 4, 8, 16 cores
    for threads in [1, 2, 4, 8, 16].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_cores", threads)),
            threads,
            |b, &threads| {
                let config = CompressionConfig::new(8, threads);
                b.iter(|| compress(black_box(&data), &config).unwrap());
            }
        );
    }

    group.finish();
}

/// Benchmark multi-core scaling for decompression
fn benchmark_decompression_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("decompression_scaling");

    // 1 GB dataset
    let size = 1024 * 1024 * 1024;
    let data = generate_compressible_text(size);

    // Pre-compress
    let config = CompressionConfig::balanced();
    let compressed = compress(&data, &config).unwrap();

    group.throughput(Throughput::Bytes(size as u64));
    group.sample_size(10);

    // Test with 1, 2, 4, 8, 16 cores
    for threads in [1, 2, 4, 8, 16].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_cores", threads)),
            threads,
            |b, &threads| {
                b.iter(|| decompress(black_box(&compressed), threads).unwrap());
            }
        );
    }

    group.finish();
}

/// Benchmark compression ratios by data type
fn benchmark_compression_ratios(c: &mut Criterion) {
    let mut group = c.benchmark_group("compression_ratios");

    // 100 MB for faster testing
    let size = 100 * 1024 * 1024;

    let test_cases = vec![
        ("random", generate_random_data(size)),
        ("text", generate_compressible_text(size)),
        ("source_code", generate_source_code(size)),
        ("zeros", generate_zeros(size)),
    ];

    group.sample_size(10);

    for (name, data) in test_cases {
        let config = CompressionConfig::balanced();

        group.bench_function(name, |b| {
            b.iter(|| {
                let compressed = compress(black_box(&data), &config).unwrap();
                let ratio = (compressed.len() as f64 / data.len() as f64) * 100.0;
                black_box(ratio);
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_compression_throughput,
    benchmark_decompression_throughput,
    benchmark_compression_scaling,
    benchmark_decompression_scaling,
    benchmark_compression_ratios
);
criterion_main!(benches);
