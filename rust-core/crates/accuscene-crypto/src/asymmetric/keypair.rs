//! Ed25519 key pair generation and management
//!
//! Provides Ed25519 public/private key pair functionality.

use crate::error::{CryptoError, CryptoResult};
use crate::random::SecureRng;
use ed25519_dalek::{SigningKey, VerifyingKey, SECRET_KEY_LENGTH};
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

/// Ed25519 public key (32 bytes)
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ed25519PublicKey {
    bytes: [u8; 32],
}

impl Ed25519PublicKey {
    /// Create a public key from bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self { bytes }
    }

    /// Create a public key from a slice
    pub fn from_slice(bytes: &[u8]) -> CryptoResult<Self> {
        if bytes.len() != 32 {
            return Err(CryptoError::InvalidKeySize {
                expected: 32,
                actual: bytes.len(),
            });
        }
        let mut array = [0u8; 32];
        array.copy_from_slice(bytes);
        Ok(Self { bytes: array })
    }

    /// Get the key bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.bytes
    }

    /// Convert to a byte array
    pub fn to_bytes(&self) -> [u8; 32] {
        self.bytes
    }

    /// Encode to base64
    pub fn to_base64(&self) -> String {
        base64::encode(self.bytes)
    }

    /// Decode from base64
    pub fn from_base64(encoded: &str) -> CryptoResult<Self> {
        let bytes = base64::decode(encoded)?;
        Self::from_slice(&bytes)
    }

    /// Convert to Ed25519-dalek verifying key
    pub(crate) fn to_dalek_verifying_key(&self) -> CryptoResult<VerifyingKey> {
        VerifyingKey::from_bytes(&self.bytes)
            .map_err(|e| CryptoError::InvalidInput(e.to_string()))
    }
}

impl AsRef<[u8]> for Ed25519PublicKey {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

/// Ed25519 secret key (32 bytes) - zeroized on drop
#[derive(Clone, Zeroize)]
#[zeroize(drop)]
pub struct Ed25519SecretKey {
    bytes: [u8; SECRET_KEY_LENGTH],
}

impl Ed25519SecretKey {
    /// Create a secret key from bytes
    pub fn from_bytes(bytes: [u8; SECRET_KEY_LENGTH]) -> Self {
        Self { bytes }
    }

    /// Create a secret key from a slice
    pub fn from_slice(bytes: &[u8]) -> CryptoResult<Self> {
        if bytes.len() != SECRET_KEY_LENGTH {
            return Err(CryptoError::InvalidKeySize {
                expected: SECRET_KEY_LENGTH,
                actual: bytes.len(),
            });
        }
        let mut array = [0u8; SECRET_KEY_LENGTH];
        array.copy_from_slice(bytes);
        Ok(Self { bytes: array })
    }

    /// Get the key bytes
    pub fn as_bytes(&self) -> &[u8; SECRET_KEY_LENGTH] {
        &self.bytes
    }

    /// Convert to a byte array
    pub fn to_bytes(&self) -> [u8; SECRET_KEY_LENGTH] {
        self.bytes
    }

    /// Encode to base64 (use with caution - secret key!)
    pub fn to_base64(&self) -> String {
        base64::encode(self.bytes)
    }

    /// Decode from base64
    pub fn from_base64(encoded: &str) -> CryptoResult<Self> {
        let bytes = base64::decode(encoded)?;
        Self::from_slice(&bytes)
    }

    /// Convert to Ed25519-dalek signing key
    pub(crate) fn to_dalek_signing_key(&self) -> SigningKey {
        SigningKey::from_bytes(&self.bytes)
    }
}

impl std::fmt::Debug for Ed25519SecretKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ed25519SecretKey([REDACTED])")
    }
}

/// Ed25519 key pair
#[derive(Clone)]
pub struct Ed25519KeyPair {
    secret_key: Ed25519SecretKey,
    public_key: Ed25519PublicKey,
}

impl Ed25519KeyPair {
    /// Generate a new random Ed25519 key pair
    pub fn generate() -> CryptoResult<Self> {
        let mut rng = SecureRng::new()?;
        let signing_key = SigningKey::generate(&mut rng);
        let verifying_key = signing_key.verifying_key();

        Ok(Self {
            secret_key: Ed25519SecretKey::from_bytes(signing_key.to_bytes()),
            public_key: Ed25519PublicKey::from_bytes(verifying_key.to_bytes()),
        })
    }

