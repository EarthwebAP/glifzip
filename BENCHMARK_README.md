# GLifzip Performance Benchmark Suite

## Quick Start

```bash
# Run complete benchmark suite (15-30 minutes)
./run_benchmarks.sh --release

# View results
cat benchmark_results/PERFORMANCE_REPORT.txt
cat benchmark_results/ZIP_COMPARISON_REPORT.txt
```

## What Gets Measured

### 1. Throughput Tests (1 GB Datasets)
- **Random data** - Worst case (uncompressible)
- **Text data** - Best case (highly compressible)
- **Source code** - Realistic scenario
- **Zeros** - Edge case

**Metrics:**
- Compression throughput (GB/s)
- Decompression throughput (GB/s)
- Compression ratios

### 2. Multi-core Scaling
Tests with 1, 2, 4, 8, and 16 cores to verify:
- Linear scaling
- Efficiency percentage
- Throughput per core

### 3. ZIP Baseline Comparison
Compares against standard ZIP (flate2/zlib):
- Compression speedup (X times faster)
- Decompression speedup (X times faster)
- Compression ratio comparison

### 4. Statistical Analysis
Criterion benchmarks provide:
- Mean, median, standard deviation
- Confidence intervals
- Outlier detection
- Regression tracking

## Output Files

```
benchmark_results/
├── benchmark_results.csv          # Main results (importable to Excel)
├── zip_comparison.csv             # ZIP comparison data
├── PERFORMANCE_REPORT.txt         # Human-readable summary
├── ZIP_COMPARISON_REPORT.txt      # ZIP comparison summary
├── throughput_comparison.png      # Bar chart: compress vs decompress
├── multicore_scaling.png          # Line chart: scaling efficiency
├── compression_ratios.png         # Bar chart: ratios by data type
├── zip_comparison.png             # Bar chart: GLifzip vs ZIP speedup
└── performance_dashboard.png      # Comprehensive 4-panel overview
```

## Individual Tests

### Performance Suite
```bash
# Full 1 GB test suite
cargo run --release --bin performance_suite

# Output: CSV + text report
```

### ZIP Comparison
```bash
# Compare against ZIP (100 MB tests)
cargo run --release --bin zip_comparison

# Output: CSV + text report
```

### Criterion Benchmarks
```bash
# Statistical micro-benchmarks
cargo bench --bench comprehensive_bench

# View HTML report
firefox target/criterion/*/report/index.html
```

### Visualizations
```bash
# Requires: pip3 install pandas matplotlib seaborn
python3 scripts/visualize_benchmarks.py

# Output: PNG graphs in benchmark_results/
```

## Performance Targets

| Metric | Target | Notes |
|--------|--------|-------|
| Compression (1 core) | ≥1.0 GB/s | Per-core throughput |
| Decompression (1 core) | ≥2.0 GB/s | Per-core throughput |
| Scaling efficiency | ≥80% | Up to 8 cores |
| ZIP speedup (compression) | 5-20× | Data-dependent |
| ZIP speedup (decompression) | 10-50× | LZ4 advantage |

## Expected Results (Reference Hardware)

**Test System:** AMD Ryzen 9 / Intel Core i9 (8+ cores)

### Throughput
- Compression: 1-2 GB/s per core
- Decompression: 2-4 GB/s per core
- 8-core total: 8-16 GB/s compression, 16-32 GB/s decompression

### Compression Ratios
- Random data: ~100% (no compression)
- Text data: 15-30%
- Source code: 30-50%
- Zeros: <1%

### ZIP Comparison
- Compression: 10-20× faster
- Decompression: 20-50× faster

**Note:** WSL/VM environments will show lower absolute numbers due to virtualization overhead, but scaling ratios should be similar.

## Troubleshooting

### "Out of memory" errors
Reduce dataset size in source files:
```rust
let size_1gb = 100 * 1024 * 1024;  // 100 MB instead
```

### Slow benchmarks
Reduce sample size:
```rust
group.sample_size(5);  // Instead of 10
```

### Python visualization fails
```bash
pip3 install pandas matplotlib seaborn numpy
```

### Inconsistent results
- Close background applications
- Disable CPU frequency scaling
- Run on native hardware (not WSL/VM)
- Run multiple times and average

## Customization

### Change Dataset Sizes
Edit `benches/performance_suite.rs`:
```rust
let size_1gb = 512 * 1024 * 1024;  // 512 MB
```

### Change Thread Counts
```rust
for threads in [1, 2, 4, 8, 16, 32].iter() {  // Add 32
```

### Change Compression Levels
```rust
let config = CompressionConfig::new(3, threads);  // Level 3
```

## CI/CD Integration

```yaml
# .github/workflows/benchmark.yml
- name: Run benchmarks
  run: |
    ./run_benchmarks.sh --release

- name: Upload results
  uses: actions/upload-artifact@v2
  with:
    name: benchmark-results
    path: benchmark_results/
```

## Data Analysis

### Import CSV to Python
```python
import pandas as pd
df = pd.read_csv('benchmark_results/benchmark_results.csv')
print(df.describe())
```

### Import CSV to Excel
1. Open Excel
2. Data → From Text/CSV
3. Select `benchmark_results.csv`
4. Create pivot tables and charts

### Compare Baselines
```bash
# Save current as baseline
cargo bench -- --save-baseline v1.0

# Later, compare
cargo bench -- --baseline v1.0
```

## Hardware Recommendations

### Optimal Testing
- Native Linux/macOS (not virtualized)
- Modern CPU ≥8 cores
- ≥16 GB RAM
- SSD storage
- No background processes

### Minimum Requirements
- 4 cores
- 8 GB RAM
- Can reduce dataset sizes

## Support

For issues:
1. Check BENCHMARKING_GUIDE.md for detailed info
2. Review existing results in `benchmark_results/`
3. Open issue on GitHub with:
   - Hardware specs
   - OS and version
   - Error messages
   - Benchmark output

---

**Last Updated:** December 2025
**GLifzip Version:** 1.0.0
**Benchmark Suite Version:** 1.0.0
