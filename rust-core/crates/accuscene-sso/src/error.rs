//! Error types for SSO module

use thiserror::Error;

/// SSO error types
#[derive(Debug, Error)]
pub enum SSOError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Provider not found
    #[error("SSO provider '{0}' not found")]
    ProviderNotFound(String),

    /// Invalid credentials
    #[error("Invalid credentials")]
    InvalidCredentials,

    /// Authentication failed
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    /// Token validation failed
    #[error("Token validation failed: {0}")]
    TokenValidationFailed(String),

    /// Token expired
    #[error("Token has expired")]
    TokenExpired,

    /// Invalid token
    #[error("Invalid token")]
    InvalidToken,

    /// Session not found
    #[error("Session not found")]
    SessionNotFound,

    /// Session expired
    #[error("Session has expired")]
    SessionExpired,

    /// MFA required
    #[error("Multi-factor authentication required")]
    MFARequired,

    /// Invalid MFA code
    #[error("Invalid MFA code")]
    InvalidMFACode,

    /// MFA not configured
    #[error("MFA not configured for user")]
    MFANotConfigured,

    /// SAML error
    #[error("SAML error: {0}")]
    SAMLError(String),

    /// OIDC error
    #[error("OIDC error: {0}")]
    OIDCError(String),

    /// OAuth2 error
    #[error("OAuth2 error: {0}")]
    OAuth2Error(String),

    /// LDAP error
    #[error("LDAP error: {0}")]
    LDAPError(String),

    /// Network error
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Invalid state parameter
    #[error("Invalid state parameter")]
    InvalidState,

    /// CSRF token mismatch
    #[error("CSRF token mismatch")]
    CSRFTokenMismatch,

    /// Rate limit exceeded
    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    /// Account locked
    #[error("Account locked due to too many failed attempts")]
    AccountLocked,

    /// Invalid redirect URI
    #[error("Invalid redirect URI")]
    InvalidRedirectURI,

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Database error
    #[error("Database error: {0}")]
    DatabaseError(String),

    /// Internal error
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Result type for SSO operations
pub type SSOResult<T> = Result<T, SSOError>;

// Implement conversions from common error types

impl From<jsonwebtoken::errors::Error> for SSOError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        match err.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => SSOError::TokenExpired,
            _ => SSOError::TokenValidationFailed(err.to_string()),
        }
    }
}

impl From<serde_json::Error> for SSOError {
    fn from(err: serde_json::Error) -> Self {
        SSOError::SerializationError(err.to_string())
    }
}

impl From<reqwest::Error> for SSOError {
    fn from(err: reqwest::Error) -> Self {
        SSOError::NetworkError(err.to_string())
    }
}

impl From<url::ParseError> for SSOError {
    fn from(err: url::ParseError) -> Self {
        SSOError::ConfigError(format!("URL parse error: {}", err))
    }
}

impl From<base64::DecodeError> for SSOError {
    fn from(err: base64::DecodeError) -> Self {
        SSOError::InternalError(format!("Base64 decode error: {}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = SSOError::InvalidCredentials;
        assert_eq!(err.to_string(), "Invalid credentials");

        let err = SSOError::ProviderNotFound("okta".to_string());
        assert_eq!(err.to_string(), "SSO provider 'okta' not found");
    }

    #[test]
    fn test_error_conversion() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let sso_err: SSOError = json_err.into();
        assert!(matches!(sso_err, SSOError::SerializationError(_)));
    }
}
