//! Refresh Token Management

use crate::{SSOUser, SSOError, SSOResult, config::SessionConfig};
use chrono::{Utc, Duration};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey, Algorithm};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Refresh token claims
#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenClaims {
    /// Subject (user ID)
    pub sub: String,

    /// User email
    pub email: String,

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

    /// Token ID (for revocation)
    pub jti: String,

    /// Provider
    pub provider: String,
}

/// Refresh token manager
pub struct RefreshTokenManager {
    config: SessionConfig,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    revoked_tokens: Arc<RwLock<HashMap<String, i64>>>, // jti -> revocation timestamp
}

impl RefreshTokenManager {
    /// Create new refresh token manager
    pub fn new(config: SessionConfig) -> Self {
        let secret = config.jwt_secret.as_bytes();
        Self {
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
            config,
            revoked_tokens: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create refresh token
    pub fn create_refresh_token(&self, user: &SSOUser, session_id: &Uuid) -> SSOResult<String> {
        let now = Utc::now();
        let exp = now + Duration::seconds(self.config.refresh_token_expiry);

        let claims = RefreshTokenClaims {
            sub: user.id.clone(),
            email: user.email.clone(),
            sid: session_id.to_string(),
            iat: now.timestamp(),
            exp: exp.timestamp(),
            iss: self.config.issuer.clone(),
            aud: self.config.audience.clone(),
            token_type: "refresh".to_string(),
            jti: Uuid::new_v4().to_string(),
            provider: user.provider.clone(),
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| SSOError::InternalError(format!("Failed to create refresh token: {}", e)))
    }

    /// Validate refresh token and return user and session ID
    pub async fn validate_refresh_token(&self, token: &str) -> SSOResult<(SSOUser, Uuid)> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[&self.config.issuer]);
        validation.set_audience(&[&self.config.audience]);

        let token_data = decode::<RefreshTokenClaims>(token, &self.decoding_key, &validation)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => SSOError::TokenExpired,
                _ => SSOError::TokenValidationFailed(e.to_string()),
            })?;

        let claims = token_data.claims;

        // Verify token type
        if claims.token_type != "refresh" {
            return Err(SSOError::InvalidToken);
        }

        // Check if token is revoked
        {
            let revoked = self.revoked_tokens.read().await;
            if revoked.contains_key(&claims.jti) {
                return Err(SSOError::InvalidToken);
            }
        }

        // Parse session ID
        let session_id = Uuid::parse_str(&claims.sid)
            .map_err(|_| SSOError::InvalidToken)?;

        // Reconstruct user from claims
        let user = SSOUser {
            id: claims.sub,
            email: claims.email,
            name: None,
            given_name: None,
            family_name: None,
            picture: None,
            metadata: serde_json::json!({
                "provider": claims.provider
            }),
            provider: claims.provider,
        };

        Ok((user, session_id))
    }

    /// Revoke refresh token
    pub async fn revoke_token(&self, token: &str) -> SSOResult<()> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[&self.config.issuer]);
        validation.set_audience(&[&self.config.audience]);
        validation.validate_exp = false; // Allow revoking expired tokens

        let token_data = decode::<RefreshTokenClaims>(token, &self.decoding_key, &validation)
            .map_err(|e| SSOError::TokenValidationFailed(e.to_string()))?;

        let claims = token_data.claims;

        // Add to revoked tokens
        {
            let mut revoked = self.revoked_tokens.write().await;
            revoked.insert(claims.jti, Utc::now().timestamp());
        }

        Ok(())
    }

    /// Clean up expired revoked tokens
    pub async fn cleanup_revoked_tokens(&self) -> usize {
        let now = Utc::now();
        let expiry_threshold = (now - Duration::seconds(self.config.refresh_token_expiry)).timestamp();

        let mut revoked = self.revoked_tokens.write().await;
        let before_count = revoked.len();

        revoked.retain(|_, &mut revocation_time| revocation_time > expiry_threshold);

        before_count - revoked.len()
    }

    /// Extract token ID without full validation
    pub fn extract_token_id(&self, token: &str) -> SSOResult<String> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[&self.config.issuer]);
        validation.set_audience(&[&self.config.audience]);
        validation.validate_exp = false;

        let token_data = decode::<RefreshTokenClaims>(token, &self.decoding_key, &validation)
            .map_err(|e| SSOError::TokenValidationFailed(e.to_string()))?;

        Ok(token_data.claims.jti)
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

    #[tokio::test]
    async fn test_refresh_token_creation_and_validation() {
        let config = create_test_config();
        let manager = RefreshTokenManager::new(config);
        let user = create_test_user();
        let session_id = Uuid::new_v4();

        let token = manager.create_refresh_token(&user, &session_id).unwrap();
        assert!(!token.is_empty());

        let (validated_user, validated_session_id) = manager.validate_refresh_token(&token).await.unwrap();
        assert_eq!(validated_user.id, user.id);
        assert_eq!(validated_session_id, session_id);
    }

    #[tokio::test]
    async fn test_token_revocation() {
        let config = create_test_config();
        let manager = RefreshTokenManager::new(config);
        let user = create_test_user();
        let session_id = Uuid::new_v4();

        let token = manager.create_refresh_token(&user, &session_id).unwrap();

        // Token should be valid initially
        assert!(manager.validate_refresh_token(&token).await.is_ok());

        // Revoke token
        manager.revoke_token(&token).await.unwrap();

        // Token should be invalid after revocation
        assert!(manager.validate_refresh_token(&token).await.is_err());
    }
}
