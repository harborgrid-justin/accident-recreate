//! # AccuScene Cryptographic Security Module
//!
//! Enterprise-grade cryptographic operations for the AccuScene accident recreation platform.
//!
//! ## Features
//!
//! - **Secure Random Generation**: Cryptographically secure random number generation
//! - **Hashing**: SHA-256, SHA-512, BLAKE3, and password hashing (Argon2id, scrypt)
//! - **Symmetric Encryption**: AES-256-GCM and ChaCha20-Poly1305 authenticated encryption
//! - **Asymmetric Cryptography**: Ed25519 signatures and X25519 key exchange
//! - **Key Derivation**: HKDF and PBKDF2 for deriving keys from passwords
//! - **Envelope Encryption**: Secure encryption for large data with DEK/KEK pattern
//! - **Secure Vault**: In-memory key storage with automatic zeroization
//! - **Token Management**: Secure token generation and validation with expiration
//! - **Integrity Verification**: File integrity checking with HMAC and signatures
//! - **Certificate System**: Simple PKI for public key distribution
//!
//! ## Security Properties
//!
//! - All sensitive data is automatically zeroized on drop
//! - Constant-time comparisons for all security-critical operations
//! - Only authenticated encryption algorithms (AEAD)
//! - Modern, audited cryptographic primitives
//! - Memory-safe implementation in Rust
//!
//! ## Quick Start
//!
//! ```rust
//! use accuscene_crypto::{
//!     symmetric::{SymmetricKey, encrypt_aes256gcm, decrypt_aes256gcm},
//!     hash::hash_password,
//!     random::generate_random_bytes,
//! };
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Generate a symmetric key
//! let key = SymmetricKey::generate()?;
//!
//! // Encrypt data
//! let plaintext = b"Secret data";
//! let encrypted = encrypt_aes256gcm(&key, plaintext, None)?;
//!
//! // Decrypt data
//! let decrypted = decrypt_aes256gcm(&key, &encrypted, None)?;
//! assert_eq!(plaintext, decrypted.as_bytes());
//!
//! // Hash a password
//! let password = "user_password";
//! let hash = hash_password(password)?;
//!
//! // Generate random bytes
//! let random = generate_random_bytes(32)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Module Organization
//!
//! - [`error`] - Error types for cryptographic operations
//! - [`random`] - Secure random number generation
//! - [`hash`] - Cryptographic hashing functions
//! - [`symmetric`] - Symmetric encryption (AES-GCM, ChaCha20-Poly1305)
//! - [`asymmetric`] - Asymmetric cryptography (Ed25519, X25519)
//! - [`kdf`] - Key derivation functions
//! - [`envelope`] - Envelope encryption for large data
//! - [`vault`] - Secure in-memory key vault
//! - [`token`] - Token generation and validation
//! - [`integrity`] - File integrity verification
//! - [`certificate`] - Simple certificate system
//! - [`secure_memory`] - Secure memory handling with zeroization

#![warn(missing_docs)]
#![warn(clippy::all)]
#![deny(unsafe_code)]

// Core modules
pub mod error;
pub mod random;
pub mod secure_memory;

// Cryptographic primitives
pub mod asymmetric;
pub mod hash;
pub mod kdf;
pub mod symmetric;

// High-level features
pub mod certificate;
pub mod envelope;
pub mod integrity;
pub mod token;
pub mod vault;

// Re-export commonly used types
pub use error::{CryptoError, CryptoResult};
pub use random::{generate_random_bytes, SecureRng};
pub use secure_memory::{SecureBytes, SecureKey, SecureString};

/// Prelude module for convenient imports
pub mod prelude {
    //! Convenient re-exports of commonly used types and functions

    // Error handling
    pub use crate::error::{CryptoError, CryptoResult};

    // Random generation
    pub use crate::random::{generate_random_bytes, generate_random_string, SecureRng};

    // Secure memory
    pub use crate::secure_memory::{SecureBytes, SecureKey, SecureString};

    // Hashing
    pub use crate::hash::{
        blake3_hash, hash_password, sha256, sha512, verify_password, Blake3Hasher,
        PasswordHasher, Sha256Hasher, Sha512Hasher,
    };

    // Symmetric encryption
    pub use crate::symmetric::{
        decrypt_aes256gcm, decrypt_chacha20poly1305, encrypt_aes256gcm, encrypt_chacha20poly1305,
        Aes256Gcm, ChaCha20Poly1305, SymmetricKey, SymmetricKey128, SymmetricKey256,
    };

    // Asymmetric cryptography
    pub use crate::asymmetric::{
        perform_key_exchange, sign_message, verify_signature, Ed25519KeyPair, Ed25519PublicKey,
        Ed25519SecretKey, Ed25519Signer, Signature, X25519KeyPair, X25519PublicKey,
        X25519SecretKey,
    };

    // Key derivation
    pub use crate::kdf::{derive_key_hkdf, derive_key_pbkdf2, Hkdf, Pbkdf2};

    // Envelope encryption
    pub use crate::envelope::{envelope_decrypt, envelope_encrypt, EnvelopeEncryptor};

    // Vault
    pub use crate::vault::{Vault, VaultEntryBuilder};

    // Tokens
    pub use crate::token::{generate_token_string, hash_token, Token, TokenGenerator};

    // Integrity
    pub use crate::integrity::{
        checksum_blake3, checksum_sha256, hmac_sha256, IntegrityAlgorithm, IntegrityProof,
        IntegrityVerifier,
    };

    // Certificates
    pub use crate::certificate::{Certificate, CertificateAuthority, CertificateBuilder};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prelude_imports() {
        use prelude::*;

        // Test that we can use prelude imports
        let key = SymmetricKey::generate().unwrap();
        assert_eq!(key.as_bytes().len(), 32);
    }

    #[test]
    fn test_module_visibility() {
        // Test that all modules are accessible
        let _ = error::CryptoError::EncryptionFailed("test".to_string());
        let _ = random::generate_random_bytes(16);
        let _ = secure_memory::SecureBytes::zeros(32);
    }
}