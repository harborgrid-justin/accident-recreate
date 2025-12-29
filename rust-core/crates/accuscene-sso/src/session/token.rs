//! JWT Token Management

use crate::{SSOUser, SSOError, SSOResult, config::SessionConfig};
use chrono::{Utc, Duration};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey, Algorithm};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// JWT token claims
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    /// Subject (user ID)
    pub sub: String,

    /// User email
    pub email: String,

    /// User name
    pub name: Option<String>,

    /// Session ID
    pub sid: String,

    /// Issued at
    pub iat: i64,

    /// Expiration time
    pub exp: i64,

    /// Issuer
    pub iss: String,

    /// Audience
    pub aud: String,

    /// Token type
    pub token_type: String,

    /// Provider
    pub provider: String,

    /// MFA verified
    pub mfa_verified: bool,
}

/// Token manager for creating and validating JWT tokens
pub struct TokenManager {
    config: SessionConfig,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl TokenManager {
    /// Create new token manager
    pub fn new(config: SessionConfig) -> Self {
        let secret = config.jwt_secret.as_bytes();
        Self {
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
            config,
        }
    }

    /// Create access token
    pub fn create_access_token(&self, user: &SSOUser, session_id: &Uuid) -> SSOResult<String> {
        let now = Utc::now();
        let exp = now + Duration::seconds(self.config.access_token_expiry);

        let claims = TokenClaims {
            sub: user.id.clone(),
            email: user.email.clone(),
            name: user.name.clone(),
            sid: session_id.to_string(),
            iat: now.timestamp(),
            exp: exp.timestamp(),
            iss: self.config.issuer.clone(),
            aud: self.config.audience.clone(),
            token_type: "access".to_string(),
            provider: user.provider.clone(),
            mfa_verified: false, // Updated after MFA verification
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| SSOError::InternalError(format!("Failed to create token: {}", e)))
    }

    /// Validate access token and return user
    pub async fn validate_access_token(&self, token: &str) -> SSOResult<SSOUser> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[&self.config.issuer]);
        validation.set_audience(&[&self.config.audience]);

        let token_data = decode::<TokenClaims>(token, &self.decoding_key, &validation)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => SSOError::TokenExpired,
                _ => SSOError::TokenValidationFailed(e.to_string()),
            })?;

        let claims = token_data.claims;

        // Verify token type
        if claims.token_type != "access" {
            return Err(SSOError::InvalidToken);
        }

        // Reconstruct user from claims
        Ok(SSOUser {
            id: claims.sub,
            email: claims.email,
            name: claims.name.clone(),
            given_name: None,
            family_name: None,
            picture: None,
            metadata: serde_json::json!({
                "provider": claims.provider,
                "mfa_verified": claims.mfa_verified
            }),
            provider: claims.provider,
        })
    }

    /// Create MFA challenge token
    pub fn create_mfa_token(&self, user: &SSOUser, session_id: &Uuid) -> SSOResult<String> {
        let now = Utc::now();
        let exp = now + Duration::minutes(5); // MFA tokens expire quickly

        let claims = TokenClaims {
            sub: user.id.clone(),
            email: user.email.clone(),
            name: user.name.clone(),
            sid: session_id.to_string(),
            iat: now.timestamp(),
            exp: exp.timestamp(),
            iss: self.config.issuer.clone(),
            aud: self.config.audience.clone(),
            token_type: "mfa_challenge".to_string(),
            provider: user.provider.clone(),
            mfa_verified: false,
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| SSOError::InternalError(format!("Failed to create MFA token: {}", e)))
    }

    /// Validate MFA token
    pub fn validate_mfa_token(&self, token: &str) -> SSOResult<TokenClaims> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[&self.config.issuer]);
        validation.set_audience(&[&self.config.audience]);

        let token_data = decode::<TokenClaims>(token, &self.decoding_key, &validation)?;

        let claims = token_data.claims;

        // Verify token type
        if claims.token_type != "mfa_challenge" {
            return Err(SSOError::InvalidToken);
        }

        Ok(claims)
    }

    /// Extract session ID from token without full validation
    pub fn extract_session_id(&self, token: &str) -> SSOResult<Uuid> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[&self.config.issuer]);
        validation.set_audience(&[&self.config.audience]);
        validation.validate_exp = false; // Don't validate expiration for extraction

        let token_data = decode::<TokenClaims>(token, &self.decoding_key, &validation)
            .map_err(|e| SSOError::TokenValidationFailed(e.to_string()))?;

        Uuid::parse_str(&token_data.claims.sid)
            .map_err(|e| SSOError::InvalidToken)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> SessionConfig {
        SessionConfig {
            jwt_secret: "test-secret-key-must-be-at-least-32-characters-long".to_string(),
            access_token_expiry: 900,
            refresh_token_expiry: 604800,
            session_timeout: 86400,
            max_sessions_per_user: 5,
            issuer: "test-issuer".to_string(),
            audience: "test-audience".to_string(),
        }
    }

    fn create_test_user() -> SSOUser {
        SSOUser {
            id: "user123".to_string(),
            email: "user@example.com".to_string(),
            name: Some("Test User".to_string()),
            given_name: Some("Test".to_string()),
            family_name: Some("User".to_string()),
            picture: None,
            metadata: serde_json::json!({}),
            provider: "test".to_string(),
        }
    }

    #[test]
    fn test_token_creation_and_validation() {
        let config = create_test_config();
        let manager = TokenManager::new(config);
        let user = create_test_user();
        let session_id = Uuid::new_v4();

        let token = manager.create_access_token(&user, &session_id).unwrap();
        assert!(!token.is_empty());
    }

    #[tokio::test]
    async fn test_token_validation() {
        let config = create_test_config();
        let manager = TokenManager::new(config);
        let user = create_test_user();
        let session_id = Uuid::new_v4();

        let token = manager.create_access_token(&user, &session_id).unwrap();
        let validated_user = manager.validate_access_token(&token).await.unwrap();

        assert_eq!(validated_user.id, user.id);
        assert_eq!(validated_user.email, user.email);
    }

    #[test]
    fn test_session_id_extraction() {
        let config = create_test_config();
        let manager = TokenManager::new(config);
        let user = create_test_user();
        let session_id = Uuid::new_v4();

        let token = manager.create_access_token(&user, &session_id).unwrap();
        let extracted_id = manager.extract_session_id(&token).unwrap();

        assert_eq!(extracted_id, session_id);
    }
}
