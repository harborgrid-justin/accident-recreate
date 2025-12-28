//! X25519 Elliptic Curve Diffie-Hellman key exchange
//!
//! Provides secure key exchange using the X25519 algorithm.

use crate::error::{CryptoError, CryptoResult};
use crate::random::SecureRng;
use crate::secure_memory::SecureBytes;
use serde::{Deserialize, Serialize};
use x25519_dalek::{PublicKey, StaticSecret};
use zeroize::Zeroize;

/// X25519 public key (32 bytes)
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct X25519PublicKey {
    bytes: [u8; 32],
}

impl X25519PublicKey {
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

    /// Convert to X25519-dalek public key
    pub(crate) fn to_dalek_public_key(&self) -> PublicKey {
        PublicKey::from(self.bytes)
    }
}

impl AsRef<[u8]> for X25519PublicKey {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

/// X25519 secret key (32 bytes) - zeroized on drop
#[derive(Clone, Zeroize)]
#[zeroize(drop)]
pub struct X25519SecretKey {
    bytes: [u8; 32],
}

impl X25519SecretKey {
    /// Create a secret key from bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self { bytes }
    }

    /// Create a secret key from a slice
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

    /// Encode to base64 (use with caution - secret key!)
    pub fn to_base64(&self) -> String {
        base64::encode(self.bytes)
    }

    /// Decode from base64
    pub fn from_base64(encoded: &str) -> CryptoResult<Self> {
        let bytes = base64::decode(encoded)?;
        Self::from_slice(&bytes)
    }

    /// Convert to X25519-dalek static secret
    pub(crate) fn to_dalek_static_secret(&self) -> StaticSecret {
        StaticSecret::from(self.bytes)
    }
}

impl std::fmt::Debug for X25519SecretKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "X25519SecretKey([REDACTED])")
    }
}

/// X25519 key pair for key exchange
#[derive(Clone)]
pub struct X25519KeyPair {
    secret_key: X25519SecretKey,
    public_key: X25519PublicKey,
}

impl X25519KeyPair {
    /// Generate a new random X25519 key pair
    pub fn generate() -> CryptoResult<Self> {
        let mut rng = SecureRng::new()?;
        let secret = StaticSecret::random_from_rng(&mut rng);
        let public = PublicKey::from(&secret);

        Ok(Self {
            secret_key: X25519SecretKey::from_bytes(secret.to_bytes()),
            public_key: X25519PublicKey::from_bytes(public.to_bytes()),
        })
    }

    /// Create a key pair from a secret key
    pub fn from_secret_key(secret_key: X25519SecretKey) -> Self {
        let dalek_secret = secret_key.to_dalek_static_secret();
        let dalek_public = PublicKey::from(&dalek_secret);

        Self {
            secret_key,
            public_key: X25519PublicKey::from_bytes(dalek_public.to_bytes()),
        }
    }

    /// Get the public key
    pub fn public_key(&self) -> &X25519PublicKey {
        &self.public_key
    }

    /// Get the secret key
    pub fn secret_key(&self) -> &X25519SecretKey {
        &self.secret_key
    }

    /// Perform Diffie-Hellman key exchange with another party's public key
    pub fn exchange(&self, their_public: &X25519PublicKey) -> CryptoResult<SecureBytes> {
        let our_secret = self.secret_key.to_dalek_static_secret();
        let their_public_key = their_public.to_dalek_public_key();

        let shared_secret = our_secret.diffie_hellman(&their_public_key);
        Ok(SecureBytes::from_slice(shared_secret.as_bytes()))
    }

    /// Export the key pair as base64 (secret and public)
    pub fn to_base64(&self) -> (String, String) {
        (self.secret_key.to_base64(), self.public_key.to_base64())
    }

