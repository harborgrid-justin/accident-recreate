//! Role definitions with hierarchical inheritance

use super::permission::{Permission, PermissionSet};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Represents a role in the RBAC system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    /// Unique role identifier
    pub id: String,

    /// Human-readable role name
    pub name: String,

    /// Role description
    pub description: String,

    /// Permissions directly assigned to this role
    pub permissions: PermissionSet,

    /// Parent roles that this role inherits from
    pub parent_roles: Vec<String>,

    /// Metadata for the role
    pub metadata: HashMap<String, String>,

    /// Whether this role is a system role (cannot be deleted)
    pub is_system: bool,
}

impl Role {
    /// Create a new role
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: String::new(),
            permissions: PermissionSet::new(),
            parent_roles: Vec::new(),
            metadata: HashMap::new(),
            is_system: false,
        }
    }

    /// Set the description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Add a permission to the role
    pub fn add_permission(&mut self, permission: Permission) {
        self.permissions.add(permission);
    }

    /// Add multiple permissions to the role
    pub fn add_permissions(&mut self, permissions: Vec<Permission>) {
        self.permissions.add_all(permissions);
    }

    /// Add a parent role
    pub fn add_parent(&mut self, parent_id: impl Into<String>) {
        self.parent_roles.push(parent_id.into());
    }

    /// Set as a system role
    pub fn as_system(mut self) -> Self {
        self.is_system = true;
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Manages role hierarchy and inheritance
#[derive(Debug, Default)]
pub struct RoleHierarchy {
    roles: HashMap<String, Role>,
}

impl RoleHierarchy {
    /// Create a new role hierarchy
    pub fn new() -> Self {
        Self {
            roles: HashMap::new(),
        }
    }

    /// Add a role to the hierarchy
    pub fn add_role(&mut self, role: Role) {
        self.roles.insert(role.id.clone(), role);
    }

    /// Get a role by ID
    pub fn get_role(&self, role_id: &str) -> Option<&Role> {
        self.roles.get(role_id)
    }

    /// Remove a role from the hierarchy
    pub fn remove_role(&mut self, role_id: &str) -> Option<Role> {
        // Check if role is a system role
        if let Some(role) = self.roles.get(role_id) {
            if role.is_system {
                return None; // Cannot remove system roles
            }
        }

        self.roles.remove(role_id)
    }

    /// Get all roles
    pub fn all_roles(&self) -> Vec<&Role> {
        self.roles.values().collect()
    }

    /// Get effective permissions for a role (including inherited permissions)
    pub fn get_effective_permissions(&self, role_id: &str) -> PermissionSet {
        let mut effective_perms = PermissionSet::new();
        let mut visited = HashSet::new();
        self.collect_permissions(role_id, &mut effective_perms, &mut visited);
        effective_perms
    }

    /// Recursively collect permissions from a role and its parents
    fn collect_permissions(
        &self,
        role_id: &str,
        permissions: &mut PermissionSet,
        visited: &mut HashSet<String>,
    ) {
        // Prevent infinite loops in case of circular dependencies
        if visited.contains(role_id) {
            return;
        }
        visited.insert(role_id.to_string());

        if let Some(role) = self.roles.get(role_id) {
            // Add permissions from this role
            permissions.merge(&role.permissions);

            // Recursively add permissions from parent roles
            for parent_id in &role.parent_roles {
                self.collect_permissions(parent_id, permissions, visited);
            }
        }
    }

    /// Check if a role has a specific permission (including inherited)
    pub fn has_permission(&self, role_id: &str, permission: &Permission) -> bool {
        let effective_perms = self.get_effective_permissions(role_id);
        effective_perms
            .permissions()
            .iter()
            .any(|p| p.allows(&permission.action, &permission.resource))
    }

    /// Get all ancestor roles (parents, grandparents, etc.)
    pub fn get_ancestors(&self, role_id: &str) -> Vec<String> {
        let mut ancestors = Vec::new();
        let mut visited = HashSet::new();
        self.collect_ancestors(role_id, &mut ancestors, &mut visited);
        ancestors
    }

    fn collect_ancestors(
        &self,
        role_id: &str,
        ancestors: &mut Vec<String>,
        visited: &mut HashSet<String>,
    ) {
        if visited.contains(role_id) {
            return;
        }
        visited.insert(role_id.to_string());

        if let Some(role) = self.roles.get(role_id) {
            for parent_id in &role.parent_roles {
                ancestors.push(parent_id.clone());
                self.collect_ancestors(parent_id, ancestors, visited);
            }
        }
    }

    /// Check if there's a circular dependency in the role hierarchy
    pub fn has_circular_dependency(&self, role_id: &str) -> bool {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        self.is_circular(role_id, &mut visited, &mut rec_stack)
    }

    fn is_circular(
        &self,
        role_id: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> bool {
        visited.insert(role_id.to_string());
        rec_stack.insert(role_id.to_string());

        if let Some(role) = self.roles.get(role_id) {
            for parent_id in &role.parent_roles {
                if !visited.contains(parent_id) {
                    if self.is_circular(parent_id, visited, rec_stack) {
                        return true;
                    }
                } else if rec_stack.contains(parent_id) {
                    return true;
                }
            }
        }

        rec_stack.remove(role_id);
        false
    }
}

/// Predefined system roles
impl RoleHierarchy {
    /// Initialize with standard system roles
    pub fn with_system_roles() -> Self {
        let mut hierarchy = Self::new();

        // Super Admin - Full system access
        let mut super_admin = Role::new("super_admin", "Super Administrator")
            .with_description("Full system access with all permissions")
            .as_system();
        super_admin.add_permission(Permission::all());
        hierarchy.add_role(super_admin);

        // Admin - Administrative access
        let mut admin = Role::new("admin", "Administrator")
            .with_description("Administrative access to manage users and resources")
            .as_system();
        admin.add_parent("super_admin");
        admin.add_permissions(Permission::crud("user"));
        admin.add_permissions(Permission::crud("organization"));
        admin.add_permissions(Permission::crud("scene"));
        hierarchy.add_role(admin);

        // Manager - Can manage scenes and users
        let mut manager = Role::new("manager", "Manager")
            .with_description("Can manage scenes and team members")
            .as_system();
        admin.add_parent("admin");
        manager.add_permissions(Permission::crud("scene"));
        manager.add_permission(Permission::read_only("user"));
        hierarchy.add_role(manager);

        // Editor - Can edit scenes
        let mut editor = Role::new("editor", "Editor")
            .with_description("Can create and edit scenes")
            .as_system();
        editor.add_permission(Permission::new(
            super::permission::Action::Create,
            super::permission::Resource::new("scene"),
        ));
        editor.add_permission(Permission::new(
            super::permission::Action::Read,
            super::permission::Resource::new("scene"),
        ));
        editor.add_permission(Permission::new(
            super::permission::Action::Update,
            super::permission::Resource::new("scene"),
        ));
        hierarchy.add_role(editor);

        // Viewer - Read-only access
        let mut viewer = Role::new("viewer", "Viewer")
            .with_description("Read-only access to scenes")
            .as_system();
        viewer.add_permission(Permission::read_only("scene"));
        hierarchy.add_role(viewer);

        hierarchy
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rbac::permission::{Action, Resource};

    #[test]
    fn test_role_creation() {
        let role = Role::new("test_role", "Test Role")
            .with_description("A test role");
        assert_eq!(role.id, "test_role");
        assert_eq!(role.name, "Test Role");
        assert_eq!(role.description, "A test role");
    }

    #[test]
    fn test_role_hierarchy() {
        let mut hierarchy = RoleHierarchy::new();

        let mut parent = Role::new("parent", "Parent Role");
        parent.add_permission(Permission::new(Action::Read, Resource::new("scene")));
        hierarchy.add_role(parent);

        let mut child = Role::new("child", "Child Role");
        child.add_parent("parent");
        child.add_permission(Permission::new(Action::Create, Resource::new("scene")));
        hierarchy.add_role(child);

        let effective_perms = hierarchy.get_effective_permissions("child");
        assert!(effective_perms.allows(&Action::Read, &Resource::new("scene")));
        assert!(effective_perms.allows(&Action::Create, &Resource::new("scene")));
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mut hierarchy = RoleHierarchy::new();

        let mut role_a = Role::new("a", "Role A");
        role_a.add_parent("b");
        hierarchy.add_role(role_a);

        let mut role_b = Role::new("b", "Role B");
        role_b.add_parent("a");
        hierarchy.add_role(role_b);

        assert!(hierarchy.has_circular_dependency("a"));
    }

    #[test]
    fn test_system_roles() {
        let hierarchy = RoleHierarchy::with_system_roles();
        let super_admin = hierarchy.get_role("super_admin").unwrap();
        assert!(super_admin.is_system);

        let perms = hierarchy.get_effective_permissions("super_admin");
        assert!(perms.allows(&Action::Manage, &Resource::new("anything")));
    }

    #[test]
    fn test_cannot_remove_system_role() {
        let mut hierarchy = RoleHierarchy::with_system_roles();
        let result = hierarchy.remove_role("super_admin");
        assert!(result.is_none());
        assert!(hierarchy.get_role("super_admin").is_some());
    }
}
