//! Policy evaluation engine for RBAC

use super::context::AccessContext;
use super::permission::{Action, Resource};
use super::policy::{Effect, Policy};
use super::role::RoleHierarchy;
use crate::error::{SecurityError, SecurityResult};
use std::sync::Arc;
use parking_lot::RwLock;

/// Policy evaluator that combines role-based and policy-based access control
#[derive(Debug)]
pub struct PolicyEvaluator {
    /// Role hierarchy
    roles: Arc<RwLock<RoleHierarchy>>,

    /// Additional policies
    policies: Arc<RwLock<Vec<Policy>>>,

    /// Default deny mode (if true, deny by default unless explicitly allowed)
    default_deny: bool,
}

impl PolicyEvaluator {
    /// Create a new policy evaluator
    pub fn new(roles: RoleHierarchy) -> Self {
        Self {
            roles: Arc::new(RwLock::new(roles)),
            policies: Arc::new(RwLock::new(Vec::new())),
            default_deny: true,
        }
    }

    /// Create with system roles
    pub fn with_system_roles() -> Self {
        Self::new(RoleHierarchy::with_system_roles())
    }

    /// Set default deny mode
    pub fn set_default_deny(&mut self, default_deny: bool) {
        self.default_deny = default_deny;
    }

    /// Add a policy
    pub fn add_policy(&self, policy: Policy) {
        self.policies.write().push(policy);
    }

    /// Evaluate access for a user with specific roles
    pub fn evaluate(
        &self,
        context: &AccessContext,
        action: &Action,
        resource: &Resource,
    ) -> SecurityResult<bool> {
        // First, check role-based permissions
        let has_role_permission = self.check_role_permissions(context, action, resource)?;

        // Then, evaluate policies
        let policy_decision = self.evaluate_policies(context, action, resource)?;

        // Combine decisions
        let allowed = match policy_decision {
            Effect::Deny => false, // Deny takes precedence
            Effect::Allow => true,
        } && has_role_permission;

        if !allowed && self.default_deny {
            return Err(SecurityError::PermissionDenied(format!(
                "Access denied for action '{}' on resource '{}'",
                action, resource
            )));
        }

        Ok(allowed)
    }

    /// Check role-based permissions
    fn check_role_permissions(
        &self,
        context: &AccessContext,
        action: &Action,
        resource: &Resource,
    ) -> SecurityResult<bool> {
        let roles = self.roles.read();

        // Check each role the user has
        for role_id in &context.user.roles {
            let effective_perms = roles.get_effective_permissions(role_id);

            if effective_perms.allows(action, resource) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Evaluate all policies
    fn evaluate_policies(
        &self,
        context: &AccessContext,
        action: &Action,
        resource: &Resource,
    ) -> SecurityResult<Effect> {
        let policies = self.policies.read();
        let flat_context = context.to_flat_context();

        let mut result = Effect::Allow;

        for policy in policies.iter() {
            let effect = policy.evaluate(action, resource, &flat_context);

            if effect == Effect::Deny {
                return Ok(Effect::Deny); // Deny takes precedence
            }

            if effect == Effect::Allow {
                result = Effect::Allow;
            }
        }

        Ok(result)
    }

    /// Check if a user can perform an action on a resource (simplified API)
    pub fn can(
        &self,
        user_id: &str,
        user_roles: &[String],
        action: &Action,
        resource: &Resource,
    ) -> SecurityResult<bool> {
        let context = AccessContext::new(
            super::context::UserContext::new(user_id, user_id)
                .with_roles(user_roles.to_vec()),
            super::context::ResourceContext::new(resource.resource_type.clone()),
        );

        self.evaluate(&context, action, resource)
    }

    /// Require permission (returns error if not allowed)
    pub fn require(
        &self,
        context: &AccessContext,
        action: &Action,
        resource: &Resource,
    ) -> SecurityResult<()> {
        if self.evaluate(context, action, resource)? {
            Ok(())
        } else {
            Err(SecurityError::PermissionDenied(format!(
                "Required permission denied for action '{}' on resource '{}'",
                action, resource
            )))
        }
    }

    /// Get all allowed actions for a user on a resource
    pub fn get_allowed_actions(
        &self,
        context: &AccessContext,
        resource: &Resource,
    ) -> SecurityResult<Vec<Action>> {
        let actions = vec![
            Action::Create,
            Action::Read,
            Action::Update,
            Action::Delete,
            Action::List,
            Action::Execute,
            Action::Manage,
        ];

        let mut allowed = Vec::new();

        for action in actions {
            if self.evaluate(context, &action, resource).unwrap_or(false) {
                allowed.push(action);
            }
        }

        Ok(allowed)
    }

    /// Add a role to the hierarchy
    pub fn add_role(&self, role: super::role::Role) {
        self.roles.write().add_role(role);
    }

    /// Get a role from the hierarchy
    pub fn get_role(&self, role_id: &str) -> Option<super::role::Role> {
        self.roles.read().get_role(role_id).cloned()
    }

    /// Get effective permissions for a role
    pub fn get_effective_permissions(&self, role_id: &str) -> super::permission::PermissionSet {
        self.roles.read().get_effective_permissions(role_id)
    }
}

impl Default for PolicyEvaluator {
    fn default() -> Self {
        Self::with_system_roles()
    }
}

/// Builder for creating complex access control scenarios
pub struct AccessControlBuilder {
    evaluator: PolicyEvaluator,
}

impl AccessControlBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            evaluator: PolicyEvaluator::with_system_roles(),
        }
    }

    /// Add a custom role
    pub fn with_role(self, role: super::role::Role) -> Self {
        self.evaluator.add_role(role);
        self
    }

    /// Add a policy
    pub fn with_policy(self, policy: Policy) -> Self {
        self.evaluator.add_policy(policy);
        self
    }

    /// Set default deny mode
    pub fn default_deny(mut self, deny: bool) -> Self {
        self.evaluator.set_default_deny(deny);
        self
    }

    /// Build the evaluator
    pub fn build(self) -> PolicyEvaluator {
        self.evaluator
    }
}

