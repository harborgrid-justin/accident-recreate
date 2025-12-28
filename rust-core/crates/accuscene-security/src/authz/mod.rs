//! Authorization framework
//!
//! Comprehensive authorization system combining RBAC and ABAC with a policy engine.

pub mod abac;
pub mod permission;
pub mod policy;
pub mod rbac;

pub use abac::{
    AbacPolicy, AbacService, Attributes, AttributeContext, AttributeValue,
    AuthorizationDecision, AuthorizationRequest, Condition, Effect,
};
pub use permission::{Permission, PermissionSet, StandardPermissions};
pub use policy::{DecisionEffect, PolicyDecision, PolicyEngine, PolicyEngineConfig, PolicyRequest};
pub use rbac::{RbacService, Role};

use crate::auth::AuthContext;
use crate::error::Result;

/// Main authorization service
pub struct AuthorizationService {
    engine: PolicyEngine,
}

impl AuthorizationService {
    /// Create a new authorization service
    pub fn new(config: PolicyEngineConfig) -> Self {
        Self {
            engine: PolicyEngine::new(config),
        }
    }

    /// Get the policy engine
    pub fn engine(&self) -> &PolicyEngine {
        &self.engine
    }

    /// Get mutable policy engine
    pub fn engine_mut(&mut self) -> &mut PolicyEngine {
        &mut self.engine
    }

    /// Authorize a request using authentication context
    pub fn authorize_context(
        &mut self,
        context: &AuthContext,
        permission: &str,
    ) -> Result<()> {
        let request = PolicyRequest::simple(context.user_id.clone(), permission.to_string());
        let decision = self.engine.authorize(request)?;
        decision.to_result()
    }

    /// Authorize with full request details
    pub fn authorize_request(&mut self, request: PolicyRequest) -> Result<()> {
        let decision = self.engine.authorize(request)?;
        decision.to_result()
    }

    /// Check if context has permission
    pub fn has_permission(&mut self, context: &AuthContext, permission: &str) -> bool {
        self.authorize_context(context, permission).is_ok()
    }

    /// Check if context has any of the permissions
    pub fn has_any_permission(&mut self, context: &AuthContext, permissions: &[&str]) -> bool {
        permissions
            .iter()
            .any(|p| self.has_permission(context, p))
    }

    /// Check if context has all permissions
    pub fn has_all_permissions(&mut self, context: &AuthContext, permissions: &[&str]) -> bool {
        permissions
            .iter()
            .all(|p| self.has_permission(context, p))
    }

    /// Get RBAC service
    pub fn rbac(&self) -> &RbacService {
        self.engine.rbac()
    }

    /// Get mutable RBAC service
    pub fn rbac_mut(&mut self) -> &mut RbacService {
        self.engine.rbac_mut()
    }

    /// Get ABAC service
    pub fn abac(&self) -> &AbacService {
        self.engine.abac()
    }

    /// Get mutable ABAC service
    pub fn abac_mut(&mut self) -> &mut AbacService {
        self.engine.abac_mut()
    }
}

impl Default for AuthorizationService {
    fn default() -> Self {
        Self::new(PolicyEngineConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::SessionMetadata;

    fn test_context() -> AuthContext {
        AuthContext {
            user_id: "user123".to_string(),
            session_id: Some("session123".to_string()),
            roles: vec!["admin".to_string()],
            permissions: vec!["users:read".to_string(), "users:write".to_string()],
            mfa_verified: true,
            session_metadata: Some(SessionMetadata::basic("192.168.1.1".to_string())),
        }
    }

    #[test]
    fn test_authorization_service_creation() {
        let _service = AuthorizationService::default();
    }

    #[test]
    fn test_context_authorization() {
        let mut service = AuthorizationService::default();
        service.rbac_mut().assign_role("user123", "admin").unwrap();

        let context = test_context();

        assert!(service.authorize_context(&context, "users:read").is_ok());
        assert!(service.has_permission(&context, "users:read"));
    }

    #[test]
    fn test_multiple_permissions() {
        let mut service = AuthorizationService::default();
        service.rbac_mut().assign_role("user123", "admin").unwrap();

        let context = test_context();

        assert!(service.has_any_permission(&context, &["users:read", "users:write"]));
        assert!(service.has_all_permissions(&context, &["users:read", "users:write"]));
        assert!(!service.has_all_permissions(&context, &["users:read", "system:shutdown"]));
    }
}
