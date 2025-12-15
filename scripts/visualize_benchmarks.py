#!/usr/bin/env python3
"""
GLifzip Performance Visualization Script

Generates performance graphs from benchmark CSV data:
- Throughput comparison charts
- Multi-core scaling analysis
- ZIP comparison bar charts
- Compression ratio visualizations
"""

import pandas as pd
import matplotlib.pyplot as plt
import numpy as np
from pathlib import Path
import sys

# Set style
plt.style.use('seaborn-v0_8-darkgrid')
colors = ['#2E86AB', '#A23B72', '#F18F01', '#C73E1D', '#6A994E']

def load_benchmark_data():
    """Load benchmark results from CSV files"""
    data_dir = Path('benchmark_results')

    if not data_dir.exists():
        print(f"Error: {data_dir} directory not found!")
        print("Please run the benchmark suite first:")
        print("  cargo run --release --bin performance_suite")
        sys.exit(1)

    results_file = data_dir / 'benchmark_results.csv'
    comparison_file = data_dir / 'zip_comparison.csv'

    if not results_file.exists():
        print(f"Error: {results_file} not found!")
        sys.exit(1)

    df = pd.read_csv(results_file)

    df_zip = None
    if comparison_file.exists():
        df_zip = pd.read_csv(comparison_file)

    return df, df_zip

def plot_throughput_comparison(df):
    """Plot compression vs decompression throughput for different data types"""
    fig, ax = plt.subplots(figsize=(12, 6))

    # Filter 1GB throughput tests
    df_throughput = df[df['test_name'].str.contains('1GB', na=False)]

    # Get unique data types
    data_types = df_throughput['data_type'].unique()

    x = np.arange(len(data_types))
    width = 0.35

    compress_throughput = []
    decompress_throughput = []

    for dt in data_types:
        comp = df_throughput[(df_throughput['data_type'] == dt) &
                             (df_throughput['operation'] == 'compression')]
        decomp = df_throughput[(df_throughput['data_type'] == dt) &
                               (df_throughput['operation'] == 'decompression')]

        compress_throughput.append(comp['throughput_gbps'].values[0] if len(comp) > 0 else 0)
        decompress_throughput.append(decomp['throughput_gbps'].values[0] if len(decomp) > 0 else 0)

    ax.bar(x - width/2, compress_throughput, width, label='Compression', color=colors[0])
    ax.bar(x + width/2, decompress_throughput, width, label='Decompression', color=colors[1])

    ax.set_xlabel('Data Type', fontsize=12, fontweight='bold')
    ax.set_ylabel('Throughput (GB/s)', fontsize=12, fontweight='bold')
    ax.set_title('GLifzip Throughput: Compression vs Decompression (1 GB datasets)',
                 fontsize=14, fontweight='bold')
    ax.set_xticks(x)
    ax.set_xticklabels(data_types, rotation=45, ha='right')
    ax.legend()
    ax.grid(True, alpha=0.3)

    # Add target lines
    ax.axhline(y=1.0, color='red', linestyle='--', alpha=0.5, label='Target: 1 GB/s compression')
    ax.axhline(y=2.0, color='orange', linestyle='--', alpha=0.5, label='Target: 2 GB/s decompression')

    plt.tight_layout()
    plt.savefig('benchmark_results/throughput_comparison.png', dpi=300, bbox_inches='tight')
    print("✓ Generated: throughput_comparison.png")
    plt.close()

