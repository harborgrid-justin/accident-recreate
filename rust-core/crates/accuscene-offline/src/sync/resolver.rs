use crate::config::ConflictResolution;
use crate::error::{OfflineError, Result};
use crate::versioning::{Ordering, Version};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Conflict between two versions of data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    /// Entity ID
    pub entity_id: String,

    /// Entity type
    pub entity_type: String,

    /// Local version
    pub local_version: Version,

    /// Local data
    pub local_data: Value,

    /// Remote version
    pub remote_version: Version,

    /// Remote data
    pub remote_data: Value,

    /// When conflict was detected
    pub detected_at: chrono::DateTime<chrono::Utc>,
}

/// Result of conflict resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionResult {
    /// Resolved data
    pub data: Value,

    /// Resolved version
    pub version: Version,

    /// Resolution strategy used
    pub strategy: ConflictResolution,

    /// Whether manual intervention was required
    pub manual: bool,

    /// Resolution metadata
    pub metadata: serde_json::Map<String, Value>,
}

/// Conflict resolver using various strategies
pub struct ConflictResolver {
    /// Default resolution strategy
    default_strategy: ConflictResolution,

    /// Custom resolution functions per entity type
    custom_resolvers: std::collections::HashMap<String, Box<dyn Fn(&Conflict) -> Result<ResolutionResult> + Send + Sync>>,
}

impl ConflictResolver {
    /// Create a new conflict resolver
    pub fn new(default_strategy: ConflictResolution) -> Self {
        Self {
            default_strategy,
            custom_resolvers: std::collections::HashMap::new(),
        }
    }

    /// Register a custom resolver for an entity type
    pub fn register_custom_resolver<F>(&mut self, entity_type: String, resolver: F)
    where
        F: Fn(&Conflict) -> Result<ResolutionResult> + Send + Sync + 'static,
    {
        self.custom_resolvers.insert(entity_type, Box::new(resolver));
    }

    /// Resolve a conflict
    pub fn resolve(&self, conflict: &Conflict) -> Result<ResolutionResult> {
        // Check for custom resolver first
        if let Some(resolver) = self.custom_resolvers.get(&conflict.entity_type) {
            return resolver(conflict);
        }

        // Use default strategy
        self.resolve_with_strategy(conflict, self.default_strategy)
    }

    /// Resolve using a specific strategy
    pub fn resolve_with_strategy(
        &self,
        conflict: &Conflict,
        strategy: ConflictResolution,
    ) -> Result<ResolutionResult> {
        match strategy {
            ConflictResolution::LastWriteWins => self.last_write_wins(conflict),
            ConflictResolution::FirstWriteWins => self.first_write_wins(conflict),
            ConflictResolution::ServerWins => self.server_wins(conflict),
            ConflictResolution::ClientWins => self.client_wins(conflict),
            ConflictResolution::Manual => self.manual_resolution(conflict),
            ConflictResolution::OperationalTransform => self.operational_transform(conflict),
            ConflictResolution::Custom => {
                Err(OfflineError::Conflict("No custom resolver registered".to_string()))
            }
        }
    }

    /// Last write wins strategy - choose version with latest timestamp
    fn last_write_wins(&self, conflict: &Conflict) -> Result<ResolutionResult> {
        let (data, version) = if conflict.local_version.timestamp > conflict.remote_version.timestamp {
            (conflict.local_data.clone(), conflict.local_version.clone())
        } else {
            (conflict.remote_data.clone(), conflict.remote_version.clone())
        };

        Ok(ResolutionResult {
            data,
            version,
            strategy: ConflictResolution::LastWriteWins,
            manual: false,
            metadata: serde_json::Map::new(),
        })
    }

    /// First write wins strategy - choose version with earliest timestamp
    fn first_write_wins(&self, conflict: &Conflict) -> Result<ResolutionResult> {
        let (data, version) = if conflict.local_version.timestamp < conflict.remote_version.timestamp {
            (conflict.local_data.clone(), conflict.local_version.clone())
        } else {
            (conflict.remote_data.clone(), conflict.remote_version.clone())
        };

        Ok(ResolutionResult {
            data,
            version,
            strategy: ConflictResolution::FirstWriteWins,
            manual: false,
            metadata: serde_json::Map::new(),
        })
    }

