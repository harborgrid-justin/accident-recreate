//! SSO integration (SAML and OIDC)
//!
//! Provides Single Sign-On capabilities for enterprise authentication.

use crate::config::SsoConfig;
use crate::error::{Result, SecurityError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// SSO service for SAML and OIDC
pub struct SsoService {
    config: SsoConfig,
}

impl SsoService {
    /// Create a new SSO service
    pub fn new(config: SsoConfig) -> Self {
        Self { config }
    }

    /// Check if SAML is enabled
    pub fn is_saml_enabled(&self) -> bool {
        self.config.saml_enabled
    }

    /// Check if OIDC is enabled
    pub fn is_oidc_enabled(&self) -> bool {
        self.config.oidc_enabled
    }

    /// Generate SAML authentication request
    pub fn generate_saml_request(&self, relay_state: Option<String>) -> Result<SamlRequest> {
        if !self.is_saml_enabled() {
            return Err(SecurityError::ConfigurationError(
                "SAML is not enabled".to_string(),
            ));
        }

        let entity_id = self.config.saml_entity_id.as_ref().ok_or_else(|| {
            SecurityError::ConfigurationError("SAML entity ID not configured".to_string())
        })?;

        Ok(SamlRequest {
            id: uuid::Uuid::new_v4().to_string(),
            entity_id: entity_id.clone(),
            relay_state,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Validate SAML response
    pub fn validate_saml_response(&self, response: &SamlResponse) -> Result<SamlAssertion> {
        if !self.is_saml_enabled() {
            return Err(SecurityError::ConfigurationError(
                "SAML is not enabled".to_string(),
            ));
        }

        // Validate response timestamp (not too old)
        let now = chrono::Utc::now();
        let age = now.signed_duration_since(response.timestamp);
        if age.num_minutes() > 5 {
            return Err(SecurityError::InvalidToken(
                "SAML response expired".to_string(),
            ));
        }

        // Validate entity ID
        let expected_entity_id = self.config.saml_entity_id.as_ref().ok_or_else(|| {
            SecurityError::ConfigurationError("SAML entity ID not configured".to_string())
        })?;

        if &response.issuer != expected_entity_id {
            return Err(SecurityError::InvalidToken(
                "SAML response issuer mismatch".to_string(),
            ));
        }

        // In production, verify signature here using X.509 certificates
        // For now, return the assertion
        Ok(response.assertion.clone())
    }

    /// Generate OIDC authorization URL
    pub fn generate_oidc_auth_url(
        &self,
        redirect_uri: &str,
        state: &str,
        nonce: &str,
    ) -> Result<String> {
        if !self.is_oidc_enabled() {
            return Err(SecurityError::ConfigurationError(
                "OIDC is not enabled".to_string(),
            ));
        }

        let issuer = self.config.oidc_issuer.as_ref().ok_or_else(|| {
            SecurityError::ConfigurationError("OIDC issuer not configured".to_string())
        })?;

        let client_id = self.config.oidc_client_id.as_ref().ok_or_else(|| {
            SecurityError::ConfigurationError("OIDC client ID not configured".to_string())
        })?;

        Ok(format!(
            "{}/authorize?client_id={}&redirect_uri={}&response_type=code&scope=openid%20profile%20email&state={}&nonce={}",
            issuer,
            urlencoding::encode(client_id),
            urlencoding::encode(redirect_uri),
            urlencoding::encode(state),
            urlencoding::encode(nonce)
        ))
    }

    /// Validate OIDC ID token
    pub fn validate_oidc_token(&self, id_token: &str) -> Result<OidcClaims> {
        if !self.is_oidc_enabled() {
            return Err(SecurityError::ConfigurationError(
                "OIDC is not enabled".to_string(),
            ));
        }

        // In production, verify JWT signature using JWKS from issuer
        // For now, decode without verification (INSECURE - only for structure)
        let parts: Vec<&str> = id_token.split('.').collect();
        if parts.len() != 3 {
            return Err(SecurityError::InvalidToken(
                "Invalid JWT format".to_string(),
            ));
        }

        let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(parts[1])
            .map_err(|e| SecurityError::InvalidToken(format!("Invalid base64: {}", e)))?;

        let claims: OidcClaims = serde_json::from_slice(&payload)
            .map_err(|e| SecurityError::InvalidToken(format!("Invalid claims: {}", e)))?;

        // Validate issuer
        let expected_issuer = self.config.oidc_issuer.as_ref().ok_or_else(|| {
            SecurityError::ConfigurationError("OIDC issuer not configured".to_string())
        })?;

        if &claims.iss != expected_issuer {
            return Err(SecurityError::InvalidToken(
                "OIDC token issuer mismatch".to_string(),
            ));
        }

        // Validate audience
        let expected_client_id = self.config.oidc_client_id.as_ref().ok_or_else(|| {
            SecurityError::ConfigurationError("OIDC client ID not configured".to_string())
        })?;

        if &claims.aud != expected_client_id {
            return Err(SecurityError::InvalidToken(
                "OIDC token audience mismatch".to_string(),
            ));
        }

        // Validate expiration
        let now = chrono::Utc::now().timestamp();
        if claims.exp < now {
            return Err(SecurityError::InvalidToken(
                "OIDC token expired".to_string(),
            ));
        }

        Ok(claims)
    }
}

/// SAML authentication request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlRequest {
    /// Request ID
    pub id: String,
    /// Service provider entity ID
    pub entity_id: String,
    /// Relay state (optional)
    pub relay_state: Option<String>,
    /// Request timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// SAML authentication response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlResponse {
    /// Response ID
    pub id: String,
    /// Identity provider issuer
    pub issuer: String,
    /// Response timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// SAML assertion
    pub assertion: SamlAssertion,
}

/// SAML assertion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlAssertion {
    /// Subject (user identifier)
    pub subject: String,
    /// Subject name ID format
    pub name_id_format: String,
    /// Assertion attributes
    pub attributes: HashMap<String, Vec<String>>,
    /// Not before timestamp
    pub not_before: chrono::DateTime<chrono::Utc>,
    /// Not on or after timestamp
    pub not_on_or_after: chrono::DateTime<chrono::Utc>,
}

impl SamlAssertion {
    /// Get email attribute
    pub fn email(&self) -> Option<&str> {
        self.attributes
            .get("email")
            .and_then(|v| v.first())
            .map(|s| s.as_str())
    }

    /// Get name attribute
    pub fn name(&self) -> Option<&str> {
        self.attributes
            .get("name")
            .and_then(|v| v.first())
            .map(|s| s.as_str())
    }

    /// Check if assertion is still valid
    pub fn is_valid(&self) -> bool {
        let now = chrono::Utc::now();
        now >= self.not_before && now < self.not_on_or_after
    }
}

/// OIDC ID token claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcClaims {
    /// Issuer
    pub iss: String,
    /// Subject (user ID)
    pub sub: String,
    /// Audience (client ID)
    pub aud: String,
    /// Expiration time
    pub exp: i64,
    /// Issued at
    pub iat: i64,
    /// Nonce
    pub nonce: Option<String>,
    /// Email
    pub email: Option<String>,
    /// Email verified
    pub email_verified: Option<bool>,
    /// Name
    pub name: Option<String>,
    /// Given name
    pub given_name: Option<String>,
    /// Family name
    pub family_name: Option<String>,
    /// Picture URL
    pub picture: Option<String>,
}

/// SSO provider type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SsoProvider {
    /// SAML 2.0
    Saml,
    /// OpenID Connect
    Oidc,
}

