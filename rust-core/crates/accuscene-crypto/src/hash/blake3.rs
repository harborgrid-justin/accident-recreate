//! BLAKE3 cryptographic hash function
//!
//! Provides high-performance BLAKE3 hashing.

use crate::error::CryptoResult;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

/// Hash data using BLAKE3
pub fn blake3_hash(data: &[u8]) -> [u8; 32] {
    let hash = blake3::hash(data);
    *hash.as_bytes()
}

/// Hash a file using BLAKE3
pub fn blake3_hash_file<P: AsRef<Path>>(path: P) -> CryptoResult<[u8; 32]> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = blake3::Hasher::new();

    let mut buffer = [0u8; 8192];
    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    Ok(*hasher.finalize().as_bytes())
}

/// BLAKE3 hasher for incremental hashing
pub struct Blake3Hasher {
    hasher: blake3::Hasher,
}

impl Blake3Hasher {
    /// Create a new BLAKE3 hasher
    pub fn new() -> Self {
        Self {
            hasher: blake3::Hasher::new(),
        }
    }

    /// Create a new BLAKE3 hasher with a key for keyed hashing
    pub fn new_keyed(key: &[u8; 32]) -> Self {
        Self {
            hasher: blake3::Hasher::new_keyed(key),
        }
    }

    /// Create a new BLAKE3 hasher in key derivation mode
    pub fn new_derive_key(context: &str) -> Self {
        Self {
            hasher: blake3::Hasher::new_derive_key(context),
        }
    }

    /// Update the hasher with more data
    pub fn update(&mut self, data: &[u8]) {
        self.hasher.update(data);
    }

    /// Finalize the hash and return the result
    pub fn finalize(self) -> [u8; 32] {
        *self.hasher.finalize().as_bytes()
    }

    /// Finalize the hash and return as hex string
    pub fn finalize_hex(self) -> String {
        self.hasher.finalize().to_hex().to_string()
    }

    /// Finalize and return a variable-length output
    pub fn finalize_variable(self, length: usize) -> Vec<u8> {
        let mut output = vec![0u8; length];
        self.hasher
            .finalize_xof()
            .fill(&mut output);
        output
    }
}

impl Default for Blake3Hasher {
    fn default() -> Self {
        Self::new()
    }
}

/// Keyed hash using BLAKE3 (for MAC/authentication)
pub fn blake3_keyed_hash(key: &[u8; 32], data: &[u8]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new_keyed(key);
    hasher.update(data);
    *hasher.finalize().as_bytes()
}

/// Derive a key using BLAKE3 key derivation
pub fn blake3_derive_key(context: &str, key_material: &[u8]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new_derive_key(context);
    hasher.update(key_material);
    *hasher.finalize().as_bytes()
}

/// Convert a hash to a hexadecimal string
pub fn hash_to_hex(hash: &[u8]) -> String {
    hex::encode(hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_blake3_hash() {
        let data = b"hello world";
        let hash = blake3_hash(data);
        // BLAKE3 produces consistent output
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_blake3_hash_empty() {
        let data = b"";
        let hash = blake3_hash(data);
        let expected = "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262";
        assert_eq!(hash_to_hex(&hash), expected);
    }

    #[test]
    fn test_blake3_hasher() {
        let mut hasher = Blake3Hasher::new();
        hasher.update(b"hello ");
        hasher.update(b"world");
        let hash = hasher.finalize();
        assert_eq!(hash.len(), 32);

        // Verify it matches single-shot hash
        let expected = blake3_hash(b"hello world");
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_blake3_keyed_hash() {
        let key = [42u8; 32];
        let data = b"test data";
        let hash1 = blake3_keyed_hash(&key, data);
        let hash2 = blake3_keyed_hash(&key, data);
        assert_eq!(hash1, hash2);

        // Different key should produce different hash
        let different_key = [43u8; 32];
        let hash3 = blake3_keyed_hash(&different_key, data);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_blake3_derive_key() {
        let context = "test-context";
        let key_material = b"secret key material";
        let key1 = blake3_derive_key(context, key_material);
        let key2 = blake3_derive_key(context, key_material);
        assert_eq!(key1, key2);

        // Different context should produce different key
        let key3 = blake3_derive_key("different-context", key_material);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_blake3_variable_length() {
        let mut hasher = Blake3Hasher::new();
        hasher.update(b"test data");
        let output = hasher.finalize_variable(64);
        assert_eq!(output.len(), 64);
    }

    #[test]
    fn test_blake3_hash_file() {
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        temp_file.write_all(b"hello world").unwrap();
        temp_file.flush().unwrap();

        let hash = blake3_hash_file(temp_file.path()).unwrap();
        let expected = blake3_hash(b"hello world");
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_finalize_hex() {
        let mut hasher = Blake3Hasher::new();
        hasher.update(b"");
        let hex = hasher.finalize_hex();
        assert_eq!(hex, "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262");
    }
}
