//! Cache key generation and hashing

use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// Cache key with namespace and identifier
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CacheKey {
    /// Namespace for the key (e.g., "physics", "query")
    pub namespace: String,
    /// Unique identifier within namespace
    pub identifier: String,
    /// Optional version for cache busting
    pub version: Option<u64>,
}

impl CacheKey {
    /// Create a new cache key
    pub fn new(namespace: impl Into<String>, identifier: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            identifier: identifier.into(),
            version: None,
        }
    }

    /// Create a cache key with version
    pub fn with_version(
        namespace: impl Into<String>,
        identifier: impl Into<String>,
        version: u64,
    ) -> Self {
        Self {
            namespace: namespace.into(),
            identifier: identifier.into(),
            version: Some(version),
        }
    }

    /// Get the full key as a string
    pub fn as_string(&self) -> String {
        match self.version {
            Some(v) => format!("{}:{}:v{}", self.namespace, self.identifier, v),
            None => format!("{}:{}", self.namespace, self.identifier),
        }
    }

    /// Compute hash of the key
    pub fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.namespace.hash(&mut hasher);
        self.identifier.hash(&mut hasher);
        if let Some(v) = self.version {
            v.hash(&mut hasher);
        }
        hasher.finish()
    }

    /// Check if key matches namespace
    pub fn in_namespace(&self, namespace: &str) -> bool {
        self.namespace == namespace
    }
}

impl Hash for CacheKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.namespace.hash(state);
        self.identifier.hash(state);
        self.version.hash(state);
    }
}

impl fmt::Display for CacheKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

impl From<String> for CacheKey {
    fn from(s: String) -> Self {
        // Parse "namespace:identifier" or "namespace:identifier:vN"
        let parts: Vec<&str> = s.split(':').collect();
        match parts.len() {
            2 => Self::new(parts[0], parts[1]),
            3 if parts[2].starts_with('v') => {
                let version = parts[2][1..].parse().unwrap_or(0);
                Self::with_version(parts[0], parts[1], version)
            }
            _ => Self::new("default", s),
        }
    }
}

/// Builder for cache keys
pub struct CacheKeyBuilder {
    namespace: String,
    parts: Vec<String>,
    version: Option<u64>,
}

impl CacheKeyBuilder {
    pub fn new(namespace: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            parts: Vec::new(),
            version: None,
        }
    }

    pub fn add_part(mut self, part: impl fmt::Display) -> Self {
        self.parts.push(part.to_string());
        self
    }

    pub fn version(mut self, version: u64) -> Self {
        self.version = Some(version);
        self
    }

    pub fn build(self) -> CacheKey {
        let identifier = self.parts.join(":");
        CacheKey {
            namespace: self.namespace,
            identifier,
            version: self.version,
        }
    }
}

/// Generate cache key from physics parameters
pub fn physics_cache_key(
    scenario_id: &str,
    vehicle_count: usize,
    timestep: f64,
) -> CacheKey {
    CacheKeyBuilder::new("physics")
        .add_part(scenario_id)
        .add_part(vehicle_count)
        .add_part(format!("{:.3}", timestep))
        .build()
}

/// Generate cache key for query results
pub fn query_cache_key(query: &str, params_hash: u64) -> CacheKey {
    CacheKeyBuilder::new("query")
        .add_part(query)
        .add_part(params_hash)
        .build()
}

/// Generate cache key for rendered images
pub fn image_cache_key(scene_id: &str, width: u32, height: u32, quality: u8) -> CacheKey {
    CacheKeyBuilder::new("image")
        .add_part(scene_id)
        .add_part(format!("{}x{}", width, height))
        .add_part(quality)
        .build()
}

/// Generate cache key for session data
pub fn session_cache_key(session_id: &str) -> CacheKey {
    CacheKey::new("session", session_id)
}

/// Generate cache key for configuration
pub fn config_cache_key(config_name: &str) -> CacheKey {
    CacheKey::new("config", config_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_creation() {
        let key = CacheKey::new("test", "id123");
        assert_eq!(key.namespace, "test");
        assert_eq!(key.identifier, "id123");
        assert_eq!(key.as_string(), "test:id123");
    }

    #[test]
    fn test_cache_key_with_version() {
        let key = CacheKey::with_version("test", "id123", 42);
        assert_eq!(key.as_string(), "test:id123:v42");
    }

    #[test]
    fn test_cache_key_builder() {
        let key = CacheKeyBuilder::new("physics")
            .add_part("scenario1")
            .add_part(2)
            .add_part(0.016)
            .build();
        assert!(key.as_string().contains("physics:scenario1:2:0.016"));
    }

    #[test]
    fn test_physics_cache_key() {
        let key = physics_cache_key("scenario_abc", 3, 0.016);
        assert_eq!(key.namespace, "physics");
        assert!(key.identifier.contains("scenario_abc"));
    }
}
