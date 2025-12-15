/// Comprehensive Performance Benchmark Suite for GLifzip
///
/// This benchmark suite measures:
/// - Compression/decompression throughput (GB/s per core)
/// - Multi-core scaling (1, 2, 4, 8, 16 cores)
/// - Compression ratios by data type
/// - Comparison with ZIP baseline
///
/// Outputs:
/// - CSV results for analysis
/// - Performance reports
/// - Raw data for visualization

use std::fs::{File, create_dir_all};
use std::io::{Write as IoWrite, Read};
use std::time::{Instant, Duration};
use std::path::Path;
use glifzip::{compress, decompress, CompressionConfig};

// Data generation functions
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

fn generate_compressible_text(size: usize) -> Vec<u8> {
    let pattern = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit. \
                     Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. \
                     Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris. ";
    let mut data = Vec::with_capacity(size);

    while data.len() < size {
        data.extend_from_slice(pattern);
    }

    data.truncate(size);
    data
}

fn generate_source_code(size: usize) -> Vec<u8> {
    let code_patterns = [
        b"fn main() {\n    println!(\"Hello, world!\");\n}\n\n",
        b"pub struct MyStruct {\n    field1: u32,\n    field2: String,\n}\n\n",
        b"impl MyTrait for MyStruct {\n    fn method(&self) -> bool {\n        true\n    }\n}\n\n",
        b"use std::collections::HashMap;\nuse std::io::{Read, Write};\n\n",
        b"#[derive(Debug, Clone, PartialEq)]\npub enum MyEnum {\n    Variant1,\n    Variant2(u32),\n}\n\n",
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

fn generate_zeros(size: usize) -> Vec<u8> {
    vec![0u8; size]
}

#[derive(Debug, Clone)]
struct BenchmarkResult {
    test_name: String,
    data_type: String,
    data_size_mb: f64,
    operation: String,
    threads: usize,
    duration_ms: f64,
    throughput_mbps: f64,
    throughput_gbps: f64,
    compression_ratio: Option<f64>,
}

impl BenchmarkResult {
    fn to_csv_header() -> String {
        "test_name,data_type,data_size_mb,operation,threads,duration_ms,throughput_mbps,throughput_gbps,compression_ratio\n".to_string()
    }

    fn to_csv_row(&self) -> String {
        format!(
            "{},{},{:.2},{},{},{:.2},{:.2},{:.4},{}\n",
            self.test_name,
            self.data_type,
            self.data_size_mb,
            self.operation,
            self.threads,
            self.duration_ms,
            self.throughput_mbps,
            self.throughput_gbps,
            self.compression_ratio.map(|r| format!("{:.2}%", r)).unwrap_or_else(|| "N/A".to_string())
        )
    }
}

fn benchmark_compression(
    name: &str,
    data_type: &str,
    data: &[u8],
    threads: usize,
    level: i32,
) -> BenchmarkResult {
    let config = CompressionConfig::new(level, threads);

    let start = Instant::now();
    let compressed = compress(data, &config).unwrap();
    let duration = start.elapsed();

    let data_size_mb = data.len() as f64 / (1024.0 * 1024.0);
    let duration_ms = duration.as_secs_f64() * 1000.0;
    let throughput_mbps = data_size_mb / duration.as_secs_f64();
    let throughput_gbps = throughput_mbps / 1024.0;
    let compression_ratio = (compressed.len() as f64 / data.len() as f64) * 100.0;

    BenchmarkResult {
        test_name: name.to_string(),
        data_type: data_type.to_string(),
        data_size_mb,
        operation: "compression".to_string(),
        threads,
        duration_ms,
        throughput_mbps,
        throughput_gbps,
        compression_ratio: Some(compression_ratio),
    }
}

fn benchmark_decompression(
    name: &str,
    data_type: &str,
    original_size: usize,
    compressed: &[u8],
    threads: usize,
) -> BenchmarkResult {
    let start = Instant::now();
    let _decompressed = decompress(compressed, threads).unwrap();
    let duration = start.elapsed();

    let data_size_mb = original_size as f64 / (1024.0 * 1024.0);
    let duration_ms = duration.as_secs_f64() * 1000.0;
    let throughput_mbps = data_size_mb / duration.as_secs_f64();
    let throughput_gbps = throughput_mbps / 1024.0;

    BenchmarkResult {
        test_name: name.to_string(),
        data_type: data_type.to_string(),
        data_size_mb,
        operation: "decompression".to_string(),
        threads,
        duration_ms,
        throughput_mbps,
        throughput_gbps,
        compression_ratio: None,
    }
}

fn run_throughput_benchmarks() -> Vec<BenchmarkResult> {
    println!("=== Running Throughput Benchmarks ===\n");
    let mut results = Vec::new();

    // 1 GB datasets
    let size_1gb = 1024 * 1024 * 1024;

    let test_cases = vec![
        ("Random Uncompressible", "random", generate_random_data(size_1gb)),
        ("Compressible Text", "text", generate_compressible_text(size_1gb)),
        ("Source Code", "source_code", generate_source_code(size_1gb)),
        ("Highly Compressible (Zeros)", "zeros", generate_zeros(size_1gb)),
    ];

    let threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(8);

    for (name, data_type, data) in test_cases {
        println!("Testing: {} (1 GB)", name);

        // Compression benchmark
        print!("  Compressing... ");
        std::io::stdout().flush().unwrap();
        let comp_result = benchmark_compression(
            &format!("{}_1GB", name),
            data_type,
            &data,
            threads,
            8,
        );
        println!("{:.2} GB/s", comp_result.throughput_gbps);
        results.push(comp_result.clone());

        // Pre-compress for decompression test
        let config = CompressionConfig::new(8, threads);
        let compressed = compress(&data, &config).unwrap();

        // Decompression benchmark
        print!("  Decompressing... ");
        std::io::stdout().flush().unwrap();
        let decomp_result = benchmark_decompression(
            &format!("{}_1GB", name),
            data_type,
            data.len(),
            &compressed,
            threads,
        );
        println!("{:.2} GB/s", decomp_result.throughput_gbps);
        results.push(decomp_result);

        println!();
    }

    results
}

fn run_scaling_benchmarks() -> Vec<BenchmarkResult> {
    println!("=== Running Multi-core Scaling Benchmarks ===\n");
    let mut results = Vec::new();

    // Use 1 GB of compressible text
    let size = 1024 * 1024 * 1024;
    let data = generate_compressible_text(size);

    for threads in [1, 2, 4, 8, 16].iter() {
        println!("Testing with {} core(s)", threads);

        // Compression
        print!("  Compression: ");
        std::io::stdout().flush().unwrap();
        let comp_result = benchmark_compression(
            &format!("Scaling_{}_cores", threads),
            "text",
            &data,
            *threads,
            8,
        );
        println!("{:.2} GB/s ({:.2} MB/s)", comp_result.throughput_gbps, comp_result.throughput_mbps);
        results.push(comp_result.clone());

        // Pre-compress for decompression
        let config = CompressionConfig::new(8, *threads);
        let compressed = compress(&data, &config).unwrap();

        // Decompression
        print!("  Decompression: ");
        std::io::stdout().flush().unwrap();
        let decomp_result = benchmark_decompression(
            &format!("Scaling_{}_cores", threads),
            "text",
            data.len(),
            &compressed,
            *threads,
        );
        println!("{:.2} GB/s ({:.2} MB/s)", decomp_result.throughput_gbps, decomp_result.throughput_mbps);
        results.push(decomp_result);

        println!();
    }

    results
}

fn run_compression_ratio_tests() -> Vec<BenchmarkResult> {
    println!("=== Running Compression Ratio Tests ===\n");
    let mut results = Vec::new();

    // 100 MB datasets for faster testing
    let size = 100 * 1024 * 1024;

    let test_cases = vec![
        ("Random", "random", generate_random_data(size)),
        ("Text", "text", generate_compressible_text(size)),
        ("Source Code", "source_code", generate_source_code(size)),
        ("Zeros", "zeros", generate_zeros(size)),
    ];

    let threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(8);

    for (name, data_type, data) in test_cases {
        let result = benchmark_compression(
            &format!("CompressionRatio_{}", name),
            data_type,
            &data,
            threads,
            8,
        );

        println!("{}: {:.2}% compression ratio",
                 name,
                 result.compression_ratio.unwrap());

        results.push(result);
    }

    println!();
    results
}

fn save_results_to_csv(results: &[BenchmarkResult], filename: &str) -> std::io::Result<()> {
    create_dir_all("benchmark_results")?;

    let path = Path::new("benchmark_results").join(filename);
    let mut file = File::create(&path)?;

    file.write_all(BenchmarkResult::to_csv_header().as_bytes())?;

    for result in results {
        file.write_all(result.to_csv_row().as_bytes())?;
    }

    println!("Results saved to: {:?}", path);
    Ok(())
}

fn generate_summary_report(results: &[BenchmarkResult]) {
    create_dir_all("benchmark_results").unwrap();

    let path = Path::new("benchmark_results").join("PERFORMANCE_REPORT.txt");
    let mut file = File::create(&path).unwrap();

    writeln!(file, "GLifzip Performance Benchmark Report").unwrap();
    writeln!(file, "=====================================").unwrap();
    writeln!(file, "Generated: {}", chrono::Local::now()).unwrap();
    writeln!(file, "").unwrap();

    // Throughput summary
    writeln!(file, "THROUGHPUT BENCHMARKS (1 GB datasets)").unwrap();
    writeln!(file, "--------------------------------------").unwrap();

    for result in results.iter().filter(|r| r.test_name.contains("1GB")) {
        writeln!(file, "{} - {}:", result.data_type, result.operation).unwrap();
        writeln!(file, "  Throughput: {:.2} GB/s ({:.2} MB/s)",
                 result.throughput_gbps, result.throughput_mbps).unwrap();
        writeln!(file, "  Duration: {:.2} ms", result.duration_ms).unwrap();
        if let Some(ratio) = result.compression_ratio {
            writeln!(file, "  Compression Ratio: {:.2}%", ratio).unwrap();
        }
        writeln!(file, "").unwrap();
    }

    // Scaling summary
    writeln!(file, "MULTI-CORE SCALING BENCHMARKS").unwrap();
    writeln!(file, "---------------------------------").unwrap();

    for threads in [1, 2, 4, 8, 16].iter() {
        let comp = results.iter()
            .find(|r| r.test_name == format!("Scaling_{}_cores", threads) && r.operation == "compression");
        let decomp = results.iter()
            .find(|r| r.test_name == format!("Scaling_{}_cores", threads) && r.operation == "decompression");

        if let (Some(comp), Some(decomp)) = (comp, decomp) {
            writeln!(file, "{} core(s):", threads).unwrap();
            writeln!(file, "  Compression: {:.2} GB/s", comp.throughput_gbps).unwrap();
            writeln!(file, "  Decompression: {:.2} GB/s", decomp.throughput_gbps).unwrap();
            writeln!(file, "").unwrap();
        }
    }

    // Compression ratios
    writeln!(file, "COMPRESSION RATIOS BY DATA TYPE").unwrap();
    writeln!(file, "--------------------------------").unwrap();

    for result in results.iter().filter(|r| r.test_name.contains("CompressionRatio")) {
        if let Some(ratio) = result.compression_ratio {
            writeln!(file, "{}: {:.2}%", result.data_type, ratio).unwrap();
        }
    }

    writeln!(file, "").unwrap();
    writeln!(file, "Report saved to: {:?}", path).unwrap();

    println!("\nPerformance report generated: {:?}", path);
}

fn main() {
    println!("╔══════════════════════════════════════════════════════╗");
    println!("║  GLifzip Comprehensive Performance Benchmark Suite  ║");
    println!("╚══════════════════════════════════════════════════════╝");
    println!();

    let mut all_results = Vec::new();

    // Run all benchmark suites
    all_results.extend(run_throughput_benchmarks());
    all_results.extend(run_scaling_benchmarks());
    all_results.extend(run_compression_ratio_tests());

    // Save results
    save_results_to_csv(&all_results, "benchmark_results.csv").unwrap();

    // Generate report
    generate_summary_report(&all_results);

    println!("\n✓ All benchmarks complete!");
    println!("  Results: benchmark_results/benchmark_results.csv");
    println!("  Report:  benchmark_results/PERFORMANCE_REPORT.txt");
}
