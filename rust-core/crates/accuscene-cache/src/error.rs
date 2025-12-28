//! Cache-specific error types

use thiserror::Error;

/// Result type for cache operations
pub type CacheResult<T> = Result<T, CacheError>;

/// Cache-specific errors
#[derive(Error, Debug)]
pub enum CacheError {
    #[error("Cache key not found: {0}")]
    KeyNotFound(String),

    #[error("Cache is full, cannot insert new entry")]
    CacheFull,

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Disk cache error: {0}")]
    DiskError(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Cache backend error: {0}")]
    BackendError(String),

    #[error("Eviction policy error: {0}")]
    EvictionError(String),

    #[error("TTL expired for key: {0}")]
    Expired(String),

    #[error("Lock acquisition failed: {0}")]
    LockError(String),

    #[error("Partition error: {0}")]
    PartitionError(String),

    #[error("Tag not found: {0}")]
    TagNotFound(String),

    #[error("Distributed cache error: {0}")]
    DistributedError(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl CacheError {
    pub fn is_retriable(&self) -> bool {
        matches!(
            self,
            CacheError::LockError(_) | CacheError::IoError(_) | CacheError::DiskError(_)
        )
    }

    pub fn is_fatal(&self) -> bool {
        matches!(
            self,
            CacheError::InvalidConfig(_) | CacheError::SerializationError(_)
        )
    }
}
