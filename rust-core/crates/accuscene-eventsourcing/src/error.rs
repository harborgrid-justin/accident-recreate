//! Comprehensive error types for the event sourcing system.

use thiserror::Error;
use uuid::Uuid;

/// Result type for event sourcing operations.
pub type Result<T> = std::result::Result<T, EventSourcingError>;

/// Comprehensive error types for event sourcing operations.
#[derive(Error, Debug, Clone)]
pub enum EventSourcingError {
    /// Event not found in the store.
    #[error("Event not found: {0}")]
    EventNotFound(Uuid),

    /// Aggregate not found.
    #[error("Aggregate not found: {0}")]
    AggregateNotFound(String),

    /// Version conflict during optimistic concurrency control.
    #[error("Version conflict for aggregate {aggregate_id}: expected {expected}, got {actual}")]
    VersionConflict {
        aggregate_id: String,
        expected: u64,
        actual: u64,
    },

    /// Invalid event sequence.
    #[error("Invalid event sequence for aggregate {aggregate_id}: expected {expected}, got {actual}")]
    InvalidSequence {
        aggregate_id: String,
        expected: u64,
        actual: u64,
    },

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Deserialization error.
    #[error("Deserialization error: {0}")]
    Deserialization(String),

    /// Database error.
    #[error("Database error: {0}")]
    Database(String),

    /// Connection error.
    #[error("Connection error: {0}")]
    Connection(String),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Snapshot error.
    #[error("Snapshot error: {0}")]
    Snapshot(String),

    /// Projection error.
    #[error("Projection error: {0}")]
    Projection(String),

    /// Command validation error.
    #[error("Command validation error: {0}")]
    CommandValidation(String),

    /// Query error.
    #[error("Query error: {0}")]
    Query(String),

    /// Event bus error.
    #[error("Event bus error: {0}")]
    EventBus(String),

    /// Command bus error.
    #[error("Command bus error: {0}")]
    CommandBus(String),

    /// Saga error.
    #[error("Saga error: {0}")]
    Saga(String),

    /// Invalid state transition.
    #[error("Invalid state transition: {0}")]
    InvalidStateTransition(String),

    /// Timeout error.
    #[error("Operation timed out: {0}")]
    Timeout(String),

    /// Lock acquisition error.
    #[error("Failed to acquire lock: {0}")]
    LockAcquisition(String),

    /// Internal error.
    #[error("Internal error: {0}")]
    Internal(String),
}

impl EventSourcingError {
    /// Creates a serialization error.
    pub fn serialization<E: std::fmt::Display>(error: E) -> Self {
        Self::Serialization(error.to_string())
    }

    /// Creates a deserialization error.
    pub fn deserialization<E: std::fmt::Display>(error: E) -> Self {
        Self::Deserialization(error.to_string())
    }

    /// Creates a database error.
    pub fn database<E: std::fmt::Display>(error: E) -> Self {
        Self::Database(error.to_string())
    }

    /// Creates a connection error.
    pub fn connection<E: std::fmt::Display>(error: E) -> Self {
        Self::Connection(error.to_string())
    }

    /// Creates a configuration error.
    pub fn configuration<E: std::fmt::Display>(error: E) -> Self {
        Self::Configuration(error.to_string())
    }

    /// Creates an internal error.
    pub fn internal<E: std::fmt::Display>(error: E) -> Self {
        Self::Internal(error.to_string())
    }
}

#[cfg(feature = "postgres")]
impl From<sqlx::Error> for EventSourcingError {
    fn from(error: sqlx::Error) -> Self {
        EventSourcingError::database(error)
    }
}

impl From<serde_json::Error> for EventSourcingError {
    fn from(error: serde_json::Error) -> Self {
        EventSourcingError::serialization(error)
    }
}

impl From<bincode::Error> for EventSourcingError {
    fn from(error: bincode::Error) -> Self {
        EventSourcingError::serialization(error)
    }
}
