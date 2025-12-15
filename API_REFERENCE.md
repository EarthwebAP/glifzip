# GLifzip API Reference

Complete API documentation for the GLifzip library (v1.0.0)

## Table of Contents

- [Overview](#overview)
- [Quick Start](#quick-start)
- [Core Functions](#core-functions)
- [Configuration](#configuration)
- [Data Types](#data-types)
- [Error Handling](#error-handling)
- [Advanced Usage](#advanced-usage)
- [Examples](#examples)

## Overview

GLifzip can be used as both a command-line tool and a Rust library. This document covers the library API for integrating GLifzip into your Rust applications.

### Adding GLifzip to Your Project

Add to your `Cargo.toml`:

```toml
[dependencies]
glifzip = { path = "../glifzip" }  # Local path
# or
glifzip = "1.0"  # When published to crates.io
```

### Basic Import

```rust
use glifzip::{
    compress, decompress, verify_archive,
    compress_file, decompress_file,
    CompressionConfig,
};
```

## Quick Start

### In-Memory Compression

```rust
use glifzip::{compress, decompress, CompressionConfig};

fn main() -> std::io::Result<()> {
    let data = b"Hello, GLifzip!";

    // Compress
    let config = CompressionConfig::default();
    let compressed = compress(data, &config)?;

    // Decompress
    let decompressed = decompress(&compressed, config.threads)?;

    assert_eq!(data.as_slice(), decompressed.as_slice());
    Ok(())
}
```

### File Compression

```rust
use glifzip::{compress_file, decompress_file, CompressionConfig};

fn main() -> std::io::Result<()> {
    let config = CompressionConfig::default();

    // Compress file
    compress_file("input.txt", "output.glif", &config)?;

    // Decompress file
    decompress_file("output.glif", "restored.txt", config.threads)?;

    Ok(())
}
```

## Core Functions

### compress

Compresses data in memory and returns a GLIF archive.

```rust
pub fn compress(data: &[u8], config: &CompressionConfig) -> Result<Vec<u8>>
```

**Parameters:**
- `data: &[u8]` - Input data to compress
- `config: &CompressionConfig` - Compression configuration

**Returns:**
- `Result<Vec<u8>>` - Complete GLIF archive (header + sidecar + compressed data)

**Example:**
```rust
let data = vec![42u8; 1_000_000];
let config = CompressionConfig::fast();
let archive = compress(&data, &config)?;

println!("Compressed {} bytes to {} bytes",
         data.len(), archive.len());
```

**Details:**
1. Calculates SHA256 hash of input data
2. Compresses using multi-threaded Zstd
3. Optionally wraps with LZ4 for fast decompression
4. Calculates SHA256 hash of compressed data
5. Creates header with metadata
6. Generates JSON sidecar
7. Returns complete archive

### decompress

Decompresses a GLIF archive and returns the original data.

```rust
pub fn decompress(archive: &[u8], threads: usize) -> Result<Vec<u8>>
```

**Parameters:**
- `archive: &[u8]` - Complete GLIF archive
- `threads: usize` - Number of threads for decompression

**Returns:**
- `Result<Vec<u8>>` - Original uncompressed data

**Example:**
```rust
let decompressed = decompress(&archive, 8)?;
println!("Decompressed to {} bytes", decompressed.len());
```

**Details:**
1. Parses and validates header
2. Reads sidecar metadata
3. Verifies archive SHA256 hash
4. Decompresses payload (LZ4 → Zstd or Zstd-only)
5. Verifies payload SHA256 hash
6. Verifies size matches header
7. Returns original data

**Errors:**
- Invalid magic number or version
- Checksum mismatch
- Hash verification failure
- Size mismatch
- Corruption detected

### compress_file

Compresses a file and saves as a GLIF archive.

```rust
pub fn compress_file<P: AsRef<Path>, Q: AsRef<Path>>(
    input_path: P,
    output_path: Q,
    config: &CompressionConfig,
) -> Result<()>
```

**Parameters:**
- `input_path` - Path to input file
- `output_path` - Path for output .glif file
- `config` - Compression configuration

**Example:**
```rust
use std::path::Path;

let config = CompressionConfig::balanced();
compress_file(
    Path::new("data.bin"),
    Path::new("data.glif"),
    &config
)?;
```

**Details:**
- Reads entire input file into memory
- Calls `compress()` internally
- Writes result to output file
- Not suitable for files larger than available RAM

### decompress_file

Decompresses a GLIF archive to a file.

```rust
pub fn decompress_file<P: AsRef<Path>, Q: AsRef<Path>>(
    input_path: P,
    output_path: Q,
    threads: usize,
) -> Result<()>
```

**Parameters:**
- `input_path` - Path to .glif archive
- `output_path` - Path for decompressed output
- `threads` - Number of threads for decompression

**Example:**
```rust
decompress_file("data.glif", "data.bin", 8)?;
```

### verify_archive

Verifies a GLIF archive without full decompression.

```rust
pub fn verify_archive(archive: &[u8]) -> Result<GlifSidecar>
```

**Parameters:**
- `archive: &[u8]` - Complete GLIF archive

**Returns:**
- `Result<GlifSidecar>` - Sidecar metadata if verification succeeds

**Example:**
```rust
use std::fs;

let archive = fs::read("data.glif")?;
let sidecar = verify_archive(&archive)?;

println!("Archive is valid!");
println!("  Payload size: {} bytes", sidecar.payload.size);
println!("  Archive size: {} bytes", sidecar.archive.size);
println!("  Compression ratio: {:.2}%",
         sidecar.payload.compression_ratio * 100.0);
```

**Details:**
- Parses header and sidecar
- Verifies archive hash (fast, no decompression)
- Returns metadata
- Much faster than full decompression

## Configuration

### CompressionConfig

Configuration structure for compression operations.

```rust
#[derive(Debug, Clone)]
pub struct CompressionConfig {
    pub level: i32,                  // Compression level (1-22)
    pub threads: usize,              // Number of threads
    pub use_lz4_decompression: bool, // Wrap with LZ4 for fast extraction
    pub deterministic: bool,         // Deterministic compression
}
```

### Constructors

#### default()

Balanced configuration suitable for most use cases.

```rust
impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            level: 8,                       // Balanced compression
            threads: num_cpus::get(),       // All available cores
            use_lz4_decompression: true,    // Fast extraction
            deterministic: true,            // Reproducible builds
        }
    }
}
```

**Example:**
```rust
let config = CompressionConfig::default();
```

#### new()

Create custom configuration.

```rust
pub fn new(level: i32, threads: usize) -> Self
```

**Example:**
```rust
let config = CompressionConfig::new(12, 16);
```

#### fast()

Optimized for speed over compression ratio.

```rust
pub fn fast() -> Self {
    Self {
        level: 3,
        threads: num_cpus::get(),
        use_lz4_decompression: true,
        deterministic: true,
    }
}
```

**Use cases:**
- CI/CD pipelines
- Temporary caches
- Quick backups

**Example:**
```rust
let config = CompressionConfig::fast();
let compressed = compress(&data, &config)?;
```

#### balanced()

Alias for `default()`.

```rust
pub fn balanced() -> Self
```

**Example:**
```rust
let config = CompressionConfig::balanced();
```

#### high_compression()

Maximum compression for archival.

```rust
pub fn high_compression() -> Self {
    Self {
        level: 16,
        threads: num_cpus::get(),
        use_lz4_decompression: false,  // Zstd-only for best ratio
        deterministic: true,
    }
}
```

**Use cases:**
- Long-term archival
- Bandwidth-constrained transfers
- Storage optimization

**Example:**
```rust
let config = CompressionConfig::high_compression();
let compressed = compress(&data, &config)?;
```

**Note:** Decompression will be slower without LZ4 wrapper.

## Data Types

### GlifHeader

Header structure for GLIF archives (116 bytes).

```rust
pub struct GlifHeader {
    pub payload_size: u64,        // Uncompressed size
    pub archive_size: u64,        // Compressed size
    pub payload_hash: [u8; 32],   // SHA256 of uncompressed data
    pub archive_hash: [u8; 32],   // SHA256 of compressed data
    pub compression_level: u32,   // Zstd level used
    pub decompression_mode: u32,  // 0=LZ4, 1=Zstd
    pub cores_used: u32,          // Threads used for compression
    pub timestamp: u64,           // Unix timestamp
    pub sidecar_size: u16,        // Size of JSON sidecar
}
```

**Methods:**

#### read()

Parse header from a reader.

```rust
pub fn read<R: Read>(reader: &mut R) -> Result<Self>
```

**Example:**
```rust
use std::io::Cursor;

let archive = fs::read("data.glif")?;
let mut cursor = Cursor::new(archive);
let header = GlifHeader::read(&mut cursor)?;

println!("Payload size: {}", header.payload_size);
```

#### write()

Write header to a writer.

```rust
pub fn write<W: Write>(&self, writer: &mut W) -> Result<()>
```

### GlifSidecar

JSON metadata structure.

```rust
pub struct GlifSidecar {
    pub format: String,                    // "glif/1.0"
    pub payload: PayloadInfo,              // Uncompressed data info
    pub archive: ArchiveInfo,              // Compressed data info
    pub cryptography: CryptographyInfo,    // Hash information
    pub metadata: MetadataInfo,            // Creation metadata
}
```

#### PayloadInfo

```rust
pub struct PayloadInfo {
    pub size: u64,                  // Uncompressed size
    pub hash: String,               // "sha256:..."
    pub compression_ratio: f32,     // archive_size / payload_size
    pub files: Option<u64>,         // Number of files (future)
    pub directories: Option<u64>,   // Number of dirs (future)
}
```

#### ArchiveInfo

```rust
pub struct ArchiveInfo {
    pub size: u64,                  // Compressed size
    pub hash: String,               // "sha256:..."
    pub compressed_with: String,    // "zstd"
    pub decompressed_with: String,  // "lz4" or "zstd"
    pub compression_level: u32,     // Zstd level
    pub threads: u32,               // Threads used
}
```

#### CryptographyInfo

```rust
pub struct CryptographyInfo {
    pub algorithm: String,          // "sha256"
    pub payload_digest: String,     // Hex hash of payload
    pub archive_digest: String,     // Hex hash of archive
    pub signature: Option<String>,  // GPG signature (future)
}
```

#### MetadataInfo

```rust
pub struct MetadataInfo {
    pub created: String,            // RFC3339 timestamp
    pub creator: String,            // "glifzip v1.0"
    pub source_platform: String,    // "linux", "windows", etc.
    pub source_architecture: String,// "x86_64", "aarch64", etc.
    pub deterministic: bool,        // true for reproducible builds
}
```

**Methods:**

#### to_json()

Serialize sidecar to JSON string.

```rust
pub fn to_json(&self) -> Result<String>
```

**Example:**
```rust
let json = sidecar.to_json()?;
println!("Sidecar: {}", json);
```

#### from_json()

Deserialize sidecar from JSON string.

```rust
pub fn from_json(json: &str) -> Result<Self>
```

## Error Handling

All functions return `std::io::Result<T>` using standard Rust error handling.

### Common Errors

```rust
use std::io::{Error, ErrorKind};

match compress_file("input.txt", "output.glif", &config) {
    Ok(_) => println!("Success!"),
    Err(e) => match e.kind() {
        ErrorKind::NotFound => eprintln!("Input file not found"),
        ErrorKind::PermissionDenied => eprintln!("Permission denied"),
        ErrorKind::InvalidData => eprintln!("Invalid data or corruption"),
        _ => eprintln!("Error: {}", e),
    }
}
```

### Error Types

| ErrorKind | Cause |
|-----------|-------|
| `NotFound` | Input file doesn't exist |
| `PermissionDenied` | Insufficient permissions |
| `InvalidData` | Corrupt archive, bad magic number, hash mismatch |
| `Other` | Compression/decompression failure |

## Advanced Usage

### Custom Compression Pipeline

```rust
use glifzip::compression::{
    compress_zstd_multithreaded,
    compress_lz4_multithreaded,
};
use glifzip::verification::calculate_sha256;

fn custom_compress(data: &[u8], level: i32, threads: usize) -> std::io::Result<Vec<u8>> {
    // Step 1: Compress with Zstd
    let zstd_compressed = compress_zstd_multithreaded(data, level, threads)?;

    // Step 2: Wrap with LZ4
    let lz4_wrapped = compress_lz4_multithreaded(&zstd_compressed, threads)?;

    // Step 3: Calculate hashes
    let payload_hash = calculate_sha256(data);
    let archive_hash = calculate_sha256(&lz4_wrapped);

    println!("Payload hash: {:x?}", payload_hash);
    println!("Archive hash: {:x?}", archive_hash);

    Ok(lz4_wrapped)
}
```

### Working with Archive Headers

```rust
use glifzip::GlifHeader;
use std::io::Cursor;
use std::fs;

fn inspect_archive(path: &str) -> std::io::Result<()> {
    let archive = fs::read(path)?;
    let mut cursor = Cursor::new(&archive);

    // Read header
    let header = GlifHeader::read(&mut cursor)?;

    println!("Archive Information:");
    println!("  Payload size: {} bytes", header.payload_size);
    println!("  Archive size: {} bytes", header.archive_size);
    println!("  Compression: {:.2}%",
             (header.archive_size as f64 / header.payload_size as f64) * 100.0);
    println!("  Level: {}", header.compression_level);
    println!("  Threads: {}", header.cores_used);
    println!("  Mode: {}", if header.decompression_mode == 0 { "LZ4" } else { "Zstd" });

    Ok(())
}
```

### Batch Processing

```rust
use std::fs;
use std::path::PathBuf;

fn batch_compress(input_dir: &str, output_dir: &str) -> std::io::Result<()> {
    let config = CompressionConfig::fast();

    for entry in fs::read_dir(input_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let mut output_path = PathBuf::from(output_dir);
            output_path.push(path.file_name().unwrap());
            output_path.set_extension("glif");

            println!("Compressing {:?} -> {:?}", path, output_path);
            compress_file(&path, &output_path, &config)?;
        }
    }

    Ok(())
}
```

### Streaming API (Future)

Note: Current version loads entire files into memory. Streaming API is planned for v2.0.

```rust
// Future API (not yet implemented)
// use glifzip::stream::{CompressWriter, DecompressReader};
//
// let mut writer = CompressWriter::new(output_file, config)?;
// io::copy(&mut input_file, &mut writer)?;
// writer.finish()?;
```

## Examples

### Example 1: Simple Compression

```rust
use glifzip::{compress, decompress, CompressionConfig};

fn main() -> std::io::Result<()> {
    let original = b"Hello, World!";
    let config = CompressionConfig::default();

    let compressed = compress(original, &config)?;
    let decompressed = decompress(&compressed, config.threads)?;

    assert_eq!(original.as_slice(), decompressed.as_slice());
    println!("Roundtrip successful!");

    Ok(())
}
```

### Example 2: File Compression with Verification

```rust
use glifzip::{compress_file, verify_archive, CompressionConfig};
use std::fs;

fn main() -> std::io::Result<()> {
    let config = CompressionConfig::default();

    // Compress
    compress_file("input.txt", "output.glif", &config)?;

    // Verify
    let archive = fs::read("output.glif")?;
    match verify_archive(&archive) {
        Ok(sidecar) => {
            println!("Verification successful!");
            println!("Compression ratio: {:.2}%",
                     sidecar.payload.compression_ratio * 100.0);
        }
        Err(e) => {
            eprintln!("Verification failed: {}", e);
        }
    }

    Ok(())
}
```

### Example 3: Performance Comparison

```rust
use glifzip::{compress, CompressionConfig};
use std::time::Instant;

fn benchmark_levels(data: &[u8]) -> std::io::Result<()> {
    let levels = vec![1, 3, 8, 12, 16];

    for level in levels {
        let config = CompressionConfig::new(level, 8);
        let start = Instant::now();

        let compressed = compress(data, &config)?;

        let duration = start.elapsed();
        let ratio = (compressed.len() as f64 / data.len() as f64) * 100.0;

        println!("Level {}: {:.2}% in {:?}", level, ratio, duration);
    }

    Ok(())
}
```

### Example 4: Error Handling

```rust
use glifzip::{compress_file, decompress_file, CompressionConfig};
use std::io::{Error, ErrorKind};

fn safe_compress(input: &str, output: &str) -> Result<(), String> {
    let config = CompressionConfig::default();

    compress_file(input, output, &config)
        .map_err(|e| match e.kind() {
            ErrorKind::NotFound => format!("Input file '{}' not found", input),
            ErrorKind::PermissionDenied => "Permission denied".to_string(),
            _ => format!("Compression failed: {}", e),
        })?;

    println!("Successfully compressed {} to {}", input, output);
    Ok(())
}
```

### Example 5: Configuration Presets

```rust
use glifzip::{compress, CompressionConfig};

fn compress_with_preset(data: &[u8], preset: &str) -> std::io::Result<Vec<u8>> {
    let config = match preset {
        "fast" => CompressionConfig::fast(),
        "balanced" => CompressionConfig::balanced(),
        "max" => CompressionConfig::high_compression(),
        _ => CompressionConfig::default(),
    };

    compress(data, &config)
}

fn main() -> std::io::Result<()> {
    let data = vec![42u8; 1_000_000];

    let fast = compress_with_preset(&data, "fast")?;
    let balanced = compress_with_preset(&data, "balanced")?;
    let max = compress_with_preset(&data, "max")?;

    println!("Fast: {} bytes", fast.len());
    println!("Balanced: {} bytes", balanced.len());
    println!("Max: {} bytes", max.len());

    Ok(())
}
```

## Constants

```rust
// Chunk size for multi-threaded processing
pub const CHUNK_SIZE: usize = 128 * 1024 * 1024;  // 128 MB

// Default compression level
pub const DEFAULT_COMPRESSION_LEVEL: i32 = 8;

// Header constants
pub const MAGIC_NUMBER: &[u8; 6] = b"GLIF01";
pub const GLIF_VERSION: u32 = 0x00000100;  // v1.0
pub const HEADER_SIZE: usize = 116;
```

## Type Aliases

```rust
use std::io::Result;

// All functions use std::io::Result
pub type GlifResult<T> = Result<T>;
```

## Thread Safety

All GLifzip functions are thread-safe and can be called from multiple threads:

```rust
use glifzip::{compress, CompressionConfig};
use std::thread;

fn parallel_compression() -> std::io::Result<()> {
    let data1 = vec![1u8; 1_000_000];
    let data2 = vec![2u8; 1_000_000];

    let config = CompressionConfig::default();

    let handle1 = thread::spawn({
        let data = data1.clone();
        let cfg = config.clone();
        move || compress(&data, &cfg)
    });

    let handle2 = thread::spawn({
        let data = data2.clone();
        let cfg = config.clone();
        move || compress(&data, &cfg)
    });

    let result1 = handle1.join().unwrap()?;
    let result2 = handle2.join().unwrap()?;

    println!("Compressed in parallel!");
    Ok(())
}
```

## Version Information

Current version: **1.0.0**

Minimum Rust version: **1.70.0** (Rust 2021 edition)

## See Also

- [User Guide](USER_GUIDE.md) - Practical usage examples
- [CLI Manual](CLI_MANUAL.md) - Command-line interface
- [Performance Guide](PERFORMANCE_GUIDE.md) - Optimization tips
- [Building](BUILDING.md) - Build instructions
