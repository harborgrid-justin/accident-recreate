//! SHA-2 family cryptographic hash functions
//!
//! Provides SHA-256 and SHA-512 hashing implementations.

use crate::error::{CryptoError, CryptoResult};
use sha2::{Digest, Sha256, Sha512};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

/// Hash data using SHA-256
pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Hash a file using SHA-256
pub fn sha256_file<P: AsRef<Path>>(path: P) -> CryptoResult<[u8; 32]> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();

    let mut buffer = [0u8; 8192];
    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    Ok(hasher.finalize().into())
}

/// Hash data using SHA-512
pub fn sha512(data: &[u8]) -> [u8; 64] {
    let mut hasher = Sha512::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Hash a file using SHA-512
pub fn sha512_file<P: AsRef<Path>>(path: P) -> CryptoResult<[u8; 64]> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha512::new();

    let mut buffer = [0u8; 8192];
    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    Ok(hasher.finalize().into())
}

/// SHA-256 hasher for incremental hashing
pub struct Sha256Hasher {
    hasher: Sha256,
}

impl Sha256Hasher {
    /// Create a new SHA-256 hasher
    pub fn new() -> Self {
        Self {
            hasher: Sha256::new(),
        }
    }

    /// Update the hasher with more data
    pub fn update(&mut self, data: &[u8]) {
        self.hasher.update(data);
    }

    /// Finalize the hash and return the result
    pub fn finalize(self) -> [u8; 32] {
        self.hasher.finalize().into()
    }

    /// Finalize the hash and return as hex string
    pub fn finalize_hex(self) -> String {
        format!("{:x}", self.hasher.finalize())
    }
}

impl Default for Sha256Hasher {
    fn default() -> Self {
        Self::new()
    }
}

/// SHA-512 hasher for incremental hashing
pub struct Sha512Hasher {
    hasher: Sha512,
}

impl Sha512Hasher {
    /// Create a new SHA-512 hasher
    pub fn new() -> Self {
        Self {
            hasher: Sha512::new(),
        }
    }

    /// Update the hasher with more data
    pub fn update(&mut self, data: &[u8]) {
        self.hasher.update(data);
    }

    /// Finalize the hash and return the result
    pub fn finalize(self) -> [u8; 64] {
        self.hasher.finalize().into()
    }

    /// Finalize the hash and return as hex string
    pub fn finalize_hex(self) -> String {
        format!("{:x}", self.hasher.finalize())
    }
}

impl Default for Sha512Hasher {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert a hash to a hexadecimal string
pub fn hash_to_hex(hash: &[u8]) -> String {
    hex::encode(hash)
}

/// Parse a hexadecimal string into a hash
pub fn hex_to_hash(hex_str: &str) -> CryptoResult<Vec<u8>> {
    hex::decode(hex_str).map_err(|e| CryptoError::DecodingError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_sha256() {
        let data = b"hello world";
        let hash = sha256(data);
        let expected = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
        assert_eq!(hash_to_hex(&hash), expected);
    }

    #[test]
    fn test_sha256_empty() {
        let data = b"";
        let hash = sha256(data);
        let expected = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        assert_eq!(hash_to_hex(&hash), expected);
    }

    #[test]
    fn test_sha512() {
        let data = b"hello world";
        let hash = sha512(data);
        let expected = "309ecc489c12d6eb4cc40f50c902f2b4d0ed77ee511a7c7a9bcd3ca86d4cd86f989dd35bc5ff499670da34255b45b0cfd830e81f605dcf7dc5542e93ae9cd76f";
        assert_eq!(hash_to_hex(&hash), expected);
    }

    #[test]
    fn test_sha256_hasher() {
        let mut hasher = Sha256Hasher::new();
        hasher.update(b"hello ");
        hasher.update(b"world");
        let hash = hasher.finalize();
        let expected = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
        assert_eq!(hash_to_hex(&hash), expected);
    }

    #[test]
    fn test_sha512_hasher() {
        let mut hasher = Sha512Hasher::new();
        hasher.update(b"hello ");
        hasher.update(b"world");
        let hash = hasher.finalize();
        let expected = "309ecc489c12d6eb4cc40f50c902f2b4d0ed77ee511a7c7a9bcd3ca86d4cd86f989dd35bc5ff499670da34255b45b0cfd830e81f605dcf7dc5542e93ae9cd76f";
        assert_eq!(hash_to_hex(&hash), expected);
    }

    #[test]
    fn test_hash_to_hex_and_back() {
        let data = b"test data";
        let hash = sha256(data);
        let hex = hash_to_hex(&hash);
        let decoded = hex_to_hash(&hex).unwrap();
        assert_eq!(hash.as_slice(), decoded.as_slice());
    }

    #[test]
    fn test_sha256_file() {
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        temp_file.write_all(b"hello world").unwrap();
        temp_file.flush().unwrap();

        let hash = sha256_file(temp_file.path()).unwrap();
        let expected = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
        assert_eq!(hash_to_hex(&hash), expected);
    }
}
