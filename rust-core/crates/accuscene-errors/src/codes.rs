//! Error code definitions and categorization

use serde::{Deserialize, Serialize};
use std::fmt;

/// Error codes for categorizing errors across AccuScene Enterprise
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    // Client Errors (4xx equivalent)
    /// Invalid input or request validation failed
    Validation,
    /// Authentication required or failed
    Authentication,
    /// Insufficient permissions
    Authorization,
    /// Requested resource not found
    NotFound,
    /// Resource already exists or state conflict
    Conflict,
    /// Request payload too large
    PayloadTooLarge,
    /// Too many requests - rate limited
    RateLimit,
    /// Invalid state transition
    InvalidState,

    // Server Errors (5xx equivalent)
    /// Internal server error
    Internal,
    /// Database operation failed
    Database,
    /// Network communication error
    Network,
    /// External service error
    ExternalService,
    /// Operation timeout
    Timeout,
    /// Service unavailable
    Unavailable,

    // Domain-Specific Errors
    /// Physics simulation error
    Physics,
    /// Rendering or visualization error
    Rendering,
    /// File I/O error
    FileSystem,
    /// Cache operation error
    Cache,
    /// Job processing error
    Job,
    /// Streaming error
    Streaming,
    /// Compression/decompression error
    Compression,
    /// Cryptography error
    Crypto,
    /// Cluster/distributed system error
    Cluster,
    /// Analytics processing error
    Analytics,
    /// Machine learning error
    MachineLearning,
    /// Search operation error
    Search,
    /// Notification delivery error
    Notification,
    /// SSO/identity provider error
    SSO,
    /// Gesture recognition error
    Gesture,
    /// Offline mode error
    Offline,
    /// Data transfer error
    Transfer,
    /// Preference/settings error
    Preferences,
    /// Accessibility error
    Accessibility,
    /// Dashboard error
    Dashboard,

    // System Errors
    /// Configuration error
    Configuration,
    /// Resource exhausted (memory, disk, etc.)
    ResourceExhausted,
    /// Unimplemented feature
    Unimplemented,
    /// Deprecated feature used
    Deprecated,
}

impl ErrorCode {
    /// Returns the default severity for this error code
    pub fn default_severity(self) -> ErrorSeverity {
        match self {
            // Critical errors
            Self::Internal
            | Self::Database
            | Self::ResourceExhausted => ErrorSeverity::Critical,

            // High severity
            Self::Authentication
            | Self::Authorization
            | Self::ExternalService
            | Self::Cluster
            | Self::Crypto => ErrorSeverity::High,

            // Medium severity
            Self::Network
            | Self::Timeout
            | Self::Unavailable
            | Self::FileSystem
            | Self::Configuration
            | Self::Job
            | Self::Streaming => ErrorSeverity::Medium,

            // Low severity
            Self::Validation
            | Self::NotFound
            | Self::Conflict
            | Self::InvalidState
            | Self::RateLimit => ErrorSeverity::Low,

            // Info/Warning
            Self::Deprecated
            | Self::Unimplemented => ErrorSeverity::Warning,

            // Domain-specific (default to medium)
            _ => ErrorSeverity::Medium,
        }
    }

    /// Returns whether this error type is typically recoverable
    pub fn is_recoverable(self) -> bool {
        match self {
            // Generally not recoverable
            Self::Internal
            | Self::Unimplemented
            | Self::Configuration
            | Self::ResourceExhausted => false,

            // Usually recoverable with retry or user action
            Self::Network
            | Self::Timeout
            | Self::Unavailable
            | Self::RateLimit
            | Self::ExternalService => true,

            // Recoverable with user correction
            Self::Validation
            | Self::Authentication
            | Self::Authorization
            | Self::NotFound
            | Self::Conflict => true,

            // Domain-specific (assume recoverable)
            _ => true,
        }
    }

    /// Returns the HTTP status code equivalent for API responses
    pub fn http_status(self) -> u16 {
        match self {
            Self::Validation | Self::InvalidState => 400,
            Self::Authentication => 401,
            Self::Authorization => 403,
            Self::NotFound => 404,
            Self::Conflict => 409,
            Self::PayloadTooLarge => 413,
            Self::RateLimit => 429,
            Self::Unimplemented => 501,
            Self::Unavailable => 503,
            Self::Timeout => 504,
            _ => 500,
        }
    }

    /// Returns a user-friendly description of the error code
    pub fn description(self) -> &'static str {
        match self {
            Self::Validation => "Invalid input or validation failed",
            Self::Authentication => "Authentication required or failed",
            Self::Authorization => "Insufficient permissions",
            Self::NotFound => "Resource not found",
            Self::Conflict => "Resource conflict",
            Self::PayloadTooLarge => "Request payload too large",
            Self::RateLimit => "Too many requests",
            Self::InvalidState => "Invalid state transition",
            Self::Internal => "Internal server error",
            Self::Database => "Database operation failed",
            Self::Network => "Network communication error",
            Self::ExternalService => "External service error",
            Self::Timeout => "Operation timeout",
            Self::Unavailable => "Service unavailable",
            Self::Physics => "Physics simulation error",
            Self::Rendering => "Rendering error",
            Self::FileSystem => "File system error",
            Self::Cache => "Cache operation error",
            Self::Job => "Job processing error",
            Self::Streaming => "Streaming error",
            Self::Compression => "Compression error",
            Self::Crypto => "Cryptography error",
            Self::Cluster => "Cluster operation error",
            Self::Analytics => "Analytics error",
            Self::MachineLearning => "Machine learning error",
            Self::Search => "Search operation error",
            Self::Notification => "Notification delivery error",
            Self::SSO => "SSO operation error",
            Self::Gesture => "Gesture recognition error",
            Self::Offline => "Offline mode error",
            Self::Transfer => "Data transfer error",
            Self::Preferences => "Preferences error",
            Self::Accessibility => "Accessibility error",
            Self::Dashboard => "Dashboard error",
            Self::Configuration => "Configuration error",
            Self::ResourceExhausted => "Resource exhausted",
            Self::Unimplemented => "Feature not implemented",
            Self::Deprecated => "Deprecated feature",
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ErrorSeverity {
    /// Informational - no action required
    Info,
    /// Warning - should be addressed but not critical
    Warning,
    /// Low severity - minor impact
    Low,
    /// Medium severity - moderate impact
    Medium,
    /// High severity - significant impact
    High,
    /// Critical - severe impact, immediate attention required
    Critical,
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Info => write!(f, "INFO"),
            Self::Warning => write!(f, "WARNING"),
            Self::Low => write!(f, "LOW"),
            Self::Medium => write!(f, "MEDIUM"),
            Self::High => write!(f, "HIGH"),
            Self::Critical => write!(f, "CRITICAL"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_severity() {
        assert_eq!(ErrorCode::Internal.default_severity(), ErrorSeverity::Critical);
        assert_eq!(ErrorCode::Validation.default_severity(), ErrorSeverity::Low);
        assert_eq!(ErrorCode::Network.default_severity(), ErrorSeverity::Medium);
    }

    #[test]
    fn test_error_code_recoverable() {
        assert!(!ErrorCode::Internal.is_recoverable());
        assert!(ErrorCode::Network.is_recoverable());
        assert!(ErrorCode::Validation.is_recoverable());
    }

    #[test]
    fn test_error_code_http_status() {
        assert_eq!(ErrorCode::NotFound.http_status(), 404);
        assert_eq!(ErrorCode::Authentication.http_status(), 401);
        assert_eq!(ErrorCode::Internal.http_status(), 500);
    }
}
