use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use glifzip::{compress, CompressionConfig};

fn benchmark_compression(c: &mut Criterion) {
    let mut group = c.benchmark_group("compression");

    // Test different data sizes
    for size in [1024 * 1024, 10 * 1024 * 1024, 100 * 1024 * 1024].iter() {
        let data: Vec<u8> = (0..*size).map(|i| (i % 256) as u8).collect();

        group.throughput(Throughput::Bytes(*size as u64));
        group.sample_size(10);

        group.bench_with_input(BenchmarkId::new("fast", size), &data, |b, data| {
            let config = CompressionConfig::fast();
            b.iter(|| compress(black_box(data), &config).unwrap());
        });

        group.bench_with_input(BenchmarkId::new("balanced", size), &data, |b, data| {
            let config = CompressionConfig::balanced();
            b.iter(|| compress(black_box(data), &config).unwrap());
        });
    }

    group.finish();
}

fn benchmark_compression_threads(c: &mut Criterion) {
    let mut group = c.benchmark_group("compression_threads");

    let size = 100 * 1024 * 1024; // 100 MB
    let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();

    group.throughput(Throughput::Bytes(size as u64));
    group.sample_size(10);

    for threads in [1, 2, 4, 8].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(threads), threads, |b, &threads| {
            let config = CompressionConfig::new(3, threads);
            b.iter(|| compress(black_box(&data), &config).unwrap());
        });
    }

    group.finish();
}

fn benchmark_compression_levels(c: &mut Criterion) {
    let mut group = c.benchmark_group("compression_levels");

    let size = 10 * 1024 * 1024; // 10 MB
    let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();

    group.throughput(Throughput::Bytes(size as u64));
    group.sample_size(10);

    for level in [1, 3, 8, 16].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(level), level, |b, &level| {
            let config = CompressionConfig::new(level, 4);
            b.iter(|| compress(black_box(&data), &config).unwrap());
        });
    }

    group.finish();
}

criterion_group!(benches, benchmark_compression, benchmark_compression_threads, benchmark_compression_levels);
criterion_main!(benches);
