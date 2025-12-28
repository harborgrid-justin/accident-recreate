//! Permission definitions and management
//!
//! Defines standard permissions for the AccuScene system.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Permission definition
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Permission {
    /// Permission identifier (e.g., "cases:read")
    pub id: String,
    /// Resource type
    pub resource: String,
    /// Action
    pub action: String,
    /// Human-readable description
    pub description: String,
}

impl Permission {
    /// Create a new permission
    pub fn new(resource: &str, action: &str, description: &str) -> Self {
        Self {
            id: format!("{}:{}", resource, action),
            resource: resource.to_string(),
            action: action.to_string(),
            description: description.to_string(),
        }
    }

    /// Parse permission from string (e.g., "cases:read")
    pub fn parse(permission: &str) -> Option<Self> {
        let parts: Vec<&str> = permission.split(':').collect();
        if parts.len() != 2 {
            return None;
        }

        Some(Self {
            id: permission.to_string(),
            resource: parts[0].to_string(),
            action: parts[1].to_string(),
            description: format!("{} on {}", parts[1], parts[0]),
        })
    }

    /// Check if permission matches a pattern (supports wildcards)
    pub fn matches(&self, pattern: &str) -> bool {
        if pattern == "*" {
            return true;
        }

        let parts: Vec<&str> = pattern.split(':').collect();
        if parts.len() != 2 {
            return false;
        }

        let resource_matches = parts[0] == "*" || parts[0] == self.resource;
        let action_matches = parts[1] == "*" || parts[1] == self.action;

        resource_matches && action_matches
    }
}

impl std::fmt::Display for Permission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

/// Standard permissions for the AccuScene system
pub struct StandardPermissions;

impl StandardPermissions {
    /// Get all standard permissions
    pub fn all() -> Vec<Permission> {
        let mut permissions = Vec::new();

        // User management
        permissions.extend(Self::user_permissions());

        // Case management
        permissions.extend(Self::case_permissions());

        // Evidence management
        permissions.extend(Self::evidence_permissions());

        // Report management
        permissions.extend(Self::report_permissions());

        // Analysis permissions
        permissions.extend(Self::analysis_permissions());

        // System permissions
        permissions.extend(Self::system_permissions());

        permissions
    }

    /// User management permissions
    pub fn user_permissions() -> Vec<Permission> {
        vec![
            Permission::new("users", "read", "View users"),
            Permission::new("users", "write", "Create and update users"),
            Permission::new("users", "delete", "Delete users"),
            Permission::new("users", "manage_roles", "Manage user roles"),
            Permission::new("users", "manage_permissions", "Manage user permissions"),
        ]
    }

    /// Case management permissions
    pub fn case_permissions() -> Vec<Permission> {
        vec![
            Permission::new("cases", "read", "View cases"),
            Permission::new("cases", "write", "Create and update cases"),
            Permission::new("cases", "delete", "Delete cases"),
            Permission::new("cases", "assign", "Assign cases to investigators"),
            Permission::new("cases", "close", "Close cases"),
            Permission::new("cases", "reopen", "Reopen closed cases"),
            Permission::new("cases", "export", "Export case data"),
        ]
    }

    /// Evidence management permissions
    pub fn evidence_permissions() -> Vec<Permission> {
        vec![
            Permission::new("evidence", "read", "View evidence"),
            Permission::new("evidence", "write", "Add and update evidence"),
            Permission::new("evidence", "delete", "Delete evidence"),
            Permission::new("evidence", "manage_chain", "Manage chain of custody"),
            Permission::new("evidence", "export", "Export evidence"),
        ]
    }

    /// Report management permissions
    pub fn report_permissions() -> Vec<Permission> {
        vec![
            Permission::new("reports", "read", "View reports"),
            Permission::new("reports", "write", "Create and edit reports"),
            Permission::new("reports", "delete", "Delete reports"),
            Permission::new("reports", "publish", "Publish reports"),
            Permission::new("reports", "export", "Export reports"),
            Permission::new("reports", "share", "Share reports"),
        ]
    }

    /// Analysis permissions
    pub fn analysis_permissions() -> Vec<Permission> {
        vec![
            Permission::new("analysis", "read", "View analysis results"),
            Permission::new("analysis", "execute", "Run analysis"),
            Permission::new("analysis", "configure", "Configure analysis parameters"),
            Permission::new("physics", "simulate", "Run physics simulations"),
            Permission::new("ml", "predict", "Run ML predictions"),
        ]
    }

    /// System permissions
    pub fn system_permissions() -> Vec<Permission> {
        vec![
            Permission::new("system", "read", "View system status"),
            Permission::new("system", "configure", "Configure system settings"),
            Permission::new("settings", "read", "View settings"),
            Permission::new("settings", "write", "Modify settings"),
            Permission::new("audit", "read", "View audit logs"),
            Permission::new("audit", "export", "Export audit logs"),
            Permission::new("security", "manage", "Manage security settings"),
        ]
    }

