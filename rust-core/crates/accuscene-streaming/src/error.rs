//! Error types for the AccuScene streaming system.

use thiserror::Error;

/// Result type alias for streaming operations.
pub type Result<T> = std::result::Result<T, StreamingError>;

/// Comprehensive error types for streaming operations.
#[derive(Debug, Error)]
pub enum StreamingError {
    /// WebSocket connection error
    #[error("WebSocket connection error: {0}")]
    WebSocketConnection(String),

    /// WebSocket protocol error
    #[error("WebSocket protocol error: {0}")]
    WebSocketProtocol(String),

    /// Event bus error
    #[error("Event bus error: {0}")]
    EventBus(String),

    /// Channel send error
    #[error("Channel send error: {0}")]
    ChannelSend(String),

    /// Channel receive error
    #[error("Channel receive error: {0}")]
    ChannelReceive(String),

    /// Broadcast error
    #[error("Broadcast error: {0}")]
    Broadcast(String),

    /// Subscription error
    #[error("Subscription error: {0}")]
    Subscription(String),

    /// Topic not found
    #[error("Topic not found: {0}")]
    TopicNotFound(String),

    /// Subscriber not found
    #[error("Subscriber not found: {0}")]
    SubscriberNotFound(String),

    /// Room not found
    #[error("Room not found: {0}")]
    RoomNotFound(String),

    /// Authentication error
    #[error("Authentication error: {0}")]
    Authentication(String),