def plot_multicore_scaling(df):
    """Plot multi-core scaling efficiency"""
    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(16, 6))

    # Filter scaling tests
    df_scaling = df[df['test_name'].str.contains('Scaling', na=False)]

    cores = [1, 2, 4, 8, 16]
    compress_throughput = []
    decompress_throughput = []

    for core_count in cores:
        comp = df_scaling[(df_scaling['threads'] == core_count) &
                          (df_scaling['operation'] == 'compression')]
        decomp = df_scaling[(df_scaling['threads'] == core_count) &
                            (df_scaling['operation'] == 'decompression')]

        compress_throughput.append(comp['throughput_gbps'].values[0] if len(comp) > 0 else 0)
        decompress_throughput.append(decomp['throughput_gbps'].values[0] if len(decomp) > 0 else 0)

    # Plot 1: Absolute throughput
    ax1.plot(cores, compress_throughput, 'o-', linewidth=2, markersize=8,
             label='Compression', color=colors[0])
    ax1.plot(cores, decompress_throughput, 's-', linewidth=2, markersize=8,
             label='Decompression', color=colors[1])

    # Ideal scaling lines
    if compress_throughput[0] > 0:
        ideal_compress = [compress_throughput[0] * c for c in cores]
        ax1.plot(cores, ideal_compress, '--', alpha=0.5, color=colors[0],
                 label='Ideal Compression Scaling')

    if decompress_throughput[0] > 0:
        ideal_decompress = [decompress_throughput[0] * c for c in cores]
        ax1.plot(cores, ideal_decompress, '--', alpha=0.5, color=colors[1],
                 label='Ideal Decompression Scaling')

    ax1.set_xlabel('Number of Cores', fontsize=12, fontweight='bold')
    ax1.set_ylabel('Throughput (GB/s)', fontsize=12, fontweight='bold')
    ax1.set_title('Multi-core Scaling: Absolute Throughput', fontsize=14, fontweight='bold')
    ax1.legend()
    ax1.grid(True, alpha=0.3)
    ax1.set_xscale('log', base=2)

    # Plot 2: Scaling efficiency
    if compress_throughput[0] > 0 and decompress_throughput[0] > 0:
        compress_efficiency = [(t / (compress_throughput[0] * c)) * 100
                               for t, c in zip(compress_throughput, cores)]
        decompress_efficiency = [(t / (decompress_throughput[0] * c)) * 100
                                 for t, c in zip(decompress_throughput, cores)]

        ax2.plot(cores, compress_efficiency, 'o-', linewidth=2, markersize=8,
                 label='Compression', color=colors[0])
        ax2.plot(cores, decompress_efficiency, 's-', linewidth=2, markersize=8,
                 label='Decompression', color=colors[1])
        ax2.axhline(y=100, color='green', linestyle='--', alpha=0.5, label='Ideal (100%)')

        ax2.set_xlabel('Number of Cores', fontsize=12, fontweight='bold')
        ax2.set_ylabel('Scaling Efficiency (%)', fontsize=12, fontweight='bold')
        ax2.set_title('Multi-core Scaling Efficiency', fontsize=14, fontweight='bold')
        ax2.legend()
        ax2.grid(True, alpha=0.3)
        ax2.set_xscale('log', base=2)
        ax2.set_ylim([0, 120])

    plt.tight_layout()
    plt.savefig('benchmark_results/multicore_scaling.png', dpi=300, bbox_inches='tight')
    print("✓ Generated: multicore_scaling.png")
    plt.close()

def plot_compression_ratios(df):
    """Plot compression ratios by data type"""
    fig, ax = plt.subplots(figsize=(10, 6))

    # Filter compression ratio tests
    df_ratios = df[df['test_name'].str.contains('CompressionRatio', na=False)]

    if len(df_ratios) == 0:
        # Fallback to 1GB tests
        df_ratios = df[(df['operation'] == 'compression') &
                       (df['compression_ratio'].notna())]

    data_types = df_ratios['data_type'].unique()
    ratios = []

    for dt in data_types:
        ratio_data = df_ratios[df_ratios['data_type'] == dt]['compression_ratio'].values
        if len(ratio_data) > 0:
            # Remove % sign if present
            ratio_val = ratio_data[0]
            if isinstance(ratio_val, str):
                ratio_val = float(ratio_val.replace('%', ''))
            ratios.append(ratio_val)
        else:
            ratios.append(0)

    x = np.arange(len(data_types))
    bars = ax.bar(x, ratios, color=colors[:len(data_types)])

    # Color bars based on compression ratio
    for i, (bar, ratio) in enumerate(zip(bars, ratios)):
        if ratio < 50:
            bar.set_color(colors[4])  # Green - excellent
        elif ratio < 80:
            bar.set_color(colors[2])  # Orange - good
        else:
            bar.set_color(colors[3])  # Red - poor

    ax.set_xlabel('Data Type', fontsize=12, fontweight='bold')
    ax.set_ylabel('Compression Ratio (%)', fontsize=12, fontweight='bold')
    ax.set_title('Compression Ratios by Data Type (Lower is Better)',
                 fontsize=14, fontweight='bold')
    ax.set_xticks(x)
    ax.set_xticklabels(data_types, rotation=45, ha='right')
    ax.grid(True, alpha=0.3, axis='y')

    # Add value labels on bars
    for i, (bar, ratio) in enumerate(zip(bars, ratios)):
        height = bar.get_height()
        ax.text(bar.get_x() + bar.get_width()/2., height + 1,
                f'{ratio:.1f}%', ha='center', va='bottom', fontweight='bold')

    plt.tight_layout()
    plt.savefig('benchmark_results/compression_ratios.png', dpi=300, bbox_inches='tight')
    print("✓ Generated: compression_ratios.png")
    plt.close()

