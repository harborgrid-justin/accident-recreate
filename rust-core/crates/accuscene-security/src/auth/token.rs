//! JWT token handling for API authentication
//!
//! Provides secure JWT token generation and validation.

use crate::config::JwtConfig;
use crate::error::{Result, SecurityError};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// JWT token service
pub struct TokenService {
    config: JwtConfig,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl TokenService {
    /// Create a new token service with a secret key
    pub fn new(config: JwtConfig, secret: &[u8]) -> Self {
        Self {
            config,
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
        }
    }

    /// Generate an access token
    pub fn generate_access_token(&self, user_id: &str, claims: TokenClaims) -> Result<String> {
        let now = chrono::Utc::now();
        let exp = now + chrono::Duration::seconds(self.config.expiry_secs as i64);

        let jwt_claims = JwtClaims {
            sub: user_id.to_string(),
            iss: self.config.issuer.clone(),
            aud: self.config.audience.clone(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            nbf: now.timestamp(),
            jti: uuid::Uuid::new_v4().to_string(),
            custom: claims,
        };

        let algorithm = self.parse_algorithm()?;
        let header = Header::new(algorithm);

        encode(&header, &jwt_claims, &self.encoding_key)
            .map_err(|e| SecurityError::Internal(format!("Token generation failed: {}", e)))
    }

    /// Generate a refresh token
    pub fn generate_refresh_token(&self, user_id: &str) -> Result<String> {
        let now = chrono::Utc::now();
        let exp = now + chrono::Duration::seconds(self.config.refresh_expiry_secs as i64);

        let jwt_claims = JwtClaims {
            sub: user_id.to_string(),
            iss: self.config.issuer.clone(),
            aud: self.config.audience.clone(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            nbf: now.timestamp(),
            jti: uuid::Uuid::new_v4().to_string(),
            custom: TokenClaims {
                token_type: TokenType::Refresh,
                ..Default::default()
            },
        };

        let algorithm = self.parse_algorithm()?;
        let header = Header::new(algorithm);

        encode(&header, &jwt_claims, &self.encoding_key)
            .map_err(|e| SecurityError::Internal(format!("Refresh token generation failed: {}", e)))
    }

    /// Validate and decode a token
    pub fn validate_token(&self, token: &str) -> Result<JwtClaims> {
        let algorithm = self.parse_algorithm()?;
        let mut validation = Validation::new(algorithm);
        validation.set_issuer(&[&self.config.issuer]);
        validation.set_audience(&[&self.config.audience]);

        let token_data = decode::<JwtClaims>(token, &self.decoding_key, &validation)
            .map_err(|e| SecurityError::InvalidToken(format!("Token validation failed: {}", e)))?;

        Ok(token_data.claims)
    }

    /// Validate refresh token
    pub fn validate_refresh_token(&self, token: &str) -> Result<JwtClaims> {
        let claims = self.validate_token(token)?;

        if claims.custom.token_type != TokenType::Refresh {
            return Err(SecurityError::InvalidToken(
                "Not a refresh token".to_string(),
            ));
        }

        Ok(claims)
    }

    /// Parse algorithm from config
    fn parse_algorithm(&self) -> Result<Algorithm> {
        match self.config.algorithm.as_str() {
            "HS256" => Ok(Algorithm::HS256),
            "HS384" => Ok(Algorithm::HS384),
            "HS512" => Ok(Algorithm::HS512),
            algo => Err(SecurityError::ConfigurationError(format!(
                "Unsupported algorithm: {}",
                algo
            ))),
        }
    }

    /// Extract user ID from token without full validation
    pub fn extract_user_id(&self, token: &str) -> Result<String> {
        let claims = self.validate_token(token)?;
        Ok(claims.sub)
    }

    /// Check if token is expired
    pub fn is_expired(&self, token: &str) -> bool {
        if let Ok(claims) = self.validate_token(token) {
            let now = chrono::Utc::now().timestamp();
            claims.exp < now
        } else {
            true
        }
    }
}

/// JWT claims structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    /// Subject (user ID)
    pub sub: String,
    /// Issuer
    pub iss: String,
    /// Audience
    pub aud: String,
    /// Expiration time
    pub exp: i64,
    /// Issued at
    pub iat: i64,
    /// Not before
    pub nbf: i64,
    /// JWT ID
    pub jti: String,
    /// Custom claims
    #[serde(flatten)]
    pub custom: TokenClaims,
}

/// Custom token claims
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenClaims {
    /// Token type
    #[serde(default)]
    pub token_type: TokenType,
    /// User roles
    #[serde(default)]
    pub roles: Vec<String>,
    /// User permissions
    #[serde(default)]
    pub permissions: Vec<String>,
    /// Session ID
    pub session_id: Option<String>,
    /// MFA verified
    #[serde(default)]
    pub mfa_verified: bool,
    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Token type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    /// Access token
    Access,
    /// Refresh token
    Refresh,
}

impl Default for TokenType {
    fn default() -> Self {
        TokenType::Access
    }
}

/// Token pair (access + refresh)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    /// Access token
    pub access_token: String,
    /// Refresh token
    pub refresh_token: String,
    /// Token type (always "Bearer")
    pub token_type: String,
    /// Expires in seconds
    pub expires_in: u64,
}

impl TokenPair {
    /// Create a new token pair
    pub fn new(access_token: String, refresh_token: String, expires_in: u64) -> Self {
        Self {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in,
        }
    }
}

/// Token blacklist for revoked tokens
pub struct TokenBlacklist {
    tokens: HashMap<String, chrono::DateTime<chrono::Utc>>,
}

impl TokenBlacklist {
    /// Create a new token blacklist
    pub fn new() -> Self {
        Self {
            tokens: HashMap::new(),
        }
    }

