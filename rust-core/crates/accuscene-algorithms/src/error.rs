//! Error types for AccuScene algorithms crate.

use thiserror::Error;

/// Result type alias for this crate.
pub type Result<T> = std::result::Result<T, AlgorithmError>;

/// Errors that can occur in algorithm operations.
#[derive(Error, Debug)]
pub enum AlgorithmError {
    /// Compression operation failed.
    #[error("Compression failed: {0}")]
    CompressionFailed(String),

    /// Decompression operation failed.
    #[error("Decompression failed: {0}")]
    DecompressionFailed(String),

    /// Invalid data format.
    #[error("Invalid data format: {0}")]
    InvalidFormat(String),

    /// Buffer too small for operation.
    #[error("Buffer too small: needed {needed}, got {available}")]
    BufferTooSmall { needed: usize, available: usize },

    /// Index out of bounds.
    #[error("Index out of bounds: {0}")]
    IndexOutOfBounds(String),

    /// I/O error occurred.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid configuration.
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Storage operation failed.
    #[error("Storage error: {0}")]
    StorageError(String),

    /// WAL (Write-Ahead Log) error.
    #[error("WAL error: {0}")]
    WalError(String),

    /// MVCC (Multi-Version Concurrency Control) error.
    #[error("MVCC error: {0}")]
    MvccError(String),

    /// Page management error.
    #[error("Page error: {0}")]
    PageError(String),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Dictionary training failed.
    #[error("Dictionary training failed: {0}")]
    DictionaryTrainingFailed(String),

    /// Spatial index error.
    #[error("Spatial index error: {0}")]
    SpatialIndexError(String),

    /// Memory mapping error.
    #[error("Memory mapping error: {0}")]
    MemoryMappingError(String),

    /// Concurrent access violation.
    #[error("Concurrent access violation: {0}")]
    ConcurrencyError(String),
}

impl From<bincode::Error> for AlgorithmError {
    fn from(e: bincode::Error) -> Self {
        AlgorithmError::SerializationError(e.to_string())
    }
}
