//! Error types for compression operations

use thiserror::Error;

/// Result type alias for compression operations
pub type Result<T> = std::result::Result<T, CompressionError>;

/// Errors that can occur during compression and decompression operations
#[derive(Debug, Error)]
pub enum CompressionError {
    /// LZ4 compression/decompression error
    #[error("LZ4 error: {0}")]
    Lz4(String),

    /// Zstandard compression/decompression error
    #[error("Zstandard error: {0}")]
    Zstd(String),

    /// Brotli compression/decompression error
    #[error("Brotli error: {0}")]
    Brotli(String),

    /// Deflate/gzip compression/decompression error
    #[error("Deflate error: {0}")]
    Deflate(String),

    /// Snappy compression/decompression error
    #[error("Snappy error: {0}")]
    Snappy(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Deserialization error
    #[error("Deserialization error: {0}")]
    Deserialization(String),

    /// Invalid compression level
    #[error("Invalid compression level: {0}")]
    InvalidLevel(i32),

    /// Invalid algorithm specified
    #[error("Invalid algorithm: {0}")]
    InvalidAlgorithm(String),

    /// Buffer too small for decompression
    #[error("Buffer too small: need at least {needed} bytes, got {available}")]
    BufferTooSmall { needed: usize, available: usize },

    /// Invalid magic number in compressed data
    #[error("Invalid magic number: expected {expected:x}, got {actual:x}")]
    InvalidMagic { expected: u32, actual: u32 },

    /// Corrupted data detected
    #[error("Corrupted data: {0}")]
    CorruptedData(String),

    /// Checksum mismatch
    #[error("Checksum mismatch: expected {expected:x}, got {actual:x}")]
    ChecksumMismatch { expected: u32, actual: u32 },

    /// Dictionary error
    #[error("Dictionary error: {0}")]
    Dictionary(String),

    /// Archive error
    #[error("Archive error: {0}")]
    Archive(String),

    /// Encryption error
    #[error("Encryption error: {0}")]
    Encryption(String),

    /// Decryption error
    #[error("Decryption error: {0}")]
    Decryption(String),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Unsupported version
    #[error("Unsupported version: {0}")]
    UnsupportedVersion(u32),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Resource limit exceeded
    #[error("Resource limit exceeded: {0}")]
    LimitExceeded(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<bincode::Error> for CompressionError {
    fn from(err: bincode::Error) -> Self {
        CompressionError::Serialization(err.to_string())
    }
}

impl From<rmp_serde::encode::Error> for CompressionError {
    fn from(err: rmp_serde::encode::Error) -> Self {
        CompressionError::Serialization(err.to_string())
    }
}

impl From<rmp_serde::decode::Error> for CompressionError {
    fn from(err: rmp_serde::decode::Error) -> Self {
        CompressionError::Deserialization(err.to_string())
    }
}

#[cfg(feature = "encryption")]
impl From<aes_gcm::Error> for CompressionError {
    fn from(err: aes_gcm::Error) -> Self {
        CompressionError::Encryption(err.to_string())
    }
}

#[cfg(feature = "encryption")]
impl From<argon2::Error> for CompressionError {
    fn from(err: argon2::Error) -> Self {
        CompressionError::Encryption(err.to_string())
    }
}