    /// Create a key pair from a secret key
    pub fn from_secret_key(secret_key: Ed25519SecretKey) -> CryptoResult<Self> {
        let signing_key = secret_key.to_dalek_signing_key();
        let verifying_key = signing_key.verifying_key();

        Ok(Self {
            secret_key,
            public_key: Ed25519PublicKey::from_bytes(verifying_key.to_bytes()),
        })
    }

    /// Get the public key
    pub fn public_key(&self) -> &Ed25519PublicKey {
        &self.public_key
    }

    /// Get the secret key
    pub fn secret_key(&self) -> &Ed25519SecretKey {
        &self.secret_key
    }

    /// Export the key pair as base64 (secret and public)
    pub fn to_base64(&self) -> (String, String) {
        (self.secret_key.to_base64(), self.public_key.to_base64())
    }

    /// Import a key pair from base64 (secret and public)
    pub fn from_base64(secret_b64: &str, public_b64: &str) -> CryptoResult<Self> {
        let secret_key = Ed25519SecretKey::from_base64(secret_b64)?;
        let public_key = Ed25519PublicKey::from_base64(public_b64)?;

        // Verify that the public key matches the secret key
        let derived_public = PublicKey::from(&secret_key.to_dalek_secret_key());
        if derived_public.to_bytes() != public_key.to_bytes() {
            return Err(CryptoError::InvalidInput(
                "Public key does not match secret key".to_string(),
            ));
        }

        Ok(Self {
            secret_key,
            public_key,
        })
    }
}

impl std::fmt::Debug for Ed25519KeyPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ed25519KeyPair")
            .field("public_key", &self.public_key)
            .field("secret_key", &"[REDACTED]")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_keypair() {
        let keypair = Ed25519KeyPair::generate().unwrap();
        assert_eq!(keypair.public_key().as_bytes().len(), 32);
        assert_eq!(keypair.secret_key().as_bytes().len(), SECRET_KEY_LENGTH);
    }

    #[test]
    fn test_public_key_base64_roundtrip() {
        let keypair = Ed25519KeyPair::generate().unwrap();
        let encoded = keypair.public_key().to_base64();
        let decoded = Ed25519PublicKey::from_base64(&encoded).unwrap();
        assert_eq!(keypair.public_key().as_bytes(), decoded.as_bytes());
    }

    #[test]
    fn test_secret_key_base64_roundtrip() {
        let keypair = Ed25519KeyPair::generate().unwrap();
        let encoded = keypair.secret_key().to_base64();
        let decoded = Ed25519SecretKey::from_base64(&encoded).unwrap();
        assert_eq!(keypair.secret_key().as_bytes(), decoded.as_bytes());
    }

    #[test]
    fn test_keypair_base64_roundtrip() {
        let keypair = Ed25519KeyPair::generate().unwrap();
        let (secret_b64, public_b64) = keypair.to_base64();
        let restored = Ed25519KeyPair::from_base64(&secret_b64, &public_b64).unwrap();

        assert_eq!(
            keypair.public_key().as_bytes(),
            restored.public_key().as_bytes()
        );
        assert_eq!(
            keypair.secret_key().as_bytes(),
            restored.secret_key().as_bytes()
        );
    }

    #[test]
    fn test_keypair_from_secret() {
        let keypair1 = Ed25519KeyPair::generate().unwrap();
        let secret = keypair1.secret_key().clone();
        let keypair2 = Ed25519KeyPair::from_secret_key(secret).unwrap();

        assert_eq!(
            keypair1.public_key().as_bytes(),
            keypair2.public_key().as_bytes()
        );
    }

    #[test]
    fn test_invalid_public_key_size() {
        let bytes = vec![0u8; 16]; // Wrong size
        let result = Ed25519PublicKey::from_slice(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_mismatched_keypair() {
        let keypair1 = Ed25519KeyPair::generate().unwrap();
        let keypair2 = Ed25519KeyPair::generate().unwrap();

        let result = Ed25519KeyPair::from_base64(
            &keypair1.secret_key().to_base64(),
            &keypair2.public_key().to_base64(),
        );

        assert!(result.is_err());
    }
}
