//! Role-Based Access Control (RBAC)
//!
//! Implements hierarchical RBAC with role inheritance.

use crate::error::{Result, SecurityError};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// RBAC service for role and permission management
pub struct RbacService {
    roles: HashMap<String, Role>,
    role_hierarchy: HashMap<String, Vec<String>>,
    user_roles: HashMap<String, HashSet<String>>,
}

impl RbacService {
    /// Create a new RBAC service
    pub fn new() -> Self {
        let mut service = Self {
            roles: HashMap::new(),
            role_hierarchy: HashMap::new(),
            user_roles: HashMap::new(),
        };

        // Initialize default roles
        service.initialize_default_roles();

        service
    }

    /// Initialize default system roles
    fn initialize_default_roles(&mut self) {
        // Super Admin
        let super_admin = Role {
            id: "super_admin".to_string(),
            name: "Super Administrator".to_string(),
            description: Some("Full system access".to_string()),
            permissions: vec!["*".to_string()].into_iter().collect(),
            inherits_from: vec![],
            metadata: HashMap::new(),
        };
        self.roles.insert("super_admin".to_string(), super_admin);

        // Admin
        let admin = Role {
            id: "admin".to_string(),
            name: "Administrator".to_string(),
            description: Some("Administrative access".to_string()),
            permissions: vec![
                "users:read".to_string(),
                "users:write".to_string(),
                "cases:read".to_string(),
                "cases:write".to_string(),
                "cases:delete".to_string(),
                "reports:read".to_string(),
                "reports:write".to_string(),
                "settings:read".to_string(),
                "settings:write".to_string(),
            ]
            .into_iter()
            .collect(),
            inherits_from: vec![],
            metadata: HashMap::new(),
        };
        self.roles.insert("admin".to_string(), admin);

        // Investigator
        let investigator = Role {
            id: "investigator".to_string(),
            name: "Investigator".to_string(),
            description: Some("Case investigation and analysis".to_string()),
            permissions: vec![
                "cases:read".to_string(),
                "cases:write".to_string(),
                "evidence:read".to_string(),
                "evidence:write".to_string(),
                "reports:read".to_string(),
                "reports:write".to_string(),
                "analysis:execute".to_string(),
            ]
            .into_iter()
            .collect(),
            inherits_from: vec![],
            metadata: HashMap::new(),
        };
        self.roles.insert("investigator".to_string(), investigator);

        // Viewer
        let viewer = Role {
            id: "viewer".to_string(),
            name: "Viewer".to_string(),
            description: Some("Read-only access".to_string()),
            permissions: vec![
                "cases:read".to_string(),
                "reports:read".to_string(),
                "evidence:read".to_string(),
            ]
            .into_iter()
            .collect(),
            inherits_from: vec![],
            metadata: HashMap::new(),
        };
        self.roles.insert("viewer".to_string(), viewer);
    }

    /// Create a new role
    pub fn create_role(&mut self, role: Role) -> Result<()> {
        if self.roles.contains_key(&role.id) {
            return Err(SecurityError::ConfigurationError(format!(
                "Role '{}' already exists",
                role.id
            )));
        }

        // Validate inherited roles exist
        for parent_role in &role.inherits_from {
            if !self.roles.contains_key(parent_role) {
                return Err(SecurityError::RoleNotFound(parent_role.clone()));
            }
        }

        self.roles.insert(role.id.clone(), role);
        Ok(())
    }

    /// Get role by ID
    pub fn get_role(&self, role_id: &str) -> Result<&Role> {
        self.roles
            .get(role_id)
            .ok_or_else(|| SecurityError::RoleNotFound(role_id.to_string()))
    }

    /// Assign role to user
    pub fn assign_role(&mut self, user_id: &str, role_id: &str) -> Result<()> {
        // Validate role exists
        if !self.roles.contains_key(role_id) {
            return Err(SecurityError::RoleNotFound(role_id.to_string()));
        }

        self.user_roles
            .entry(user_id.to_string())
            .or_insert_with(HashSet::new)
            .insert(role_id.to_string());

        Ok(())
    }

    /// Revoke role from user
    pub fn revoke_role(&mut self, user_id: &str, role_id: &str) -> Result<()> {
        if let Some(roles) = self.user_roles.get_mut(user_id) {
            roles.remove(role_id);
        }
        Ok(())
    }