    /// Server wins strategy - always choose remote version
    fn server_wins(&self, conflict: &Conflict) -> Result<ResolutionResult> {
        Ok(ResolutionResult {
            data: conflict.remote_data.clone(),
            version: conflict.remote_version.clone(),
            strategy: ConflictResolution::ServerWins,
            manual: false,
            metadata: serde_json::Map::new(),
        })
    }

    /// Client wins strategy - always choose local version
    fn client_wins(&self, conflict: &Conflict) -> Result<ResolutionResult> {
        Ok(ResolutionResult {
            data: conflict.local_data.clone(),
            version: conflict.local_version.clone(),
            strategy: ConflictResolution::ClientWins,
            manual: false,
            metadata: serde_json::Map::new(),
        })
    }

    /// Manual resolution - requires user intervention
    fn manual_resolution(&self, conflict: &Conflict) -> Result<ResolutionResult> {
        Err(OfflineError::Conflict(format!(
            "Manual resolution required for {} {}",
            conflict.entity_type, conflict.entity_id
        )))
    }

    /// Operational transformation for mergeable data
    fn operational_transform(&self, conflict: &Conflict) -> Result<ResolutionResult> {
        // Check if versions are concurrent
        match conflict.local_version.compare(&conflict.remote_version) {
            Ordering::Concurrent => {
                // Attempt to merge concurrent changes
                let merged_data = self.merge_objects(&conflict.local_data, &conflict.remote_data)?;

                // Create merged version
                let mut merged_version = conflict.local_version.clone();
                merged_version.clock.merge(&conflict.remote_version.clock);
                merged_version.timestamp = chrono::Utc::now();

                // Recompute hash
                let data_str = serde_json::to_string(&merged_data)?;
                merged_version.content_hash = format!("{:x}", blake3::hash(data_str.as_bytes()));

                Ok(ResolutionResult {
                    data: merged_data,
                    version: merged_version,
                    strategy: ConflictResolution::OperationalTransform,
                    manual: false,
                    metadata: serde_json::Map::new(),
                })
            }
            _ => {
                // Not concurrent, use last write wins
                self.last_write_wins(conflict)
            }
        }
    }

    /// Merge two JSON objects (3-way merge for objects, last-write-wins for conflicts)
    fn merge_objects(&self, local: &Value, remote: &Value) -> Result<Value> {
        match (local, remote) {
            (Value::Object(local_obj), Value::Object(remote_obj)) => {
                let mut merged = serde_json::Map::new();

                // Add all keys from both objects
                for key in local_obj.keys().chain(remote_obj.keys()) {
                    let local_val = local_obj.get(key);
                    let remote_val = remote_obj.get(key);

                    let merged_val = match (local_val, remote_val) {
                        (Some(l), Some(r)) => {
                            // Both have the value, try to merge recursively
                            if l == r {
                                l.clone()
                            } else {
                                self.merge_objects(l, r)?
                            }
                        }
                        (Some(l), None) => l.clone(),
                        (None, Some(r)) => r.clone(),
                        (None, None) => continue,
                    };

                    merged.insert(key.clone(), merged_val);
                }

                Ok(Value::Object(merged))
            }
            (Value::Array(local_arr), Value::Array(remote_arr)) => {
                // For arrays, combine and deduplicate
                let mut merged = local_arr.clone();
                for item in remote_arr {
                    if !merged.contains(item) {
                        merged.push(item.clone());
                    }
                }
                Ok(Value::Array(merged))
            }
            _ => {
                // For primitives, use remote value (server wins for non-mergeable types)
                Ok(remote.clone())
            }
        }
    }

    /// Detect if there's a conflict between two versions
    pub fn has_conflict(local: &Version, remote: &Version) -> bool {
        matches!(local.compare(remote), Ordering::Concurrent)
    }

    /// Create a conflict record
    pub fn create_conflict(
        entity_id: String,
        entity_type: String,
        local_version: Version,
        local_data: Value,
        remote_version: Version,
        remote_data: Value,
    ) -> Conflict {
        Conflict {
            entity_id,
            entity_type,
            local_version,
            local_data,
            remote_version,
            remote_data,
            detected_at: chrono::Utc::now(),
        }
    }
}

