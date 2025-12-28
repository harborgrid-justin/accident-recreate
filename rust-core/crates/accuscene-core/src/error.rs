//! Central error types for AccuScene Core
//!
//! This module provides comprehensive error handling using thiserror.
//! All errors implement std::error::Error and can be easily converted
//! across the FFI boundary.

use thiserror::Error;

/// Result type alias for AccuScene operations
pub type Result<T> = std::result::Result<T, AccuSceneError>;

/// Central error type for all AccuScene operations
#[derive(Error, Debug, Clone)]
pub enum AccuSceneError {
    /// Validation errors for input data
    #[error("Validation error: {message}")]
    ValidationError {
        /// Description of the validation failure
        message: String,
        /// Field that failed validation (if applicable)
        field: Option<String>,
    },

    /// Physics calculation errors
    #[error("Physics calculation error: {0}")]
    PhysicsError(String),

    /// Serialization/deserialization errors
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Resource not found errors
    #[error("Resource not found: {resource_type} with id {id}")]
    NotFound {
        /// Type of resource that was not found
        resource_type: String,
        /// ID of the missing resource
        id: String,
    },

    /// Permission/authorization errors
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Data consistency errors
    #[error("Data integrity error: {0}")]
    IntegrityError(String),

    /// Threading/concurrency errors
    #[error("Concurrency error: {0}")]
    ConcurrencyError(String),

    /// I/O errors
    #[error("IO error: {0}")]
    IoError(String),

    /// Generic internal errors
    #[error("Internal error: {0}")]
    InternalError(String),

    /// Mathematical domain errors (e.g., division by zero, sqrt of negative)
    #[error("Mathematical domain error: {0}")]
    MathError(String),

    /// Invalid state errors
    #[error("Invalid state: {0}")]
    InvalidState(String),
}

impl AccuSceneError {
    /// Create a new validation error
    pub fn validation<S: Into<String>>(message: S) -> Self {
        Self::ValidationError {
            message: message.into(),
            field: None,
        }
    }

    /// Create a validation error with a specific field
    pub fn validation_field<S: Into<String>>(message: S, field: S) -> Self {
        Self::ValidationError {
            message: message.into(),
            field: Some(field.into()),
        }
    }

    /// Create a not found error
    pub fn not_found<S: Into<String>>(resource_type: S, id: S) -> Self {
        Self::NotFound {
            resource_type: resource_type.into(),
            id: id.into(),
        }
    }

    /// Create a physics error
    pub fn physics<S: Into<String>>(message: S) -> Self {
        Self::PhysicsError(message.into())
    }

    /// Create a math error
    pub fn math<S: Into<String>>(message: S) -> Self {
        Self::MathError(message.into())
    }

    /// Get error category for telemetry
    pub fn category(&self) -> &str {
        match self {
            Self::ValidationError { .. } => "validation",
            Self::PhysicsError(_) => "physics",
            Self::SerializationError(_) => "serialization",
            Self::ConfigError(_) => "config",
            Self::NotFound { .. } => "not_found",
            Self::PermissionDenied(_) => "permission",
            Self::IntegrityError(_) => "integrity",
            Self::ConcurrencyError(_) => "concurrency",
            Self::IoError(_) => "io",
            Self::InternalError(_) => "internal",
            Self::MathError(_) => "math",
            Self::InvalidState(_) => "invalid_state",
        }
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::ConcurrencyError(_) | Self::IoError(_) | Self::InternalError(_)
        )
    }
}

// Conversion from serde_json::Error
impl From<serde_json::Error> for AccuSceneError {
    fn from(err: serde_json::Error) -> Self {
        Self::SerializationError(err.to_string())
    }
}

// Conversion from std::io::Error
impl From<std::io::Error> for AccuSceneError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err.to_string())
    }
}

// Implement serde traits for error serialization
impl serde::Serialize for AccuSceneError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("AccuSceneError", 3)?;
        state.serialize_field("category", self.category())?;
        state.serialize_field("message", &self.to_string())?;
        state.serialize_field("retryable", &self.is_retryable())?;
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = AccuSceneError::validation("Invalid input");
        assert_eq!(err.category(), "validation");
        assert!(!err.is_retryable());
    }

    #[test]
    fn test_error_serialization() {
        let err = AccuSceneError::physics("Collision detection failed");
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("physics"));
    }

    #[test]
    fn test_not_found_error() {
        let err = AccuSceneError::not_found("Vehicle", "abc-123");
        assert!(err.to_string().contains("Vehicle"));
        assert!(err.to_string().contains("abc-123"));
    }
}