impl Default for AccessControlBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rbac::permission::Permission;
    use crate::rbac::role::Role;

    #[test]
    fn test_role_based_evaluation() {
        let evaluator = PolicyEvaluator::with_system_roles();

        let user = super::super::context::UserContext::new("user1", "user1")
            .with_role("viewer");

        let resource = super::super::context::ResourceContext::new("scene");

        let context = AccessContext::new(user, resource);

        assert!(evaluator
            .evaluate(&context, &Action::Read, &Resource::new("scene"))
            .unwrap());

        assert!(!evaluator
            .evaluate(&context, &Action::Delete, &Resource::new("scene"))
            .unwrap_or(false));
    }

    #[test]
    fn test_custom_role() {
        let evaluator = PolicyEvaluator::new(RoleHierarchy::new());

        let mut custom_role = Role::new("custom", "Custom Role");
        custom_role.add_permission(Permission::new(Action::Read, Resource::new("scene")));
        evaluator.add_role(custom_role);

        let user = super::super::context::UserContext::new("user1", "user1")
            .with_role("custom");

        let resource = super::super::context::ResourceContext::new("scene");

        let context = AccessContext::new(user, resource);

        assert!(evaluator
            .evaluate(&context, &Action::Read, &Resource::new("scene"))
            .unwrap());
    }

    #[test]
    fn test_policy_evaluation() {
        let evaluator = PolicyEvaluator::with_system_roles();

        let policy = Policy::owner_policy();
        evaluator.add_policy(policy);

        let user = super::super::context::UserContext::new("user1", "user1");

        let resource = super::super::context::ResourceContext::new("scene")
            .with_owner("user1");

        let context = AccessContext::new(user, resource);

        // Owner policy allows management
        assert!(evaluator
            .evaluate(&context, &Action::Manage, &Resource::new("scene"))
            .unwrap_or(false));
    }

    #[test]
    fn test_require_permission() {
        let evaluator = PolicyEvaluator::with_system_roles();

        let user = super::super::context::UserContext::new("user1", "user1")
            .with_role("viewer");

        let resource = super::super::context::ResourceContext::new("scene");

        let context = AccessContext::new(user, resource);

        assert!(evaluator
            .require(&context, &Action::Read, &Resource::new("scene"))
            .is_ok());

        assert!(evaluator
            .require(&context, &Action::Delete, &Resource::new("scene"))
            .is_err());
    }

    #[test]
    fn test_get_allowed_actions() {
        let evaluator = PolicyEvaluator::with_system_roles();

        let user = super::super::context::UserContext::new("user1", "user1")
            .with_role("editor");

        let resource = super::super::context::ResourceContext::new("scene");

        let context = AccessContext::new(user, resource);

        let allowed = evaluator.get_allowed_actions(&context, &Resource::new("scene")).unwrap();

        assert!(allowed.contains(&Action::Read));
        assert!(allowed.contains(&Action::Create));
        assert!(allowed.contains(&Action::Update));
    }
}
