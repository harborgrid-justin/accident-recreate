use thiserror::Error;

/// Result type for offline sync operations
pub type Result<T> = std::result::Result<T, OfflineError>;

/// Comprehensive error types for offline sync operations
#[derive(Error, Debug)]
pub enum OfflineError {
    #[error("Storage error: {0}")]
    Storage(String),

    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Binary serialization error: {0}")]
    BinarySerialization(#[from] bincode::Error),

    #[error("Conflict detected: {0}")]
    Conflict(String),

    #[error("Version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: String, actual: String },

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Sync failed: {0}")]
    SyncFailed(String),

    #[error("Queue full: cannot enqueue operation")]
    QueueFull,

    #[error("Operation timeout after {0}ms")]
    Timeout(u64),

    #[error("Database lock timeout")]
    LockTimeout,

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Network unavailable")]
    NetworkUnavailable,

    #[error("Authentication required")]
    AuthRequired,

    #[error("Rate limit exceeded, retry after {0}s")]
    RateLimited(u64),

    #[error("Data corruption detected: {0}")]
    DataCorruption(String),

    #[error("Insufficient storage space")]
    InsufficientStorage,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl OfflineError {
    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            OfflineError::Network(_)
                | OfflineError::NetworkUnavailable
                | OfflineError::Timeout(_)
                | OfflineError::LockTimeout
                | OfflineError::RateLimited(_)
        )
    }

    /// Check if error is due to conflict
    pub fn is_conflict(&self) -> bool {
        matches!(self, OfflineError::Conflict(_) | OfflineError::VersionMismatch { .. })
    }

    /// Get retry delay in milliseconds
    pub fn retry_delay(&self) -> Option<u64> {
        match self {
            OfflineError::RateLimited(secs) => Some(secs * 1000),
            OfflineError::NetworkUnavailable => Some(5000),
            OfflineError::Network(_) => Some(1000),
            OfflineError::Timeout(_) => Some(2000),
            _ => None,
        }
    }
}
