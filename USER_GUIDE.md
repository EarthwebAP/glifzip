# GLifzip User Guide

## Table of Contents

- [Introduction](#introduction)
- [Getting Started](#getting-started)
- [Basic Usage](#basic-usage)
- [Real-World Examples](#real-world-examples)
- [Advanced Features](#advanced-features)
- [Best Practices](#best-practices)
- [Integration Examples](#integration-examples)

## Introduction

GLifzip is a high-performance compression tool designed for the GlyphOS ecosystem. It achieves **10-100x faster compression and decompression** than traditional ZIP formats through multi-threaded Zstd compression and ultra-fast LZ4 decompression.

### Key Features

- **Blazing Fast**: Multi-threaded compression scales linearly with CPU cores
- **Deterministic**: Identical inputs always produce identical outputs
- **Verified**: Built-in SHA256 verification ensures data integrity
- **Cross-Platform**: Works on Windows, Linux, and macOS
- **Dual-Layer**: Zstd for compression, LZ4 for fast extraction

### When to Use GLifzip

**Best Use Cases:**
- Large file archives that need fast compression/decompression
- Build artifacts and deployment packages
- Database backups requiring quick restore times
- Data transfers where speed is critical
- Reproducible builds requiring deterministic compression

**Not Recommended For:**
- Small files under 1 MB (overhead isn't worth it)
- Already compressed formats (JPEG, PNG, MP4, etc.)
- Maximum compression ratio (use gzip/xz instead)

## Getting Started

### Installation

```bash
# Build from source
cd glifzip
cargo build --release

# The binary will be at:
# target/release/glifzip

# Optional: Install to system
cargo install --path .
```

### Quick Start

```bash
# Compress a file
glifzip create document.pdf -o document.glif

# Extract a file
glifzip extract document.glif -o document.pdf

# Verify integrity
glifzip verify document.glif
```

## Basic Usage

### Compressing Files

The simplest way to compress a file:

```bash
glifzip create myfile.txt -o myfile.glif
```

This uses default settings:
- Compression level: 8 (balanced)
- Threads: Auto-detected based on CPU cores
- Deterministic: Yes (reproducible builds)

### Extracting Files

Extract a compressed archive:

```bash
glifzip extract myfile.glif -o myfile.txt
```

The extraction is automatically verified using SHA256 hashes embedded in the archive.

### Verifying Archives

Check archive integrity without extracting:

```bash
glifzip verify myfile.glif
```

Output example:
```
Archive verified successfully!
  Payload size: 10485760 bytes
  Archive size: 3670016 bytes
  Compression ratio: 35.00%
  Compression level: 8
  Threads used: 8
```

## Real-World Examples

### Example 1: Compressing Build Artifacts

Scenario: You have a build output directory with executables and assets.

```bash
# Compress with fast settings for CI/CD
glifzip create build/release/app -o artifacts.glif --level=3 --threads=16

# Later, extract on deployment server
glifzip extract artifacts.glif -o app --threads=8
```

**Why this works:**
- Level 3 provides good compression with minimal CPU time
- 16 threads maximizes CI server resources
- Fast extraction (level 3 decompresses faster than level 8)

### Example 2: Database Backup

Scenario: Daily database dumps that need quick restore times.

```bash
# Backup script
pg_dump mydb | glifzip create /dev/stdin -o backup-$(date +%Y%m%d).glif --level=8

# Restore script
glifzip extract backup-20251214.glif -o /dev/stdout | psql mydb
```

**Why this works:**
- Level 8 balances compression ratio and speed
- SHA256 verification ensures backup integrity
- LZ4 decompression layer provides fast restores

### Example 3: Log File Archival

Scenario: Compress old log files for long-term storage.

```bash
# Compress logs with high compression
glifzip create /var/log/app-2024.log -o logs-2024.glif --level=16

# Verify the archive
glifzip verify logs-2024.glif
```

**Why this works:**
- Level 16 provides excellent compression for text logs
- Deterministic compression allows deduplication
- Verification ensures archival integrity

### Example 4: Data Transfer

Scenario: Transfer large datasets between servers.

```bash
# On source server - compress with fast settings
glifzip create dataset.csv -o dataset.glif --level=3 --threads=24

# Transfer
scp dataset.glif remote-server:/data/

# On remote server - extract quickly
glifzip extract dataset.glif -o dataset.csv --threads=24
```

**Why this works:**
- Level 3 minimizes compression time
- Reduced file size speeds up network transfer
- Fast extraction on remote end
- Built-in verification prevents corruption

### Example 5: Software Distribution

Scenario: Distribute software packages to users.

```bash
# Create deterministic release package
glifzip create app-v1.0/ -o app-v1.0.glif --level=8

# Generate checksum for website
sha256sum app-v1.0.glif > app-v1.0.glif.sha256

# Users verify and extract
sha256sum -c app-v1.0.glif.sha256
glifzip extract app-v1.0.glif -o app-v1.0/
```

**Why this works:**
- Deterministic builds = reproducible packages
- Double verification (SHA256 file + embedded hashes)
- Professional distribution workflow

### Example 6: Scientific Data Archives

Scenario: Archive research data with full provenance.

```bash
# Compress experiment results
glifzip create experiment-001.csv -o experiment-001.glif --level=8

# Extract metadata
glifzip verify experiment-001.glif | tee metadata.txt
```

**Why this works:**
- SHA256 hashes provide cryptographic proof of integrity
- Sidecar JSON contains creation timestamp and platform info
- Deterministic compression aids in data provenance

## Advanced Features

### Compression Levels

GLifzip supports compression levels 1-22:

| Level | Speed | Ratio | Use Case |
|-------|-------|-------|----------|
| 1-3   | Fastest | Good | CI/CD, quick backups |
| 4-8   | Fast | Better | General purpose |
| 9-15  | Moderate | Great | Archival, distribution |
| 16-22 | Slow | Best | Long-term storage |

Example:
```bash
# Maximum speed
glifzip create bigfile.bin -o bigfile.glif --level=1

# Balanced (default)
glifzip create bigfile.bin -o bigfile.glif --level=8

# Maximum compression
glifzip create bigfile.bin -o bigfile.glif --level=22
```

### Thread Control

Control parallelism for optimal performance:

```bash
# Use all available cores (default)
glifzip create data.bin -o data.glif

# Limit to 4 threads (laptop with other work)
glifzip create data.bin -o data.glif --threads=4

# Use all 32 cores (server environment)
glifzip create data.bin -o data.glif --threads=32
```

**Thread Count Guidelines:**
- Development laptop: Use 50% of cores (e.g., --threads=4 on 8-core CPU)
- CI/CD server: Use all cores (maximize throughput)
- Shared server: Limit to avoid impacting others
- Single file: More than 16 threads shows diminishing returns

### Understanding Archive Structure

GLifzip archives contain three parts:

1. **Header (116 bytes)**: Magic number, version, sizes, hashes, metadata
2. **Sidecar (variable)**: JSON metadata with detailed information
3. **Payload (variable)**: Compressed data (Zstd → LZ4 wrapped)

You can inspect the sidecar with `verify`:
```bash
glifzip verify archive.glif
```

The sidecar includes:
- Payload and archive sizes
- SHA256 digests
- Compression parameters
- Creation timestamp
- Platform information

## Best Practices

### DO

1. **Use appropriate compression levels**
   ```bash
   # Quick daily backup
   glifzip create data/ -o daily-backup.glif --level=3

   # Monthly archival
   glifzip create data/ -o monthly-archive.glif --level=12
   ```

2. **Always verify archives after creation**
   ```bash
   glifzip create important.db -o important.glif && glifzip verify important.glif
   ```

3. **Use deterministic builds for reproducibility**
   - Default behavior ensures identical outputs
   - Great for version control and deduplication

4. **Match thread count to workload**
   ```bash
   # Interactive use - leave headroom
   glifzip create file.bin -o file.glif --threads=6  # on 8-core CPU

   # Batch processing - use everything
   glifzip create file.bin -o file.glif --threads=16  # on 16-core server
   ```

### DON'T

1. **Don't compress already compressed files**
   ```bash
   # BAD - JPEGs are already compressed
   glifzip create photos.zip -o photos.glif  # Won't help!

   # GOOD - Compress uncompressed formats
   glifzip create photos/*.raw -o photos.glif  # Much better!
   ```

2. **Don't use maximum compression for frequently accessed files**
   ```bash
   # BAD - level 22 is slow to compress AND decompress
   glifzip create cache.bin -o cache.glif --level=22

   # GOOD - level 3 for speed
   glifzip create cache.bin -o cache.glif --level=3
   ```

3. **Don't forget to verify archives**
   ```bash
   # BAD - no verification
   glifzip create data.bin -o data.glif

   # GOOD - verify immediately
   glifzip create data.bin -o data.glif && glifzip verify data.glif
   ```

### Performance Tips

1. **For maximum speed**: Use level 1-3 with all available threads
   ```bash
   glifzip create huge.bin -o huge.glif --level=1 --threads=32
   ```

2. **For best compression**: Use level 16-20 (level 22 rarely helps)
   ```bash
   glifzip create archive.tar -o archive.glif --level=16
   ```

3. **For balanced performance**: Use default settings
   ```bash
   glifzip create file.bin -o file.glif  # level=8, auto threads
   ```

## Integration Examples

### Shell Scripts

```bash
#!/bin/bash
# backup.sh - Daily backup script

DATE=$(date +%Y%m%d)
BACKUP_DIR="/var/backups"
DATA_DIR="/var/data"

# Compress data
glifzip create "$DATA_DIR" -o "$BACKUP_DIR/backup-$DATE.glif" --level=8 --threads=8

# Verify
if glifzip verify "$BACKUP_DIR/backup-$DATE.glif"; then
    echo "Backup successful: backup-$DATE.glif"

    # Remove old backups (keep 7 days)
    find "$BACKUP_DIR" -name "backup-*.glif" -mtime +7 -delete
else
    echo "ERROR: Backup verification failed!"
    exit 1
fi
```

### Python Integration

```python
import subprocess
import sys

def compress_file(input_path, output_path, level=8, threads=8):
    """Compress a file using glifzip"""
    cmd = [
        'glifzip', 'create', input_path,
        '-o', output_path,
        '--level', str(level),
        '--threads', str(threads)
    ]

    result = subprocess.run(cmd, capture_output=True, text=True)

    if result.returncode != 0:
        print(f"Compression failed: {result.stderr}", file=sys.stderr)
        return False

    # Verify
    verify_cmd = ['glifzip', 'verify', output_path]
    verify_result = subprocess.run(verify_cmd, capture_output=True, text=True)

    return verify_result.returncode == 0

# Usage
if compress_file('data.csv', 'data.glif', level=8, threads=16):
    print("Compression successful!")
else:
    print("Compression failed!")
```

### Makefile Integration

```makefile
# Makefile for project with glifzip integration

GLIFZIP := glifzip
LEVEL := 8
THREADS := 8

dist: build
	$(GLIFZIP) create build/release -o dist/app-$(VERSION).glif \
		--level=$(LEVEL) --threads=$(THREADS)
	$(GLIFZIP) verify dist/app-$(VERSION).glif
	@echo "Distribution package created: dist/app-$(VERSION).glif"

backup:
	$(GLIFZIP) create data/ -o backups/backup-$$(date +%Y%m%d).glif \
		--level=12 --threads=$(THREADS)
	$(GLIFZIP) verify backups/backup-$$(date +%Y%m%d).glif

.PHONY: dist backup
```

### CI/CD Integration (GitHub Actions)

```yaml
name: Build and Package

on: [push]

jobs:
  build:
    runs-on: ubuntu-latest

    steps:
    - uses: actions/checkout@v3

    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable

    - name: Build GLifzip
      run: |
        cargo build --release
        cp target/release/glifzip /usr/local/bin/

    - name: Build Project
      run: make build

    - name: Create Archive
      run: |
        glifzip create build/release -o artifacts.glif --level=3 --threads=4
        glifzip verify artifacts.glif

    - name: Upload Artifacts
      uses: actions/upload-artifact@v3
      with:
        name: build-artifacts
        path: artifacts.glif
```

## Next Steps

- Read the [CLI Manual](CLI_MANUAL.md) for detailed command reference
- Check [API Reference](API_REFERENCE.md) for library integration
- See [Performance Guide](PERFORMANCE_GUIDE.md) for optimization tips
- Review [Troubleshooting](TROUBLESHOOTING.md) for common issues
- Browse [FAQ](FAQ.md) for frequently asked questions

## Getting Help

- Report bugs: GitHub Issues
- Documentation: This guide and associated docs
- Performance questions: See PERFORMANCE_GUIDE.md
- Build issues: See BUILDING.md
