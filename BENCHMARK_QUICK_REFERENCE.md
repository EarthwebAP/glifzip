# GLifzip Benchmark Quick Reference Card

## One-Command Run

```bash
./run_benchmarks.sh --release    # Full suite (15-30 min)
```

## Individual Components

| Command | Time | Purpose |
|---------|------|---------|
| `cargo run --release --bin performance_suite` | 10-20 min | Throughput + scaling |
| `cargo run --release --bin zip_comparison` | 5-10 min | ZIP comparison |
| `cargo bench --bench comprehensive_bench` | 15-30 min | Statistical analysis |
| `python3 scripts/visualize_benchmarks.py` | <1 min | Generate graphs |

## Output Files

```
benchmark_results/
├── benchmark_results.csv          # Import to Excel/Python
├── zip_comparison.csv             # ZIP comparison data
├── PERFORMANCE_REPORT.txt         # Read this first
├── ZIP_COMPARISON_REPORT.txt      # ZIP speedup summary
└── *.png (5 files)                # Graphs
```

## Performance Targets

| Metric | Target |
|--------|--------|
| Compression (1 core) | ≥1.0 GB/s |
| Decompression (1 core) | ≥2.0 GB/s |
| Scaling efficiency | ≥80% (8 cores) |
| ZIP speedup | 10-100× |

## Quick Check

```bash
# View summary
cat benchmark_results/PERFORMANCE_REPORT.txt

# Check if targets met
grep "GB/s" benchmark_results/PERFORMANCE_REPORT.txt

# View ZIP speedup
grep "Speedup" benchmark_results/ZIP_COMPARISON_REPORT.txt
```

## View Results

```bash
# Text reports
cat benchmark_results/PERFORMANCE_REPORT.txt
cat benchmark_results/ZIP_COMPARISON_REPORT.txt

# Graphs
xdg-open benchmark_results/performance_dashboard.png

# Criterion HTML
firefox target/criterion/*/report/index.html

# CSV in Excel
libreoffice benchmark_results/benchmark_results.csv
```

## Troubleshooting

| Problem | Solution |
|---------|----------|
| Out of memory | Edit bench files, reduce size to 100 MB |
| Python error | `pip3 install pandas matplotlib seaborn` |
| Slow tests | Reduce `sample_size(10)` to `(5)` |
| Low performance | Close apps, disable power saving |

## Documentation

- **BENCHMARK_README.md** - Quick start guide
- **BENCHMARKING_GUIDE.md** - Complete reference (3000+ words)
- **BENCHMARK_SUITE_SUMMARY.md** - Implementation details

## Expected Results (8-core CPU)

| Test | Expected |
|------|----------|
| Compression (8 cores) | 8-16 GB/s |
| Decompression (8 cores) | 16-32 GB/s |
| ZIP speedup (compression) | 10-20× |
| ZIP speedup (decompression) | 20-50× |
| Text compression ratio | 15-30% |
| Random data ratio | ~100% |

## Visualization Examples

```bash
# After running benchmarks
python3 scripts/visualize_benchmarks.py

# View dashboard
xdg-open benchmark_results/performance_dashboard.png
```

## CI/CD Snippet

```yaml
- run: ./run_benchmarks.sh --release
- uses: actions/upload-artifact@v2
  with:
    name: benchmarks
    path: benchmark_results/
```

## Data Analysis

### Python
```python
import pandas as pd
df = pd.read_csv('benchmark_results/benchmark_results.csv')
print(df[df['operation'] == 'compression']['throughput_gbps'].describe())
```

### Excel
1. Open `benchmark_results.csv`
2. Create pivot table
3. Chart compression vs decompression

## Regression Testing

```bash
# Save baseline
cargo bench -- --save-baseline main

# Later, compare
cargo bench -- --baseline main
```

---

**Quick Commands:**
- Full suite: `./run_benchmarks.sh --release`
- View results: `cat benchmark_results/PERFORMANCE_REPORT.txt`
- View graphs: `xdg-open benchmark_results/performance_dashboard.png`
