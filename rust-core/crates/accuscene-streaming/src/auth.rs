//! WebSocket authentication and authorization.

use crate::error::{Result, StreamingError};
use crate::event::UserId;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Authentication token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    /// Token value
    pub token: String,
    /// User ID associated with this token
    pub user_id: UserId,
    /// Token expiration time
    pub expires_at: Option<DateTime<Utc>>,
    /// Additional claims
    pub claims: HashMap<String, serde_json::Value>,
}

impl AuthToken {
    /// Create a new auth token
    pub fn new(token: impl Into<String>, user_id: impl Into<UserId>) -> Self {
        Self {
            token: token.into(),
            user_id: user_id.into(),
            expires_at: None,
            claims: HashMap::new(),
        }
    }

    /// Set expiration time
    pub fn with_expiration(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    /// Add a claim
    pub fn with_claim(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.claims.insert(key.into(), value);
        self
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    /// Get a claim value
    pub fn get_claim(&self, key: &str) -> Option<&serde_json::Value> {
        self.claims.get(key)
    }
}

/// Authentication result
#[derive(Debug, Clone)]
pub struct AuthResult {
    /// Whether authentication succeeded
    pub success: bool,
    /// User ID if authenticated
    pub user_id: Option<UserId>,
    /// Error message if failed
    pub error: Option<String>,
    /// Authenticated token
    pub token: Option<AuthToken>,
}

impl AuthResult {
    /// Create a successful auth result
    pub fn success(user_id: impl Into<UserId>, token: AuthToken) -> Self {
        Self {
            success: true,
            user_id: Some(user_id.into()),
            error: None,
            token: Some(token),
        }
    }

    /// Create a failed auth result
    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            success: false,
            user_id: None,
            error: Some(error.into()),
            token: None,
        }
    }
}

/// Authenticator trait for validating tokens
#[async_trait]
pub trait Authenticator: Send + Sync {
    /// Authenticate a token
    async fn authenticate(&self, token: &str) -> Result<AuthResult>;

    /// Validate a token (check if still valid)
    async fn validate(&self, token: &str) -> Result<bool>;

    /// Revoke a token
    async fn revoke(&self, token: &str) -> Result<()>;
}

/// Simple in-memory authenticator
pub struct InMemoryAuthenticator {
    /// Valid tokens
    tokens: Arc<RwLock<HashMap<String, AuthToken>>>,
}

