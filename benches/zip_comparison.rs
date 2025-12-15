/// ZIP Baseline Comparison Benchmark
///
/// Compares GLifzip performance against standard ZIP compression
/// (using the `zip` crate which is the Rust equivalent of Windows/macOS ZIP)

use std::fs::{File, create_dir_all};
use std::io::Write as IoWrite;
use std::time::Instant;
use std::path::Path;
use glifzip::{compress as glifzip_compress, decompress as glifzip_decompress, CompressionConfig};

// Data generation (same as performance_suite)
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
        b"fn main() {\n    println!(\"Hello, world!\");\n}\n\n".as_slice(),
        b"pub struct MyStruct {\n    field1: u32,\n    field2: String,\n}\n\n".as_slice(),
        b"impl MyTrait for MyStruct {\n    fn method(&self) -> bool {\n        true\n    }\n}\n\n".as_slice(),
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

#[derive(Debug)]
struct ComparisonResult {
    data_type: String,
    data_size_mb: f64,
    glifzip_compress_ms: f64,
    glifzip_decompress_ms: f64,
    glifzip_compress_gbps: f64,
    glifzip_decompress_gbps: f64,
    glifzip_ratio: f64,
    zip_compress_ms: f64,
    zip_decompress_ms: f64,
    zip_compress_mbps: f64,
    zip_decompress_mbps: f64,
    zip_ratio: f64,
    speedup_compression: f64,
    speedup_decompression: f64,
}

impl ComparisonResult {
    fn to_csv_header() -> String {
        "data_type,data_size_mb,\
         glifzip_compress_ms,glifzip_decompress_ms,\
         glifzip_compress_gbps,glifzip_decompress_gbps,glifzip_ratio,\
         zip_compress_ms,zip_decompress_ms,\
         zip_compress_mbps,zip_decompress_mbps,zip_ratio,\
         speedup_compression,speedup_decompression\n".to_string()
    }

    fn to_csv_row(&self) -> String {
        format!(
            "{},{:.2},{:.2},{:.2},{:.4},{:.4},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2}x,{:.2}x\n",
            self.data_type,
            self.data_size_mb,
            self.glifzip_compress_ms,
            self.glifzip_decompress_ms,
            self.glifzip_compress_gbps,
            self.glifzip_decompress_gbps,
            self.glifzip_ratio,
            self.zip_compress_ms,
            self.zip_decompress_ms,
            self.zip_compress_mbps,
            self.zip_decompress_mbps,
            self.zip_ratio,
            self.speedup_compression,
            self.speedup_decompression,
        )
    }
}

fn benchmark_glifzip(data: &[u8]) -> (f64, f64, f64, f64, f64) {
    let threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(8);

    let config = CompressionConfig::new(8, threads);

    // Compression
    let start = Instant::now();
    let compressed = glifzip_compress(data, &config).unwrap();
    let compress_time = start.elapsed().as_secs_f64();

    // Decompression
    let start = Instant::now();
    let _decompressed = glifzip_decompress(&compressed, threads).unwrap();
    let decompress_time = start.elapsed().as_secs_f64();

    let data_size_gb = data.len() as f64 / (1024.0 * 1024.0 * 1024.0);
    let compress_gbps = data_size_gb / compress_time;
    let decompress_gbps = data_size_gb / decompress_time;
    let ratio = (compressed.len() as f64 / data.len() as f64) * 100.0;

    (
        compress_time * 1000.0,
        decompress_time * 1000.0,
        compress_gbps,
        decompress_gbps,
        ratio,
    )
}

fn benchmark_zip_flate2(data: &[u8]) -> (f64, f64, f64, f64, f64) {
    use flate2::Compression;
    use flate2::write::ZlibEncoder;
    use flate2::read::ZlibDecoder;
    use std::io::Read;

    // Compression
    let start = Instant::now();
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(6));
    encoder.write_all(data).unwrap();
    let compressed = encoder.finish().unwrap();
    let compress_time = start.elapsed().as_secs_f64();

    // Decompression
    let start = Instant::now();
    let mut decoder = ZlibDecoder::new(&compressed[..]);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed).unwrap();
    let decompress_time = start.elapsed().as_secs_f64();

    let data_size_mb = data.len() as f64 / (1024.0 * 1024.0);
    let compress_mbps = data_size_mb / compress_time;
    let decompress_mbps = data_size_mb / decompress_time;
    let ratio = (compressed.len() as f64 / data.len() as f64) * 100.0;

    (
        compress_time * 1000.0,
        decompress_time * 1000.0,
        compress_mbps,
        decompress_mbps,
        ratio,
    )
}