    /// Import a key pair from base64 (secret and public)
    pub fn from_base64(secret_b64: &str, public_b64: &str) -> CryptoResult<Self> {
        let secret_key = X25519SecretKey::from_base64(secret_b64)?;
        let public_key = X25519PublicKey::from_base64(public_b64)?;

        // Verify that the public key matches the secret key
        let derived_public = PublicKey::from(&secret_key.to_dalek_static_secret());
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

impl std::fmt::Debug for X25519KeyPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("X25519KeyPair")
            .field("public_key", &self.public_key)
            .field("secret_key", &"[REDACTED]")
            .finish()
    }
}

/// Perform a key exchange between two parties
pub fn perform_key_exchange(
    our_keypair: &X25519KeyPair,
    their_public: &X25519PublicKey,
) -> CryptoResult<SecureBytes> {
    our_keypair.exchange(their_public)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_keypair() {
        let keypair = X25519KeyPair::generate().unwrap();
        assert_eq!(keypair.public_key().as_bytes().len(), 32);
        assert_eq!(keypair.secret_key().as_bytes().len(), 32);
    }

    #[test]
    fn test_key_exchange() {
        let alice = X25519KeyPair::generate().unwrap();
        let bob = X25519KeyPair::generate().unwrap();

        let alice_shared = alice.exchange(bob.public_key()).unwrap();
        let bob_shared = bob.exchange(alice.public_key()).unwrap();

        // Both parties should derive the same shared secret
        assert_eq!(alice_shared.as_bytes(), bob_shared.as_bytes());
    }

    #[test]
    fn test_key_exchange_function() {
        let alice = X25519KeyPair::generate().unwrap();
        let bob = X25519KeyPair::generate().unwrap();

        let alice_shared = perform_key_exchange(&alice, bob.public_key()).unwrap();
        let bob_shared = perform_key_exchange(&bob, alice.public_key()).unwrap();

        assert_eq!(alice_shared.as_bytes(), bob_shared.as_bytes());
    }

    #[test]
    fn test_public_key_base64_roundtrip() {
        let keypair = X25519KeyPair::generate().unwrap();
        let encoded = keypair.public_key().to_base64();
        let decoded = X25519PublicKey::from_base64(&encoded).unwrap();
        assert_eq!(keypair.public_key().as_bytes(), decoded.as_bytes());
    }

    #[test]
    fn test_secret_key_base64_roundtrip() {
        let keypair = X25519KeyPair::generate().unwrap();
        let encoded = keypair.secret_key().to_base64();
        let decoded = X25519SecretKey::from_base64(&encoded).unwrap();
        assert_eq!(keypair.secret_key().as_bytes(), decoded.as_bytes());
    }

    #[test]
    fn test_keypair_base64_roundtrip() {
        let keypair = X25519KeyPair::generate().unwrap();
        let (secret_b64, public_b64) = keypair.to_base64();
        let restored = X25519KeyPair::from_base64(&secret_b64, &public_b64).unwrap();

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
        let keypair1 = X25519KeyPair::generate().unwrap();
        let secret = keypair1.secret_key().clone();
        let keypair2 = X25519KeyPair::from_secret_key(secret);

        assert_eq!(
            keypair1.public_key().as_bytes(),
            keypair2.public_key().as_bytes()
        );
    }

    #[test]
    fn test_different_keypairs_different_shared_secrets() {
        let alice = X25519KeyPair::generate().unwrap();
        let bob = X25519KeyPair::generate().unwrap();
        let charlie = X25519KeyPair::generate().unwrap();

        let alice_bob_shared = alice.exchange(bob.public_key()).unwrap();
        let alice_charlie_shared = alice.exchange(charlie.public_key()).unwrap();

        assert_ne!(
            alice_bob_shared.as_bytes(),
            alice_charlie_shared.as_bytes()
        );
    }

    #[test]
    fn test_invalid_public_key_size() {
        let bytes = vec![0u8; 16]; // Wrong size
        let result = X25519PublicKey::from_slice(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_mismatched_keypair() {
        let keypair1 = X25519KeyPair::generate().unwrap();
        let keypair2 = X25519KeyPair::generate().unwrap();

        let result = X25519KeyPair::from_base64(
            &keypair1.secret_key().to_base64(),
            &keypair2.public_key().to_base64(),
        );

        assert!(result.is_err());
    }
}
