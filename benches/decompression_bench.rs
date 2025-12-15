use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use glifzip::{compress, decompress, CompressionConfig};

fn benchmark_decompression(c: &mut Criterion) {
    let mut group = c.benchmark_group("decompression");

    // Test different data sizes
    for size in [1024 * 1024, 10 * 1024 * 1024, 100 * 1024 * 1024].iter() {
        let data: Vec<u8> = (0..*size).map(|i| (i % 256) as u8).collect();

        // Pre-compress the data
        let config = CompressionConfig::fast();
        let compressed = compress(&data, &config).unwrap();

        group.throughput(Throughput::Bytes(*size as u64));
        group.sample_size(10);

        group.bench_with_input(BenchmarkId::from_parameter(size), &compressed, |b, compressed| {
            b.iter(|| decompress(black_box(compressed), config.threads).unwrap());
        });
    }

    group.finish();
}

fn benchmark_decompression_threads(c: &mut Criterion) {
    let mut group = c.benchmark_group("decompression_threads");

    let size = 100 * 1024 * 1024; // 100 MB
    let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();

    // Pre-compress the data
    let config = CompressionConfig::fast();
    let compressed = compress(&data, &config).unwrap();

    group.throughput(Throughput::Bytes(size as u64));
    group.sample_size(10);

    for threads in [1, 2, 4, 8].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(threads), threads, |b, &threads| {
            b.iter(|| decompress(black_box(&compressed), threads).unwrap());
        });
    }

    group.finish();
}

fn benchmark_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("roundtrip");

    let size = 10 * 1024 * 1024; // 10 MB
    let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();

    group.throughput(Throughput::Bytes(size as u64));
    group.sample_size(10);

    let config = CompressionConfig::fast();

    group.bench_function("compress_decompress", |b| {
        b.iter(|| {
            let compressed = compress(black_box(&data), &config).unwrap();
            let _decompressed = decompress(black_box(&compressed), config.threads).unwrap();
        });
    });

    group.finish();
}

criterion_group!(benches, benchmark_decompression, benchmark_decompression_threads, benchmark_roundtrip);
criterion_main!(benches);
