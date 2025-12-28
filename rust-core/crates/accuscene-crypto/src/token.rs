//! Secure token generation and validation
//!
//! Provides cryptographic token generation and validation with expiration.

use crate::error::{CryptoError, CryptoResult};
use crate::hash::sha::sha256;
use crate::random::generate_random_bytes;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use subtle::ConstantTimeEq;

/// A cryptographically secure token
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Token {
    /// The token value
    value: String,
    /// When the token was created (Unix timestamp)
    created_at: u64,
    /// When the token expires (Unix timestamp)
    expires_at: u64,
    /// Optional metadata
    metadata: TokenMetadata,
}

/// Metadata for tokens
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenMetadata {
    /// User-defined purpose
    pub purpose: Option<String>,
    /// User-defined subject (e.g., user ID)
    pub subject: Option<String>,
    /// Custom claims
    pub claims: std::collections::HashMap<String, String>,
}

impl Default for TokenMetadata {
    fn default() -> Self {
        Self {
            purpose: None,
            subject: None,
            claims: std::collections::HashMap::new(),
        }
    }
}

impl Token {
    /// Generate a new random token with a specific lifetime
    pub fn generate(lifetime: Duration) -> CryptoResult<Self> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let expires_at = now + lifetime.as_secs();

        // Generate 32 bytes of random data
        let random_bytes = generate_random_bytes(32)?;
        let value = base64::encode(random_bytes.as_bytes());

        Ok(Self {
            value,
            created_at: now,
            expires_at,
            metadata: TokenMetadata::default(),
        })
    }

    /// Generate a new token with metadata
    pub fn generate_with_metadata(
        lifetime: Duration,
        metadata: TokenMetadata,
    ) -> CryptoResult<Self> {
        let mut token = Self::generate(lifetime)?;
        token.metadata = metadata;
        Ok(token)
    }

    /// Get the token value
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Check if the token is expired
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now >= self.expires_at
    }

    /// Check if the token is valid (not expired)
    pub fn is_valid(&self) -> bool {
        !self.is_expired()
    }

    /// Get the remaining lifetime of the token
    pub fn remaining_lifetime(&self) -> Option<Duration> {
        if self.is_expired() {
            return None;
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Some(Duration::from_secs(self.expires_at - now))
    }

    /// Get when the token was created
    pub fn created_at(&self) -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(self.created_at)
    }

    /// Get when the token expires
    pub fn expires_at(&self) -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(self.expires_at)
    }

    /// Get the metadata
    pub fn metadata(&self) -> &TokenMetadata {
        &self.metadata
    }

    /// Encode the token to a string
    pub fn to_string(&self) -> CryptoResult<String> {
        let json = serde_json::to_string(self)?;
        Ok(base64::encode(json.as_bytes()))
    }

    /// Decode a token from a string
    pub fn from_string(encoded: &str) -> CryptoResult<Self> {
        let json_bytes = base64::decode(encoded)?;
        let json_str = std::str::from_utf8(&json_bytes)
            .map_err(|e| CryptoError::DecodingError(e.to_string()))?;
        let token = serde_json::from_str(json_str)?;
        Ok(token)
    }

    /// Verify that this token matches another token value (constant-time comparison)
    pub fn verify(&self, other: &str) -> bool {
        self.value.as_bytes().ct_eq(other.as_bytes()).into()
    }
}

/// Token generator for creating tokens with consistent settings
pub struct TokenGenerator {
    default_lifetime: Duration,
    default_metadata: TokenMetadata,
}

impl TokenGenerator {
    /// Create a new token generator with default settings
    pub fn new(default_lifetime: Duration) -> Self {
        Self {
            default_lifetime,
            default_metadata: TokenMetadata::default(),
        }
    }

    /// Set default metadata
    pub fn with_metadata(mut self, metadata: TokenMetadata) -> Self {
        self.default_metadata = metadata;
        self
    }

    /// Generate a new token with default settings
    pub fn generate(&self) -> CryptoResult<Token> {
        Token::generate_with_metadata(self.default_lifetime, self.default_metadata.clone())
    }

    /// Generate a token with custom lifetime
    pub fn generate_with_lifetime(&self, lifetime: Duration) -> CryptoResult<Token> {
        Token::generate_with_metadata(lifetime, self.default_metadata.clone())
    }

    /// Generate a token with custom metadata
    pub fn generate_custom(&self, metadata: TokenMetadata) -> CryptoResult<Token> {
        Token::generate_with_metadata(self.default_lifetime, metadata)
    }
}

/// Token validator for validating tokens
pub struct TokenValidator {
    /// Whether to allow expired tokens
    allow_expired: bool,
}

impl TokenValidator {
    /// Create a new token validator
    pub fn new() -> Self {
        Self {
            allow_expired: false,
        }
    }

    /// Allow expired tokens
    pub fn allow_expired(mut self) -> Self {
        self.allow_expired = true;
        self
    }

