//! Security error types
//!
//! Comprehensive error handling for all security operations.

use thiserror::Error;

/// Main security error type
#[derive(Error, Debug)]
pub enum SecurityError {
    // Authentication errors
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Password too weak: {0}")]
    WeakPassword(String),

    #[error("MFA required")]
    MfaRequired,

    #[error("Invalid MFA token")]
    InvalidMfaToken,

    #[error("Session expired")]
    SessionExpired,

    #[error("Session not found")]
    SessionNotFound,

    #[error("Invalid token: {0}")]
    InvalidToken(String),

    // Authorization errors
    #[error("Access denied: {0}")]
    AccessDenied(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Policy violation: {0}")]
    PolicyViolation(String),

    #[error("Role not found: {0}")]
    RoleNotFound(String),

    // Audit errors
    #[error("Audit log write failed: {0}")]
    AuditLogFailed(String),

    #[error("Audit trail compromised: {0}")]
    AuditTrailCompromised(String),

    #[error("Audit query failed: {0}")]
    AuditQueryFailed(String),

    // Encryption errors
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("Key not found: {0}")]
    KeyNotFound(String),

    #[error("Key rotation failed: {0}")]
    KeyRotationFailed(String),

    // Secrets management errors
    #[error("Secret not found: {0}")]
    SecretNotFound(String),

    #[error("Secret expired")]
    SecretExpired,

    #[error("Vault sealed")]
    VaultSealed,

    // Validation errors
    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("Sanitization failed: {0}")]
    SanitizationFailed(String),

    #[error("Input too large: {max} bytes maximum")]
    InputTooLarge { max: usize },

    // Threat detection errors
    #[error("Rate limit exceeded: retry after {retry_after_secs} seconds")]
    RateLimitExceeded { retry_after_secs: u64 },

    #[error("Brute force attack detected from {source}")]
    BruteForceDetected { source: String },

    #[error("Anomaly detected: {0}")]
    AnomalyDetected(String),

    // Compliance errors
    #[error("SOC2 compliance violation: {0}")]
    Soc2Violation(String),

    #[error("GDPR compliance violation: {0}")]
    GdprViolation(String),

    #[error("HIPAA compliance violation: {0}")]
    HipaaViolation(String),

    // Domain-specific errors
    #[error("Case access denied: {case_id}")]
    CaseAccessDenied { case_id: String },

    #[error("Evidence chain of custody broken: {evidence_id}")]
    ChainOfCustodyBroken { evidence_id: String },

    #[error("Report access denied: {report_id}")]
    ReportAccessDenied { report_id: String },

    // Configuration errors
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    // Generic errors
    #[error("Internal security error: {0}")]
    Internal(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Specialized result type for security operations
pub type Result<T> = std::result::Result<T, SecurityError>;

impl SecurityError {
    /// Check if error is authentication-related
    pub fn is_auth_error(&self) -> bool {
        matches!(
            self,
            SecurityError::AuthenticationFailed(_)
                | SecurityError::InvalidCredentials
                | SecurityError::MfaRequired
                | SecurityError::InvalidMfaToken
                | SecurityError::SessionExpired
                | SecurityError::SessionNotFound
                | SecurityError::InvalidToken(_)
        )
    }

    /// Check if error is authorization-related
    pub fn is_authz_error(&self) -> bool {
        matches!(
            self,
            SecurityError::AccessDenied(_)
                | SecurityError::PermissionDenied(_)
                | SecurityError::PolicyViolation(_)
        )
    }

    /// Check if error is threat-related
    pub fn is_threat_error(&self) -> bool {
        matches!(
            self,
            SecurityError::RateLimitExceeded { .. }
                | SecurityError::BruteForceDetected { .. }
                | SecurityError::AnomalyDetected(_)
        )
    }

    /// Check if error should be logged as security incident
    pub fn is_security_incident(&self) -> bool {
        matches!(
            self,
            SecurityError::BruteForceDetected { .. }
                | SecurityError::AnomalyDetected(_)
                | SecurityError::AuditTrailCompromised(_)
                | SecurityError::ChainOfCustodyBroken { .. }
                | SecurityError::PolicyViolation(_)
        )
    }

    /// Get error severity level
    pub fn severity(&self) -> Severity {
        match self {
            SecurityError::BruteForceDetected { .. }
            | SecurityError::AuditTrailCompromised(_)
            | SecurityError::ChainOfCustodyBroken { .. } => Severity::Critical,

            SecurityError::AnomalyDetected(_)
            | SecurityError::PolicyViolation(_)
            | SecurityError::AccessDenied(_) => Severity::High,

            SecurityError::RateLimitExceeded { .. }
            | SecurityError::ValidationFailed(_)
            | SecurityError::PermissionDenied(_) => Severity::Medium,

            _ => Severity::Low,
        }
    }
}

/// Security event severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Low => write!(f, "LOW"),
            Severity::Medium => write!(f, "MEDIUM"),
            Severity::High => write!(f, "HIGH"),
            Severity::Critical => write!(f, "CRITICAL"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_classification() {
        let auth_err = SecurityError::InvalidCredentials;
        assert!(auth_err.is_auth_error());
        assert!(!auth_err.is_authz_error());

        let authz_err = SecurityError::AccessDenied("test".to_string());
        assert!(!authz_err.is_auth_error());
        assert!(authz_err.is_authz_error());
    }

    #[test]
    fn test_severity_levels() {
        let brute_force = SecurityError::BruteForceDetected {
            source: "192.168.1.1".to_string(),
        };
        assert_eq!(brute_force.severity(), Severity::Critical);

        let rate_limit = SecurityError::RateLimitExceeded {
            retry_after_secs: 60,
        };
        assert_eq!(rate_limit.severity(), Severity::Medium);
    }

    #[test]
    fn test_security_incidents() {
        let brute_force = SecurityError::BruteForceDetected {
            source: "test".to_string(),
        };
        assert!(brute_force.is_security_incident());

        let weak_password = SecurityError::WeakPassword("test".to_string());
        assert!(!weak_password.is_security_incident());
    }
}