    /// Add token to blacklist
    pub fn revoke(&mut self, jti: String, expires_at: chrono::DateTime<chrono::Utc>) {
        self.tokens.insert(jti, expires_at);
    }

    /// Check if token is revoked
    pub fn is_revoked(&self, jti: &str) -> bool {
        self.tokens.contains_key(jti)
    }

    /// Clean up expired tokens from blacklist
    pub fn cleanup_expired(&mut self) -> usize {
        let now = chrono::Utc::now();
        let before_count = self.tokens.len();
        self.tokens.retain(|_, expires_at| *expires_at > now);
        before_count - self.tokens.len()
    }
}

impl Default for TokenBlacklist {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> JwtConfig {
        JwtConfig {
            expiry_secs: 3600,
            refresh_expiry_secs: 86400,
            issuer: "accuscene-test".to_string(),
            audience: "accuscene-api".to_string(),
            algorithm: "HS256".to_string(),
        }
    }

    fn test_service() -> TokenService {
        TokenService::new(test_config(), b"test-secret-key-32-bytes-long!!!")
    }

    #[test]
    fn test_access_token_generation() {
        let service = test_service();
        let claims = TokenClaims {
            roles: vec!["admin".to_string()],
            ..Default::default()
        };

        let token = service.generate_access_token("user123", claims).unwrap();
        assert!(!token.is_empty());
    }

    #[test]
    fn test_token_validation() {
        let service = test_service();
        let claims = TokenClaims::default();

        let token = service.generate_access_token("user123", claims).unwrap();
        let validated = service.validate_token(&token).unwrap();

        assert_eq!(validated.sub, "user123");
        assert_eq!(validated.iss, "accuscene-test");
        assert_eq!(validated.aud, "accuscene-api");
    }

    #[test]
    fn test_refresh_token() {
        let service = test_service();

        let token = service.generate_refresh_token("user123").unwrap();
        let validated = service.validate_refresh_token(&token).unwrap();

        assert_eq!(validated.sub, "user123");
        assert_eq!(validated.custom.token_type, TokenType::Refresh);
    }

    #[test]
    fn test_invalid_token() {
        let service = test_service();
        let result = service.validate_token("invalid.token.here");
        assert!(result.is_err());
    }

    #[test]
    fn test_user_id_extraction() {
        let service = test_service();
        let claims = TokenClaims::default();

        let token = service.generate_access_token("user123", claims).unwrap();
        let user_id = service.extract_user_id(&token).unwrap();

        assert_eq!(user_id, "user123");
    }

    #[test]
    fn test_token_blacklist() {
        let mut blacklist = TokenBlacklist::new();
        let jti = "token-id-123".to_string();
        let expires = chrono::Utc::now() + chrono::Duration::hours(1);

        assert!(!blacklist.is_revoked(&jti));

        blacklist.revoke(jti.clone(), expires);
        assert!(blacklist.is_revoked(&jti));
    }
}
