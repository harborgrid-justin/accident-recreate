//! Cryptography module
//!
//! Provides cryptographic primitives and utilities:
//! - AES-256-GCM encryption/decryption
//! - Ed25519 digital signatures
//! - Key derivation (HKDF, PBKDF2)
//! - Cryptographically secure RNG

pub mod encryption;
pub mod signing;
pub mod key_derivation;
pub mod secure_random;

pub use encryption::{AesGcmEncryptor, EncryptedData};
pub use signing::{Ed25519Signer, Signature};
pub use key_derivation::{KeyDerivation, DerivedKey};
pub use secure_random::SecureRandom;