fn run_comparison(data_type: &str, data: &[u8]) -> ComparisonResult {
    println!("Testing: {} ({:.2} MB)", data_type, data.len() as f64 / (1024.0 * 1024.0));

    print!("  GLifzip... ");
    std::io::stdout().flush().unwrap();
    let (glif_comp_ms, glif_decomp_ms, glif_comp_gbps, glif_decomp_gbps, glif_ratio) =
        benchmark_glifzip(data);
    println!("✓ (comp: {:.2} GB/s, decomp: {:.2} GB/s)", glif_comp_gbps, glif_decomp_gbps);

    print!("  ZIP (flate2)... ");
    std::io::stdout().flush().unwrap();
    let (zip_comp_ms, zip_decomp_ms, zip_comp_mbps, zip_decomp_mbps, zip_ratio) =
        benchmark_zip_flate2(data);
    println!("✓ (comp: {:.2} MB/s, decomp: {:.2} MB/s)", zip_comp_mbps, zip_decomp_mbps);

    let speedup_comp = zip_comp_ms / glif_comp_ms;
    let speedup_decomp = zip_decomp_ms / glif_decomp_ms;

    println!("  Speedup: {:.2}x compression, {:.2}x decompression\n", speedup_comp, speedup_decomp);

    ComparisonResult {
        data_type: data_type.to_string(),
        data_size_mb: data.len() as f64 / (1024.0 * 1024.0),
        glifzip_compress_ms: glif_comp_ms,
        glifzip_decompress_ms: glif_decomp_ms,
        glifzip_compress_gbps: glif_comp_gbps,
        glifzip_decompress_gbps: glif_decomp_gbps,
        glifzip_ratio: glif_ratio,
        zip_compress_ms: zip_comp_ms,
        zip_decompress_ms: zip_decomp_ms,
        zip_compress_mbps: zip_comp_mbps,
        zip_decompress_mbps: zip_decomp_mbps,
        zip_ratio: zip_ratio,
        speedup_compression: speedup_comp,
        speedup_decompression: speedup_decomp,
    }
}

fn save_comparison_csv(results: &[ComparisonResult]) {
    create_dir_all("benchmark_results").unwrap();

    let path = Path::new("benchmark_results").join("zip_comparison.csv");
    let mut file = File::create(&path).unwrap();

    file.write_all(ComparisonResult::to_csv_header().as_bytes()).unwrap();

    for result in results {
        file.write_all(result.to_csv_row().as_bytes()).unwrap();
    }

    println!("Comparison results saved to: {:?}", path);
}

fn generate_comparison_report(results: &[ComparisonResult]) {
    create_dir_all("benchmark_results").unwrap();

    let path = Path::new("benchmark_results").join("ZIP_COMPARISON_REPORT.txt");
    let mut file = File::create(&path).unwrap();

    writeln!(file, "GLifzip vs ZIP Baseline Comparison").unwrap();
    writeln!(file, "====================================").unwrap();
    writeln!(file, "Generated: {}", chrono::Local::now()).unwrap();
    writeln!(file, "").unwrap();

    for result in results {
        writeln!(file, "Data Type: {}", result.data_type).unwrap();
        writeln!(file, "Data Size: {:.2} MB", result.data_size_mb).unwrap();
        writeln!(file, "").unwrap();

        writeln!(file, "COMPRESSION:").unwrap();
        writeln!(file, "  GLifzip: {:.2} ms ({:.2} GB/s) - Ratio: {:.2}%",
                 result.glifzip_compress_ms, result.glifzip_compress_gbps, result.glifzip_ratio).unwrap();
        writeln!(file, "  ZIP:     {:.2} ms ({:.2} MB/s) - Ratio: {:.2}%",
                 result.zip_compress_ms, result.zip_compress_mbps, result.zip_ratio).unwrap();
        writeln!(file, "  Speedup: {:.2}x FASTER", result.speedup_compression).unwrap();
        writeln!(file, "").unwrap();

        writeln!(file, "DECOMPRESSION:").unwrap();
        writeln!(file, "  GLifzip: {:.2} ms ({:.2} GB/s)",
                 result.glifzip_decompress_ms, result.glifzip_decompress_gbps).unwrap();
        writeln!(file, "  ZIP:     {:.2} ms ({:.2} MB/s)",
                 result.zip_decompress_ms, result.zip_decompress_mbps).unwrap();
        writeln!(file, "  Speedup: {:.2}x FASTER", result.speedup_decompression).unwrap();
        writeln!(file, "").unwrap();
        writeln!(file, "---").unwrap();
        writeln!(file, "").unwrap();
    }

    println!("Comparison report generated: {:?}", path);
}

fn main() {
    println!("╔═══════════════════════════════════════════════╗");
    println!("║  GLifzip vs ZIP Baseline Comparison Suite   ║");
    println!("╚═══════════════════════════════════════════════╝");
    println!();

    let mut results = Vec::new();

    // Test with 100 MB datasets for faster benchmarking
    let size = 100 * 1024 * 1024;

    let test_cases = vec![
        ("Random", generate_random_data(size)),
        ("Compressible_Text", generate_compressible_text(size)),
        ("Source_Code", generate_source_code(size)),
    ];

    for (name, data) in test_cases {
        results.push(run_comparison(name, &data));
    }

    save_comparison_csv(&results);
    generate_comparison_report(&results);

    println!("\n✓ ZIP comparison complete!");
    println!("  CSV:    benchmark_results/zip_comparison.csv");
    println!("  Report: benchmark_results/ZIP_COMPARISON_REPORT.txt");
}