    /// Get permission by ID
    pub fn get(id: &str) -> Option<Permission> {
        Self::all().into_iter().find(|p| p.id == id)
    }

    /// Get permissions for a resource
    pub fn for_resource(resource: &str) -> Vec<Permission> {
        Self::all()
            .into_iter()
            .filter(|p| p.resource == resource)
            .collect()
    }
}

/// Permission set for easy management
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PermissionSet {
    permissions: HashSet<String>,
}

impl PermissionSet {
    /// Create a new permission set
    pub fn new() -> Self {
        Self {
            permissions: HashSet::new(),
        }
    }

    /// Add a permission
    pub fn add(&mut self, permission: impl Into<String>) {
        self.permissions.insert(permission.into());
    }

    /// Remove a permission
    pub fn remove(&mut self, permission: &str) {
        self.permissions.remove(permission);
    }

    /// Check if permission exists
    pub fn contains(&self, permission: &str) -> bool {
        // Check exact match
        if self.permissions.contains(permission) {
            return true;
        }

        // Check for wildcard
        if self.permissions.contains("*") {
            return true;
        }

        // Check for resource wildcard
        if let Some(pos) = permission.rfind(':') {
            let resource_wildcard = format!("{}:*", &permission[..pos]);
            if self.permissions.contains(&resource_wildcard) {
                return true;
            }
        }

        false
    }

    /// Check if has all permissions
    pub fn has_all(&self, permissions: &[String]) -> bool {
        permissions.iter().all(|p| self.contains(p))
    }

    /// Check if has any permission
    pub fn has_any(&self, permissions: &[String]) -> bool {
        permissions.iter().any(|p| self.contains(p))
    }

    /// Get all permissions
    pub fn list(&self) -> Vec<String> {
        self.permissions.iter().cloned().collect()
    }

    /// Merge with another permission set
    pub fn merge(&mut self, other: &PermissionSet) {
        self.permissions.extend(other.permissions.iter().cloned());
    }

    /// Get permission count
    pub fn len(&self) -> usize {
        self.permissions.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.permissions.is_empty()
    }
}

impl From<Vec<String>> for PermissionSet {
    fn from(permissions: Vec<String>) -> Self {
        Self {
            permissions: permissions.into_iter().collect(),
        }
    }
}

impl FromIterator<String> for PermissionSet {
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        Self {
            permissions: iter.into_iter().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_creation() {
        let perm = Permission::new("cases", "read", "View cases");
        assert_eq!(perm.id, "cases:read");
        assert_eq!(perm.resource, "cases");
        assert_eq!(perm.action, "read");
    }

    #[test]
    fn test_permission_parsing() {
        let perm = Permission::parse("cases:read").unwrap();
        assert_eq!(perm.resource, "cases");
        assert_eq!(perm.action, "read");

        assert!(Permission::parse("invalid").is_none());
    }

    #[test]
    fn test_permission_matching() {
        let perm = Permission::new("cases", "read", "View cases");

        assert!(perm.matches("*"));
        assert!(perm.matches("cases:*"));
        assert!(perm.matches("cases:read"));
        assert!(!perm.matches("cases:write"));
        assert!(!perm.matches("users:read"));
    }

    #[test]
    fn test_standard_permissions() {
        let all = StandardPermissions::all();
        assert!(!all.is_empty());

        let case_perms = StandardPermissions::case_permissions();
        assert!(case_perms.iter().any(|p| p.id == "cases:read"));
        assert!(case_perms.iter().any(|p| p.id == "cases:write"));
    }

    #[test]
    fn test_permission_set() {
        let mut set = PermissionSet::new();

        set.add("cases:read".to_string());
        set.add("cases:write".to_string());

        assert!(set.contains("cases:read"));
        assert!(set.contains("cases:write"));
        assert!(!set.contains("cases:delete"));
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_permission_set_wildcard() {
        let mut set = PermissionSet::new();
        set.add("cases:*".to_string());

        assert!(set.contains("cases:read"));
        assert!(set.contains("cases:write"));
        assert!(set.contains("cases:delete"));
        assert!(!set.contains("users:read"));
    }

    #[test]
    fn test_permission_set_merge() {
        let mut set1 = PermissionSet::new();
        set1.add("cases:read".to_string());

        let mut set2 = PermissionSet::new();
        set2.add("cases:write".to_string());

        set1.merge(&set2);

        assert!(set1.contains("cases:read"));
        assert!(set1.contains("cases:write"));
        assert_eq!(set1.len(), 2);
    }

    #[test]
    fn test_permission_set_has_all() {
        let mut set = PermissionSet::new();
        set.add("cases:read".to_string());
        set.add("cases:write".to_string());

        assert!(set.has_all(&vec!["cases:read".to_string(), "cases:write".to_string()]));
        assert!(!set.has_all(&vec!["cases:read".to_string(), "cases:delete".to_string()]));
    }
}
