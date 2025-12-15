# GLifzip Frequently Asked Questions (FAQ)

Common questions and answers about GLifzip.

## Table of Contents

- [General Questions](#general-questions)
- [Technical Questions](#technical-questions)
- [Performance Questions](#performance-questions)
- [Compatibility Questions](#compatibility-questions)
- [Usage Questions](#usage-questions)
- [Comparison Questions](#comparison-questions)

## General Questions

### What is GLifzip?

GLifzip is a high-performance compression tool designed for the GlyphOS ecosystem. It uses multi-threaded Zstd compression and ultra-fast LZ4 decompression to achieve 10-100x better performance than traditional ZIP formats.

### Why should I use GLifzip instead of ZIP/GZIP?

**Advantages over ZIP/GZIP:**
- 10-100x faster compression and decompression
- Multi-threaded by default (linear scaling with CPU cores)
- Deterministic compression (reproducible builds)
- Built-in SHA256 verification
- Better compression ratios at similar speeds

**When to stick with ZIP/GZIP:**
- Need maximum compatibility (every platform supports ZIP)
- Working with very small files (<1 MB)
- Single-core systems

### Is GLifzip free and open source?

Yes! GLifzip is licensed under the MIT License. You can:
- Use it for any purpose (commercial or personal)
- Modify the source code
- Distribute it freely
- Contribute improvements

### What does "GLIF" stand for?

GLIF is the file format name used by GLifzip. It's part of the GlyphOS ecosystem naming convention.

### What platforms does GLifzip support?

GLifzip runs on:
- Linux (x86_64, ARM64)
- macOS (Intel, Apple Silicon)
- Windows (x86_64)
- FreeBSD and other Unix-like systems

Built with Rust for maximum portability.

### How mature is GLifzip?

**Current Status (v1.0.0):**
- Core compression/decompression: Production-ready
- File operations: Stable
- API: Stable
- Advanced features: In development (v2.0+)

Week 1 complete with 100% test pass rate. See [WEEK1_COMPLETION.md](WEEK1_COMPLETION.md) for details.

## Technical Questions

### How does GLifzip achieve such high performance?

**Multi-threading:**
- Zstd compression uses all available CPU cores
- Chunk-based processing (128 MB per chunk)
- Rayon-based work stealing for optimal load balancing

**Dual-layer compression:**
- Zstd for high compression ratios
- LZ4 wrapper for ultra-fast decompression (10x faster than Zstd)

**Modern algorithms:**
- Zstd: State-of-the-art compression (Facebook/Meta)
- LZ4: Fastest decompressor available

### What is deterministic compression?

Deterministic compression means identical inputs always produce identical outputs, byte-for-byte.

**Benefits:**
- Reproducible builds
- Deduplication (same archive = same hash)
- Version control friendly
- Cryptographic verification

**How GLifzip achieves it:**
- Fixed timestamps (epoch or RFC3339 format)
- Consistent byte ordering (big-endian)
- No random elements

### What is the GLIF file format?

```
GLIF Archive Structure:
┌────────────────────────────┐
│ Header (116 bytes)         │  Magic, version, sizes, hashes
├────────────────────────────┤
│ Sidecar (variable JSON)    │  Metadata, parameters, digests
├────────────────────────────┤
│ Compressed Payload         │  Zstd → LZ4 wrapped data
└────────────────────────────┘
```

**Header fields:**
- Magic number: "GLIF01"
- Version: 1.0
- Payload size (uncompressed)
- Archive size (compressed)
- SHA256 hashes (payload and archive)
- Compression parameters
- Adler-32 checksum

**Sidecar contains:**
- Detailed metadata (JSON format)
- Creation timestamp
- Platform information
- Compression parameters
- Hash digests

### How does verification work?

**During compression:**
1. Calculate SHA256 of uncompressed data
2. Compress data
3. Calculate SHA256 of compressed data
4. Embed both hashes in header

**During decompression:**
1. Read header
2. Verify archive hash (before decompression)
3. Decompress data
4. Verify payload hash
5. Verify size matches

**Fast verification (no decompression):**
- `glifzip verify` only checks archive hash
- Much faster than full extraction

### Can archives be corrupted without detection?

**Extremely unlikely.** SHA256 provides cryptographic-level integrity:
- Collision probability: 2^-256 (essentially impossible)
- Detects any bit flip or modification
- Multiple layers of verification

**Protection against:**
- Disk corruption
- Network transmission errors
- Bit rot
- Malicious modification

**Cannot protect against:**
- Corrupted source data (garbage in, garbage out)
- Corrupted original before compression

### What compression levels should I use?

**Quick Guide:**

| Level | Speed | Ratio | Use Case |
|-------|-------|-------|----------|
| 1-3   | Very fast | Good | CI/CD, quick backups |
| 4-8   | Fast | Better | General purpose, daily use |
| 9-15  | Moderate | Great | Distribution, archival |
| 16-22 | Slow | Best | Long-term storage |

**Default: Level 8** (balanced performance and compression)

See [Performance Guide](PERFORMANCE_GUIDE.md) for detailed benchmarks.

### How much RAM does GLifzip need?

**Formula:**
```
Required RAM = File Size + (Threads × 128 MB) + 500 MB overhead
```

**Examples:**
- 1 GB file, 8 threads: ~2.5 GB RAM
- 4 GB file, 16 threads: ~6.5 GB RAM
- 10 GB file, 4 threads: ~11 GB RAM

**Recommendation:** Leave 2 GB free for OS.

### Does GLifzip support streaming?

**v1.0:** No, files are loaded entirely into memory.

**v2.0 (planned):** Yes, streaming API for large files.

**Current workaround:**
```bash
# Split large files
split -b 2G hugefile.bin chunk-

# Compress chunks
for chunk in chunk-*; do
  glifzip create "$chunk" -o "$chunk.glif"
done
```

### Can I compress directories?

**v1.0:** No, single files only.

**Workaround:**
```bash
# Create tarball first
tar -cf archive.tar directory/

# Compress tarball
glifzip create archive.tar -o archive.tar.glif
```

**v2.0 (planned):** Native directory support with:
- File metadata preservation
- Recursive traversal
- Exclude patterns

### Is GLifzip thread-safe?

Yes! All GLifzip functions are thread-safe and can be called concurrently from multiple threads.

**Example:**
```rust
use std::thread;
use glifzip::{compress, CompressionConfig};

let config = CompressionConfig::default();

let handle1 = thread::spawn(move || {
    compress(&data1, &config)
});

let handle2 = thread::spawn(move || {
    compress(&data2, &config)
});
```

## Performance Questions

### How fast is GLifzip compared to ZIP?

**Typical results (1 GB file, 8 cores):**

| Tool | Compression | Decompression | Threads |
|------|-------------|---------------|---------|
| GLifzip | 8s | 3s | 8 |
| ZIP (pigz) | 60s | 20s | 8 |
| ZIP (gzip) | 180s | 15s | 1 |

**Speedup: 10-20x compression, 5-10x decompression**

### Why is my compression slower than advertised?

**Common causes:**

1. **High compression level**
   - Level 16+: Much slower
   - Use level 3-8 for speed

2. **Insufficient threads**
   - Check: `nproc` (Linux) or `sysctl -n hw.ncpu` (macOS)
   - Set explicitly: `--threads=16`

3. **I/O bottleneck**
   - HDD is slower than SSD
   - Network storage adds latency

4. **CPU throttling**
   - Check temperatures
   - Laptop power-saving modes

5. **Resource contention**
   - Other processes using CPU
   - Close unnecessary programs

### How can I make compression faster?

1. **Lower compression level**
   ```bash
   glifzip create file.bin -o file.glif --level=3  # Fast!
   ```

2. **Use all CPU cores**
   ```bash
   glifzip create file.bin -o file.glif --threads=$(nproc)
   ```

3. **Use faster storage**
   - SSD instead of HDD
   - Local disk instead of network

4. **Process on a dedicated machine**
   - No competing processes

### How can I get better compression ratios?

1. **Increase compression level**
   ```bash
   glifzip create file.bin -o file.glif --level=16
   ```

2. **Use Zstd-only mode (no LZ4 wrapper)**
   ```bash
   # Use high_compression config in API
   let config = CompressionConfig::high_compression();
   ```

3. **Pre-process data**
   - Sort data for better compression
   - Remove redundant information

4. **Accept that some data won't compress**
   - Already compressed: JPEG, MP4, ZIP
   - Random/encrypted data
   - Binary executables (limited compression)

### Does GLifzip work well on SSDs vs. HDDs?

**SSD:**
- Ideal for GLifzip
- I/O is not a bottleneck
- Full multi-threaded performance

**HDD:**
- Compression is still CPU-bound (works fine)
- Decompression may be I/O-limited
- Consider fewer threads to match disk speed

**NVMe:**
- Maximum performance
- No I/O bottlenecks even with 32+ threads

## Compatibility Questions

### Can I extract GLIF archives on any system?

**Requirements:**
- GLifzip binary for your platform
- OR the GLifzip Rust library

**Cross-platform:**
- Archive created on Linux works on Windows/macOS
- Archive created on Intel works on ARM
- Deterministic compression ensures identical results

### Are GLIF archives compatible across GLifzip versions?

**v1.0 format (GLIF01):**
- Forward compatible: Future versions can read v1.0 archives
- Backward compatible: v1.0 can read v1.0 archives only

**Best practice:**
- Note GLifzip version used for compression
- Keep old versions for old archives (if needed)

### Can other tools extract GLIF archives?

**No.** GLIF is a GLifzip-specific format.

**To extract without GLifzip:**
1. You need the source code
2. Implement GLIF parser (see `src/format/`)
3. Use Zstd and LZ4 libraries

**Or:** Just use GLifzip (it's free and cross-platform)

### Can I convert ZIP to GLIF or vice versa?

**ZIP → GLIF:**
```bash
# Extract ZIP
unzip archive.zip -d extracted/

# Compress with GLifzip
glifzip create extracted/ -o archive.glif  # v2.0 feature

# v1.0 workaround:
tar -cf extracted.tar extracted/
glifzip create extracted.tar -o archive.tar.glif
```

**GLIF → ZIP:**
```bash
# Extract GLIF
glifzip extract archive.glif -o data.tar

# Create ZIP
tar -xf data.tar
zip -r archive.zip extracted/
```

## Usage Questions

### How do I compress a file?

```bash
glifzip create input.txt -o output.glif
```

### How do I extract an archive?

```bash
glifzip extract archive.glif -o output.txt
```

### How do I verify an archive?

```bash
glifzip verify archive.glif
```

### How do I compress with maximum speed?

```bash
glifzip create file.bin -o file.glif --level=1 --threads=16
```

### How do I compress with maximum compression?

```bash
glifzip create file.bin -o file.glif --level=20 --threads=8
```

### Can I compress multiple files at once?

**v1.0:** No, one file at a time.

**Workaround:**
```bash
# Bash script
for file in *.txt; do
  glifzip create "$file" -o "$file.glif"
done
```

**v2.0 (planned):** Multiple file support in archives.

### How do I automate compression?

**Shell script:**
```bash
#!/bin/bash
# compress-backups.sh

for backup in /backups/*.sql; do
  glifzip create "$backup" -o "$backup.glif" --level=8
  if glifzip verify "$backup.glif"; then
    rm "$backup"  # Remove original after verification
  fi
done
```

**Cron job:**
```
0 2 * * * /usr/local/bin/compress-backups.sh
```

**Python:**
```python
import subprocess

subprocess.run([
    'glifzip', 'create', 'data.csv',
    '-o', 'data.glif',
    '--level', '8'
], check=True)
```

### How do I integrate GLifzip into my application?

**CLI (any language):**
```python
# Python example
import subprocess

subprocess.run(['glifzip', 'create', 'file.bin', '-o', 'file.glif'])
```

**Rust library:**
```rust
use glifzip::{compress, CompressionConfig};

let config = CompressionConfig::default();
let compressed = compress(&data, &config)?;
```

**Other languages:** Use FFI bindings (not yet available)

### Can I use GLifzip in Docker?

**Yes!**

```dockerfile
FROM rust:1.70 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/glifzip /usr/local/bin/
CMD ["glifzip"]
```

### What file extension should I use?

**Recommended:** `.glif`

**Also acceptable:**
- `.glif.zst` (if you want to indicate Zstd)
- `.glif.lz4` (if you want to indicate LZ4 wrapper)

**Note:** Extension doesn't affect functionality, but `.glif` is standard.

## Comparison Questions

### GLifzip vs. gzip?

| Feature | GLifzip | gzip |
|---------|---------|------|
| Multi-threading | Yes (default) | No (unless pigz) |
| Speed | 10-20x faster | Baseline |
| Compression ratio | Similar | Good |
| Compatibility | GlyphOS | Universal |
| Deterministic | Yes | Yes (with -n) |

**Use GLifzip when:** Speed matters, multi-core available
**Use gzip when:** Maximum compatibility needed

### GLifzip vs. bzip2?

| Feature | GLifzip | bzip2 |
|---------|---------|-------|
| Speed | 20-30x faster | Very slow |
| Compression ratio | Better | Good |
| Multi-threading | Yes | No |
| Memory usage | Higher | Lower |

**Use GLifzip when:** Speed matters
**Use bzip2 when:** Single-core, memory-constrained (rare)

### GLifzip vs. xz?

| Feature | GLifzip | xz |
|---------|---------|-----|
| Speed | 50-100x faster | Very slow |
| Compression ratio | Good | Best |
| Multi-threading | Yes | Limited |
| Decompression | Very fast | Slow |

**Use GLifzip when:** Speed matters
**Use xz when:** Maximum compression, time not critical

### GLifzip vs. Zstd?

| Feature | GLifzip | Zstd |
|---------|---------|------|
| Algorithm | Zstd + LZ4 | Zstd only |
| Decompression | Faster (LZ4 wrapper) | Fast |
| Verification | Built-in SHA256 | External |
| Deterministic | Built-in | Requires flags |
| File format | GLIF | .zst |

**Use GLifzip when:** Want fast decompression, verification, determinism
**Use Zstd when:** Want simplicity, standard .zst format

### GLifzip vs. 7-Zip?

| Feature | GLifzip | 7-Zip |
|---------|---------|-------|
| Compression ratio | Good | Better (LZMA2) |
| Speed | Much faster | Slower |
| Multi-threading | Excellent | Good |
| GUI | No (v1.0) | Yes |
| Cross-platform | Yes | Yes |

**Use GLifzip when:** Speed is priority
**Use 7-Zip when:** Maximum compression, need GUI

## Common Misconceptions

### "Higher compression level is always better"

**False.** Beyond level 12-16, compression ratio improvements are minimal while time increases dramatically.

**Reality:** Level 8-12 is optimal for most use cases.

### "More threads = always faster"

**False.** Diminishing returns beyond 16 threads.

**Reality:** Match threads to CPU cores. Beyond that, overhead increases.

### "GLifzip can compress any file to 10% of original size"

**False.** Compression ratio depends on data characteristics.

**Reality:**
- Text, logs: 10-30% (excellent)
- Binary data: 50-70% (good)
- Already compressed: 95-100% (no gain)
- Random/encrypted: ~100% (no compression possible)

### "GLIF archives are incompatible across platforms"

**False.** GLIF format is designed for cross-platform compatibility.

**Reality:** Archive created on Linux works on Windows/macOS/ARM.

### "GLifzip is only for GlyphOS"

**False.** GLifzip works on any platform with Rust support.

**Reality:** Designed for GlyphOS but fully cross-platform.

## Troubleshooting Quick Answers

### Why does compression fail?

Most common:
1. File not found (check path)
2. Permission denied (check permissions)
3. Out of disk space (check `df -h`)
4. Out of memory (reduce threads or use smaller files)

### Why does decompression fail?

Most common:
1. Corrupted archive (verify with `glifzip verify`)
2. Not a GLIF archive (check with `file archive.glif`)
3. Wrong version (check GLifzip version)

### Why is compression slow?

Most common:
1. High compression level (use level 3-8)
2. Low thread count (use all cores)
3. Slow disk (use SSD)

See [Troubleshooting Guide](TROUBLESHOOTING.md) for detailed solutions.

## Getting Started

### I'm new to GLifzip. Where do I start?

1. **Install GLifzip**
   ```bash
   cargo install --path /path/to/glifzip
   ```

2. **Try basic compression**
   ```bash
   echo "Hello, GLifzip!" > test.txt
   glifzip create test.txt -o test.glif
   ```

3. **Verify and extract**
   ```bash
   glifzip verify test.glif
   glifzip extract test.glif -o restored.txt
   cat restored.txt
   ```

4. **Read the guides**
   - [User Guide](USER_GUIDE.md) - Practical examples
   - [CLI Manual](CLI_MANUAL.md) - Command reference

### What are the essential commands?

**Three commands you need:**
1. `glifzip create` - Compress
2. `glifzip extract` - Decompress
3. `glifzip verify` - Check integrity

**Everything else is optional configuration.**

### Where can I get help?

1. **Documentation:** This FAQ and other guides
2. **GitHub Issues:** Report bugs
3. **GitHub Discussions:** Ask questions
4. **Source Code:** Read the implementation

## Future Features

### What's planned for v2.0?

- Streaming API for large files
- Directory compression with metadata
- Progress indicators
- File listing in archives
- Incremental compression
- Windows context menu integration

See [README.md](README.md) roadmap section.

### Can I request features?

Yes! Open a GitHub issue with:
- Feature description
- Use case
- Why it's important
- Proposed implementation (optional)

### Can I contribute?

Absolutely! GLifzip is open source. See [BUILDING.md](BUILDING.md) for development setup.

**Ways to contribute:**
- Bug reports
- Feature requests
- Code improvements
- Documentation
- Testing
- Performance optimizations

## Still Have Questions?

- Check other documentation files
- Search GitHub Issues
- Ask in GitHub Discussions
- Read the source code

**Not finding answers?**
Create an issue: Include your question, context, and what you've already tried.

We're here to help!
