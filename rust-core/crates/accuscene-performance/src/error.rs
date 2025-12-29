//! Error types for the performance crate

use thiserror::Error;

/// Result type for performance operations
pub type Result<T> = std::result::Result<T, PerformanceError>;

/// Errors that can occur in the performance crate
#[derive(Error, Debug, Clone)]
pub enum PerformanceError {
    /// Stream has been closed
    #[error("Stream closed")]
    StreamClosed,

    /// Buffer is full
    #[error("Buffer full: capacity {capacity}, attempted size {size}")]
    BufferFull { capacity: usize, size: usize },

    /// Buffer is empty
    #[error("Buffer empty")]
    BufferEmpty,

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Allocation failed
    #[error("Allocation failed: {0}")]
    AllocationFailed(String),

    /// Channel send error
    #[error("Channel send error")]
    ChannelSend,

    /// Channel receive error
    #[error("Channel receive error")]
    ChannelReceive,

    /// Timeout error
    #[error("Operation timed out after {duration_ms}ms")]
    Timeout { duration_ms: u64 },

    /// Capacity exceeded
    #[error("Capacity exceeded: max {max}, requested {requested}")]
    CapacityExceeded { max: usize, requested: usize },

    /// Invalid state
    #[error("Invalid state: {0}")]
    InvalidState(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Deserialization error
    #[error("Deserialization error: {0}")]
    Deserialization(String),

    /// Custom error
    #[error("{0}")]
    Custom(String),
}

impl PerformanceError {
    /// Create a custom error
    pub fn custom(msg: impl Into<String>) -> Self {
        Self::Custom(msg.into())
    }

    /// Check if error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::BufferFull { .. } | Self::Timeout { .. } | Self::ChannelSend | Self::ChannelReceive
        )
    }

    /// Check if error is fatal
    pub fn is_fatal(&self) -> bool {
        !self.is_recoverable()
    }
}

impl From<std::io::Error> for PerformanceError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

impl From<serde_json::Error> for PerformanceError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}

impl<T> From<flume::SendError<T>> for PerformanceError {
    fn from(_: flume::SendError<T>) -> Self {
        Self::ChannelSend
    }
}

impl From<flume::RecvError> for PerformanceError {
    fn from(_: flume::RecvError) -> Self {
        Self::ChannelReceive
    }
}
