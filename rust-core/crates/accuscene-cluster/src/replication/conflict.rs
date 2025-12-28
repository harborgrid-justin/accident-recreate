//! Conflict resolution strategies.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Versioned value with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionedValue {
    /// Value data
    pub value: Vec<u8>,

    /// Version vector clock
    pub version: VectorClock,

    /// Last modified timestamp
    pub timestamp: DateTime<Utc>,

    /// Node that made the update
    pub node_id: Uuid,

    /// Checksum
    pub checksum: u32,
}

impl VersionedValue {
    /// Create a new versioned value.
    pub fn new(value: Vec<u8>, node_id: Uuid, version: VectorClock) -> Self {
        let checksum = crc32fast::hash(&value);

        Self {
            value,
            version,
            timestamp: Utc::now(),
            node_id,
            checksum,
        }
    }

    /// Verify integrity.
    pub fn verify(&self) -> bool {
        crc32fast::hash(&self.value) == self.checksum
    }
}

/// Vector clock for causal ordering.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VectorClock {
    /// Clock values for each node
    clocks: HashMap<Uuid, u64>,
}

impl VectorClock {
    /// Create a new vector clock.
    pub fn new() -> Self {
        Self {
            clocks: HashMap::new(),
        }
    }

    /// Increment clock for a node.
    pub fn increment(&mut self, node_id: Uuid) {
        *self.clocks.entry(node_id).or_insert(0) += 1;
    }

    /// Get clock value for a node.
    pub fn get(&self, node_id: &Uuid) -> u64 {
        self.clocks.get(node_id).copied().unwrap_or(0)
    }

    /// Merge with another vector clock (taking max of each).
    pub fn merge(&mut self, other: &VectorClock) {
        for (node_id, value) in &other.clocks {
            let current = self.clocks.entry(*node_id).or_insert(0);
            *current = (*current).max(*value);
        }
    }

    /// Compare with another vector clock.
    pub fn compare(&self, other: &VectorClock) -> VectorClockOrdering {
        let mut less = false;
        let mut greater = false;

        // Get all node IDs from both clocks
        let mut all_nodes: Vec<Uuid> = self.clocks.keys().copied().collect();
        all_nodes.extend(other.clocks.keys().copied());
        all_nodes.sort();
        all_nodes.dedup();

        for node_id in all_nodes {
            let self_val = self.get(&node_id);
            let other_val = other.get(&node_id);

            if self_val < other_val {
                less = true;
            } else if self_val > other_val {
                greater = true;
            }
        }

        match (less, greater) {
            (false, false) => VectorClockOrdering::Equal,
            (true, false) => VectorClockOrdering::Before,
            (false, true) => VectorClockOrdering::After,
            (true, true) => VectorClockOrdering::Concurrent,
        }
    }

    /// Check if this clock happens before another.
    pub fn happens_before(&self, other: &VectorClock) -> bool {
        matches!(self.compare(other), VectorClockOrdering::Before)
    }

    /// Check if this clock is concurrent with another.
    pub fn is_concurrent(&self, other: &VectorClock) -> bool {
        matches!(self.compare(other), VectorClockOrdering::Concurrent)
    }
}

impl Default for VectorClock {
    fn default() -> Self {
        Self::new()
    }
}

/// Vector clock ordering relationship.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VectorClockOrdering {
    /// Clocks are equal
    Equal,
    /// This clock is before the other
    Before,
    /// This clock is after the other
    After,
    /// Clocks are concurrent (conflict)
    Concurrent,
}

/// Conflict resolver.
pub struct ConflictResolver;

impl ConflictResolver {
    /// Resolve conflict using last-write-wins strategy.
    pub fn last_write_wins(values: &[VersionedValue]) -> Option<VersionedValue> {
        values
            .iter()
            .max_by_key(|v| v.timestamp)
            .cloned()
    }

    /// Resolve conflict using vector clocks.
    pub fn vector_clock_resolve(values: &[VersionedValue]) -> ConflictResolution {
        if values.is_empty() {
            return ConflictResolution::NoValue;
        }

        if values.len() == 1 {
            return ConflictResolution::Resolved(values[0].clone());
        }

        // Find values that are not dominated by others
        let mut winners = Vec::new();

        for (i, value) in values.iter().enumerate() {
            let mut dominated = false;

            for (j, other) in values.iter().enumerate() {
                if i != j && value.version.happens_before(&other.version) {
                    dominated = true;
                    break;
                }
            }

            if !dominated {
                winners.push(value.clone());
            }
        }

        match winners.len() {
            0 => ConflictResolution::NoValue,
            1 => ConflictResolution::Resolved(winners[0].clone()),
            _ => ConflictResolution::Conflict(winners),
        }
    }

    /// Merge concurrent values (application-specific).
    pub fn merge_values(values: Vec<VersionedValue>) -> VersionedValue {
        // Simple merge: concatenate values and merge clocks
        let mut merged_data = Vec::new();
        let mut merged_clock = VectorClock::new();
        let mut latest_timestamp = Utc::now();
        let mut latest_node = Uuid::nil();

        for value in &values {
            merged_data.extend_from_slice(&value.value);
            merged_clock.merge(&value.version);

            if value.timestamp > latest_timestamp {
                latest_timestamp = value.timestamp;
                latest_node = value.node_id;
            }
        }

        VersionedValue::new(merged_data, latest_node, merged_clock)
    }
}

/// Conflict resolution result.
#[derive(Debug, Clone)]
pub enum ConflictResolution {
    /// Conflict resolved to single value
    Resolved(VersionedValue),
    /// Multiple concurrent values (siblings)
    Conflict(Vec<VersionedValue>),
    /// No value
    NoValue,
}

impl ConflictResolution {
    /// Check if resolved.
    pub fn is_resolved(&self) -> bool {
        matches!(self, ConflictResolution::Resolved(_))
    }

    /// Get resolved value.
    pub fn resolved_value(&self) -> Option<&VersionedValue> {
        match self {
            ConflictResolution::Resolved(v) => Some(v),
            _ => None,
        }
    }

    /// Get conflicting values.
    pub fn conflicting_values(&self) -> Option<&[VersionedValue]> {
        match self {
            ConflictResolution::Conflict(values) => Some(values),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_clock() {
        let node1 = Uuid::new_v4();
        let node2 = Uuid::new_v4();

        let mut clock1 = VectorClock::new();
        clock1.increment(node1);
        clock1.increment(node1);

        let mut clock2 = VectorClock::new();
        clock2.increment(node1);
        clock2.increment(node2);

        assert_eq!(clock1.get(&node1), 2);
        assert_eq!(clock2.get(&node2), 1);

        // clock1 and clock2 are concurrent
        assert!(clock1.is_concurrent(&clock2));
    }

    #[test]
    fn test_conflict_resolution() {
        let node1 = Uuid::new_v4();
        let node2 = Uuid::new_v4();

        let mut clock1 = VectorClock::new();
        clock1.increment(node1);

        let mut clock2 = VectorClock::new();
        clock2.increment(node2);

        let val1 = VersionedValue::new(vec![1, 2, 3], node1, clock1);
        let val2 = VersionedValue::new(vec![4, 5, 6], node2, clock2);

        let resolution = ConflictResolver::vector_clock_resolve(&[val1, val2]);

        // Should be concurrent conflict
        assert!(!resolution.is_resolved());
        assert_eq!(resolution.conflicting_values().unwrap().len(), 2);
    }
}
