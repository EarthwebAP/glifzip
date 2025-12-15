use rayon::prelude::*;
use std::io::Result;

use super::CHUNK_SIZE;

pub fn compress_zstd(data: &[u8], level: i32) -> Result<Vec<u8>> {
    zstd::encode_all(data, level)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}

pub fn compress_zstd_multithreaded(data: &[u8], level: i32, threads: usize) -> Result<Vec<u8>> {
    if data.len() <= CHUNK_SIZE || threads <= 1 {
        return compress_zstd(data, level);
    }

    // Build thread pool with specified number of threads
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    // Split data into chunks
    let chunks: Vec<&[u8]> = data.chunks(CHUNK_SIZE).collect();

    // Compress chunks in parallel
    let compressed_chunks: Result<Vec<Vec<u8>>> = pool.install(|| {
        chunks
            .par_iter()
            .map(|chunk| compress_zstd(chunk, level))
            .collect()
    });

    let compressed_chunks = compressed_chunks?;

    // Build the final compressed data with chunk metadata
    let mut result = Vec::new();

    // Write number of chunks (4 bytes, big-endian)
    result.extend_from_slice(&(compressed_chunks.len() as u32).to_be_bytes());

    // Write each chunk with its size
    for chunk in &compressed_chunks {
        // Write chunk size (8 bytes, big-endian)
        result.extend_from_slice(&(chunk.len() as u64).to_be_bytes());
        // Write chunk data
        result.extend_from_slice(chunk);
    }

    Ok(result)
}

pub fn decompress_zstd(data: &[u8]) -> Result<Vec<u8>> {
    zstd::decode_all(data)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}

pub fn decompress_zstd_multithreaded(data: &[u8], threads: usize) -> Result<Vec<u8>> {
    if threads <= 1 {
        return decompress_zstd(data);
    }

    // Read number of chunks
    if data.len() < 4 {
        return decompress_zstd(data);
    }

    let num_chunks = u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as usize;

    if num_chunks == 0 {
        return decompress_zstd(data);
    }

    // Parse chunk metadata
    let mut offset = 4;
    let mut chunk_infos = Vec::new();

    for _ in 0..num_chunks {
        if offset + 8 > data.len() {
            return decompress_zstd(data);
        }

        let chunk_size = u64::from_be_bytes([
            data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
            data[offset + 4], data[offset + 5], data[offset + 6], data[offset + 7],
        ]) as usize;
        offset += 8;

        if offset + chunk_size > data.len() {
            return decompress_zstd(data);
        }

        chunk_infos.push((offset, chunk_size));
        offset += chunk_size;
    }

    // Build thread pool
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    // Decompress chunks in parallel
    let decompressed_chunks: Result<Vec<Vec<u8>>> = pool.install(|| {
        chunk_infos
            .par_iter()
            .map(|(offset, size)| {
                let chunk = &data[*offset..*offset + *size];
                decompress_zstd(chunk)
            })
            .collect()
    });

    let decompressed_chunks = decompressed_chunks?;

    // Merge decompressed chunks
    let total_size: usize = decompressed_chunks.iter().map(|c| c.len()).sum();
    let mut result = Vec::with_capacity(total_size);

    for chunk in decompressed_chunks {
        result.extend_from_slice(&chunk);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zstd_compression_roundtrip() {
        let data = b"Hello, GLifzip! This is a test of Zstd compression.";
        let compressed = compress_zstd(data, 8).unwrap();
        let decompressed = decompress_zstd(&compressed).unwrap();
        assert_eq!(data.as_slice(), decompressed.as_slice());
    }

    #[test]
    fn test_multithreaded_compression_roundtrip() {
        // Create large enough data to trigger multithreading
        let data: Vec<u8> = (0..256 * 1024 * 1024).map(|i| (i % 256) as u8).collect();

        let compressed = compress_zstd_multithreaded(&data, 3, 4).unwrap();
        let decompressed = decompress_zstd_multithreaded(&compressed, 4).unwrap();

        assert_eq!(data.len(), decompressed.len());
        assert_eq!(data, decompressed);
    }

    #[test]
    fn test_deterministic_compression() {
        let data = vec![42u8; 10_000];
        let result1 = compress_zstd(&data, 8).unwrap();
        let result2 = compress_zstd(&data, 8).unwrap();
        assert_eq!(result1, result2, "Compression not deterministic");
    }
}