def plot_zip_comparison(df_zip):
    """Plot GLifzip vs ZIP comparison"""
    if df_zip is None or len(df_zip) == 0:
        print("⚠ No ZIP comparison data available")
        return

    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(16, 6))

    data_types = df_zip['data_type'].unique()
    x = np.arange(len(data_types))
    width = 0.35

    # Compression speedup
    speedup_comp = df_zip['speedup_compression'].values
    speedup_decomp = df_zip['speedup_decompression'].values

    ax1.bar(x - width/2, speedup_comp, width, label='Compression', color=colors[0])
    ax1.bar(x + width/2, speedup_decomp, width, label='Decompression', color=colors[1])
    ax1.axhline(y=1.0, color='red', linestyle='--', alpha=0.5, label='Baseline (1x)')

    ax1.set_xlabel('Data Type', fontsize=12, fontweight='bold')
    ax1.set_ylabel('Speedup vs ZIP (x)', fontsize=12, fontweight='bold')
    ax1.set_title('GLifzip vs ZIP: Performance Speedup', fontsize=14, fontweight='bold')
    ax1.set_xticks(x)
    ax1.set_xticklabels(data_types, rotation=45, ha='right')
    ax1.legend()
    ax1.grid(True, alpha=0.3)

    # Add value labels
    for i, (comp, decomp) in enumerate(zip(speedup_comp, speedup_decomp)):
        ax1.text(i - width/2, comp + 0.5, f'{comp:.1f}x',
                ha='center', va='bottom', fontweight='bold')
        ax1.text(i + width/2, decomp + 0.5, f'{decomp:.1f}x',
                ha='center', va='bottom', fontweight='bold')

    # Throughput comparison
    glifzip_comp = df_zip['glifzip_compress_gbps'].values
    glifzip_decomp = df_zip['glifzip_decompress_gbps'].values
    zip_comp = df_zip['zip_compress_mbps'].values / 1024.0  # Convert to GB/s
    zip_decomp = df_zip['zip_decompress_mbps'].values / 1024.0

    x2 = np.arange(len(data_types)) * 3
    width2 = 0.4

    ax2.bar(x2 - width2*1.5, glifzip_comp, width2, label='GLifzip Compression', color=colors[0])
    ax2.bar(x2 - width2*0.5, glifzip_decomp, width2, label='GLifzip Decompression', color=colors[1])
    ax2.bar(x2 + width2*0.5, zip_comp, width2, label='ZIP Compression', color=colors[2], alpha=0.7)
    ax2.bar(x2 + width2*1.5, zip_decomp, width2, label='ZIP Decompression', color=colors[3], alpha=0.7)

    ax2.set_xlabel('Data Type', fontsize=12, fontweight='bold')
    ax2.set_ylabel('Throughput (GB/s)', fontsize=12, fontweight='bold')
    ax2.set_title('GLifzip vs ZIP: Absolute Throughput', fontsize=14, fontweight='bold')
    ax2.set_xticks(x2)
    ax2.set_xticklabels(data_types, rotation=45, ha='right')
    ax2.legend()
    ax2.grid(True, alpha=0.3)

    plt.tight_layout()
    plt.savefig('benchmark_results/zip_comparison.png', dpi=300, bbox_inches='tight')
    print("✓ Generated: zip_comparison.png")
    plt.close()

