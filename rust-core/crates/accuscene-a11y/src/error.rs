//! Error types for accessibility operations

use thiserror::Error;

/// Result type for accessibility operations
pub type Result<T> = std::result::Result<T, A11yError>;

/// Accessibility-related errors
#[derive(Error, Debug)]
pub enum A11yError {
    /// Color parsing error
    #[error("Invalid color format: {0}")]
    InvalidColor(String),

    /// Contrast ratio below minimum
    #[error("Contrast ratio {actual:.2} is below minimum {required:.2}")]
    InsufficientContrast { actual: f64, required: f64 },

    /// Text size too small
    #[error("Text size {0}px is below minimum recommended size")]
    TextSizeTooSmall(f32),

    /// Missing ARIA attribute
    #[error("Required ARIA attribute missing: {0}")]
    MissingAriaAttribute(String),

    /// Invalid ARIA value
    #[error("Invalid ARIA value for {attribute}: {value}")]
    InvalidAriaValue { attribute: String, value: String },

    /// Focus management error
    #[error("Focus management error: {0}")]
    FocusError(String),

    /// Keyboard navigation error
    #[error("Keyboard navigation error: {0}")]
    NavigationError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Audit failure
    #[error("Accessibility audit failed with {0} violations")]
    AuditFailed(usize),

    /// Screen reader error
    #[error("Screen reader error: {0}")]
    ScreenReaderError(String),

    /// Generic error
    #[error("Accessibility error: {0}")]
    Generic(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

impl A11yError {
    /// Create a new generic error
    pub fn new(msg: impl Into<String>) -> Self {
        Self::Generic(msg.into())
    }

    /// Check if error is critical (should halt operations)
    pub fn is_critical(&self) -> bool {
        matches!(
            self,
            Self::MissingAriaAttribute(_) | Self::AuditFailed(_)
        )
    }

    /// Get error severity level
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Self::InsufficientContrast { .. } => ErrorSeverity::High,
            Self::MissingAriaAttribute(_) => ErrorSeverity::Critical,
            Self::InvalidAriaValue { .. } => ErrorSeverity::High,
            Self::TextSizeTooSmall(_) => ErrorSeverity::Medium,
            Self::FocusError(_) => ErrorSeverity::High,
            Self::NavigationError(_) => ErrorSeverity::Medium,
            Self::AuditFailed(_) => ErrorSeverity::Critical,
            _ => ErrorSeverity::Low,
        }
    }
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl ErrorSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_severity() {
        let error = A11yError::InsufficientContrast {
            actual: 2.0,
            required: 4.5,
        };
        assert_eq!(error.severity(), ErrorSeverity::High);

        let error = A11yError::MissingAriaAttribute("label".to_string());
        assert_eq!(error.severity(), ErrorSeverity::Critical);
        assert!(error.is_critical());
    }
}
