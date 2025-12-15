use sha2::{Sha256, Digest};
use std::io::{Result, Error, ErrorKind};

pub fn calculate_sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}

pub fn verify_sha256(data: &[u8], expected_hash: &[u8; 32]) -> Result<()> {
    let calculated_hash = calculate_sha256(data);

    if &calculated_hash == expected_hash {
        Ok(())
    } else {
        Err(Error::new(
            ErrorKind::InvalidData,
            format!(
                "SHA256 hash mismatch. Expected: {}, Got: {}",
                hex_encode(expected_hash),
                hex_encode(&calculated_hash)
            )
        ))
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_deterministic() {
        let data = b"Hello, GLifzip!";
        let hash1 = calculate_sha256(data);
        let hash2 = calculate_sha256(data);
        assert_eq!(hash1, hash2, "SHA256 calculation not deterministic");
    }

    #[test]
    fn test_sha256_verification_success() {
        let data = b"Test data for verification";
        let hash = calculate_sha256(data);
        assert!(verify_sha256(data, &hash).is_ok());
    }

    #[test]
    fn test_sha256_verification_failure() {
        let data = b"Test data for verification";
        let wrong_hash = [0u8; 32];
        assert!(verify_sha256(data, &wrong_hash).is_err());
    }
}
