# GLifzip Performance Benchmarking Guide

## Overview

This guide describes the comprehensive performance benchmarking suite for GLifzip, designed to measure and verify:

- **Compression throughput** (target: ≥1 GB/s per core)
- **Decompression throughput** (target: ≥2 GB/s per core)
- **Multi-core scaling** (linear scaling verification)
- **ZIP baseline comparison** (10-100× performance improvement)
- **Compression ratios** by data type

## Quick Start

### Run All Benchmarks

```bash
cd /home/daveswo/glifzip
./run_benchmarks.sh --release
```

This will:
1. Build the benchmark suite
2. Run performance benchmarks (1 GB datasets)
3. Compare against ZIP baseline
4. Generate Criterion micro-benchmarks
5. Create visualization graphs

**Estimated time:** 15-30 minutes depending on hardware

### View Results

```bash
# Text reports
cat benchmark_results/PERFORMANCE_REPORT.txt
cat benchmark_results/ZIP_COMPARISON_REPORT.txt

# CSV data for analysis
less benchmark_results/benchmark_results.csv
less benchmark_results/zip_comparison.csv

# Visualizations (PNG)
xdg-open benchmark_results/performance_dashboard.png

# Criterion HTML reports
firefox target/criterion/*/report/index.html
```

## Benchmark Components

### 1. Performance Suite

**Location:** `benches/performance_suite.rs`

**Purpose:** Measures real-world throughput with 1 GB datasets

**Test Cases:**
- Random uncompressible data (worst case)
- Compressible text data (best case)
- Source code (realistic scenario)
- Highly compressible zeros (edge case)

**Operations:**
- Compression throughput (GB/s)
- Decompression throughput (GB/s)
- Multi-core scaling (1, 2, 4, 8, 16 cores)
- Compression ratios

**Run individually:**
```bash
cargo run --release --bin performance_suite
```

**Output:**
- `benchmark_results/benchmark_results.csv` - Raw data
- `benchmark_results/PERFORMANCE_REPORT.txt` - Summary report

### 2. ZIP Comparison Suite

**Location:** `benches/zip_comparison.rs`

**Purpose:** Compares GLifzip against standard ZIP (flate2/zlib)

**Test Cases:**
- Random data (100 MB)
- Compressible text (100 MB)
- Source code (100 MB)

**Metrics:**
- Compression speed comparison
- Decompression speed comparison
- Speedup factors (how many times faster)
- Compression ratio comparison

**Run individually:**
```bash
cargo run --release --bin zip_comparison
```

**Output:**
- `benchmark_results/zip_comparison.csv` - Raw comparison data
- `benchmark_results/ZIP_COMPARISON_REPORT.txt` - Summary report

### 3. Criterion Micro-benchmarks

**Location:** `benches/comprehensive_bench.rs`

**Purpose:** Statistical analysis with confidence intervals

**Features:**
- Multiple iterations for accuracy
- Statistical analysis
- Outlier detection
- Regression tracking
- HTML reports with graphs

**Run individually:**
```bash
cargo bench --bench comprehensive_bench
```

**Output:**
- `target/criterion/*/report/index.html` - Interactive HTML reports

### 4. Visualization Generator

**Location:** `scripts/visualize_benchmarks.py`

**Purpose:** Generate performance graphs from CSV data

**Requirements:**
```bash
pip3 install pandas matplotlib seaborn
```

**Graphs Generated:**
1. **Throughput Comparison** - Compression vs decompression by data type
2. **Multi-core Scaling** - Absolute throughput and efficiency
3. **Compression Ratios** - By data type
4. **ZIP Comparison** - Speedup and absolute throughput
5. **Performance Dashboard** - Comprehensive overview

**Run individually:**
```bash
python3 scripts/visualize_benchmarks.py
```

**Output:**
- `benchmark_results/*.png` - High-resolution graphs (300 DPI)

## Performance Targets

### Throughput Targets (Per Core)

| Cores | Compression Target | Decompression Target |
|-------|-------------------|----------------------|
| 1     | ≥1.0 GB/s        | ≥2.0 GB/s           |
| 2     | ≥2.0 GB/s        | ≥4.0 GB/s           |
| 4     | ≥4.0 GB/s        | ≥8.0 GB/s           |
| 8     | ≥8.0 GB/s        | ≥16.0 GB/s          |
| 16    | ≥16.0 GB/s       | ≥32.0 GB/s          |

