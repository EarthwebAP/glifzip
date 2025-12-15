pub mod format;
pub mod compression;
pub mod verification;
pub mod archive;
pub mod platform;

use std::io::{Write, Read, Result, Error, ErrorKind};
use std::fs::File;
use std::path::Path;

pub use format::{GlifHeader, GlifSidecar};
pub use compression::{compress_zstd_multithreaded, decompress_lz4_multithreaded, CHUNK_SIZE, DEFAULT_COMPRESSION_LEVEL};
pub use verification::{calculate_sha256, verify_sha256, hex_encode, hex_decode};
pub use archive::{ArchiveManifest, FileEntry, DirectoryCompressor};
pub use archive::directory_compressor::DirectoryCompressionConfig;

/// Configuration for compression
#[derive(Debug, Clone)]
pub struct CompressionConfig {
    pub level: i32,
    pub threads: usize,
    pub use_lz4_decompression: bool,
    pub deterministic: bool,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            level: DEFAULT_COMPRESSION_LEVEL,
            threads: num_cpus::get(),
            use_lz4_decompression: true,
            deterministic: true,
        }
    }
}

impl CompressionConfig {
    pub fn new(level: i32, threads: usize) -> Self {
        Self {
            level,
            threads,
            use_lz4_decompression: true,
            deterministic: true,
        }
    }

    pub fn fast() -> Self {
        Self {
            level: 3,
            threads: num_cpus::get(),
            use_lz4_decompression: true,
            deterministic: true,
        }
    }

    pub fn balanced() -> Self {
        Self::default()
    }

    pub fn high_compression() -> Self {
        Self {
            level: 16,
            threads: num_cpus::get(),
            use_lz4_decompression: false,
            deterministic: true,
        }
    }
}

/// Compress data and create a GLIF archive
pub fn compress(data: &[u8], config: &CompressionConfig) -> Result<Vec<u8>> {
    // Calculate SHA256 of uncompressed data
    let payload_hash = calculate_sha256(data);

    // Compress data using Zstd
    let compressed_data = compress_zstd_multithreaded(data, config.level, config.threads)?;

    // If using LZ4 decompression mode, we need to recompress with LZ4
    let (archive_data, decompression_mode) = if config.use_lz4_decompression {
        let lz4_compressed = compression::compress_lz4_multithreaded(&compressed_data, config.threads)?;
        (lz4_compressed, 0u32)
    } else {
        (compressed_data, 1u32)
    };

    // Calculate SHA256 of compressed data
    let archive_hash = calculate_sha256(&archive_data);

    // Create sidecar metadata
    let timestamp = if config.deterministic {
        Some("2025-01-01T00:00:00.000000000+00:00".to_string())
    } else {
        None
    };

    let sidecar = format::GlifSidecar::new_with_timestamp(
        data.len() as u64,
        archive_data.len() as u64,
        &payload_hash,
        &archive_hash,
        config.level as u32,
        config.threads as u32,
        decompression_mode,
        timestamp,
    );

    let sidecar_json = sidecar.to_json()?;
    let sidecar_size = sidecar_json.len() as u16;

    // Create header
    let header_timestamp = if config.deterministic {
        Some(0) // Unix epoch for deterministic builds
    } else {
        None
    };

    let header = format::GlifHeader::new_with_timestamp(
        data.len() as u64,
        archive_data.len() as u64,
        payload_hash,
        archive_hash,
        config.level as u32,
        decompression_mode,
        config.threads as u32,
        sidecar_size,
        header_timestamp,
    );

    // Build final archive
    let mut result = Vec::new();
    header.write(&mut result)?;
    result.write_all(sidecar_json.as_bytes())?;
    result.write_all(&archive_data)?;

    Ok(result)
}

