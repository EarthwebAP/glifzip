use std::io::{Read, Write, Result, Error, ErrorKind};

pub const MAGIC_NUMBER: &[u8; 6] = b"GLIF01";
pub const GLIF_VERSION: u32 = 0x00000100; // v1.0
pub const HEADER_SIZE: usize = 116;

#[derive(Debug, Clone)]
pub struct GlifHeader {
    pub payload_size: u64,
    pub archive_size: u64,
    pub payload_hash: [u8; 32],
    pub archive_hash: [u8; 32],
    pub compression_level: u32,
    pub decompression_mode: u32, // 0=LZ4, 1=Zstd
    pub cores_used: u32,
    pub timestamp: u64,
    pub sidecar_size: u16,
}

impl GlifHeader {
    pub fn new(
        payload_size: u64,
        archive_size: u64,
        payload_hash: [u8; 32],
        archive_hash: [u8; 32],
        compression_level: u32,
        decompression_mode: u32,
        cores_used: u32,
        sidecar_size: u16,
    ) -> Self {
        Self::new_with_timestamp(
            payload_size,
            archive_size,
            payload_hash,
            archive_hash,
            compression_level,
            decompression_mode,
            cores_used,
            sidecar_size,
            None,
        )
    }

    pub fn new_with_timestamp(
        payload_size: u64,
        archive_size: u64,
        payload_hash: [u8; 32],
        archive_hash: [u8; 32],
        compression_level: u32,
        decompression_mode: u32,
        cores_used: u32,
        sidecar_size: u16,
        timestamp: Option<u64>,
    ) -> Self {
        let timestamp = timestamp.unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        });

        Self {
            payload_size,
            archive_size,
            payload_hash,
            archive_hash,
            compression_level,
            decompression_mode,
            cores_used,
            timestamp,
            sidecar_size,
        }
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        // Magic number (6 bytes)
        writer.write_all(MAGIC_NUMBER)?;

        // Version (4 bytes, big-endian)
        writer.write_all(&GLIF_VERSION.to_be_bytes())?;

        // Payload size (8 bytes, big-endian)
        writer.write_all(&self.payload_size.to_be_bytes())?;

        // Archive size (8 bytes, big-endian)
        writer.write_all(&self.archive_size.to_be_bytes())?;

        // Payload hash (32 bytes)
        writer.write_all(&self.payload_hash)?;

        // Archive hash (32 bytes)
        writer.write_all(&self.archive_hash)?;

        // Compression level (4 bytes, big-endian)
        writer.write_all(&self.compression_level.to_be_bytes())?;

        // Decompression mode (4 bytes, big-endian)
        writer.write_all(&self.decompression_mode.to_be_bytes())?;

        // Cores used (4 bytes, big-endian)
        writer.write_all(&self.cores_used.to_be_bytes())?;

        // Timestamp (8 bytes, big-endian)
        writer.write_all(&self.timestamp.to_be_bytes())?;

        // Calculate and write Adler-32 checksum (4 bytes)
        let checksum = self.calculate_checksum();
        writer.write_all(&checksum.to_be_bytes())?;

        // Sidecar size (2 bytes, big-endian)
        writer.write_all(&self.sidecar_size.to_be_bytes())?;

        Ok(())
    }

    pub fn read<R: Read>(reader: &mut R) -> Result<Self> {
        // Read and verify magic number
        let mut magic = [0u8; 6];
        reader.read_exact(&mut magic)?;
        if &magic != MAGIC_NUMBER {
            return Err(Error::new(ErrorKind::InvalidData, "Invalid GLIF magic number"));
        }

        // Read version
        let mut version_bytes = [0u8; 4];
        reader.read_exact(&mut version_bytes)?;
        let version = u32::from_be_bytes(version_bytes);
        if version != GLIF_VERSION {
            return Err(Error::new(ErrorKind::InvalidData, "Unsupported GLIF version"));
        }

        // Read payload size
        let mut payload_size_bytes = [0u8; 8];
        reader.read_exact(&mut payload_size_bytes)?;
        let payload_size = u64::from_be_bytes(payload_size_bytes);

        // Read archive size
        let mut archive_size_bytes = [0u8; 8];
        reader.read_exact(&mut archive_size_bytes)?;
        let archive_size = u64::from_be_bytes(archive_size_bytes);

        // Read payload hash
        let mut payload_hash = [0u8; 32];
        reader.read_exact(&mut payload_hash)?;

        // Read archive hash
        let mut archive_hash = [0u8; 32];
        reader.read_exact(&mut archive_hash)?;

        // Read compression level
        let mut compression_level_bytes = [0u8; 4];
        reader.read_exact(&mut compression_level_bytes)?;
        let compression_level = u32::from_be_bytes(compression_level_bytes);

        // Read decompression mode
        let mut decompression_mode_bytes = [0u8; 4];
        reader.read_exact(&mut decompression_mode_bytes)?;
        let decompression_mode = u32::from_be_bytes(decompression_mode_bytes);

        // Read cores used
        let mut cores_used_bytes = [0u8; 4];
        reader.read_exact(&mut cores_used_bytes)?;
        let cores_used = u32::from_be_bytes(cores_used_bytes);

        // Read timestamp
        let mut timestamp_bytes = [0u8; 8];
        reader.read_exact(&mut timestamp_bytes)?;
        let timestamp = u64::from_be_bytes(timestamp_bytes);

        // Read and verify checksum
        let mut checksum_bytes = [0u8; 4];
        reader.read_exact(&mut checksum_bytes)?;
        let stored_checksum = u32::from_be_bytes(checksum_bytes);

        // Read sidecar size
        let mut sidecar_size_bytes = [0u8; 2];
        reader.read_exact(&mut sidecar_size_bytes)?;
        let sidecar_size = u16::from_be_bytes(sidecar_size_bytes);

        let header = Self {
            payload_size,
            archive_size,
            payload_hash,
            archive_hash,
            compression_level,
            decompression_mode,
            cores_used,
            timestamp,
            sidecar_size,
        };

        // Verify checksum
        let calculated_checksum = header.calculate_checksum();
        if calculated_checksum != stored_checksum {
            return Err(Error::new(ErrorKind::InvalidData, "Header checksum mismatch"));
        }

        Ok(header)
    }

    fn calculate_checksum(&self) -> u32 {
        let mut data = Vec::new();
        data.extend_from_slice(&self.payload_size.to_be_bytes());
        data.extend_from_slice(&self.archive_size.to_be_bytes());
        data.extend_from_slice(&self.payload_hash);
        data.extend_from_slice(&self.archive_hash);
        data.extend_from_slice(&self.compression_level.to_be_bytes());
        data.extend_from_slice(&self.decompression_mode.to_be_bytes());
        data.extend_from_slice(&self.cores_used.to_be_bytes());
        data.extend_from_slice(&self.timestamp.to_be_bytes());

        adler::adler32_slice(&data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_header_roundtrip() {
        let header = GlifHeader::new(
            1000000,
            500000,
            [1u8; 32],
            [2u8; 32],
            8,
            0,
            8,
            100,
        );

        let mut buffer = Vec::new();
        header.write(&mut buffer).unwrap();

        let mut cursor = Cursor::new(buffer);
        let read_header = GlifHeader::read(&mut cursor).unwrap();

        assert_eq!(header.payload_size, read_header.payload_size);
        assert_eq!(header.archive_size, read_header.archive_size);
        assert_eq!(header.payload_hash, read_header.payload_hash);
        assert_eq!(header.archive_hash, read_header.archive_hash);
        assert_eq!(header.compression_level, read_header.compression_level);
        assert_eq!(header.decompression_mode, read_header.decompression_mode);
        assert_eq!(header.cores_used, read_header.cores_used);
        assert_eq!(header.sidecar_size, read_header.sidecar_size);
    }
}
