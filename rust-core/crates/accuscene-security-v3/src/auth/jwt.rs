//! JWT token generation and validation
//!
//! Provides secure JWT token management for authentication and authorization.

use crate::config::JwtConfig;
use crate::error::{SecurityError, SecurityResult};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use zeroize::Zeroizing;

/// JWT claims structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,

    /// Issuer
    pub iss: String,

    /// Audience
    pub aud: String,

    /// Expiration time (Unix timestamp)
    pub exp: i64,

    /// Issued at (Unix timestamp)
    pub iat: i64,

    /// Not before (Unix timestamp)
    pub nbf: i64,

    /// JWT ID
    pub jti: String,

    /// User roles
    #[serde(default)]
    pub roles: Vec<String>,

    /// User permissions
    #[serde(default)]
    pub permissions: Vec<String>,

    /// Custom claims
    #[serde(flatten)]
    pub custom: std::collections::HashMap<String, serde_json::Value>,
}

impl Claims {
    /// Create new claims
    pub fn new(user_id: impl Into<String>, issuer: impl Into<String>, audience: impl Into<String>) -> Self {
        let now = Utc::now();

        Self {
            sub: user_id.into(),
            iss: issuer.into(),
            aud: audience.into(),
            exp: (now + Duration::hours(1)).timestamp(),
            iat: now.timestamp(),
            nbf: now.timestamp(),
            jti: Uuid::new_v4().to_string(),
            roles: Vec::new(),
            permissions: Vec::new(),
            custom: std::collections::HashMap::new(),
        }
    }

    /// Set expiration
    pub fn with_expiration(mut self, exp: i64) -> Self {
        self.exp = exp;
        self
    }

    /// Set roles
    pub fn with_roles(mut self, roles: Vec<String>) -> Self {
        self.roles = roles;
        self
    }

    /// Set permissions
    pub fn with_permissions(mut self, permissions: Vec<String>) -> Self {
        self.permissions = permissions;
        self
    }

    /// Add custom claim
    pub fn with_custom(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.custom.insert(key.into(), value);
        self
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() > self.exp
    }

    /// Get time until expiration
    pub fn time_until_expiry(&self) -> Option<Duration> {
        let now = Utc::now().timestamp();
        if self.exp > now {
            Some(Duration::seconds(self.exp - now))
        } else {
            None
        }
    }
}

/// Token type for different use cases
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    /// Access token (short-lived)
    Access,
    /// Refresh token (long-lived)
    Refresh,
}

/// JWT manager for token operations
#[derive(Debug)]
pub struct JwtManager {
    config: JwtConfig,
    algorithm: Algorithm,
}

impl JwtManager {
    /// Create a new JWT manager
    pub fn new(config: JwtConfig) -> SecurityResult<Self> {
        if config.secret.is_empty() {
            return Err(SecurityError::ConfigError(
                "JWT secret cannot be empty".to_string(),
            ));
        }

        let algorithm = Self::parse_algorithm(&config.algorithm)?;

        Ok(Self { config, algorithm })
    }

    /// Parse algorithm string
    fn parse_algorithm(algo: &str) -> SecurityResult<Algorithm> {
        match algo.to_uppercase().as_str() {
            "HS256" => Ok(Algorithm::HS256),
            "HS384" => Ok(Algorithm::HS384),
            "HS512" => Ok(Algorithm::HS512),
            "RS256" => Ok(Algorithm::RS256),
            "RS384" => Ok(Algorithm::RS384),
            "RS512" => Ok(Algorithm::RS512),
            "ES256" => Ok(Algorithm::ES256),
            "ES384" => Ok(Algorithm::ES384),
            _ => Err(SecurityError::ConfigError(format!(
                "Unsupported JWT algorithm: {}",
                algo
            ))),
        }
    }

    /// Generate an access token
    pub fn generate_access_token(&self, user_id: impl Into<String>, roles: Vec<String>) -> SecurityResult<String> {
        let claims = Claims::new(user_id, &self.config.issuer, &self.config.audience)
            .with_expiration((Utc::now() + Duration::seconds(self.config.access_token_expiry_secs)).timestamp())
            .with_roles(roles);

        self.encode_token(&claims)
    }

    /// Generate a refresh token
    pub fn generate_refresh_token(&self, user_id: impl Into<String>) -> SecurityResult<String> {
        let claims = Claims::new(user_id, &self.config.issuer, &self.config.audience)
            .with_expiration((Utc::now() + Duration::seconds(self.config.refresh_token_expiry_secs)).timestamp())
            .with_custom("token_type".to_string(), serde_json::json!("refresh"));

        self.encode_token(&claims)
    }

    /// Generate both access and refresh tokens
    pub fn generate_token_pair(
        &self,
        user_id: impl Into<String>,
        roles: Vec<String>,
    ) -> SecurityResult<TokenPair> {
        let user_id = user_id.into();
        let access_token = self.generate_access_token(&user_id, roles)?;
        let refresh_token = self.generate_refresh_token(&user_id)?;

        Ok(TokenPair {
            access_token,
            refresh_token,
        })
    }

    /// Encode a token
    fn encode_token(&self, claims: &Claims) -> SecurityResult<String> {
        let secret = Zeroizing::new(self.config.secret.as_bytes());
        let key = EncodingKey::from_secret(&secret);

        let header = Header::new(self.algorithm);

        encode(&header, claims, &key).map_err(|e| {
            SecurityError::TokenError(format!("Failed to encode token: {}", e))
        })
    }