    /// Validate a token
    pub fn validate(&self, token: &Token) -> CryptoResult<()> {
        if !self.allow_expired && token.is_expired() {
            return Err(CryptoError::TokenExpired);
        }

        Ok(())
    }

    /// Validate and verify a token value
    pub fn validate_value(&self, token: &Token, value: &str) -> CryptoResult<bool> {
        self.validate(token)?;
        Ok(token.verify(value))
    }
}

impl Default for TokenValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate a simple random token string
pub fn generate_token_string(byte_length: usize) -> CryptoResult<String> {
    let random_bytes = generate_random_bytes(byte_length)?;
    Ok(base64::encode(random_bytes.as_bytes()))
}

/// Generate a token hash for secure storage
pub fn hash_token(token: &str) -> String {
    let hash = sha256(token.as_bytes());
    hex::encode(hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_token() {
        let token = Token::generate(Duration::from_secs(3600)).unwrap();
        assert!(!token.value().is_empty());
        assert!(token.is_valid());
        assert!(!token.is_expired());
    }

    #[test]
    fn test_token_expiration() {
        let token = Token::generate(Duration::from_secs(0)).unwrap();
        std::thread::sleep(Duration::from_millis(10));
        assert!(token.is_expired());
        assert!(!token.is_valid());
    }

    #[test]
    fn test_token_remaining_lifetime() {
        let token = Token::generate(Duration::from_secs(3600)).unwrap();
        let remaining = token.remaining_lifetime().unwrap();
        assert!(remaining.as_secs() > 3590); // Allow some margin
        assert!(remaining.as_secs() <= 3600);
    }

    #[test]
    fn test_expired_token_no_remaining_lifetime() {
        let token = Token::generate(Duration::from_secs(0)).unwrap();
        std::thread::sleep(Duration::from_millis(10));
        assert!(token.remaining_lifetime().is_none());
    }

    #[test]
    fn test_token_string_roundtrip() {
        let token = Token::generate(Duration::from_secs(3600)).unwrap();
        let encoded = token.to_string().unwrap();
        let decoded = Token::from_string(&encoded).unwrap();

        assert_eq!(token.value(), decoded.value());
        assert_eq!(token.created_at, decoded.created_at);
        assert_eq!(token.expires_at, decoded.expires_at);
    }

    #[test]
    fn test_token_verify() {
        let token = Token::generate(Duration::from_secs(3600)).unwrap();
        assert!(token.verify(token.value()));
        assert!(!token.verify("wrong_value"));
    }

    #[test]
    fn test_token_with_metadata() {
        let mut metadata = TokenMetadata::default();
        metadata.purpose = Some("api_access".to_string());
        metadata.subject = Some("user123".to_string());
        metadata
            .claims
            .insert("role".to_string(), "admin".to_string());

        let token = Token::generate_with_metadata(Duration::from_secs(3600), metadata).unwrap();

        assert_eq!(
            token.metadata().purpose,
            Some("api_access".to_string())
        );
        assert_eq!(token.metadata().subject, Some("user123".to_string()));
        assert_eq!(
            token.metadata().claims.get("role"),
            Some(&"admin".to_string())
        );
    }

    #[test]
    fn test_token_generator() {
        let generator = TokenGenerator::new(Duration::from_secs(3600));
        let token = generator.generate().unwrap();

        assert!(token.is_valid());
    }

    #[test]
    fn test_token_validator() {
        let validator = TokenValidator::new();
        let token = Token::generate(Duration::from_secs(3600)).unwrap();

        assert!(validator.validate(&token).is_ok());
    }

    #[test]
    fn test_token_validator_expired() {
        let validator = TokenValidator::new();
        let token = Token::generate(Duration::from_secs(0)).unwrap();
        std::thread::sleep(Duration::from_millis(10));

        assert!(validator.validate(&token).is_err());
    }

    #[test]
    fn test_token_validator_allow_expired() {
        let validator = TokenValidator::new().allow_expired();
        let token = Token::generate(Duration::from_secs(0)).unwrap();
        std::thread::sleep(Duration::from_millis(10));

        assert!(validator.validate(&token).is_ok());
    }

    #[test]
    fn test_generate_token_string() {
        let token1 = generate_token_string(32).unwrap();
        let token2 = generate_token_string(32).unwrap();

        assert_ne!(token1, token2);
        assert!(!token1.is_empty());
    }

    #[test]
    fn test_hash_token() {
        let token = "my_secret_token";
        let hash = hash_token(token);

        assert_eq!(hash.len(), 64); // SHA-256 produces 64 hex characters
    }

    #[test]
    fn test_hash_token_deterministic() {
        let token = "my_secret_token";
        let hash1 = hash_token(token);
        let hash2 = hash_token(token);

        assert_eq!(hash1, hash2);
    }
}
