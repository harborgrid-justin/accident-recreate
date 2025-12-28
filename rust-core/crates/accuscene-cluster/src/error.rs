//! Cluster-specific error types.

use thiserror::Error;
use uuid::Uuid;

/// Result type for cluster operations.
pub type Result<T> = std::result::Result<T, ClusterError>;

/// Comprehensive cluster error types.
#[derive(Debug, Error)]
pub enum ClusterError {
    #[error("Node not found: {0}")]
    NodeNotFound(Uuid),

    #[error("Leader not available")]
    NoLeader,

    #[error("Not the leader (current leader: {0:?})")]
    NotLeader(Option<Uuid>),

    #[error("Quorum not reached: {current}/{required}")]
    QuorumNotReached { current: usize, required: usize },

    #[error("Split brain detected")]
    SplitBrain,

    #[error("Network partition detected")]
    NetworkPartition,

    #[error("Node already exists: {0}")]
    NodeAlreadyExists(Uuid),

    #[error("Invalid cluster configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Consensus timeout")]
    ConsensusTimeout,

    #[error("Replication failed: {0}")]
    ReplicationFailed(String),

    #[error("Conflict resolution failed: {0}")]
    ConflictResolution(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Join error: {0}")]
    Join(#[from] tokio::task::JoinError),

    #[error("Node unhealthy: {0}")]
    NodeUnhealthy(String),

    #[error("Invalid message: {0}")]
    InvalidMessage(String),

    #[error("Checksum mismatch")]
    ChecksumMismatch,

    #[error("Operation aborted: {0}")]
    Aborted(String),

    #[error("Timeout: {0}")]
    Timeout(String),
}

impl From<bincode::Error> for ClusterError {
    fn from(err: bincode::Error) -> Self {
        ClusterError::Serialization(err.to_string())
    }
}

impl From<serde_json::Error> for ClusterError {
    fn from(err: serde_json::Error) -> Self {
        ClusterError::Serialization(err.to_string())
    }
}
