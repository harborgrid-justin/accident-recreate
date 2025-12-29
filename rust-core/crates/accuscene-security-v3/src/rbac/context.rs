//! Access control context for policy evaluation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Access control context containing user, resource, and request information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessContext {
    /// User information
    pub user: UserContext,

    /// Resource information
    pub resource: ResourceContext,

    /// Request information
    pub request: RequestContext,

    /// Additional context attributes
    pub attributes: HashMap<String, String>,
}

/// User context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContext {
    /// User ID
    pub id: String,

    /// Username
    pub username: String,

    /// User roles
    pub roles: Vec<String>,

    /// User department/organization
    pub department: Option<String>,

    /// User groups
    pub groups: Vec<String>,

    /// User attributes
    pub attributes: HashMap<String, String>,
}

/// Resource context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceContext {
    /// Resource ID
    pub id: Option<String>,

    /// Resource type
    pub resource_type: String,

    /// Resource owner
    pub owner: Option<String>,

    /// Resource organization
    pub organization: Option<String>,

    /// Resource classification (public, private, confidential, etc.)
    pub classification: Option<String>,

    /// Resource tags
    pub tags: Vec<String>,

    /// Resource attributes
    pub attributes: HashMap<String, String>,
}

/// Request context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestContext {
    /// Request ID
    pub id: String,

    /// Client IP address
    pub ip: Option<String>,

    /// User agent
    pub user_agent: Option<String>,

    /// Request timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Request source (web, api, mobile, etc.)
    pub source: Option<String>,

    /// Geographic location
    pub location: Option<String>,

    /// Request attributes
    pub attributes: HashMap<String, String>,
}

impl AccessContext {
    /// Create a new access context
    pub fn new(user: UserContext, resource: ResourceContext) -> Self {
        Self {
            user,
            resource,
            request: RequestContext::default(),
            attributes: HashMap::new(),
        }
    }

    /// Add a custom attribute
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }

    /// Set request context
    pub fn with_request(mut self, request: RequestContext) -> Self {
        self.request = request;
        self
    }

    /// Convert to a flat HashMap for policy evaluation
    pub fn to_flat_context(&self) -> HashMap<String, String> {
        let mut context = HashMap::new();

        // User context
        context.insert("user.id".to_string(), self.user.id.clone());
        context.insert("user.username".to_string(), self.user.username.clone());
        context.insert("user.roles".to_string(), self.user.roles.join(","));

        if let Some(dept) = &self.user.department {
            context.insert("user.department".to_string(), dept.clone());
        }

        for (key, value) in &self.user.attributes {
            context.insert(format!("user.{}", key), value.clone());
        }

        // Resource context
        context.insert(
            "resource.type".to_string(),
            self.resource.resource_type.clone(),
        );

        if let Some(id) = &self.resource.id {
            context.insert("resource.id".to_string(), id.clone());
        }

        if let Some(owner) = &self.resource.owner {
            context.insert("resource.owner".to_string(), owner.clone());
        }

        if let Some(org) = &self.resource.organization {
            context.insert("resource.organization".to_string(), org.clone());
        }

        if let Some(classification) = &self.resource.classification {
            context.insert("resource.classification".to_string(), classification.clone());
        }

        for (key, value) in &self.resource.attributes {
            context.insert(format!("resource.{}", key), value.clone());
        }

        // Request context
        context.insert("request.id".to_string(), self.request.id.clone());

        if let Some(ip) = &self.request.ip {
            context.insert("request.ip".to_string(), ip.clone());
        }

        if let Some(source) = &self.request.source {
            context.insert("request.source".to_string(), source.clone());
        }

        context.insert(
            "time.timestamp".to_string(),
            self.request.timestamp.to_rfc3339(),
        );
        context.insert(
            "time.hour".to_string(),
            self.request.timestamp.hour().to_string(),
        );
        context.insert(
            "time.weekday".to_string(),
            self.request.timestamp.weekday().to_string(),
        );

        // Custom attributes
        for (key, value) in &self.attributes {
            context.insert(key.clone(), value.clone());
        }

        context
    }
}

impl UserContext {
    /// Create a new user context
    pub fn new(id: impl Into<String>, username: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            username: username.into(),
            roles: Vec::new(),
            department: None,
            groups: Vec::new(),
            attributes: HashMap::new(),
        }
    }

    /// Add a role
    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        self.roles.push(role.into());
        self
    }

    /// Add roles
    pub fn with_roles(mut self, roles: Vec<String>) -> Self {
        self.roles.extend(roles);
        self
    }

    /// Set department
    pub fn with_department(mut self, department: impl Into<String>) -> Self {
        self.department = Some(department.into());
        self
    }

    /// Add an attribute
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }
}

impl ResourceContext {
    /// Create a new resource context
    pub fn new(resource_type: impl Into<String>) -> Self {
        Self {
            id: None,
            resource_type: resource_type.into(),
            owner: None,
            organization: None,
            classification: None,
            tags: Vec::new(),
            attributes: HashMap::new(),
        }
    }

    /// Set resource ID
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set owner
    pub fn with_owner(mut self, owner: impl Into<String>) -> Self {
        self.owner = Some(owner.into());
        self
    }

    /// Set organization
    pub fn with_organization(mut self, org: impl Into<String>) -> Self {
        self.organization = Some(org.into());
        self
    }

    /// Set classification
    pub fn with_classification(mut self, classification: impl Into<String>) -> Self {
        self.classification = Some(classification.into());
        self
    }

    /// Add a tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Add an attribute
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }
}

impl Default for RequestContext {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            ip: None,
            user_agent: None,
            timestamp: chrono::Utc::now(),
            source: None,
            location: None,
            attributes: HashMap::new(),
        }
    }
}

impl RequestContext {
    /// Create a new request context
    pub fn new() -> Self {
        Self::default()
    }

    /// Set IP address
    pub fn with_ip(mut self, ip: impl Into<String>) -> Self {
        self.ip = Some(ip.into());
        self
    }

    /// Set user agent
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    /// Set source
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Set location
    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_access_context_creation() {
        let user = UserContext::new("user123", "john.doe")
            .with_role("admin")
            .with_department("engineering");

        let resource = ResourceContext::new("scene")
            .with_id("scene123")
            .with_owner("user123");

        let context = AccessContext::new(user, resource);

        assert_eq!(context.user.id, "user123");
        assert_eq!(context.resource.resource_type, "scene");
    }

    #[test]
    fn test_flat_context_conversion() {
        let user = UserContext::new("user123", "john.doe")
            .with_role("admin");

        let resource = ResourceContext::new("scene")
            .with_owner("user123");

        let context = AccessContext::new(user, resource);
        let flat = context.to_flat_context();

        assert_eq!(flat.get("user.id").unwrap(), "user123");
        assert_eq!(flat.get("resource.type").unwrap(), "scene");
        assert_eq!(flat.get("resource.owner").unwrap(), "user123");
    }
}