impl std::fmt::Display for SsoProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SsoProvider::Saml => write!(f, "SAML"),
            SsoProvider::Oidc => write!(f, "OIDC"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_saml_config() -> SsoConfig {
        SsoConfig {
            saml_enabled: true,
            saml_entity_id: Some("https://accuscene.com/saml".to_string()),
            oidc_enabled: false,
            oidc_issuer: None,
            oidc_client_id: None,
        }
    }

    fn test_oidc_config() -> SsoConfig {
        SsoConfig {
            saml_enabled: false,
            saml_entity_id: None,
            oidc_enabled: true,
            oidc_issuer: Some("https://auth.example.com".to_string()),
            oidc_client_id: Some("accuscene-client".to_string()),
        }
    }

    #[test]
    fn test_saml_request_generation() {
        let service = SsoService::new(test_saml_config());
        let request = service.generate_saml_request(Some("test-state".to_string())).unwrap();

        assert!(!request.id.is_empty());
        assert_eq!(request.entity_id, "https://accuscene.com/saml");
        assert_eq!(request.relay_state, Some("test-state".to_string()));
    }

    #[test]
    fn test_oidc_auth_url_generation() {
        let service = SsoService::new(test_oidc_config());
        let url = service
            .generate_oidc_auth_url(
                "https://accuscene.com/callback",
                "random-state",
                "random-nonce",
            )
            .unwrap();

        assert!(url.contains("https://auth.example.com/authorize"));
        assert!(url.contains("client_id=accuscene-client"));
        assert!(url.contains("state=random-state"));
        assert!(url.contains("nonce=random-nonce"));
    }

    #[test]
    fn test_saml_assertion_validity() {
        let now = chrono::Utc::now();
        let assertion = SamlAssertion {
            subject: "user@example.com".to_string(),
            name_id_format: "urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress".to_string(),
            attributes: HashMap::new(),
            not_before: now - chrono::Duration::minutes(5),
            not_on_or_after: now + chrono::Duration::minutes(5),
        };

        assert!(assertion.is_valid());

        let expired = SamlAssertion {
            not_on_or_after: now - chrono::Duration::minutes(1),
            ..assertion
        };

        assert!(!expired.is_valid());
    }
}