    /// Validate and decode a token
    pub fn validate_token(&self, token: &str) -> SecurityResult<Claims> {
        let secret = Zeroizing::new(self.config.secret.as_bytes());
        let key = DecodingKey::from_secret(&secret);

        let mut validation = Validation::new(self.algorithm);
        validation.set_issuer(&[&self.config.issuer]);
        validation.set_audience(&[&self.config.audience]);

        let token_data = decode::<Claims>(token, &key, &validation).map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => SecurityError::TokenExpired,
            jsonwebtoken::errors::ErrorKind::InvalidToken => SecurityError::InvalidToken,
            jsonwebtoken::errors::ErrorKind::InvalidSignature => SecurityError::InvalidToken,
            jsonwebtoken::errors::ErrorKind::InvalidIssuer => {
                SecurityError::TokenError("Invalid issuer".to_string())
            }
            jsonwebtoken::errors::ErrorKind::InvalidAudience => {
                SecurityError::TokenError("Invalid audience".to_string())
            }
            _ => SecurityError::TokenError(format!("Token validation failed: {}", e)),
        })?;

        Ok(token_data.claims)
    }

    /// Refresh an access token using a refresh token
    pub fn refresh_access_token(&self, refresh_token: &str, roles: Vec<String>) -> SecurityResult<String> {
        let claims = self.validate_token(refresh_token)?;

        // Verify it's a refresh token
        if claims.custom.get("token_type") != Some(&serde_json::json!("refresh")) {
            return Err(SecurityError::TokenError(
                "Invalid token type for refresh".to_string(),
            ));
        }

        self.generate_access_token(&claims.sub, roles)
    }

    /// Extract user ID from token without full validation (use with caution)
    pub fn extract_user_id(&self, token: &str) -> SecurityResult<String> {
        let claims = self.validate_token(token)?;
        Ok(claims.sub)
    }

    /// Extract roles from token
    pub fn extract_roles(&self, token: &str) -> SecurityResult<Vec<String>> {
        let claims = self.validate_token(token)?;
        Ok(claims.roles)
    }
}

/// Token pair (access + refresh)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    /// Access token
    pub access_token: String,
    /// Refresh token
    pub refresh_token: String,
}

impl TokenPair {
    /// Get access token
    pub fn access(&self) -> &str {
        &self.access_token
    }

    /// Get refresh token
    pub fn refresh(&self) -> &str {
        &self.refresh_token
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> JwtConfig {
        JwtConfig {
            secret: "test-secret-key-that-is-long-enough".to_string(),
            issuer: "test-issuer".to_string(),
            audience: "test-audience".to_string(),
            access_token_expiry_secs: 900,
            refresh_token_expiry_secs: 604800,
            algorithm: "HS256".to_string(),
        }
    }

    #[test]
    fn test_token_generation() {
        let config = create_test_config();
        let manager = JwtManager::new(config).unwrap();

        let token = manager
            .generate_access_token("user123", vec!["admin".to_string()])
            .unwrap();

        assert!(!token.is_empty());
    }

    #[test]
    fn test_token_validation() {
        let config = create_test_config();
        let manager = JwtManager::new(config).unwrap();

        let token = manager
            .generate_access_token("user123", vec!["admin".to_string()])
            .unwrap();

        let claims = manager.validate_token(&token).unwrap();

        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.roles, vec!["admin".to_string()]);
        assert_eq!(claims.iss, "test-issuer");
        assert_eq!(claims.aud, "test-audience");
    }

    #[test]
    fn test_token_pair_generation() {
        let config = create_test_config();
        let manager = JwtManager::new(config).unwrap();

        let pair = manager
            .generate_token_pair("user123", vec!["admin".to_string()])
            .unwrap();

        assert!(!pair.access_token.is_empty());
        assert!(!pair.refresh_token.is_empty());

        // Validate both tokens
        let access_claims = manager.validate_token(&pair.access_token).unwrap();
        let refresh_claims = manager.validate_token(&pair.refresh_token).unwrap();

        assert_eq!(access_claims.sub, "user123");
        assert_eq!(refresh_claims.sub, "user123");
    }

    #[test]
    fn test_token_refresh() {
        let config = create_test_config();
        let manager = JwtManager::new(config).unwrap();

        let refresh_token = manager.generate_refresh_token("user123").unwrap();
        let new_access_token = manager
            .refresh_access_token(&refresh_token, vec!["admin".to_string()])
            .unwrap();

        let claims = manager.validate_token(&new_access_token).unwrap();
        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.roles, vec!["admin".to_string()]);
    }

    #[test]
    fn test_invalid_token() {
        let config = create_test_config();
        let manager = JwtManager::new(config).unwrap();

        let result = manager.validate_token("invalid.token.here");
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_user_id() {
        let config = create_test_config();
        let manager = JwtManager::new(config).unwrap();

        let token = manager
            .generate_access_token("user123", vec![])
            .unwrap();

        let user_id = manager.extract_user_id(&token).unwrap();
        assert_eq!(user_id, "user123");
    }

    #[test]
    fn test_claims_expiration_check() {
        let claims = Claims::new("user123", "issuer", "audience")
            .with_expiration((Utc::now() - Duration::hours(1)).timestamp());

        assert!(claims.is_expired());

        let claims = Claims::new("user123", "issuer", "audience")
            .with_expiration((Utc::now() + Duration::hours(1)).timestamp());

        assert!(!claims.is_expired());
    }
}
