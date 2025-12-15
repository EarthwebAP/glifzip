# GLifzip CLI Manual

Complete command-line reference for GLifzip v1.0.0

## Table of Contents

- [Synopsis](#synopsis)
- [Description](#description)
- [Commands](#commands)
- [Global Options](#global-options)
- [Environment Variables](#environment-variables)
- [Exit Codes](#exit-codes)
- [Examples](#examples)
- [Shell Integration](#shell-integration)

## Synopsis

```
glifzip <COMMAND> [OPTIONS]

Commands:
  create   Create a GLIF archive from a file
  extract  Extract a GLIF archive
  verify   Verify a GLIF archive
  help     Print help information
```

## Description

GLifzip is a high-performance compression tool for the GlyphOS ecosystem. It provides multi-threaded Zstd compression with ultra-fast LZ4 decompression, achieving 10-100x better performance than traditional ZIP formats.

### File Format

GLifzip uses the `.glif` file extension for archives. Each archive contains:
- A 116-byte header with magic number, hashes, and metadata
- A variable-length JSON sidecar with detailed information
- Compressed payload using Zstd and optionally LZ4

### Design Philosophy

- **Fast**: Multi-threaded compression scales linearly with CPU cores
- **Safe**: SHA256 verification ensures data integrity
- **Deterministic**: Identical inputs produce identical outputs
- **Cross-platform**: Works on Windows, Linux, and macOS

## Commands

### create

Create a GLIF archive from an input file.

```
glifzip create <INPUT> -o <OUTPUT> [OPTIONS]
```

#### Arguments

**INPUT** (required)
- Path to the input file to compress
- Can be any file type
- Must exist and be readable
- Loaded entirely into memory (ensure sufficient RAM)

**-o, --output** (required)
- Path for the output .glif archive
- Creates directories if they don't exist
- Overwrites existing files without warning
- Recommended extension: `.glif`

#### Options

**-l, --level** (optional)
- Compression level: 1-22
- Default: 8
- Lower = faster, less compression
- Higher = slower, better compression
- Sweet spot: 3-12 for most use cases

**-t, --threads** (optional)
- Number of threads to use
- Default: Auto-detect (all available cores)
- Range: 1 to system maximum
- More threads = faster compression (with diminishing returns)

#### Examples

Basic usage:
```bash
glifzip create document.pdf -o document.glif
```

Fast compression for CI/CD:
```bash
glifzip create build/ -o artifacts.glif --level=3 --threads=16
```

Maximum compression for archival:
```bash
glifzip create data.tar -o data.glif --level=20 --threads=8
```

Using short flags:
```bash
glifzip create file.bin -o file.glif -l 12 -t 4
```

#### Output

Success:
```
Compressing input.txt to output.glif (level=8, threads=8)
```

Error (file not found):
```
Error: No such file or directory (os error 2)
```

#### Exit Codes

- `0`: Success
- `1`: Error (file not found, permission denied, etc.)

### extract

Extract a GLIF archive to a file.

```
glifzip extract <INPUT> -o <OUTPUT> [OPTIONS]
```

#### Arguments

**INPUT** (required)
- Path to the .glif archive
- Must be a valid GLIF archive
- Automatically verifies SHA256 hashes
- Rejects corrupted archives

**-o, --output** (required)
- Path for the extracted output file
- Creates directories if they don't exist
- Overwrites existing files without warning

#### Options

**-t, --threads** (optional)
- Number of threads to use
- Default: Auto-detect (all available cores)
- More threads = faster decompression

#### Examples

Basic usage:
```bash
glifzip extract document.glif -o document.pdf
```

With explicit thread count:
```bash
glifzip extract data.glif -o data.bin --threads=16
```

Extract to different directory:
```bash
glifzip extract backup.glif -o /restore/data.bin
```

#### Output

Success:
```
Extracting archive.glif to output.txt (threads=8)
```

Error (invalid archive):
```
Error: Invalid GLIF magic number
```

Error (corrupted data):
```
Error: Hash verification failed
```

#### Verification

Extraction automatically performs:
1. Header checksum validation
2. Archive SHA256 verification
3. Payload SHA256 verification
4. Size validation

If any check fails, extraction is aborted and no file is written.

#### Exit Codes

- `0`: Success
- `1`: Error (invalid archive, corruption, permission denied, etc.)

### verify

Verify a GLIF archive without extracting.

```
glifzip verify <INPUT>
```

#### Arguments

**INPUT** (required)
- Path to the .glif archive
- Verifies structure and hashes
- Does not decompress the full payload (fast)

#### Examples

Basic usage:
```bash
glifzip verify archive.glif
```

Multiple files:
```bash
for f in *.glif; do
  glifzip verify "$f" && echo "$f: OK"
done
```

#### Output

Success:
```
Verifying archive.glif...
Archive verified successfully!
  Payload size: 10485760 bytes
  Archive size: 3670016 bytes
  Compression ratio: 35.00%
  Compression level: 8
  Threads used: 8
```

Error (corrupted):
```
Verifying archive.glif...
Error: Header checksum mismatch
```

Error (invalid format):
```
Verifying archive.glif...
Error: Invalid GLIF magic number
```

#### What Gets Verified

1. **Header structure**: Magic number, version, checksum
2. **Archive hash**: SHA256 of compressed data
3. **Sidecar**: JSON structure validity
4. **Metadata consistency**: Sizes, parameters

Note: Does NOT decompress payload, so it's much faster than extraction.

#### Exit Codes

- `0`: Verification successful
- `1`: Verification failed

### help

Print help information.

```
glifzip help [COMMAND]
```

#### Examples

General help:
```bash
glifzip help
```

Command-specific help:
```bash
glifzip help create
glifzip help extract
glifzip help verify
```

Using `--help` flag:
```bash
glifzip --help
glifzip create --help
```

## Global Options

### --help, -h

Print help information for glifzip or a specific command.

```bash
glifzip --help
glifzip create --help
```

### --version, -V

Print version information.

```bash
glifzip --version
```

Output:
```
glifzip 1.0.0
```

## Environment Variables

GLifzip respects the following environment variables:

### GLIFZIP_THREADS

Override default thread count.

```bash
export GLIFZIP_THREADS=8
glifzip create file.bin -o file.glif  # Uses 8 threads
```

Note: Command-line `--threads` flag takes precedence.

### GLIFZIP_LEVEL

Override default compression level.

```bash
export GLIFZIP_LEVEL=12
glifzip create file.bin -o file.glif  # Uses level 12
```

Note: Command-line `--level` flag takes precedence.

### RUST_LOG

Control logging output (for debugging).

```bash
export RUST_LOG=debug
glifzip create file.bin -o file.glif
```

Levels: `error`, `warn`, `info`, `debug`, `trace`

## Exit Codes

GLifzip uses standard UNIX exit codes:

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Invalid usage (wrong arguments) |

### Error Examples

```bash
# Success
glifzip create file.txt -o file.glif
echo $?  # 0

# File not found
glifzip create missing.txt -o output.glif
echo $?  # 1

# Invalid arguments
glifzip create
echo $?  # 2
```

## Examples

### Example 1: Basic Compression Workflow

```bash
# Create archive
glifzip create document.txt -o document.glif

# Verify archive
glifzip verify document.glif

# Extract archive
glifzip extract document.glif -o restored.txt

# Compare files
diff document.txt restored.txt
```

### Example 2: CI/CD Pipeline

```bash
#!/bin/bash
# build-and-package.sh

# Build artifacts
make build

# Compress with fast settings
glifzip create build/release -o artifacts.glif --level=3 --threads=16

# Verify before upload
if glifzip verify artifacts.glif; then
  echo "Upload artifacts.glif to server"
  scp artifacts.glif deploy@server:/releases/
else
  echo "ERROR: Archive verification failed!"
  exit 1
fi
```

### Example 3: Backup Script

```bash
#!/bin/bash
# daily-backup.sh

DATE=$(date +%Y%m%d)
BACKUP_FILE="backup-$DATE.glif"

# Compress database dump
glifzip create /var/data/database.sql -o "$BACKUP_FILE" --level=8

# Verify backup
if glifzip verify "$BACKUP_FILE"; then
  echo "Backup successful: $BACKUP_FILE"

  # Upload to cloud storage
  aws s3 cp "$BACKUP_FILE" s3://backups/

  # Remove old backups (keep 7 days)
  find . -name "backup-*.glif" -mtime +7 -delete
else
  echo "ERROR: Backup verification failed!"
  exit 1
fi
```

### Example 4: Batch Processing

```bash
#!/bin/bash
# compress-all.sh

# Compress all .txt files in current directory
for file in *.txt; do
  echo "Compressing $file..."
  glifzip create "$file" -o "${file%.txt}.glif" --level=8

  if glifzip verify "${file%.txt}.glif"; then
    echo "  OK: ${file%.txt}.glif"
  else
    echo "  FAILED: ${file%.txt}.glif"
  fi
done
```

### Example 5: Extract and Verify

```bash
#!/bin/bash
# safe-extract.sh

ARCHIVE="$1"
OUTPUT="$2"

# Verify before extracting
if ! glifzip verify "$ARCHIVE"; then
  echo "ERROR: Archive verification failed!"
  exit 1
fi

# Extract
glifzip extract "$ARCHIVE" -o "$OUTPUT"

# Verify output exists
if [ -f "$OUTPUT" ]; then
  echo "Successfully extracted to $OUTPUT"
else
  echo "ERROR: Extraction failed!"
  exit 1
fi
```

### Example 6: Performance Testing

```bash
#!/bin/bash
# benchmark-levels.sh

FILE="testdata.bin"

for level in 1 3 8 12 16 20; do
  echo "Testing level $level..."

  # Time compression
  time glifzip create "$FILE" -o "test-L$level.glif" --level=$level --threads=8

  # Show compression ratio
  ORIGINAL=$(stat -f%z "$FILE")
  COMPRESSED=$(stat -f%z "test-L$level.glif")
  RATIO=$(echo "scale=2; $COMPRESSED * 100 / $ORIGINAL" | bc)

  echo "  Compression ratio: $RATIO%"
  echo ""
done
```

### Example 7: Conditional Compression

```bash
#!/bin/bash
# smart-compress.sh

FILE="$1"
SIZE=$(stat -f%z "$FILE")

# Use different levels based on file size
if [ $SIZE -lt 1048576 ]; then
  # Small files: fast compression
  LEVEL=3
elif [ $SIZE -lt 104857600 ]; then
  # Medium files: balanced
  LEVEL=8
else
  # Large files: higher compression
  LEVEL=12
fi

echo "Compressing $FILE (size: $SIZE bytes, level: $LEVEL)"
glifzip create "$FILE" -o "$FILE.glif" --level=$LEVEL
```

## Shell Integration

### Bash Completion

Add to `~/.bashrc`:

```bash
# GLifzip completion
_glifzip_completions() {
  local cur prev
  cur="${COMP_WORDS[COMP_CWORD]}"
  prev="${COMP_WORDS[COMP_CWORD-1]}"

  case "$prev" in
    glifzip)
      COMPREPLY=( $(compgen -W "create extract verify help" -- "$cur") )
      return 0
      ;;
    -o|--output)
      COMPREPLY=( $(compgen -f -- "$cur") )
      return 0
      ;;
    -l|--level)
      COMPREPLY=( $(compgen -W "1 3 8 12 16 20" -- "$cur") )
      return 0
      ;;
  esac

  COMPREPLY=( $(compgen -f -- "$cur") )
}

complete -F _glifzip_completions glifzip
```

### Shell Aliases

Add to `~/.bashrc` or `~/.zshrc`:

```bash
# GLifzip aliases
alias gzip='glifzip create'
alias gunzip='glifzip extract'
alias gverify='glifzip verify'

# Quick compression
alias gzfast='glifzip create -l 3'
alias gzmax='glifzip create -l 16'

# Verify then extract
gzextract() {
  if glifzip verify "$1"; then
    glifzip extract "$1" -o "$2"
  else
    echo "ERROR: Archive verification failed!"
    return 1
  fi
}
```

### Fish Shell Completion

Add to `~/.config/fish/completions/glifzip.fish`:

```fish
# GLifzip completions for fish shell
complete -c glifzip -n "__fish_use_subcommand" -a "create" -d "Create archive"
complete -c glifzip -n "__fish_use_subcommand" -a "extract" -d "Extract archive"
complete -c glifzip -n "__fish_use_subcommand" -a "verify" -d "Verify archive"
complete -c glifzip -n "__fish_use_subcommand" -a "help" -d "Show help"

complete -c glifzip -s o -l output -d "Output file" -r
complete -c glifzip -s l -l level -d "Compression level (1-22)" -x
complete -c glifzip -s t -l threads -d "Thread count" -x
complete -c glifzip -s h -l help -d "Show help"
complete -c glifzip -s V -l version -d "Show version"
```

### Directory Shortcuts

Add to `~/.bashrc`:

```bash
# Compress current directory
gzdir() {
  DIR="${1:-.}"
  BASENAME=$(basename "$DIR")
  glifzip create "$DIR" -o "$BASENAME.glif" --level=8
}

# Extract to current directory
gunzdir() {
  ARCHIVE="$1"
  BASENAME=$(basename "$ARCHIVE" .glif)
  glifzip extract "$ARCHIVE" -o "$BASENAME"
}
```

## Advanced Usage

### Piping (Future Feature)

Note: Piping is planned for v2.0. Current version requires file paths.

```bash
# Future API (not yet implemented)
# cat file.txt | glifzip create - -o file.glif
# glifzip extract file.glif -o - | less
```

Current workaround:
```bash
# Use temporary files
cat file.txt > /tmp/input.txt
glifzip create /tmp/input.txt -o file.glif
glifzip extract file.glif -o /tmp/output.txt
cat /tmp/output.txt
```

### Compression Level Guidelines

| Level | Speed | Ratio | CPU | RAM | Use Case |
|-------|-------|-------|-----|-----|----------|
| 1 | Fastest | 2-3x | Low | Low | Quick caching |
| 3 | Very fast | 3-5x | Low | Low | CI/CD, temp files |
| 8 | Fast | 4-7x | Moderate | Moderate | General purpose |
| 12 | Moderate | 5-10x | High | High | Distribution |
| 16 | Slow | 6-12x | Very high | Very high | Long-term storage |
| 20 | Very slow | 6-13x | Extreme | Extreme | Maximum compression |
| 22 | Extremely slow | 6-13x | Extreme | Extreme | Rarely useful |

### Thread Count Guidelines

| Threads | Use Case |
|---------|----------|
| 1 | Minimal CPU usage, debugging |
| 2-4 | Laptop with background tasks |
| 8 | Desktop workstation |
| 16 | Server, CI/CD |
| 24+ | High-performance server |

Diminishing returns beyond 16 threads for most workloads.

### Performance Tips

1. **Match level to use case**:
   - Temporary files: level 1-3
   - Daily backups: level 8
   - Archival: level 12-16

2. **Use all available cores** for batch processing:
   ```bash
   glifzip create bigfile.bin -o bigfile.glif --threads=$(nproc)
   ```

3. **Verify archives immediately** after creation:
   ```bash
   glifzip create file.bin -o file.glif && glifzip verify file.glif
   ```

4. **Use fast levels for CI/CD** to minimize build time:
   ```bash
   glifzip create build/ -o artifacts.glif --level=3
   ```

## Troubleshooting

### Common Errors

**Error: No such file or directory**
```
Solution: Check input file path, ensure it exists
```

**Error: Permission denied**
```
Solution: Check file permissions, run with appropriate user
```

**Error: Invalid GLIF magic number**
```
Solution: File is not a GLIF archive or is corrupted
```

**Error: Hash verification failed**
```
Solution: Archive is corrupted, re-download or re-compress
```

**Error: Header checksum mismatch**
```
Solution: Archive header is corrupted, file is invalid
```

### Debugging

Enable verbose logging:
```bash
export RUST_LOG=debug
glifzip create file.bin -o file.glif
```

Check version:
```bash
glifzip --version
```

Verify installation:
```bash
which glifzip
glifzip help
```

## See Also

- [User Guide](USER_GUIDE.md) - Practical examples and best practices
- [API Reference](API_REFERENCE.md) - Library API for Rust integration
- [Performance Guide](PERFORMANCE_GUIDE.md) - Optimization and benchmarking
- [Troubleshooting](TROUBLESHOOTING.md) - Detailed troubleshooting guide
- [Building](BUILDING.md) - Build from source instructions

## Reporting Bugs

Report bugs via GitHub Issues with:
- GLifzip version (`glifzip --version`)
- Operating system and version
- Command that failed
- Error message
- Expected vs. actual behavior

## License

GLifzip is licensed under the MIT License. See LICENSE file for details.
