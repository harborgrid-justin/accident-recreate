//! State synchronization between nodes.

use crate::error::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// State snapshot for synchronization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    /// Snapshot ID
    pub id: Uuid,

    /// Node that created snapshot
    pub node_id: Uuid,

    /// Snapshot timestamp
    pub timestamp: DateTime<Utc>,

    /// Snapshot version
    pub version: u64,

    /// State data
    pub data: HashMap<String, Vec<u8>>,

    /// Checksum for integrity
    pub checksum: u32,
}

impl StateSnapshot {
    /// Create a new state snapshot.
    pub fn new(node_id: Uuid, version: u64, data: HashMap<String, Vec<u8>>) -> Self {
        let checksum = Self::calculate_checksum(&data);

        Self {
            id: Uuid::new_v4(),
            node_id,
            timestamp: Utc::now(),
            version,
            data,
            checksum,
        }
    }

    /// Verify snapshot integrity.
    pub fn verify(&self) -> bool {
        let calculated = Self::calculate_checksum(&self.data);
        calculated == self.checksum
    }

    /// Calculate checksum of data.
    fn calculate_checksum(data: &HashMap<String, Vec<u8>>) -> u32 {
        let mut hasher = crc32fast::Hasher::new();

        // Sort keys for deterministic hashing
        let mut keys: Vec<_> = data.keys().collect();
        keys.sort();

        for key in keys {
            if let Some(value) = data.get(key) {
                hasher.update(key.as_bytes());
                hasher.update(value);
            }
        }

        hasher.finalize()
    }

    /// Get size in bytes.
    pub fn size(&self) -> usize {
        self.data.values().map(|v| v.len()).sum()
    }
}

/// State delta for incremental updates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDelta {
    /// Delta ID
    pub id: Uuid,

    /// Base version
    pub base_version: u64,

    /// Target version
    pub target_version: u64,

    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// Added/updated entries
    pub updates: HashMap<String, Vec<u8>>,

    /// Removed entries
    pub removals: Vec<String>,
}

impl StateDelta {
    /// Create a new state delta.
    pub fn new(base_version: u64, target_version: u64) -> Self {
        Self {
            id: Uuid::new_v4(),
            base_version,
            target_version,
            timestamp: Utc::now(),
            updates: HashMap::new(),
            removals: Vec::new(),
        }
    }

    /// Add an update.
    pub fn add_update(&mut self, key: String, value: Vec<u8>) {
        self.updates.insert(key, value);
    }

    /// Add a removal.
    pub fn add_removal(&mut self, key: String) {
        self.removals.push(key);
    }

    /// Check if delta is empty.
    pub fn is_empty(&self) -> bool {
        self.updates.is_empty() && self.removals.is_empty()
    }

    /// Apply delta to a snapshot.
    pub fn apply_to(&self, snapshot: &mut StateSnapshot) -> Result<()> {
        // Apply updates
        for (key, value) in &self.updates {
            snapshot.data.insert(key.clone(), value.clone());
        }

        // Apply removals
        for key in &self.removals {
            snapshot.data.remove(key);
        }

        // Update version
        snapshot.version = self.target_version;

        // Recalculate checksum
        snapshot.checksum = StateSnapshot::calculate_checksum(&snapshot.data);

        Ok(())
    }
}

/// Synchronization request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncRequest {
    /// Request full snapshot
    RequestSnapshot {
        requester_version: u64,
    },

    /// Request delta from version
    RequestDelta {
        base_version: u64,
    },

    /// Anti-entropy sync request
    AntiEntropy {
        keys: Vec<String>,
    },
}

/// Synchronization response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncResponse {
    /// Full snapshot
    Snapshot(StateSnapshot),

    /// Delta update
    Delta(StateDelta),

    /// Anti-entropy response
    AntiEntropyData {
        data: HashMap<String, Vec<u8>>,
    },

    /// Error response
    Error(String),
}