def generate_summary_dashboard(df, df_zip):
    """Generate a comprehensive summary dashboard"""
    fig = plt.figure(figsize=(20, 12))
    gs = fig.add_gridspec(3, 3, hspace=0.3, wspace=0.3)

    # Title
    fig.suptitle('GLifzip Performance Benchmark Dashboard',
                 fontsize=20, fontweight='bold', y=0.98)

    # 1. Throughput by data type
    ax1 = fig.add_subplot(gs[0, :2])
    df_throughput = df[df['test_name'].str.contains('1GB', na=False)]
    data_types = df_throughput['data_type'].unique()
    x = np.arange(len(data_types))
    width = 0.35

    compress_tp = [df_throughput[(df_throughput['data_type'] == dt) &
                                 (df_throughput['operation'] == 'compression')]['throughput_gbps'].values[0]
                   if len(df_throughput[(df_throughput['data_type'] == dt) &
                                       (df_throughput['operation'] == 'compression')]) > 0 else 0
                   for dt in data_types]
    decompress_tp = [df_throughput[(df_throughput['data_type'] == dt) &
                                   (df_throughput['operation'] == 'decompression')]['throughput_gbps'].values[0]
                     if len(df_throughput[(df_throughput['data_type'] == dt) &
                                         (df_throughput['operation'] == 'decompression')]) > 0 else 0
                     for dt in data_types]

    ax1.bar(x - width/2, compress_tp, width, label='Compression', color=colors[0])
    ax1.bar(x + width/2, decompress_tp, width, label='Decompression', color=colors[1])
    ax1.set_ylabel('Throughput (GB/s)')
    ax1.set_title('Throughput by Data Type (1 GB)', fontweight='bold')
    ax1.set_xticks(x)
    ax1.set_xticklabels(data_types, rotation=45, ha='right')
    ax1.legend()
    ax1.grid(True, alpha=0.3)

    # 2. Compression ratios
    ax2 = fig.add_subplot(gs[0, 2])
    df_ratios = df[(df['operation'] == 'compression') & (df['compression_ratio'].notna())]
    if len(df_ratios) > 0:
        ratios_by_type = df_ratios.groupby('data_type')['compression_ratio'].mean()
        ax2.barh(range(len(ratios_by_type)), ratios_by_type.values, color=colors[2])
        ax2.set_yticks(range(len(ratios_by_type)))
        ax2.set_yticklabels(ratios_by_type.index)
        ax2.set_xlabel('Ratio (%)')
        ax2.set_title('Compression Ratios', fontweight='bold')
        ax2.grid(True, alpha=0.3, axis='x')

    # 3. Multi-core scaling
    ax3 = fig.add_subplot(gs[1, :])
    df_scaling = df[df['test_name'].str.contains('Scaling', na=False)]
    cores = [1, 2, 4, 8, 16]

    compress_scaling = [df_scaling[(df_scaling['threads'] == c) &
                                   (df_scaling['operation'] == 'compression')]['throughput_gbps'].values[0]
                        if len(df_scaling[(df_scaling['threads'] == c) &
                                         (df_scaling['operation'] == 'compression')]) > 0 else 0
                        for c in cores]
    decompress_scaling = [df_scaling[(df_scaling['threads'] == c) &
                                     (df_scaling['operation'] == 'decompression')]['throughput_gbps'].values[0]
                          if len(df_scaling[(df_scaling['threads'] == c) &
                                           (df_scaling['operation'] == 'decompression')]) > 0 else 0
                          for c in cores]

    ax3.plot(cores, compress_scaling, 'o-', linewidth=2, markersize=8,
             label='Compression', color=colors[0])
    ax3.plot(cores, decompress_scaling, 's-', linewidth=2, markersize=8,
             label='Decompression', color=colors[1])
    ax3.set_xlabel('Number of Cores')
    ax3.set_ylabel('Throughput (GB/s)')
    ax3.set_title('Multi-core Scaling Performance', fontweight='bold')
    ax3.legend()
    ax3.grid(True, alpha=0.3)
    ax3.set_xscale('log', base=2)

    # 4. ZIP comparison (if available)
    if df_zip is not None and len(df_zip) > 0:
        ax4 = fig.add_subplot(gs[2, :])
        zip_data_types = df_zip['data_type'].unique()
        x_zip = np.arange(len(zip_data_types))
        width_zip = 0.35

        speedup_comp = df_zip['speedup_compression'].values
        speedup_decomp = df_zip['speedup_decompression'].values

        ax4.bar(x_zip - width_zip/2, speedup_comp, width_zip,
                label='Compression Speedup', color=colors[0])
        ax4.bar(x_zip + width_zip/2, speedup_decomp, width_zip,
                label='Decompression Speedup', color=colors[1])
        ax4.axhline(y=1.0, color='red', linestyle='--', alpha=0.5, label='Baseline')
        ax4.set_ylabel('Speedup vs ZIP (x)')
        ax4.set_title('GLifzip vs ZIP Performance Comparison', fontweight='bold')
        ax4.set_xticks(x_zip)
        ax4.set_xticklabels(zip_data_types, rotation=45, ha='right')
        ax4.legend()
        ax4.grid(True, alpha=0.3)

    plt.savefig('benchmark_results/performance_dashboard.png', dpi=300, bbox_inches='tight')
    print("✓ Generated: performance_dashboard.png")
    plt.close()

def main():
    print("╔════════════════════════════════════════════════╗")
    print("║  GLifzip Performance Visualization Generator  ║")
    print("╚════════════════════════════════════════════════╝")
    print()

    # Load data
    print("Loading benchmark data...")
    df, df_zip = load_benchmark_data()
    print(f"  Loaded {len(df)} benchmark results")
    if df_zip is not None:
        print(f"  Loaded {len(df_zip)} ZIP comparison results")
    print()

    # Generate visualizations
    print("Generating visualizations...")

    plot_throughput_comparison(df)
    plot_multicore_scaling(df)
    plot_compression_ratios(df)

    if df_zip is not None:
        plot_zip_comparison(df_zip)

    generate_summary_dashboard(df, df_zip)

    print()
    print("✓ All visualizations generated successfully!")
    print("  Output directory: benchmark_results/")
    print()
    print("Generated files:")
    print("  - throughput_comparison.png")
    print("  - multicore_scaling.png")
    print("  - compression_ratios.png")
    if df_zip is not None:
        print("  - zip_comparison.png")
    print("  - performance_dashboard.png")

if __name__ == '__main__':
    main()