    /// Authorization error
    #[error("Authorization error: {0}")]
    Authorization(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Compression error
    #[error("Compression error: {0}")]
    Compression(String),

    /// Decompression error
    #[error("Decompression error: {0}")]
    Decompression(String),

    /// Message too large
    #[error("Message too large: {size} bytes exceeds limit of {limit} bytes")]
    MessageTooLarge { size: usize, limit: usize },

    /// Invalid message format
    #[error("Invalid message format: {0}")]
    InvalidMessageFormat(String),

    /// Timeout error
    #[error("Operation timed out: {0}")]
    Timeout(String),

    /// Connection closed
    #[error("Connection closed: {0}")]
    ConnectionClosed(String),

    /// Already connected
    #[error("Already connected: {0}")]
    AlreadyConnected(String),

    /// Not connected
    #[error("Not connected: {0}")]
    NotConnected(String),

    /// Reconnection failed
    #[error("Reconnection failed after {attempts} attempts: {reason}")]
    ReconnectionFailed { attempts: usize, reason: String },

    /// Heartbeat timeout
    #[error("Heartbeat timeout: no response for {duration} seconds")]
    HeartbeatTimeout { duration: u64 },

    /// Presence error
    #[error("Presence error: {0}")]
    Presence(String),

    /// Replay error
    #[error("Event replay error: {0}")]
    Replay(String),

    /// Stream processing error
    #[error("Stream processing error: {0}")]
    StreamProcessing(String),

    /// Filter error
    #[error("Filter error: {0}")]
    Filter(String),

    /// Transform error
    #[error("Transform error: {0}")]
    Transform(String),

    /// Aggregation error
    #[error("Aggregation error: {0}")]
    Aggregation(String),

    /// Windowing error
    #[error("Windowing error: {0}")]
    Windowing(String),

    /// Join error
    #[error("Join error: {0}")]
    Join(String),

    /// Partitioning error
    #[error("Partitioning error: {0}")]
    Partitioning(String),

    /// Checkpoint error
    #[error("Checkpoint error: {0}")]
    Checkpoint(String),

    /// Checkpoint restoration failed
    #[error("Failed to restore from checkpoint: {0}")]
    CheckpointRestore(String),

    /// Watermark error
    #[error("Watermark error: {0}")]
    Watermark(String),

    /// State backend error
    #[error("State backend error: {0}")]
    StateBackend(String),

    /// State not found
    #[error("State not found: {0}")]
    StateNotFound(String),

    /// Backpressure error
    #[error("Backpressure error: {0}")]
    Backpressure(String),

    /// Buffer overflow
    #[error("Buffer overflow: capacity {capacity} exceeded")]
    BufferOverflow { capacity: usize },

    /// Source error
    #[error("Source error: {0}")]
    Source(String),

    /// Sink error
    #[error("Sink error: {0}")]
    Sink(String),

    /// Pipeline error
    #[error("Pipeline error: {0}")]
    Pipeline(String),

    /// Runtime error
    #[error("Runtime error: {0}")]
    Runtime(String),

    /// Arrow error
    #[error("Arrow error: {0}")]
    Arrow(String),

    /// Parquet error
    #[error("Parquet error: {0}")]
    Parquet(String),

    /// File rotation error
    #[error("File rotation error: {0}")]
    FileRotation(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Resource limit exceeded
    #[error("Resource limit exceeded: {resource} ({current}/{limit})")]
    ResourceLimitExceeded {
        resource: String,
        current: usize,
        limit: usize,
    },

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl StreamingError {
    /// Check if the error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            StreamingError::WebSocketConnection(_)
                | StreamingError::ChannelSend(_)
                | StreamingError::ChannelReceive(_)
                | StreamingError::Timeout(_)
                | StreamingError::ConnectionClosed(_)
                | StreamingError::HeartbeatTimeout { .. }
        )
    }

    /// Check if the error is fatal (requires shutdown)
    pub fn is_fatal(&self) -> bool {
        matches!(
            self,
            StreamingError::Authentication(_)
                | StreamingError::Authorization(_)
                | StreamingError::Configuration(_)
                | StreamingError::Internal(_)
        )
    }

    /// Get error category for metrics and logging
    pub fn category(&self) -> &'static str {
        match self {
            StreamingError::WebSocketConnection(_) | StreamingError::WebSocketProtocol(_) => {
                "websocket"
            }
            StreamingError::EventBus(_) => "event_bus",
            StreamingError::ChannelSend(_) | StreamingError::ChannelReceive(_) => "channel",
            StreamingError::Broadcast(_) => "broadcast",
            StreamingError::Subscription(_)
            | StreamingError::TopicNotFound(_)
            | StreamingError::SubscriberNotFound(_) => "pubsub",
            StreamingError::RoomNotFound(_) => "room",
            StreamingError::Authentication(_) | StreamingError::Authorization(_) => "auth",
            StreamingError::Serialization(_) => "serialization",
            StreamingError::Compression(_) | StreamingError::Decompression(_) => "compression",
            StreamingError::MessageTooLarge { .. } | StreamingError::InvalidMessageFormat(_) => {
                "message"
            }
            StreamingError::Timeout(_) => "timeout",
            StreamingError::ConnectionClosed(_)
            | StreamingError::AlreadyConnected(_)
            | StreamingError::NotConnected(_)
            | StreamingError::ReconnectionFailed { .. } => "connection",
            StreamingError::HeartbeatTimeout { .. } => "heartbeat",
            StreamingError::Presence(_) => "presence",
            StreamingError::Replay(_) => "replay",
            StreamingError::StreamProcessing(_)
            | StreamingError::Filter(_)
            | StreamingError::Transform(_)
            | StreamingError::Aggregation(_)
            | StreamingError::Windowing(_)
            | StreamingError::Join(_) => "stream",
            StreamingError::Partitioning(_) => "partitioning",
            StreamingError::Checkpoint(_) | StreamingError::CheckpointRestore(_) => "checkpoint",
            StreamingError::Watermark(_) => "watermark",
            StreamingError::StateBackend(_) | StreamingError::StateNotFound(_) => "state",
            StreamingError::Backpressure(_) | StreamingError::BufferOverflow { .. } => {
                "backpressure"
            }
            StreamingError::Source(_) => "source",
            StreamingError::Sink(_) => "sink",
            StreamingError::Pipeline(_) => "pipeline",
            StreamingError::Runtime(_) => "runtime",
            StreamingError::Arrow(_) => "arrow",
            StreamingError::Parquet(_) => "parquet",
            StreamingError::FileRotation(_) => "file_rotation",
            StreamingError::Configuration(_) => "config",
            StreamingError::ResourceLimitExceeded { .. } => "resource",
            StreamingError::Internal(_) => "internal",
            StreamingError::Io(_) => "io",
        }
    }
}
