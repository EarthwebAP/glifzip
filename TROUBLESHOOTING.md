# GLifzip Troubleshooting Guide

Complete troubleshooting reference for GLifzip issues.

## Table of Contents

- [Common Errors](#common-errors)
- [Installation Issues](#installation-issues)
- [Compression Problems](#compression-problems)
- [Decompression Problems](#decompression-problems)
- [Verification Failures](#verification-failures)
- [Performance Issues](#performance-issues)
- [Platform-Specific Issues](#platform-specific-issues)
- [Debug Mode](#debug-mode)
- [Getting Help](#getting-help)

## Common Errors

### Error: No such file or directory (os error 2)

**Symptom:**
```
Error: No such file or directory (os error 2)
```

**Cause:** Input file doesn't exist or path is incorrect.

**Solution:**
```bash
# Check if file exists
ls -la input.txt

# Use absolute path
glifzip create /full/path/to/input.txt -o output.glif

# Check current directory
pwd
ls -la
```

**Prevention:**
- Always verify file paths before running commands
- Use tab completion in shells
- Use absolute paths when in doubt

---

### Error: Permission denied (os error 13)

**Symptom:**
```
Error: Permission denied (os error 13)
```

**Cause:** Insufficient permissions to read input or write output.

**Solution:**
```bash
# Check file permissions
ls -la input.txt

# Make file readable
chmod 644 input.txt

# Check output directory permissions
ls -ld output/
chmod 755 output/

# Run with appropriate user
sudo glifzip create /protected/file.txt -o output.glif
```

**Common Scenarios:**
- Input file owned by another user
- Output directory not writable
- SELinux/AppArmor restrictions

---

### Error: Invalid GLIF magic number

**Symptom:**
```
Error: Invalid GLIF magic number
```

**Cause:** File is not a GLIF archive or is corrupted.

**Solution:**
```bash
# Check file type
file archive.glif

# Verify it's actually a GLIF archive
hexdump -C archive.glif | head -n 1
# Should start with: 47 4c 49 46 30 31 (GLIF01)

# Re-compress if original file is available
glifzip create original.txt -o new-archive.glif
```

**Common Causes:**
- Downloaded incomplete file
- Trying to extract non-GLIF file
- File extension renamed but not actually GLIF
- Corrupted during transfer

---

### Error: Unsupported GLIF version

**Symptom:**
```
Error: Unsupported GLIF version
```

**Cause:** Archive created with incompatible version.

**Solution:**
```bash
# Check GLifzip version
glifzip --version

# Update GLifzip
cd glifzip
git pull
cargo build --release

# Archive might be from future version
# Downgrade or wait for update
```

**Note:** GLifzip v1.0 only supports GLIF01 format.

---

### Error: Header checksum mismatch

**Symptom:**
```
Error: Header checksum mismatch
```

**Cause:** Archive header is corrupted.

**Solution:**
```bash
# Archive is corrupted - cannot be recovered
# Re-download or re-compress from original

# If downloaded, verify download
sha256sum archive.glif

# Check for partial download
ls -lh archive.glif  # Compare to expected size
```

**Prevention:**
- Verify downloads with checksums
- Use reliable transfer methods
- Verify immediately after compression

---

### Error: Hash verification failed

**Symptom:**
```
Error: Hash verification failed
```

**Cause:** Archive data is corrupted (payload or archive hash mismatch).

**Solution:**
```bash
# Archive is corrupted
# If available, re-compress from original
glifzip create original.txt -o new-archive.glif

# If downloaded, re-download
wget -c https://example.com/archive.glif

# Check disk for errors
fsck /dev/sdX  # Linux
diskutil verifyVolume /  # macOS
```

**Common Causes:**
- Disk corruption
- Bad RAM
- Network transmission error
- Storage device failure

---

### Error: Decompressed size mismatch

**Symptom:**
```
Error: Decompressed size mismatch: expected 1000000, got 999999
```

**Cause:** Decompressed data size doesn't match header.

**Solution:**
```bash
# Archive is corrupted or truncated
# Cannot be recovered
# Re-download or re-compress
```

**Common Causes:**
- Incomplete compression (interrupted)
- Disk full during compression
- Corrupted archive

---

### Error: Out of memory

**Symptom:**
```
Error: Cannot allocate memory
```
or system crashes/freezes during compression.

**Cause:** Insufficient RAM for file size + thread count.

**Solution:**
```bash
# Reduce thread count
glifzip create hugefile.bin -o hugefile.glif --threads=2

# Check available memory
free -h

# Close other applications

# Use smaller files or increase RAM
```

**Memory Calculation:**
```
Required RAM = File Size + (Threads × 128 MB) + 500 MB overhead

Example: 4 GB file, 8 threads
= 4 GB + (8 × 128 MB) + 500 MB
= 4 GB + 1 GB + 0.5 GB
= 5.5 GB required
```

## Installation Issues

### Cargo build fails

**Symptom:**
```
error: linking with `cc` failed
```

**Solution:**
```bash
# Install build dependencies (Ubuntu/Debian)
sudo apt-get install build-essential pkg-config

# Install build dependencies (Fedora/RHEL)
sudo dnf groupinstall "Development Tools"

# Install build dependencies (macOS)
xcode-select --install

# Update Rust
rustup update stable
```

---

### Dependency resolution errors

**Symptom:**
```
error: failed to select a version for `dependency`
```

**Solution:**
```bash
# Update Cargo.lock
cargo update

# Clean and rebuild
cargo clean
cargo build --release

# Use specific Rust version
rustup install 1.70.0
rustup default 1.70.0
```

---

### Tests fail during build

**Symptom:**
```
test result: FAILED. 32 passed; 2 failed
```

**Solution:**
```bash
# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name -- --nocapture

# Skip tests during build
cargo build --release --no-default-features
```

## Compression Problems

### Compression is very slow

**Symptom:** Compression takes much longer than expected.

**Diagnosis:**
```bash
# Check compression level
glifzip create file.bin -o file.glif --level=8  # Should be fast

# Test with lower level
glifzip create file.bin -o file.glif --level=3  # Faster

# Monitor CPU usage
top -p $(pgrep glifzip)  # Should be >90% CPU
```

**Solutions:**

**If CPU usage is low (<50%):**
```bash
# Increase thread count
glifzip create file.bin -o file.glif --threads=16

# Check available cores
nproc
```

**If CPU usage is high (>90%):**
```bash
# Lower compression level
glifzip create file.bin -o file.glif --level=3

# This is normal for high levels (12+)
```

**If I/O-bound:**
```bash
# Move to faster storage (SSD)
# Or reduce thread count to match disk speed
glifzip create file.bin -o file.glif --threads=4
```

---

### Compression ratio is poor

**Symptom:** Compressed file is nearly as large as original.

**Diagnosis:**
```bash
# Check file type
file input.bin

# Check compression ratio
glifzip verify archive.glif | grep "Compression ratio"
```

**Causes:**

**Already compressed files:**
```bash
# These won't compress further:
# - JPEG, PNG, MP4, ZIP, GZIP, etc.

# Solution: Don't compress already compressed files
```

**Random/encrypted data:**
```bash
# Random data doesn't compress
# Check entropy
ent input.bin

# Solution: This is expected behavior
```

**Low compression level:**
```bash
# Increase level
glifzip create file.bin -o file.glif --level=16
```

---

### Compression produces different outputs

**Symptom:** Same input creates different archives on different runs.

**Expected:** GLifzip is deterministic by default.

**Diagnosis:**
```bash
# Compress twice
glifzip create file.txt -o file1.glif --level=8
glifzip create file.txt -o file2.glif --level=8

# Compare
diff file1.glif file2.glif  # Should be identical
sha256sum file*.glif        # Should match
```

**If different:**
```bash
# Check for non-deterministic config
# Ensure deterministic mode is enabled

# Check file modification times
ls -la file.txt
# File metadata shouldn't affect compression in v1.0
```

**Note:** Future versions may support metadata preservation, which would affect determinism.

---

### Compression runs out of disk space

**Symptom:**
```
Error: No space left on device (os error 28)
```

**Solution:**
```bash
# Check available disk space
df -h

# Estimate compressed size (roughly 30-70% of original)
du -h input.file

# Free up space
rm -rf /tmp/old-files

# Use different output location
glifzip create input.bin -o /other/disk/output.glif
```

## Decompression Problems

### Decompression is very slow

**Symptom:** Extraction takes much longer than expected.

**Diagnosis:**
```bash
# Monitor CPU and I/O
iostat -x 1

# Check thread count
glifzip extract archive.glif -o output.bin --threads=16
```

**Solutions:**

**If CPU-bound:**
```bash
# This is normal for archives created without LZ4 wrapper
# Archives created with --level=16+ may use Zstd-only mode

# Increase threads
glifzip extract archive.glif -o output.bin --threads=24
```

**If I/O-bound:**
```bash
# Extract to faster storage (SSD)
glifzip extract archive.glif -o /ssd/output.bin

# Reduce thread count to match disk speed
glifzip extract archive.glif -o output.bin --threads=4
```

---

### Extracted file is corrupted

**Symptom:** Extracted file exists but is incorrect.

**This should never happen** - GLifzip verifies hashes during extraction.

**If it does happen:**
```bash
# 1. Verify the archive
glifzip verify archive.glif

# 2. Re-extract
glifzip extract archive.glif -o output2.bin

# 3. Compare
diff output.bin output2.bin

# 4. Report bug with:
glifzip --version
uname -a
sha256sum archive.glif
```

---

### Cannot extract: file already exists

**Symptom:** Output file exists and extraction fails.

**Note:** GLifzip v1.0 **overwrites** existing files without warning.

**If extraction fails:**
```bash
# Check output path permissions
ls -la output.bin

# Remove or rename existing file
mv output.bin output.bin.old
glifzip extract archive.glif -o output.bin
```

## Verification Failures

### Verify fails on valid archive

**Symptom:**
```
Error: Hash verification failed
```
on an archive that previously verified.

**Causes:**
- File was modified (corrupted)
- Disk corruption
- Memory corruption

**Diagnosis:**
```bash
# Check file integrity
sha256sum archive.glif
# Compare with original checksum

# Check disk health
smartctl -a /dev/sda  # Linux
diskutil info /  # macOS

# Test RAM
memtest86+  # Run overnight
```

**Solution:**
```bash
# Re-download or restore from backup
# If disk/RAM is faulty, fix hardware first
```

---

### Verify shows wrong information

**Symptom:** Payload size or compression ratio seems incorrect.

**Diagnosis:**
```bash
# Check archive header
glifzip verify archive.glif

# Compare with original file
ls -lh original.txt
```

**If metadata is incorrect:**
- Archive was created incorrectly (bug)
- Archive was modified externally
- Header corruption

**Solution:**
```bash
# Re-compress from original
glifzip create original.txt -o new-archive.glif

# Report bug if reproducible
```

## Performance Issues

### CPU usage is low during compression

**Symptom:** GLifzip uses <50% CPU with default settings.

**Cause:** Insufficient thread count or I/O bottleneck.

**Solution:**
```bash
# Explicitly set threads to all cores
glifzip create file.bin -o file.glif --threads=$(nproc)

# Check if I/O-bound
iostat -x 1
# If disk util >80%, you're I/O-bound
```

---

### Memory usage is very high

**Symptom:** System becomes unresponsive or swaps excessively.

**Cause:** Too many threads for available RAM.

**Solution:**
```bash
# Reduce thread count
glifzip create file.bin -o file.glif --threads=4

# Calculate safe thread count
AVAILABLE_GB=$(free -g | awk '/^Mem:/{print $7}')
FILE_SIZE_GB=$(du -BG file.bin | awk '{print $1}' | sed 's/G//')
SAFE_THREADS=$(echo "($AVAILABLE_GB - $FILE_SIZE_GB - 1) / 0.128" | bc)
echo "Safe threads: $SAFE_THREADS"
```

---

### Compression is slower than expected

**Expected Performance:**
- Level 3: ~250 MB/s per core
- Level 8: ~125 MB/s per core
- Level 16: ~17 MB/s per core

**If much slower:**
```bash
# 1. Check CPU usage (should be >90%)
top

# 2. Check for thermal throttling
sensors  # Linux
sudo powermetrics  # macOS

# 3. Check for resource contention
ps aux | grep -v glifzip | sort -k3 -r | head

# 4. Run on dedicated system or with nice
nice -n -10 glifzip create file.bin -o file.glif
```

---

### Decompression is slower than compression

**Symptom:** Extraction takes longer than compression.

**Cause:** Archive created without LZ4 wrapper (high_compression mode).

**Diagnosis:**
```bash
# Check decompression mode in verify output
glifzip verify archive.glif
# Look for "decompressed_with": "zstd" vs "lz4"
```

**Solution:**
- Use default settings for future compressions
- Or accept slower decompression for better ratio

## Platform-Specific Issues

### Linux

#### Error: libzstd.so: cannot open shared object file

**Cause:** Missing zstd library.

**Solution:**
```bash
# Ubuntu/Debian
sudo apt-get install libzstd-dev

# Fedora/RHEL
sudo dnf install libzstd-devel

# Arch
sudo pacman -S zstd

# Rebuild
cargo clean
cargo build --release
```

#### Error: /tmp full during compression

**Cause:** Insufficient space in /tmp.

**Solution:**
```bash
# Use different temp directory
export TMPDIR=/var/tmp
glifzip create file.bin -o file.glif

# Or increase /tmp size
sudo mount -o remount,size=10G /tmp
```

### macOS

#### Error: Code signature invalid

**Cause:** Binary not signed or quarantined.

**Solution:**
```bash
# Remove quarantine attribute
xattr -d com.apple.quarantine glifzip

# Or compile from source
cargo build --release
```

#### Error: xcrun: error: invalid active developer path

**Cause:** Xcode command-line tools not installed.

**Solution:**
```bash
xcode-select --install
```

### Windows

#### Error: MSVCR120.dll missing

**Cause:** Missing Visual C++ redistributable.

**Solution:**
```
Download and install:
Microsoft Visual C++ Redistributable
From: https://aka.ms/vs/17/release/vc_redist.x64.exe
```

#### Error: Access denied

**Cause:** Antivirus blocking or UAC restrictions.

**Solution:**
```
1. Add glifzip.exe to antivirus exceptions
2. Run as Administrator
3. Check file permissions in Security tab
```

## Debug Mode

### Enable Debug Logging

```bash
# Set log level
export RUST_LOG=debug
glifzip create file.bin -o file.glif

# Maximum verbosity
export RUST_LOG=trace
glifzip create file.bin -o file.glif

# Filter by module
export RUST_LOG=glifzip::compression=debug
glifzip create file.bin -o file.glif
```

### Capture Debug Output

```bash
# Save to file
RUST_LOG=debug glifzip create file.bin -o file.glif 2> debug.log

# View in less
RUST_LOG=debug glifzip create file.bin -o file.glif 2>&1 | less
```

### Run Tests with Debug Info

```bash
# Run all tests with output
cargo test -- --nocapture

# Run specific test with debug
RUST_LOG=debug cargo test test_compress_decompress_roundtrip -- --nocapture

# Run with backtrace on panic
RUST_BACKTRACE=1 cargo test
```

### Inspect Archive Structure

```bash
# View hex dump of header
hexdump -C archive.glif | head -n 20

# Extract sidecar JSON
# (First 116 bytes are header, then sidecar)
dd if=archive.glif bs=1 skip=116 count=500 2>/dev/null | head

# Check file structure
file archive.glif
```

## Getting Help

### Before Asking for Help

Gather this information:

```bash
# 1. Version
glifzip --version

# 2. System info
uname -a

# 3. Command that failed
# (copy exact command)

# 4. Error message
# (copy full error output)

# 5. File info
ls -lh input.txt
file input.txt

# 6. Archive info (if applicable)
glifzip verify archive.glif 2>&1
```

### Reporting Bugs

Create a GitHub issue with:

1. **Title**: Brief description (e.g., "Hash verification fails on 4GB files")

2. **Environment**:
   - GLifzip version
   - OS and version
   - Rust version (if building from source)

3. **Steps to reproduce**:
   ```bash
   glifzip create testfile.bin -o testfile.glif --level=8
   glifzip verify testfile.glif  # FAILS
   ```

4. **Expected behavior**: What should happen

5. **Actual behavior**: What actually happens

6. **Error messages**: Full error output

7. **Additional context**:
   - File size
   - Available RAM
   - CPU cores
   - Storage type (HDD/SSD/NVMe)

### Community Support

- GitHub Discussions: General questions
- GitHub Issues: Bug reports
- Documentation: This guide and others
- Source code: Read the implementation

### Known Issues

Check GitHub Issues for known problems:
```
https://github.com/your-org/glifzip/issues
```

## Workarounds

### Large File Limitations (v1.0)

**Issue:** Files larger than available RAM cannot be compressed.

**Workaround:**
```bash
# Split large file
split -b 2G largefile.bin chunk-

# Compress chunks
for chunk in chunk-*; do
  glifzip create "$chunk" -o "$chunk.glif"
done

# Later, extract and reassemble
for chunk in chunk-*.glif; do
  glifzip extract "$chunk" -o "${chunk%.glif}"
done
cat chunk-* > largefile.bin
```

**Permanent fix:** Streaming API planned for v2.0

---

### Directory Compression (v1.0)

**Issue:** Cannot compress directories directly.

**Workaround:**
```bash
# Create tarball first
tar -cf archive.tar directory/

# Compress tarball
glifzip create archive.tar -o archive.tar.glif

# Later, extract
glifzip extract archive.tar.glif -o archive.tar
tar -xf archive.tar
```

**Permanent fix:** Native directory support planned for v2.0

---

### Progress Indication (v1.0)

**Issue:** No progress bar during compression.

**Workaround:**
```bash
# Monitor file size growth
watch -n 1 'ls -lh output.glif'

# Or use pv (pipe viewer)
# (Not directly supported in v1.0)
```

**Permanent fix:** Progress bars planned for v2.0

## See Also

- [User Guide](USER_GUIDE.md) - Usage examples
- [Performance Guide](PERFORMANCE_GUIDE.md) - Optimization tips
- [CLI Manual](CLI_MANUAL.md) - Command reference
- [FAQ](FAQ.md) - Frequently asked questions
- [Building](BUILDING.md) - Build instructions

## Still Stuck?

If this guide didn't help:

1. Search GitHub Issues for similar problems
2. Ask in GitHub Discussions
3. Create a new issue with all relevant information
4. Contact the GlyphOS team

We're here to help!