### Scaling Efficiency

- **Target:** ≥80% linear scaling up to 8 cores
- **Acceptable:** ≥60% linear scaling up to 16 cores

### ZIP Comparison

- **Target:** 10-100× faster than standard ZIP
- **Compression:** 5-20× speedup (varies by data type)
- **Decompression:** 10-50× speedup (LZ4 advantage)

## Data Types and Test Scenarios

### 1. Random Uncompressible Data

**Purpose:** Worst-case scenario testing

**Characteristics:**
- Pseudo-random data
- Minimal compression possible
- Tests raw throughput
- Expected ratio: ~100% (no compression)

### 2. Compressible Text Data

**Purpose:** Best-case scenario

**Characteristics:**
- Repeated text patterns
- High redundancy
- Excellent compression
- Expected ratio: 10-30%

### 3. Source Code

**Purpose:** Realistic development scenario

**Characteristics:**
- Mix of text and structure
- Moderate redundancy
- Real-world representation
- Expected ratio: 30-50%

### 4. Highly Compressible (Zeros)

**Purpose:** Edge case testing

**Characteristics:**
- All zeros
- Maximum compression
- Tests compression limits
- Expected ratio: <1%

## Understanding Results

### CSV Format

**benchmark_results.csv:**
```csv
test_name,data_type,data_size_mb,operation,threads,duration_ms,throughput_mbps,throughput_gbps,compression_ratio
Random_1GB,random,1024.00,compression,8,5234.56,195.67,0.191,99.8%
```

**zip_comparison.csv:**
```csv
data_type,data_size_mb,glifzip_compress_ms,glifzip_decompress_ms,glifzip_compress_gbps,glifzip_decompress_gbps,glifzip_ratio,zip_compress_ms,zip_decompress_ms,zip_compress_mbps,zip_decompress_mbps,zip_ratio,speedup_compression,speedup_decompression
```

### Interpreting Throughput

**Good Performance:**
- Compression: ≥1 GB/s on single core
- Decompression: ≥2 GB/s on single core
- Multi-core: Near-linear scaling

**Factors Affecting Performance:**
- CPU speed and architecture
- Memory bandwidth
- Data characteristics
- Virtualization overhead (WSL, VM)
- Background processes

### Compression Ratio Guide

- **<30%** - Excellent compression
- **30-60%** - Good compression
- **60-80%** - Moderate compression
- **>80%** - Poor compression (incompressible data)

## Advanced Usage

### Custom Benchmark Parameters

Edit `benches/performance_suite.rs` to customize:

```rust
// Change dataset size
let size_1gb = 1024 * 1024 * 1024;  // Modify this

// Change thread counts tested
for threads in [1, 2, 4, 8, 16, 32].iter() {  // Add more

// Change compression level
let config = CompressionConfig::new(8, threads);  // Change level
```

### Running Individual Benchmarks

```bash
# Only compression benchmarks
cargo bench --bench compression_bench

# Only decompression benchmarks
cargo bench --bench decompression_bench

# Specific test
cargo bench --bench comprehensive_bench -- compression_throughput
```

### Baseline Comparison (Regression Testing)

```bash
# Save baseline
cargo bench --bench comprehensive_bench -- --save-baseline main

# Compare against baseline
cargo bench --bench comprehensive_bench -- --baseline main
```

### Export Criterion Data

Criterion saves data in JSON format:
```bash
# View raw data
cat target/criterion/compression_throughput/*/estimates.json

# Compare baselines
cargo bench -- --baseline old --load-baseline new
```

## Visualization Customization

### Custom Graphs

Edit `scripts/visualize_benchmarks.py`:

```python
# Change color scheme
colors = ['#2E86AB', '#A23B72', '#F18F01', '#C73E1D', '#6A994E']

# Change figure size
fig, ax = plt.subplots(figsize=(16, 8))  # Larger

# Change DPI (resolution)
plt.savefig('output.png', dpi=600)  # Higher quality
```

### Export to Different Formats

```python
# Add to visualize_benchmarks.py
plt.savefig('output.pdf')  # PDF for papers
plt.savefig('output.svg')  # SVG for editing
```