impl Default for ConflictResolver {
    fn default() -> Self {
        Self::new(ConflictResolution::LastWriteWins)
    }
}

/// Three-way merge for structured data
pub struct ThreeWayMerge;

impl ThreeWayMerge {
    /// Perform three-way merge given base, local, and remote versions
    pub fn merge(base: &Value, local: &Value, remote: &Value) -> Result<Value> {
        match (base, local, remote) {
            (Value::Object(base_obj), Value::Object(local_obj), Value::Object(remote_obj)) => {
                let mut merged = serde_json::Map::new();

                // Collect all keys
                let all_keys: std::collections::HashSet<_> = base_obj
                    .keys()
                    .chain(local_obj.keys())
                    .chain(remote_obj.keys())
                    .collect();

                for key in all_keys {
                    let base_val = base_obj.get(key);
                    let local_val = local_obj.get(key);
                    let remote_val = remote_obj.get(key);

                    let merged_val = match (base_val, local_val, remote_val) {
                        // No changes
                        (Some(b), Some(l), Some(r)) if b == l && l == r => b.clone(),

                        // Local changed, remote unchanged
                        (Some(b), Some(l), Some(r)) if b == r && b != l => l.clone(),

                        // Remote changed, local unchanged
                        (Some(b), Some(l), Some(r)) if b == l && b != r => r.clone(),

                        // Both changed to same value
                        (Some(_), Some(l), Some(r)) if l == r => l.clone(),

                        // Both changed to different values - conflict
                        (Some(b), Some(l), Some(r)) if l != r => {
                            // Try recursive merge
                            Self::merge(b, l, r)?
                        }

                        // Added in local only
                        (None, Some(l), None) => l.clone(),

                        // Added in remote only
                        (None, None, Some(r)) => r.clone(),

                        // Added in both with same value
                        (None, Some(l), Some(r)) if l == r => l.clone(),

                        // Added in both with different values - use remote
                        (None, Some(_), Some(r)) => r.clone(),

                        // Deleted in local, unchanged in remote
                        (Some(_), None, Some(_)) => continue,

                        // Deleted in remote, unchanged in local
                        (Some(_), Some(_), None) => continue,

                        // All other cases
                        _ => continue,
                    };

                    merged.insert(key.clone(), merged_val);
                }

                Ok(Value::Object(merged))
            }
            _ => {
                // For non-objects, choose remote if different
                if local != remote {
                    Ok(remote.clone())
                } else {
                    Ok(local.clone())
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::versioning::VectorClock;

    fn create_test_version(node_id: &str, timestamp_offset: i64) -> Version {
        let mut clock = VectorClock::new();
        clock.increment(&node_id.to_string());

        Version {
            clock,
            node_id: node_id.to_string(),
            timestamp: chrono::Utc::now() + chrono::Duration::seconds(timestamp_offset),
            content_hash: "test-hash".to_string(),
        }
    }

    #[test]
    fn test_last_write_wins() {
        let resolver = ConflictResolver::default();

        let old_version = create_test_version("node1", -10);
        let new_version = create_test_version("node2", 0);

        let conflict = Conflict {
            entity_id: "entity1".to_string(),
            entity_type: "test".to_string(),
            local_version: old_version,
            local_data: serde_json::json!({"value": "old"}),
            remote_version: new_version.clone(),
            remote_data: serde_json::json!({"value": "new"}),
            detected_at: chrono::Utc::now(),
        };

        let result = resolver.resolve(&conflict).unwrap();
        assert_eq!(result.data, serde_json::json!({"value": "new"}));
        assert_eq!(result.version.node_id, new_version.node_id);
    }

    #[test]
    fn test_merge_objects() {
        let resolver = ConflictResolver::default();

        let local = serde_json::json!({
            "name": "Alice",
            "age": 30,
            "local_only": "value"
        });

        let remote = serde_json::json!({
            "name": "Alice",
            "age": 31,
            "remote_only": "value"
        });

        let merged = resolver.merge_objects(&local, &remote).unwrap();

        assert_eq!(merged["name"], "Alice");
        assert_eq!(merged["local_only"], "value");
        assert_eq!(merged["remote_only"], "value");
    }
}
