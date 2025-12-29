//! Core error types for AccuScene Enterprise

use crate::codes::{ErrorCode, ErrorSeverity};
use crate::context::ErrorContext;
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Result type alias for AccuScene operations
pub type Result<T, E = AccuSceneError> = std::result::Result<T, E>;

/// Main error type for AccuScene Enterprise
///
/// This type provides a comprehensive error representation with:
/// - Error code and severity
/// - Human-readable message
/// - Optional context chain
/// - Unique error ID for tracking
/// - Timestamp
/// - Source location (file, line)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccuSceneError {
    /// Unique identifier for this error instance
    id: Uuid,

    /// Error code categorizing the error type
    code: ErrorCode,

    /// Error severity level
    severity: ErrorSeverity,

    /// Human-readable error message
    message: String,

    /// Optional detailed description
    details: Option<String>,

    /// Error context chain
    context: Option<ErrorContext>,

    /// Timestamp when error occurred
    timestamp: chrono::DateTime<chrono::Utc>,

    /// Source location (file:line)
    location: Option<String>,

    /// Additional metadata
    metadata: std::collections::HashMap<String, String>,

    /// Whether this error can be recovered from
    recoverable: bool,
}

impl AccuSceneError {
    /// Creates a new error with the given code and message
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        let severity = code.default_severity();

        Self {
            id: Uuid::new_v4(),
            code,
            severity,
            message: message.into(),
            details: None,
            context: None,
            timestamp: chrono::Utc::now(),
            location: None,
            metadata: std::collections::HashMap::new(),
            recoverable: code.is_recoverable(),
        }
    }

    /// Creates a validation error
    pub fn validation(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Validation, message)
    }

    /// Creates an authentication error
    pub fn authentication(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Authentication, message)
    }

    /// Creates an authorization error
    pub fn authorization(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Authorization, message)
    }

    /// Creates a not found error
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::NotFound, message)
    }

    /// Creates a conflict error
    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Conflict, message)
    }

    /// Creates an internal error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Internal, message)
    }

    /// Creates a database error
    pub fn database(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Database, message)
    }

    /// Creates a network error
    pub fn network(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Network, message)
    }

    /// Creates a timeout error
    pub fn timeout(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Timeout, message)
    }

    /// Creates a rate limit error
    pub fn rate_limit(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::RateLimit, message)
    }

    /// Adds detailed description to the error
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    /// Adds context to the error
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        let ctx = ErrorContext::new(context.into());
        self.context = Some(match self.context {
            Some(existing) => existing.with_parent(ctx),
            None => ctx,
        });
        self
    }

    /// Adds source location to the error
    pub fn with_location(mut self, file: &str, line: u32) -> Self {
        self.location = Some(format!("{}:{}", file, line));
        self
    }

    /// Adds metadata to the error
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Sets the severity of the error
    pub fn with_severity(mut self, severity: ErrorSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Sets whether the error is recoverable
    pub fn with_recoverable(mut self, recoverable: bool) -> Self {
        self.recoverable = recoverable;
        self
    }

    // Getters

    /// Returns the error ID
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Returns the error code
    pub fn code(&self) -> ErrorCode {
        self.code
    }

    /// Returns the error severity
    pub fn severity(&self) -> ErrorSeverity {
        self.severity
    }

    /// Returns the error message
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the error details
    pub fn details(&self) -> Option<&str> {
        self.details.as_deref()
    }

    /// Returns the error context
    pub fn context(&self) -> Option<&ErrorContext> {
        self.context.as_ref()
    }

    /// Returns the timestamp
    pub fn timestamp(&self) -> &chrono::DateTime<chrono::Utc> {
        &self.timestamp
    }

    /// Returns the source location
    pub fn location(&self) -> Option<&str> {
        self.location.as_deref()
    }

    /// Returns the metadata
    pub fn metadata(&self) -> &std::collections::HashMap<String, String> {
        &self.metadata
    }

    /// Returns whether the error is recoverable
    pub fn is_recoverable(&self) -> bool {
        self.recoverable
    }

    /// Returns a formatted error report
    pub fn report(&self) -> String {
        use crate::reporting::ErrorReporter;
        ErrorReporter::format_error(self)
    }
}

impl fmt::Display for AccuSceneError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}: {}", self.code, self.severity, self.message)?;

        if let Some(details) = &self.details {
            write!(f, "\nDetails: {}", details)?;
        }

        if let Some(ctx) = &self.context {
            write!(f, "\nContext: {}", ctx)?;
        }

        Ok(())
    }
}

impl std::error::Error for AccuSceneError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

// Conversions from common error types

impl From<std::io::Error> for AccuSceneError {
    fn from(err: std::io::Error) -> Self {
        AccuSceneError::internal(format!("I/O error: {}", err))
            .with_details(format!("{:?}", err))
    }
}

impl From<serde_json::Error> for AccuSceneError {
    fn from(err: serde_json::Error) -> Self {
        AccuSceneError::validation(format!("JSON error: {}", err))
            .with_details(format!("{:?}", err))
    }
}

impl From<anyhow::Error> for AccuSceneError {
    fn from(err: anyhow::Error) -> Self {
        AccuSceneError::internal(format!("{}", err))
            .with_details(format!("{:?}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = AccuSceneError::validation("Test error");
        assert_eq!(error.code(), ErrorCode::Validation);
        assert_eq!(error.message(), "Test error");
    }

    #[test]
    fn test_error_with_details() {
        let error = AccuSceneError::internal("Error")
            .with_details("Additional details");
        assert_eq!(error.details(), Some("Additional details"));
    }

    #[test]
    fn test_error_with_context() {
        let error = AccuSceneError::database("Connection failed")
            .with_context("During initialization");
        assert!(error.context().is_some());
    }

    #[test]
    fn test_error_with_metadata() {
        let error = AccuSceneError::network("Request failed")
            .with_metadata("url", "https://api.example.com")
            .with_metadata("method", "GET");
        assert_eq!(error.metadata().len(), 2);
    }
}