impl InMemoryAuthenticator {
    /// Create a new in-memory authenticator
    pub fn new() -> Self {
        Self {
            tokens: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a valid token
    pub fn add_token(&self, token: AuthToken) {
        self.tokens.write().insert(token.token.clone(), token);
    }

    /// Remove a token
    pub fn remove_token(&self, token: &str) {
        self.tokens.write().remove(token);
    }

    /// Get all tokens for a user
    pub fn get_user_tokens(&self, user_id: &UserId) -> Vec<AuthToken> {
        self.tokens
            .read()
            .values()
            .filter(|t| &t.user_id == user_id)
            .cloned()
            .collect()
    }

    /// Clear expired tokens
    pub fn clear_expired(&self) {
        self.tokens.write().retain(|_, token| !token.is_expired());
    }
}

impl Default for InMemoryAuthenticator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Authenticator for InMemoryAuthenticator {
    async fn authenticate(&self, token: &str) -> Result<AuthResult> {
        let tokens = self.tokens.read();

        if let Some(auth_token) = tokens.get(token) {
            if auth_token.is_expired() {
                return Ok(AuthResult::failure("Token expired"));
            }

            Ok(AuthResult::success(
                auth_token.user_id.clone(),
                auth_token.clone(),
            ))
        } else {
            Ok(AuthResult::failure("Invalid token"))
        }
    }

    async fn validate(&self, token: &str) -> Result<bool> {
        let tokens = self.tokens.read();

        if let Some(auth_token) = tokens.get(token) {
            Ok(!auth_token.is_expired())
        } else {
            Ok(false)
        }
    }

    async fn revoke(&self, token: &str) -> Result<()> {
        self.tokens.write().remove(token);
        Ok(())
    }
}

/// Permission-based authorization
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Permission {
    /// Read events
    ReadEvents,
    /// Publish events
    PublishEvents,
    /// Join rooms
    JoinRooms,
    /// Create rooms
    CreateRooms,
    /// Manage users
    ManageUsers,
    /// Admin access
    Admin,
    /// Custom permission
    Custom(String),
}

/// Authorizer trait for checking permissions
#[async_trait]
pub trait Authorizer: Send + Sync {
    /// Check if user has permission
    async fn has_permission(&self, user_id: &UserId, permission: &Permission) -> Result<bool>;

    /// Grant permission to user
    async fn grant_permission(&self, user_id: &UserId, permission: Permission) -> Result<()>;

    /// Revoke permission from user
    async fn revoke_permission(&self, user_id: &UserId, permission: &Permission) -> Result<()>;
}

/// Simple role-based authorizer
pub struct RoleBasedAuthorizer {
    /// User permissions
    permissions: Arc<RwLock<HashMap<UserId, Vec<Permission>>>>,
}

impl RoleBasedAuthorizer {
    /// Create a new role-based authorizer
    pub fn new() -> Self {
        Self {
            permissions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Set user permissions
    pub fn set_permissions(&self, user_id: impl Into<UserId>, permissions: Vec<Permission>) {
        self.permissions
            .write()
            .insert(user_id.into(), permissions);
    }

    /// Get user permissions
    pub fn get_permissions(&self, user_id: &UserId) -> Vec<Permission> {
        self.permissions
            .read()
            .get(user_id)
            .cloned()
            .unwrap_or_default()
    }
}

impl Default for RoleBasedAuthorizer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Authorizer for RoleBasedAuthorizer {
    async fn has_permission(&self, user_id: &UserId, permission: &Permission) -> Result<bool> {
        let permissions = self.permissions.read();

        if let Some(user_perms) = permissions.get(user_id) {
            // Admin has all permissions
            if user_perms.contains(&Permission::Admin) {
                return Ok(true);
            }

            Ok(user_perms.contains(permission))
        } else {
            Ok(false)
        }
    }

    async fn grant_permission(&self, user_id: &UserId, permission: Permission) -> Result<()> {
        let mut permissions = self.permissions.write();

        permissions
            .entry(user_id.clone())
            .or_insert_with(Vec::new)
            .push(permission);

        Ok(())
    }

    async fn revoke_permission(&self, user_id: &UserId, permission: &Permission) -> Result<()> {
        let mut permissions = self.permissions.write();

        if let Some(user_perms) = permissions.get_mut(user_id) {
            user_perms.retain(|p| p != permission);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_token() {
        let token = AuthToken::new("token123", "user1");

        assert_eq!(token.token, "token123");
        assert_eq!(token.user_id, "user1");
        assert!(!token.is_expired());
    }

    #[test]
    fn test_token_expiration() {
        let expires = Utc::now() - chrono::Duration::seconds(60);
        let token = AuthToken::new("token123", "user1").with_expiration(expires);

        assert!(token.is_expired());
    }

    #[test]
    fn test_token_claims() {
        let token = AuthToken::new("token123", "user1")
            .with_claim("role", serde_json::json!("admin"))
            .with_claim("scope", serde_json::json!("full"));

        assert_eq!(token.get_claim("role").unwrap(), &serde_json::json!("admin"));
        assert_eq!(token.get_claim("scope").unwrap(), &serde_json::json!("full"));
    }

    #[tokio::test]
    async fn test_in_memory_authenticator() {
        let auth = InMemoryAuthenticator::new();

        let token = AuthToken::new("valid_token", "user1");
        auth.add_token(token);

        let result = auth.authenticate("valid_token").await.unwrap();
        assert!(result.success);
        assert_eq!(result.user_id.unwrap(), "user1");

        let result = auth.authenticate("invalid_token").await.unwrap();
        assert!(!result.success);
    }

    #[tokio::test]
    async fn test_token_validation() {
        let auth = InMemoryAuthenticator::new();

        let token = AuthToken::new("valid_token", "user1");
        auth.add_token(token);

        let is_valid = auth.validate("valid_token").await.unwrap();
        assert!(is_valid);

        let is_valid = auth.validate("invalid_token").await.unwrap();
        assert!(!is_valid);
    }

    #[tokio::test]
    async fn test_token_revocation() {
        let auth = InMemoryAuthenticator::new();

        let token = AuthToken::new("token123", "user1");
        auth.add_token(token);

        auth.revoke("token123").await.unwrap();

        let is_valid = auth.validate("token123").await.unwrap();
        assert!(!is_valid);
    }

    #[tokio::test]
    async fn test_role_based_authorization() {
        let authz = RoleBasedAuthorizer::new();

        authz
            .grant_permission(&"user1".to_string(), Permission::ReadEvents)
            .await
            .unwrap();

        let has_perm = authz
            .has_permission(&"user1".to_string(), &Permission::ReadEvents)
            .await
            .unwrap();
        assert!(has_perm);

        let has_perm = authz
            .has_permission(&"user1".to_string(), &Permission::Admin)
            .await
            .unwrap();
        assert!(!has_perm);
    }

    #[tokio::test]
    async fn test_admin_permission() {
        let authz = RoleBasedAuthorizer::new();

        authz
            .grant_permission(&"admin".to_string(), Permission::Admin)
            .await
            .unwrap();

        // Admin should have all permissions
        let has_perm = authz
            .has_permission(&"admin".to_string(), &Permission::ReadEvents)
            .await
            .unwrap();
        assert!(has_perm);

        let has_perm = authz
            .has_permission(&"admin".to_string(), &Permission::ManageUsers)
            .await
            .unwrap();
        assert!(has_perm);
    }

    #[tokio::test]
    async fn test_revoke_permission() {
        let authz = RoleBasedAuthorizer::new();

        authz
            .grant_permission(&"user1".to_string(), Permission::ReadEvents)
            .await
            .unwrap();

        let has_perm = authz
            .has_permission(&"user1".to_string(), &Permission::ReadEvents)
            .await
            .unwrap();
        assert!(has_perm);

        authz
            .revoke_permission(&"user1".to_string(), &Permission::ReadEvents)
            .await
            .unwrap();

        let has_perm = authz
            .has_permission(&"user1".to_string(), &Permission::ReadEvents)
            .await
            .unwrap();
        assert!(!has_perm);
    }
}
