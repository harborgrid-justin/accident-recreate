//! Error types for the security module

use thiserror::Error;

/// Result type alias for security operations
pub type SecurityResult<T> = Result<T, SecurityError>;

/// Comprehensive error types for security operations
#[derive(Error, Debug)]
pub enum SecurityError {
    /// Authentication errors
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    /// Authorization errors
    #[error("Authorization denied: {0}")]
    AuthorizationDenied(String),

    /// Invalid credentials
    #[error("Invalid credentials")]
    InvalidCredentials,

    /// Token errors
    #[error("Token error: {0}")]
    TokenError(String),

    /// Token expired
    #[error("Token expired")]
    TokenExpired,

    /// Invalid token
    #[error("Invalid token")]
    InvalidToken,

    /// Session errors
    #[error("Session error: {0}")]
    SessionError(String),

    /// Session expired
    #[error("Session expired")]
    SessionExpired,

    /// Cryptography errors
    #[error("Cryptography error: {0}")]
    CryptoError(String),

    /// Encryption failed
    #[error("Encryption failed")]
    EncryptionFailed,

    /// Decryption failed
    #[error("Decryption failed")]
    DecryptionFailed,

    /// Key derivation failed
    #[error("Key derivation failed")]
    KeyDerivationFailed,

    /// Invalid key
    #[error("Invalid key: {0}")]
    InvalidKey(String),

    /// RBAC errors
    #[error("RBAC error: {0}")]
    RbacError(String),

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Role not found
    #[error("Role not found: {0}")]
    RoleNotFound(String),

    /// Policy evaluation failed
    #[error("Policy evaluation failed: {0}")]
    PolicyEvaluationFailed(String),

    /// MFA errors
    #[error("MFA error: {0}")]
    MfaError(String),

    /// Invalid TOTP code
    #[error("Invalid TOTP code")]
    InvalidTotpCode,

    /// WebAuthn error
    #[error("WebAuthn error: {0}")]
    WebAuthnError(String),

    /// OAuth errors
    #[error("OAuth error: {0}")]
    OAuthError(String),

    /// SAML errors
    #[error("SAML error: {0}")]
    SamlError(String),

    /// Audit errors
    #[error("Audit error: {0}")]
    AuditError(String),

    /// Compliance errors
    #[error("Compliance error: {0}")]
    ComplianceError(String),

    /// Data retention policy violation
    #[error("Data retention policy violation: {0}")]
    RetentionPolicyViolation(String),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Invalid input
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// UTF-8 error
    #[error("UTF-8 error: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),

    /// Base64 decode error
    #[error("Base64 decode error: {0}")]
    Base64DecodeError(#[from] base64::DecodeError),

    /// Generic error
    #[error("Security error: {0}")]
    Other(String),
}

impl SecurityError {
    /// Check if error is related to authentication
    pub fn is_auth_error(&self) -> bool {
        matches!(
            self,
            Self::AuthenticationFailed(_)
                | Self::InvalidCredentials
                | Self::TokenError(_)
                | Self::TokenExpired
                | Self::InvalidToken
                | Self::SessionError(_)
                | Self::SessionExpired
        )
    }

    /// Check if error is related to authorization
    pub fn is_authz_error(&self) -> bool {
        matches!(
            self,
            Self::AuthorizationDenied(_)
                | Self::PermissionDenied(_)
                | Self::RbacError(_)
                | Self::PolicyEvaluationFailed(_)
        )
    }

    /// Check if error is related to cryptography
    pub fn is_crypto_error(&self) -> bool {
        matches!(
            self,
            Self::CryptoError(_)
                | Self::EncryptionFailed
                | Self::DecryptionFailed
                | Self::KeyDerivationFailed
                | Self::InvalidKey(_)
        )
    }

    /// Convert to a safe error message that doesn't leak sensitive information
    pub fn safe_message(&self) -> String {
        match self {
            Self::AuthenticationFailed(_) => "Authentication failed".to_string(),
            Self::InvalidCredentials => "Invalid credentials".to_string(),
            Self::TokenExpired => "Token expired".to_string(),
            Self::InvalidToken => "Invalid token".to_string(),
            Self::SessionExpired => "Session expired".to_string(),
            Self::PermissionDenied(_) => "Permission denied".to_string(),
            Self::InvalidTotpCode => "Invalid verification code".to_string(),
            _ => "Security error occurred".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_classification() {
        let auth_error = SecurityError::AuthenticationFailed("test".to_string());
        assert!(auth_error.is_auth_error());
        assert!(!auth_error.is_authz_error());

        let authz_error = SecurityError::PermissionDenied("test".to_string());
        assert!(!authz_error.is_auth_error());
        assert!(authz_error.is_authz_error());

        let crypto_error = SecurityError::EncryptionFailed;
        assert!(crypto_error.is_crypto_error());
    }

    #[test]
    fn test_safe_message() {
        let error = SecurityError::AuthenticationFailed("User password incorrect".to_string());
        assert_eq!(error.safe_message(), "Authentication failed");
        assert!(!error.safe_message().contains("password"));
    }
}