## Hardware Considerations

### Optimal Testing Environment

**Recommended:**
- Native Linux/macOS (not virtualized)
- Modern CPU with ≥8 cores
- ≥16 GB RAM
- SSD storage
- No background processes

**Avoid:**
- WSL/WSL2 (virtualization overhead)
- VMs (performance penalty)
- Active development (file watchers, etc.)
- Power saving mode

### CPU Frequency Scaling

For consistent results:

```bash
# Linux: Disable frequency scaling
sudo cpupower frequency-set --governor performance

# Restore after benchmarking
sudo cpupower frequency-set --governor powersave
```

### Memory Considerations

1 GB datasets require:
- ~1 GB for original data
- ~1 GB for compressed data (worst case)
- ~1 GB for decompression buffer
- Total: ~3-4 GB per thread

**Recommended RAM:** 16 GB minimum for 16-thread tests

## Troubleshooting

### Out of Memory Errors

**Reduce dataset size:**
```rust
let size = 100 * 1024 * 1024;  // 100 MB instead of 1 GB
```

### Slow Benchmarks

**Reduce sample size:**
```rust
group.sample_size(5);  // Instead of 10
```

### Python Visualization Errors

**Install dependencies:**
```bash
pip3 install pandas matplotlib seaborn numpy
```

**Fix matplotlib backend:**
```bash
export MPLBACKEND=Agg  # Use non-interactive backend
```

### Criterion Timeout

**Increase measurement time:**
```rust
group.measurement_time(std::time::Duration::from_secs(30));
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Performance Benchmarks

on: [push, pull_request]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run benchmarks
        run: |
          cargo bench --bench comprehensive_bench -- --save-baseline main
      - name: Upload results
        uses: actions/upload-artifact@v2
        with:
          name: criterion-results
          path: target/criterion/
```

### Regression Detection

```bash
# In CI pipeline
cargo bench -- --baseline main > results.txt
if grep -q "Performance regressed" results.txt; then
    exit 1
fi
```

## Best Practices

### 1. Consistent Environment

- Same hardware for comparisons
- Minimal background processes
- Fixed CPU frequency
- Same OS/kernel version

### 2. Multiple Runs

Run benchmarks 3-5 times and average results:
```bash
for i in {1..5}; do
    ./run_benchmarks.sh --release
    mv benchmark_results benchmark_results_run_$i
done
```

### 3. Statistical Significance

Use Criterion's confidence intervals:
- Look for non-overlapping intervals
- Check for consistent trends
- Require multiple confirmations

### 4. Documentation

Document any changes:
- Hardware specifications
- OS and kernel version
- GLifzip version
- Environmental factors

## Performance Analysis Checklist

- [ ] Run on native hardware (not VM/WSL)
- [ ] Close all unnecessary applications
- [ ] Disable CPU frequency scaling
- [ ] Run benchmarks 3+ times
- [ ] Check for consistent results (±5%)
- [ ] Compare against baseline
- [ ] Document environment details
- [ ] Generate visualizations
- [ ] Review Criterion HTML reports
- [ ] Archive results with version tag

## Output Files Reference

```
benchmark_results/
├── benchmark_results.csv          # Main performance data
├── zip_comparison.csv             # ZIP comparison data
├── PERFORMANCE_REPORT.txt         # Human-readable summary
├── ZIP_COMPARISON_REPORT.txt      # ZIP comparison summary
├── throughput_comparison.png      # Throughput graph
├── multicore_scaling.png          # Scaling analysis
├── compression_ratios.png         # Ratio comparison
├── zip_comparison.png             # ZIP speedup graphs
└── performance_dashboard.png      # Comprehensive overview

target/criterion/
├── compression_throughput/
│   └── report/index.html          # Criterion HTML report
├── decompression_throughput/
│   └── report/index.html
├── compression_scaling/
│   └── report/index.html
└── ...
```

## Support and Questions

For issues or questions:
- Check existing benchmark results in `benchmark_results/`
- Review Criterion documentation: https://bheisler.github.io/criterion.rs/
- Consult the main README.md
- Open an issue on the GlyphOS repository

---

**Last Updated:** December 2025
**GLifzip Version:** 1.0.0
**Benchmark Suite Version:** 1.0.0