    /// Get user roles
    pub fn get_user_roles(&self, user_id: &str) -> Vec<String> {
        self.user_roles
            .get(user_id)
            .map(|roles| roles.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Get all permissions for a user (including inherited)
    pub fn get_user_permissions(&self, user_id: &str) -> HashSet<String> {
        let mut permissions = HashSet::new();

        if let Some(role_ids) = self.user_roles.get(user_id) {
            for role_id in role_ids {
                if let Some(role) = self.roles.get(role_id) {
                    self.collect_permissions(role, &mut permissions);
                }
            }
        }

        permissions
    }

    /// Recursively collect permissions from role and inherited roles
    fn collect_permissions(&self, role: &Role, permissions: &mut HashSet<String>) {
        // Add role's own permissions
        permissions.extend(role.permissions.iter().cloned());

        // Add permissions from inherited roles
        for parent_role_id in &role.inherits_from {
            if let Some(parent_role) = self.roles.get(parent_role_id) {
                self.collect_permissions(parent_role, permissions);
            }
        }
    }

    /// Check if user has a specific permission
    pub fn has_permission(&self, user_id: &str, permission: &str) -> bool {
        let permissions = self.get_user_permissions(user_id);

        // Check for wildcard permission
        if permissions.contains("*") {
            return true;
        }

        // Check exact permission
        if permissions.contains(permission) {
            return true;
        }

        // Check for wildcard within resource (e.g., "cases:*" matches "cases:read")
        if let Some(pos) = permission.rfind(':') {
            let resource_wildcard = format!("{}:*", &permission[..pos]);
            if permissions.contains(&resource_wildcard) {
                return true;
            }
        }

        false
    }

    /// Check if user has a specific role
    pub fn has_role(&self, user_id: &str, role_id: &str) -> bool {
        if let Some(roles) = self.user_roles.get(user_id) {
            return roles.contains(role_id);
        }
        false
    }

    /// Check authorization for user
    pub fn authorize(&self, user_id: &str, permission: &str) -> Result<()> {
        if self.has_permission(user_id, permission) {
            Ok(())
        } else {
            Err(SecurityError::PermissionDenied(format!(
                "User '{}' does not have permission '{}'",
                user_id, permission
            )))
        }
    }
}

impl Default for RbacService {
    fn default() -> Self {
        Self::new()
    }
}

/// Role definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    /// Unique role identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Role description
    pub description: Option<String>,
    /// Direct permissions granted by this role
    pub permissions: HashSet<String>,
    /// Roles this role inherits from
    pub inherits_from: Vec<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl Role {
    /// Create a new role
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            description: None,
            permissions: HashSet::new(),
            inherits_from: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add a permission to the role
    pub fn add_permission(mut self, permission: String) -> Self {
        self.permissions.insert(permission);
        self
    }

    /// Add multiple permissions
    pub fn add_permissions(mut self, permissions: Vec<String>) -> Self {
        self.permissions.extend(permissions);
        self
    }

    /// Set role inheritance
    pub fn inherits_from(mut self, parent_roles: Vec<String>) -> Self {
        self.inherits_from = parent_roles;
        self
    }

    /// Set description
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rbac_service_creation() {
        let service = RbacService::new();
        assert!(service.get_role("admin").is_ok());
        assert!(service.get_role("investigator").is_ok());
        assert!(service.get_role("viewer").is_ok());
    }

    #[test]
    fn test_role_assignment() {
        let mut service = RbacService::new();
        service.assign_role("user123", "admin").unwrap();

        assert!(service.has_role("user123", "admin"));
        assert!(!service.has_role("user123", "viewer"));
    }

    #[test]
    fn test_permission_check() {
        let mut service = RbacService::new();
        service.assign_role("user123", "admin").unwrap();

        assert!(service.has_permission("user123", "users:read"));
        assert!(service.has_permission("user123", "cases:write"));
        assert!(!service.has_permission("user123", "system:shutdown"));
    }

    #[test]
    fn test_wildcard_permissions() {
        let mut service = RbacService::new();
        service.assign_role("user123", "super_admin").unwrap();

        assert!(service.has_permission("user123", "anything:anywhere"));
        assert!(service.has_permission("user123", "users:delete"));
    }

    #[test]
    fn test_authorization() {
        let mut service = RbacService::new();
        service.assign_role("user123", "viewer").unwrap();

        assert!(service.authorize("user123", "cases:read").is_ok());
        assert!(service.authorize("user123", "cases:delete").is_err());
    }

    #[test]
    fn test_role_revocation() {
        let mut service = RbacService::new();
        service.assign_role("user123", "admin").unwrap();
        assert!(service.has_role("user123", "admin"));

        service.revoke_role("user123", "admin").unwrap();
        assert!(!service.has_role("user123", "admin"));
    }

    #[test]
    fn test_custom_role_creation() {
        let mut service = RbacService::new();

        let custom_role = Role::new("analyst".to_string(), "Data Analyst".to_string())
            .add_permission("analytics:read".to_string())
            .add_permission("analytics:execute".to_string())
            .with_description("Data analysis role".to_string());

        service.create_role(custom_role).unwrap();
        service.assign_role("user123", "analyst").unwrap();

        assert!(service.has_permission("user123", "analytics:read"));
    }
}