/// Decompress a GLIF archive
pub fn decompress(archive: &[u8], threads: usize) -> Result<Vec<u8>> {
    // Parse header
    let mut cursor = std::io::Cursor::new(archive);
    let header = GlifHeader::read(&mut cursor)?;

    // Read sidecar (not currently used but needed to advance cursor position)
    let _sidecar = GlifSidecar::read(&mut cursor, header.sidecar_size)?;

    // Get current position (start of compressed data)
    let header_and_sidecar_size = cursor.position() as usize;

    // Extract compressed data
    let compressed_data = &archive[header_and_sidecar_size..];

    // Verify archive hash
    verify_sha256(compressed_data, &header.archive_hash)?;

    // Decompress based on mode
    let decompressed_data = if header.decompression_mode == 0 {
        // LZ4 mode
        let lz4_decompressed = compression::decompress_lz4_multithreaded(compressed_data, threads)?;
        // The LZ4 layer decompresses to Zstd-compressed data
        compression::decompress_zstd_multithreaded(&lz4_decompressed, threads)?
    } else {
        // Zstd-only mode
        compression::decompress_zstd_multithreaded(compressed_data, threads)?
    };

    // Verify payload hash
    verify_sha256(&decompressed_data, &header.payload_hash)?;

    // Verify size matches
    if decompressed_data.len() != header.payload_size as usize {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!(
                "Decompressed size mismatch: expected {}, got {}",
                header.payload_size,
                decompressed_data.len()
            )
        ));
    }

    Ok(decompressed_data)
}

/// Compress a file and save as GLIF archive
pub fn compress_file<P: AsRef<Path>, Q: AsRef<Path>>(
    input_path: P,
    output_path: Q,
    config: &CompressionConfig,
) -> Result<()> {
    // Read input file
    let mut file = File::open(input_path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    // Compress
    let compressed = compress(&data, config)?;

    // Write output file
    let mut output = File::create(output_path)?;
    output.write_all(&compressed)?;

    Ok(())
}

/// Decompress a GLIF archive file
pub fn decompress_file<P: AsRef<Path>, Q: AsRef<Path>>(
    input_path: P,
    output_path: Q,
    threads: usize,
) -> Result<()> {
    // Read archive file
    let mut file = File::open(input_path)?;
    let mut archive = Vec::new();
    file.read_to_end(&mut archive)?;

    // Decompress
    let decompressed = decompress(&archive, threads)?;

    // Write output file
    let mut output = File::create(output_path)?;
    output.write_all(&decompressed)?;

    Ok(())
}

/// Verify a GLIF archive without decompressing
pub fn verify_archive(archive: &[u8]) -> Result<GlifSidecar> {
    let mut cursor = std::io::Cursor::new(archive);
    let header = GlifHeader::read(&mut cursor)?;
    let sidecar = GlifSidecar::read(&mut cursor, header.sidecar_size)?;

    // Get compressed data position
    let header_and_sidecar_size = cursor.position() as usize;
    let compressed_data = &archive[header_and_sidecar_size..];

    // Verify archive hash
    verify_sha256(compressed_data, &header.archive_hash)?;

    Ok(sidecar)
}

// Helper function to get number of CPUs (we'll use rayon's default if num_cpus isn't available)
mod num_cpus {
    pub fn get() -> usize {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_decompress_roundtrip() {
        let original_data = b"Hello, GLifzip! This is a test of the compression system.";
        let config = CompressionConfig::default();

        let compressed = compress(original_data, &config).unwrap();
        let decompressed = decompress(&compressed, config.threads).unwrap();

        assert_eq!(original_data.as_slice(), decompressed.as_slice());
    }

    #[test]
    fn test_compress_deterministic() {
        let data = vec![42u8; 10_000];
        let config = CompressionConfig::default();

        let result1 = compress(&data, &config).unwrap();
        let result2 = compress(&data, &config).unwrap();

        assert_eq!(result1, result2, "Compression not deterministic");
    }

    #[test]
    fn test_verify_archive() {
        let data = b"Test data for archive verification";
        let config = CompressionConfig::default();

        let archive = compress(data, &config).unwrap();
        let sidecar = verify_archive(&archive).unwrap();

        assert_eq!(sidecar.payload.size, data.len() as u64);
    }

    #[test]
    fn test_large_data_compression() {
        // Test with 10 MB of data
        let data: Vec<u8> = (0..10 * 1024 * 1024).map(|i| (i % 256) as u8).collect();
        let config = CompressionConfig::fast();

        let compressed = compress(&data, &config).unwrap();
        let decompressed = decompress(&compressed, config.threads).unwrap();

        assert_eq!(data.len(), decompressed.len());
        assert_eq!(data, decompressed);
    }
}
