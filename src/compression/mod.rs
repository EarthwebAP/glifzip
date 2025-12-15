pub mod zstd_compressor;
pub mod lz4_decompressor;

pub use zstd_compressor::{compress_zstd, compress_zstd_multithreaded, decompress_zstd, decompress_zstd_multithreaded};
pub use lz4_decompressor::{compress_lz4, compress_lz4_multithreaded, decompress_lz4, decompress_lz4_multithreaded};

// Chunk size for multi-threaded processing (128 MB)
pub const CHUNK_SIZE: usize = 128 * 1024 * 1024;

// Default compression level (balanced)
pub const DEFAULT_COMPRESSION_LEVEL: i32 = 8;
