//! Symmetric encryption using authenticated encryption algorithms
//!
//! Provides AES-256-GCM and ChaCha20-Poly1305 authenticated encryption.

pub mod aes;
pub mod chacha;
pub mod key;

pub use self::aes::{Aes256Gcm, decrypt_aes256gcm, encrypt_aes256gcm};
pub use self::chacha::{ChaCha20Poly1305, decrypt_chacha20poly1305, encrypt_chacha20poly1305};
pub use self::key::{SymmetricKey, SymmetricKey128, SymmetricKey256};
