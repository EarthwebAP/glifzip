#!/bin/bash
#
# GLifzip Comprehensive Benchmark Suite Runner
#
# This script executes the full benchmark suite including:
# - Performance benchmarks (1 GB datasets)
# - Multi-core scaling tests
# - ZIP baseline comparison
# - Visualization generation
#

set -e

echo "╔══════════════════════════════════════════════════════════╗"
echo "║  GLifzip Comprehensive Performance Benchmark Suite      ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""

# Color codes
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if running in release mode
if [ "$1" != "--release" ] && [ "$1" != "-r" ]; then
    echo -e "${YELLOW}Warning: Running in debug mode. Use --release for accurate benchmarks.${NC}"
    echo ""
fi

# Create results directory
mkdir -p benchmark_results

echo -e "${BLUE}[1/5] Building benchmark suite...${NC}"
cargo build --release --bin performance_suite --bin zip_comparison
echo ""

echo -e "${BLUE}[2/5] Running performance benchmarks...${NC}"
echo "This will test 1 GB datasets and may take several minutes."
echo ""
cargo run --release --bin performance_suite
echo ""

echo -e "${BLUE}[3/5] Running ZIP baseline comparison...${NC}"
echo "Comparing GLifzip against standard ZIP compression."
echo ""
cargo run --release --bin zip_comparison
echo ""

echo -e "${BLUE}[4/5] Running Criterion benchmarks...${NC}"
echo "Detailed micro-benchmarks with statistical analysis."
echo ""
cargo bench --bench comprehensive_bench
echo ""

echo -e "${BLUE}[5/5] Generating visualizations...${NC}"
echo "Creating performance graphs and charts."
echo ""

# Check if Python and required packages are available
if command -v python3 &> /dev/null; then
    # Check if required packages are installed
    python3 -c "import pandas, matplotlib" 2>/dev/null
    if [ $? -eq 0 ]; then
        python3 scripts/visualize_benchmarks.py
    else
        echo -e "${YELLOW}Warning: Python packages not found. Install with:${NC}"
        echo "  pip3 install pandas matplotlib seaborn"
        echo ""
        echo "Skipping visualization generation."
    fi
else
    echo -e "${YELLOW}Warning: Python 3 not found. Skipping visualization generation.${NC}"
    echo "Install Python 3 and run: python3 scripts/visualize_benchmarks.py"
fi

echo ""
echo -e "${GREEN}╔══════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║  Benchmark Suite Complete!                              ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════════════════════╝${NC}"
echo ""
echo "Results saved to:"
echo "  📊 CSV Data:"
echo "     - benchmark_results/benchmark_results.csv"
echo "     - benchmark_results/zip_comparison.csv"
echo ""
echo "  📄 Reports:"
echo "     - benchmark_results/PERFORMANCE_REPORT.txt"
echo "     - benchmark_results/ZIP_COMPARISON_REPORT.txt"
echo ""
echo "  📈 Graphs (if generated):"
echo "     - benchmark_results/throughput_comparison.png"
echo "     - benchmark_results/multicore_scaling.png"
echo "     - benchmark_results/compression_ratios.png"
echo "     - benchmark_results/zip_comparison.png"
echo "     - benchmark_results/performance_dashboard.png"
echo ""
echo "  📊 Criterion Reports:"
echo "     - target/criterion/*/report/index.html"
echo ""
echo "Next steps:"
echo "  1. Review the text reports in benchmark_results/"
echo "  2. Open the visualizations to analyze performance"
echo "  3. Check Criterion HTML reports for detailed statistics"
echo ""