/// State synchronizer.
pub struct StateSynchronizer {
    /// Current snapshot
    current_snapshot: parking_lot::RwLock<StateSnapshot>,

    /// Delta history
    deltas: parking_lot::RwLock<Vec<StateDelta>>,

    /// Maximum delta history size
    max_delta_history: usize,
}

impl StateSynchronizer {
    /// Create a new state synchronizer.
    pub fn new(node_id: Uuid) -> Self {
        let initial_snapshot = StateSnapshot::new(node_id, 0, HashMap::new());

        Self {
            current_snapshot: parking_lot::RwLock::new(initial_snapshot),
            deltas: parking_lot::RwLock::new(Vec::new()),
            max_delta_history: 100,
        }
    }

    /// Get current version.
    pub fn current_version(&self) -> u64 {
        self.current_snapshot.read().version
    }

    /// Get current snapshot.
    pub fn get_snapshot(&self) -> StateSnapshot {
        self.current_snapshot.read().clone()
    }

    /// Update state with delta.
    pub fn apply_delta(&self, delta: StateDelta) -> Result<()> {
        let mut snapshot = self.current_snapshot.write();
        delta.apply_to(&mut *snapshot)?;

        // Store delta in history
        let mut deltas = self.deltas.write();
        deltas.push(delta);

        // Trim history if needed
        if deltas.len() > self.max_delta_history {
            deltas.remove(0);
        }

        Ok(())
    }

    /// Get delta from version.
    pub fn get_delta(&self, base_version: u64) -> Option<StateDelta> {
        let deltas = self.deltas.read();

        // Try to find continuous deltas from base_version
        let mut combined_delta = None;

        for delta in deltas.iter() {
            if delta.base_version == base_version && combined_delta.is_none() {
                combined_delta = Some(delta.clone());
            } else if let Some(ref mut combined) = combined_delta {
                if delta.base_version == combined.target_version {
                    // Merge deltas
                    combined.updates.extend(delta.updates.clone());
                    combined.removals.extend(delta.removals.clone());
                    combined.target_version = delta.target_version;
                }
            }
        }

        combined_delta
    }

    /// Handle sync request.
    pub fn handle_request(&self, request: SyncRequest) -> SyncResponse {
        match request {
            SyncRequest::RequestSnapshot { .. } => {
                SyncResponse::Snapshot(self.get_snapshot())
            }

            SyncRequest::RequestDelta { base_version } => {
                if let Some(delta) = self.get_delta(base_version) {
                    SyncResponse::Delta(delta)
                } else {
                    // Fall back to full snapshot
                    SyncResponse::Snapshot(self.get_snapshot())
                }
            }

            SyncRequest::AntiEntropy { keys } => {
                let snapshot = self.current_snapshot.read();
                let mut data = HashMap::new();

                for key in keys {
                    if let Some(value) = snapshot.data.get(&key) {
                        data.insert(key, value.clone());
                    }
                }

                SyncResponse::AntiEntropyData { data }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_snapshot() {
        let mut data = HashMap::new();
        data.insert("key1".to_string(), vec![1, 2, 3]);
        data.insert("key2".to_string(), vec![4, 5, 6]);

        let snapshot = StateSnapshot::new(Uuid::new_v4(), 1, data);
        assert!(snapshot.verify());
    }

    #[test]
    fn test_state_delta() {
        let mut data = HashMap::new();
        data.insert("key1".to_string(), vec![1, 2, 3]);

        let mut snapshot = StateSnapshot::new(Uuid::new_v4(), 1, data);

        let mut delta = StateDelta::new(1, 2);
        delta.add_update("key2".to_string(), vec![4, 5, 6]);
        delta.add_removal("key1".to_string());

        delta.apply_to(&mut snapshot).unwrap();

        assert_eq!(snapshot.version, 2);
        assert!(!snapshot.data.contains_key("key1"));
        assert!(snapshot.data.contains_key("key2"));
    }
}
