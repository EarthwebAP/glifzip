use serde::{Deserialize, Serialize};
use std::io::{Read, Write, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlifSidecar {
    pub format: String,
    pub payload: PayloadInfo,
    pub archive: ArchiveInfo,
    pub cryptography: CryptographyInfo,
    pub metadata: MetadataInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayloadInfo {
    pub size: u64,
    pub hash: String,
    pub compression_ratio: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub directories: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveInfo {
    pub size: u64,
    pub hash: String,
    pub compressed_with: String,
    pub decompressed_with: String,
    pub compression_level: u32,
    pub threads: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptographyInfo {
    pub algorithm: String,
    pub payload_digest: String,
    pub archive_digest: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataInfo {
    pub created: String,
    pub creator: String,
    pub source_platform: String,
    pub source_architecture: String,
    pub deterministic: bool,
}

impl GlifSidecar {
    pub fn new(
        payload_size: u64,
        archive_size: u64,
        payload_hash: &[u8; 32],
        archive_hash: &[u8; 32],
        compression_level: u32,
        threads: u32,
        decompression_mode: u32,
    ) -> Self {
        Self::new_with_timestamp(
            payload_size,
            archive_size,
            payload_hash,
            archive_hash,
            compression_level,
            threads,
            decompression_mode,
            None,
        )
    }

    pub fn new_with_timestamp(
        payload_size: u64,
        archive_size: u64,
        payload_hash: &[u8; 32],
        archive_hash: &[u8; 32],
        compression_level: u32,
        threads: u32,
        decompression_mode: u32,
        timestamp: Option<String>,
    ) -> Self {
        let payload_hash_str = hex_encode(payload_hash);
        let archive_hash_str = hex_encode(archive_hash);

        let compression_ratio = if payload_size > 0 {
            archive_size as f32 / payload_size as f32
        } else {
            0.0
        };

        let timestamp = timestamp.unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
        let platform = std::env::consts::OS.to_string();
        let arch = std::env::consts::ARCH.to_string();

        let decompressed_with = if decompression_mode == 0 {
            "lz4".to_string()
        } else {
            "zstd".to_string()
        };

        Self {
            format: "glif/1.0".to_string(),
            payload: PayloadInfo {
                size: payload_size,
                hash: format!("sha256:{}", payload_hash_str),
                compression_ratio,
                files: None,
                directories: None,
            },
            archive: ArchiveInfo {
                size: archive_size,
                hash: format!("sha256:{}", archive_hash_str),
                compressed_with: "zstd".to_string(),
                decompressed_with,
                compression_level,
                threads,
            },
            cryptography: CryptographyInfo {
                algorithm: "sha256".to_string(),
                payload_digest: payload_hash_str.clone(),
                archive_digest: archive_hash_str.clone(),
                signature: None,
            },
            metadata: MetadataInfo {
                created: timestamp,
                creator: "glifzip v1.0".to_string(),
                source_platform: platform,
                source_architecture: arch,
                deterministic: true,
            },
        }
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        let json = self.to_json()?;
        writer.write_all(json.as_bytes())?;
        Ok(())
    }

    pub fn read<R: Read>(reader: &mut R, size: u16) -> Result<Self> {
        let mut buffer = vec![0u8; size as usize];
        reader.read_exact(&mut buffer)?;
        let json = String::from_utf8(buffer)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Self::from_json(&json)
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sidecar_json_roundtrip() {
        let payload_hash = [1u8; 32];
        let archive_hash = [2u8; 32];

        let sidecar = GlifSidecar::new(1000000, 500000, &payload_hash, &archive_hash, 8, 8, 0);
        let json = sidecar.to_json().unwrap();
        let parsed = GlifSidecar::from_json(&json).unwrap();

        assert_eq!(sidecar.payload.size, parsed.payload.size);
        assert_eq!(sidecar.archive.size, parsed.archive.size);
        assert_eq!(sidecar.archive.compression_level, parsed.archive.compression_level);
    }
}
