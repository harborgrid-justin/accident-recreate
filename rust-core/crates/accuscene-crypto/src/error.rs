//! Cryptographic error types for AccuScene
//!
//! Provides comprehensive error handling for all cryptographic operations.

use thiserror::Error;

/// Result type for cryptographic operations
pub type CryptoResult<T> = Result<T, CryptoError>;

/// Cryptographic error types
#[derive(Debug, Error)]
pub enum CryptoError {
    /// Error during encryption operation
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    /// Error during decryption operation
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    /// Error during key generation
    #[error("Key generation failed: {0}")]
    KeyGenerationFailed(String),

    /// Error during key derivation
    #[error("Key derivation failed: {0}")]
    KeyDerivationFailed(String),

    /// Invalid key size
    #[error("Invalid key size: expected {expected}, got {actual}")]
    InvalidKeySize { expected: usize, actual: usize },

    /// Invalid nonce/IV size
    #[error("Invalid nonce size: expected {expected}, got {actual}")]
    InvalidNonceSize { expected: usize, actual: usize },

    /// Invalid salt size
    #[error("Invalid salt size: expected {expected}, got {actual}")]
    InvalidSaltSize { expected: usize, actual: usize },

    /// Error during hashing operation
    #[error("Hashing failed: {0}")]
    HashingFailed(String),

    /// Error during password hashing
    #[error("Password hashing failed: {0}")]
    PasswordHashingFailed(String),

    /// Error during password verification
    #[error("Password verification failed: {0}")]
    PasswordVerificationFailed(String),

    /// Error during signature generation
    #[error("Signature generation failed: {0}")]
    SignatureFailed(String),

    /// Error during signature verification
    #[error("Signature verification failed: {0}")]
    VerificationFailed(String),

    /// Error during key exchange
    #[error("Key exchange failed: {0}")]
    KeyExchangeFailed(String),

    /// Invalid signature
    #[error("Invalid signature")]
    InvalidSignature,

    /// Invalid certificate
    #[error("Invalid certificate: {0}")]
    InvalidCertificate(String),

    /// Certificate expired
    #[error("Certificate expired")]
    CertificateExpired,

    /// Token generation failed
    #[error("Token generation failed: {0}")]
    TokenGenerationFailed(String),

    /// Token validation failed
    #[error("Token validation failed: {0}")]
    TokenValidationFailed(String),

    /// Invalid token format
    #[error("Invalid token format: {0}")]
    InvalidTokenFormat(String),

    /// Token expired
    #[error("Token expired")]
    TokenExpired,

    /// Invalid input data
    #[error("Invalid input data: {0}")]
    InvalidInput(String),

    /// Invalid output data
    #[error("Invalid output data: {0}")]
    InvalidOutput(String),

    /// Encoding error
    #[error("Encoding error: {0}")]
    EncodingError(String),

    /// Decoding error
    #[error("Decoding error: {0}")]
    DecodingError(String),

    /// Random number generation failed
    #[error("Random number generation failed: {0}")]
    RandomGenerationFailed(String),

    /// Vault error
    #[error("Vault error: {0}")]
    VaultError(String),

    /// Key not found in vault
    #[error("Key not found in vault: {0}")]
    KeyNotFound(String),

    /// Integrity check failed
    #[error("Integrity check failed: {0}")]
    IntegrityCheckFailed(String),

    /// I/O error
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Deserialization error
    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    /// Internal error
    #[error("Internal cryptographic error: {0}")]
    InternalError(String),
}

// Implement conversions from specific error types
impl From<base64::DecodeError> for CryptoError {
    fn from(err: base64::DecodeError) -> Self {
        CryptoError::DecodingError(err.to_string())
    }
}

impl From<aes_gcm::Error> for CryptoError {
    fn from(err: aes_gcm::Error) -> Self {
        CryptoError::DecryptionFailed(err.to_string())
    }
}

impl From<chacha20poly1305::Error> for CryptoError {
    fn from(err: chacha20poly1305::Error) -> Self {
        CryptoError::DecryptionFailed(err.to_string())
    }
}

impl From<argon2::Error> for CryptoError {
    fn from(err: argon2::Error) -> Self {
        CryptoError::PasswordHashingFailed(err.to_string())
    }
}

impl From<argon2::password_hash::Error> for CryptoError {
    fn from(err: argon2::password_hash::Error) -> Self {
        CryptoError::PasswordHashingFailed(err.to_string())
    }
}

impl From<ed25519_dalek::SignatureError> for CryptoError {
    fn from(err: ed25519_dalek::SignatureError) -> Self {
        CryptoError::VerificationFailed(err.to_string())
    }
}

impl From<serde_json::Error> for CryptoError {
    fn from(err: serde_json::Error) -> Self {
        CryptoError::SerializationError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = CryptoError::EncryptionFailed("test error".to_string());
        assert_eq!(err.to_string(), "Encryption failed: test error");
    }

    #[test]
    fn test_invalid_key_size_error() {
        let err = CryptoError::InvalidKeySize {
            expected: 32,
            actual: 16,
        };
        assert!(err.to_string().contains("expected 32"));
        assert!(err.to_string().contains("got 16"));
    }
}
