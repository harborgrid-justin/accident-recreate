//! Fine-grained permission system

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;

/// Represents an action that can be performed
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// Create a resource
    Create,
    /// Read a resource
    Read,
    /// Update a resource
    Update,
    /// Delete a resource
    Delete,
    /// List resources
    List,
    /// Execute an operation
    Execute,
    /// Manage (full control)
    Manage,
    /// Custom action
    Custom(String),
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Create => write!(f, "create"),
            Self::Read => write!(f, "read"),
            Self::Update => write!(f, "update"),
            Self::Delete => write!(f, "delete"),
            Self::List => write!(f, "list"),
            Self::Execute => write!(f, "execute"),
            Self::Manage => write!(f, "manage"),
            Self::Custom(name) => write!(f, "{}", name),
        }
    }
}

/// Represents a resource type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Resource {
    /// Resource type (e.g., "scene", "user", "organization")
    pub resource_type: String,
    /// Optional resource ID for specific resource instances
    pub resource_id: Option<String>,
    /// Resource attributes for fine-grained control
    pub attributes: HashSet<String>,
}

impl Resource {
    /// Create a new resource
    pub fn new(resource_type: impl Into<String>) -> Self {
        Self {
            resource_type: resource_type.into(),
            resource_id: None,
            attributes: HashSet::new(),
        }
    }

    /// Create a new resource with a specific ID
    pub fn with_id(resource_type: impl Into<String>, resource_id: impl Into<String>) -> Self {
        Self {
            resource_type: resource_type.into(),
            resource_id: Some(resource_id.into()),
            attributes: HashSet::new(),
        }
    }

    /// Add an attribute to the resource
    pub fn with_attribute(mut self, attribute: impl Into<String>) -> Self {
        self.attributes.insert(attribute.into());
        self
    }

    /// Check if resource matches another resource (considers wildcards)
    pub fn matches(&self, other: &Self) -> bool {
        // Check resource type
        if self.resource_type != other.resource_type && self.resource_type != "*" {
            return false;
        }

        // Check resource ID
        match (&self.resource_id, &other.resource_id) {
            (Some(id1), Some(id2)) if id1 != id2 && id1 != "*" => return false,
            (Some(_), None) => return false,
            _ => {}
        }

        true
    }
}

impl fmt::Display for Resource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(id) = &self.resource_id {
            write!(f, "{}:{}", self.resource_type, id)
        } else {
            write!(f, "{}", self.resource_type)
        }
    }
}

/// Represents a permission (action on a resource)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Permission {
    /// The action to perform
    pub action: Action,
    /// The resource to act upon
    pub resource: Resource,
    /// Optional conditions for the permission
    pub conditions: Vec<String>,
}

impl Permission {
    /// Create a new permission
    pub fn new(action: Action, resource: Resource) -> Self {
        Self {
            action,
            resource,
            conditions: Vec::new(),
        }
    }

    /// Add a condition to the permission
    pub fn with_condition(mut self, condition: impl Into<String>) -> Self {
        self.conditions.push(condition.into());
        self
    }

    /// Create a wildcard permission (all actions on all resources)
    pub fn all() -> Self {
        Self {
            action: Action::Manage,
            resource: Resource::new("*"),
            conditions: Vec::new(),
        }
    }

    /// Create a read-only permission for a resource type
    pub fn read_only(resource_type: impl Into<String>) -> Self {
        Self {
            action: Action::Read,
            resource: Resource::new(resource_type),
            conditions: Vec::new(),
        }
    }

    /// Create a full CRUD permission set for a resource type
    pub fn crud(resource_type: impl Into<String>) -> Vec<Self> {
        let resource_type = resource_type.into();
        vec![
            Self::new(Action::Create, Resource::new(resource_type.clone())),
            Self::new(Action::Read, Resource::new(resource_type.clone())),
            Self::new(Action::Update, Resource::new(resource_type.clone())),
            Self::new(Action::Delete, Resource::new(resource_type)),
        ]
    }

    /// Check if this permission allows the given action on the given resource
    pub fn allows(&self, action: &Action, resource: &Resource) -> bool {
        // Check if action matches
        let action_matches = match (&self.action, action) {
            (Action::Manage, _) => true, // Manage allows all actions
            (a1, a2) if a1 == a2 => true,
            _ => false,
        };

        if !action_matches {
            return false;
        }

        // Check if resource matches
        self.resource.matches(resource)
    }
}

impl fmt::Display for Permission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.action, self.resource)
    }
}

/// A set of permissions
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PermissionSet {
    permissions: Vec<Permission>,
}

impl PermissionSet {
    /// Create a new empty permission set
    pub fn new() -> Self {
        Self {
            permissions: Vec::new(),
        }
    }

    /// Add a permission to the set
    pub fn add(&mut self, permission: Permission) {
        self.permissions.push(permission);
    }

    /// Add multiple permissions to the set
    pub fn add_all(&mut self, permissions: Vec<Permission>) {
        self.permissions.extend(permissions);
    }

    /// Check if the set contains a permission for the given action and resource
    pub fn allows(&self, action: &Action, resource: &Resource) -> bool {
        self.permissions
            .iter()
            .any(|p| p.allows(action, resource))
    }

    /// Get all permissions
    pub fn permissions(&self) -> &[Permission] {
        &self.permissions
    }

    /// Merge another permission set into this one
    pub fn merge(&mut self, other: &Self) {
        self.permissions.extend(other.permissions.clone());
    }
}

impl FromIterator<Permission> for PermissionSet {
    fn from_iter<T: IntoIterator<Item = Permission>>(iter: T) -> Self {
        Self {
            permissions: iter.into_iter().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_matches() {
        let r1 = Resource::new("scene");
        let r2 = Resource::new("scene");
        assert!(r1.matches(&r2));

        let r3 = Resource::new("*");
        assert!(r3.matches(&r1));

        let r4 = Resource::with_id("scene", "123");
        let r5 = Resource::with_id("scene", "123");
        assert!(r4.matches(&r5));

        let r6 = Resource::with_id("scene", "456");
        assert!(!r4.matches(&r6));
    }

    #[test]
    fn test_permission_allows() {
        let perm = Permission::new(Action::Read, Resource::new("scene"));
        assert!(perm.allows(&Action::Read, &Resource::new("scene")));
        assert!(!perm.allows(&Action::Write, &Resource::new("scene")));
        assert!(!perm.allows(&Action::Read, &Resource::new("user")));

        let manage_perm = Permission::new(Action::Manage, Resource::new("scene"));
        assert!(manage_perm.allows(&Action::Read, &Resource::new("scene")));
        assert!(manage_perm.allows(&Action::Create, &Resource::new("scene")));
        assert!(manage_perm.allows(&Action::Delete, &Resource::new("scene")));
    }

    #[test]
    fn test_permission_set() {
        let mut set = PermissionSet::new();
        set.add(Permission::new(Action::Read, Resource::new("scene")));
        set.add(Permission::new(Action::Write, Resource::new("scene")));

        assert!(set.allows(&Action::Read, &Resource::new("scene")));
        assert!(set.allows(&Action::Write, &Resource::new("scene")));
        assert!(!set.allows(&Action::Delete, &Resource::new("scene")));
    }

    #[test]
    fn test_crud_permissions() {
        let perms = Permission::crud("scene");
        assert_eq!(perms.len(), 4);
        assert!(perms.iter().any(|p| p.action == Action::Create));
        assert!(perms.iter().any(|p| p.action == Action::Read));
        assert!(perms.iter().any(|p| p.action == Action::Update));
        assert!(perms.iter().any(|p| p.action == Action::Delete));
    }
}
