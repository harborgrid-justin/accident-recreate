//! Symmetric key management
//!
//! Provides secure symmetric key generation and storage.

use crate::error::CryptoResult;
use crate::random::generate_random_bytes;
use crate::secure_memory::SecureKey;

/// 128-bit (16 byte) symmetric key
pub type SymmetricKey128 = SecureKey<16>;

/// 256-bit (32 byte) symmetric key
pub type SymmetricKey256 = SecureKey<32>;

/// Symmetric encryption key
#[derive(Clone, Debug)]
pub struct SymmetricKey {
    key: SymmetricKey256,
}

impl SymmetricKey {
    /// Generate a new random 256-bit symmetric key
    pub fn generate() -> CryptoResult<Self> {
        let bytes = generate_random_bytes(32)?;
        let key = SymmetricKey256::from_slice(bytes.as_bytes())?;
        Ok(Self { key })
    }

    /// Create a symmetric key from bytes
    pub fn from_bytes(bytes: &[u8]) -> CryptoResult<Self> {
        let key = SymmetricKey256::from_slice(bytes)?;
        Ok(Self { key })
    }

    /// Create a symmetric key from a fixed-size array
    pub fn from_array(bytes: [u8; 32]) -> Self {
        Self {
            key: SymmetricKey256::new(bytes),
        }
    }

    /// Get the key bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        self.key.as_bytes()
    }

    /// Convert to a byte array
    pub fn into_bytes(self) -> [u8; 32] {
        self.key.into_bytes()
    }

    /// Export the key as base64
    pub fn to_base64(&self) -> String {
        base64::encode(self.key.as_bytes())
    }

    /// Import a key from base64
    pub fn from_base64(encoded: &str) -> CryptoResult<Self> {
        let bytes = base64::decode(encoded)?;
        Self::from_bytes(&bytes)
    }
}

impl AsRef<[u8]> for SymmetricKey {
    fn as_ref(&self) -> &[u8] {
        self.key.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_symmetric_key() {
        let key = SymmetricKey::generate().unwrap();
        assert_eq!(key.as_bytes().len(), 32);
    }

    #[test]
    fn test_key_from_bytes() {
        let bytes = [42u8; 32];
        let key = SymmetricKey::from_bytes(&bytes).unwrap();
        assert_eq!(key.as_bytes(), &bytes);
    }

    #[test]
    fn test_key_from_array() {
        let bytes = [42u8; 32];
        let key = SymmetricKey::from_array(bytes);
        assert_eq!(key.as_bytes(), &bytes);
    }

    #[test]
    fn test_key_base64_roundtrip() {
        let key = SymmetricKey::generate().unwrap();
        let encoded = key.to_base64();
        let decoded = SymmetricKey::from_base64(&encoded).unwrap();
        assert_eq!(key.as_bytes(), decoded.as_bytes());
    }

    #[test]
    fn test_invalid_key_size() {
        let bytes = [42u8; 16]; // Too short
        let result = SymmetricKey::from_bytes(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_key_128() {
        let key = SymmetricKey128::new([42u8; 16]);
        assert_eq!(key.as_bytes().len(), 16);
    }

    #[test]
    fn test_key_256() {
        let key = SymmetricKey256::new([42u8; 32]);
        assert_eq!(key.as_bytes().len(), 32);
    }
}
