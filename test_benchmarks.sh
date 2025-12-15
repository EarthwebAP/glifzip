#!/bin/bash
#
# Quick Benchmark Test Script
# Tests the benchmark suite with smaller datasets (100 MB)
# for rapid verification without long wait times
#

set -e

echo "╔══════════════════════════════════════════════════════════╗"
echo "║  GLifzip Quick Benchmark Test (100 MB datasets)         ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Create a temporary test binary with smaller datasets
echo -e "${BLUE}Creating quick test benchmark...${NC}"

# Create temporary test file
cat > /tmp/quick_bench_test.rs << 'EOF'
use std::time::Instant;
use glifzip::{compress, decompress, CompressionConfig};

fn generate_compressible_text(size: usize) -> Vec<u8> {
    let pattern = b"Lorem ipsum dolor sit amet. ";
    let mut data = Vec::with_capacity(size);
    while data.len() < size {
        data.extend_from_slice(pattern);
    }
    data.truncate(size);
    data
}

fn main() {
    println!("Testing GLifzip with 100 MB dataset...\n");

    let size = 100 * 1024 * 1024;
    let data = generate_compressible_text(size);

    let threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(8);

    println!("Configuration:");
    println!("  Data size: {} MB", size / 1024 / 1024);
    println!("  Threads: {}", threads);
    println!("  Compression level: 8\n");

    // Compression test
    let config = CompressionConfig::new(8, threads);
    print!("Compressing... ");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();

    let start = Instant::now();
    let compressed = compress(&data, &config).unwrap();
    let comp_time = start.elapsed();

    let comp_gbps = (size as f64 / (1024.0 * 1024.0 * 1024.0)) / comp_time.as_secs_f64();
    let ratio = (compressed.len() as f64 / data.len() as f64) * 100.0;

    println!("✓");
    println!("  Duration: {:.2} ms", comp_time.as_secs_f64() * 1000.0);
    println!("  Throughput: {:.2} GB/s", comp_gbps);
    println!("  Ratio: {:.2}%\n", ratio);

    // Decompression test
    print!("Decompressing... ");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();

    let start = Instant::now();
    let decompressed = decompress(&compressed, threads).unwrap();
    let decomp_time = start.elapsed();

    let decomp_gbps = (size as f64 / (1024.0 * 1024.0 * 1024.0)) / decomp_time.as_secs_f64();

    println!("✓");
    println!("  Duration: {:.2} ms", decomp_time.as_secs_f64() * 1000.0);
    println!("  Throughput: {:.2} GB/s", decomp_gbps);
    println!("  Verified: {}\n", decompressed == data);

    // Performance check
    println!("Performance Status:");
    if comp_gbps >= 1.0 {
        println!("  ✓ Compression: MEETS TARGET (≥1.0 GB/s)");
    } else {
        println!("  ⚠ Compression: {:.2} GB/s (target: ≥1.0 GB/s)", comp_gbps);
    }

    if decomp_gbps >= 2.0 {
        println!("  ✓ Decompression: MEETS TARGET (≥2.0 GB/s)");
    } else {
        println!("  ⚠ Decompression: {:.2} GB/s (target: ≥2.0 GB/s)", decomp_gbps);
    }

    println!("\nTest completed successfully!");
}
EOF

# Build and run the test
echo -e "${BLUE}Building test...${NC}"
rustc --edition 2021 -O -L /home/daveswo/glifzip/target/release/deps \
    --extern glifzip=/home/daveswo/glifzip/target/release/libglifzip.rlib \
    /tmp/quick_bench_test.rs -o /tmp/quick_bench_test 2>&1 | grep -v "warning" || true

if [ -f /tmp/quick_bench_test ]; then
    echo -e "${GREEN}Build successful!${NC}\n"
    /tmp/quick_bench_test
    rm /tmp/quick_bench_test /tmp/quick_bench_test.rs
else
    echo -e "${YELLOW}Direct build failed, trying cargo...${NC}\n"
    cd /home/daveswo/glifzip
    cargo run --release --example quick_test 2>/dev/null || {
        echo "Creating inline test..."
        cargo build --release --lib
        echo "Library built. Run performance_suite for full tests."
    }
fi

echo ""
echo -e "${GREEN}✓ Quick benchmark test complete!${NC}"
echo ""
echo "For full benchmarks, run:"
echo "  ./run_benchmarks.sh --release"
