use rayon::prelude::*;
use std::io::Result;

use super::CHUNK_SIZE;

pub fn compress_lz4(data: &[u8]) -> Result<Vec<u8>> {
    lz4::block::compress(data, None, false)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}

pub fn decompress_lz4(data: &[u8], uncompressed_size: Option<usize>) -> Result<Vec<u8>> {
    let size = uncompressed_size.unwrap_or_else(|| {
        // Use a larger estimate for safety - LZ4 can compress very well
        // so decompressed size could be much larger than compressed size
        std::cmp::max(data.len() * 100, 1024 * 1024 * 1024) // At least 1GB buffer
    });
    lz4::block::decompress(data, Some(size as i32))
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}

pub fn compress_lz4_multithreaded(data: &[u8], threads: usize) -> Result<Vec<u8>> {
    if data.len() <= CHUNK_SIZE || threads <= 1 {
        return compress_lz4(data);
    }

    // Build thread pool
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
            .map(|chunk| compress_lz4(chunk))
            .collect()
    });

    let compressed_chunks = compressed_chunks?;

    // Build the final compressed data with chunk metadata
    let mut result = Vec::new();

    // Write number of chunks (4 bytes, big-endian)
    result.extend_from_slice(&(compressed_chunks.len() as u32).to_be_bytes());

    // Write original chunk size (for decompression sizing)
    result.extend_from_slice(&(CHUNK_SIZE as u64).to_be_bytes());

    // Write total uncompressed size
    result.extend_from_slice(&(data.len() as u64).to_be_bytes());

    // Write each chunk with its size
    for chunk in &compressed_chunks {
        // Write chunk size (8 bytes, big-endian)
        result.extend_from_slice(&(chunk.len() as u64).to_be_bytes());
        // Write chunk data
        result.extend_from_slice(chunk);
    }

    Ok(result)
}

pub fn decompress_lz4_multithreaded(data: &[u8], threads: usize) -> Result<Vec<u8>> {
    if threads <= 1 || data.len() < 20 {
        return decompress_lz4(data, None);
    }

    // Try to read chunk metadata
    let num_chunks = u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as usize;

    if num_chunks == 0 {
        return decompress_lz4(data, None);
    }

    let chunk_size = u64::from_be_bytes([
        data[4], data[5], data[6], data[7],
        data[8], data[9], data[10], data[11],
    ]) as usize;

    let total_size = u64::from_be_bytes([
        data[12], data[13], data[14], data[15],
        data[16], data[17], data[18], data[19],
    ]) as usize;

    // Parse chunk metadata
    let mut offset = 20;
    let mut chunk_infos = Vec::new();

    for i in 0..num_chunks {
        if offset + 8 > data.len() {
            return decompress_lz4(data, None);
        }

        let compressed_size = u64::from_be_bytes([
            data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
            data[offset + 4], data[offset + 5], data[offset + 6], data[offset + 7],
        ]) as usize;
        offset += 8;

        if offset + compressed_size > data.len() {
            return decompress_lz4(data, None);
        }

        // Calculate uncompressed size for this chunk (last chunk might be smaller)
        let uncompressed_size = if i == num_chunks - 1 {
            total_size - (i * chunk_size)
        } else {
            chunk_size
        };

        chunk_infos.push((offset, compressed_size, uncompressed_size));
        offset += compressed_size;
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
            .map(|(offset, compressed_size, uncompressed_size)| {
                let chunk = &data[*offset..*offset + *compressed_size];
                decompress_lz4(chunk, Some(*uncompressed_size))
            })
            .collect()
    });

    let decompressed_chunks = decompressed_chunks?;

    // Merge decompressed chunks
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
    fn test_lz4_compression_roundtrip() {
        let data = b"Hello, GLifzip! This is a test of LZ4 compression.";
        let compressed = compress_lz4(data).unwrap();
        let decompressed = decompress_lz4(&compressed, Some(data.len())).unwrap();
        assert_eq!(data.as_slice(), decompressed.as_slice());
    }

    #[test]
    fn test_multithreaded_lz4_roundtrip() {
        // Create large enough data to trigger multithreading
        let data: Vec<u8> = (0..256 * 1024 * 1024).map(|i| (i % 256) as u8).collect();

        let compressed = compress_lz4_multithreaded(&data, 4).unwrap();
        let decompressed = decompress_lz4_multithreaded(&compressed, 4).unwrap();

        assert_eq!(data.len(), decompressed.len());
        assert_eq!(data, decompressed);
    }
}
