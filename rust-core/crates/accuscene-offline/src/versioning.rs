use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Node identifier for distributed system
pub type NodeId = String;

/// Vector clock for distributed versioning and causality tracking
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VectorClock {
    /// Clock values per node
    clocks: HashMap<NodeId, u64>,
}

impl VectorClock {
    /// Create a new empty vector clock
    pub fn new() -> Self {
        Self {
            clocks: HashMap::new(),
        }
    }

    /// Create a vector clock with initial value for a node
    pub fn new_with_node(node_id: NodeId) -> Self {
        let mut clocks = HashMap::new();
        clocks.insert(node_id, 1);
        Self { clocks }
    }

    /// Increment the clock for a specific node
    pub fn increment(&mut self, node_id: &NodeId) {
        *self.clocks.entry(node_id.clone()).or_insert(0) += 1;
    }

    /// Get the clock value for a node
    pub fn get(&self, node_id: &NodeId) -> u64 {
        self.clocks.get(node_id).copied().unwrap_or(0)
    }

    /// Merge with another vector clock (take maximum of each component)
    pub fn merge(&mut self, other: &VectorClock) {
        for (node_id, &value) in &other.clocks {
            let entry = self.clocks.entry(node_id.clone()).or_insert(0);
            *entry = (*entry).max(value);
        }
    }

    /// Compare two vector clocks
    pub fn compare(&self, other: &VectorClock) -> Ordering {
        let mut less = false;
        let mut greater = false;

        // Get all unique node IDs
        let mut all_nodes: Vec<_> = self.clocks.keys().chain(other.clocks.keys()).collect();
        all_nodes.sort();
        all_nodes.dedup();

        for node_id in all_nodes {
            let self_val = self.get(node_id);
            let other_val = other.get(node_id);

            if self_val < other_val {
                less = true;
            } else if self_val > other_val {
                greater = true;
            }
        }

        match (less, greater) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            (false, false) => Ordering::Equal,
            (true, true) => Ordering::Concurrent,
        }
    }

    /// Check if this vector clock is strictly less than another
    pub fn is_less_than(&self, other: &VectorClock) -> bool {
        matches!(self.compare(other), Ordering::Less)
    }

    /// Check if this vector clock is concurrent with another
    pub fn is_concurrent(&self, other: &VectorClock) -> bool {
        matches!(self.compare(other), Ordering::Concurrent)
    }

    /// Check if this vector clock dominates another (greater or equal)
    pub fn dominates(&self, other: &VectorClock) -> bool {
        matches!(self.compare(other), Ordering::Greater | Ordering::Equal)
    }

    /// Get all node IDs in this clock
    pub fn node_ids(&self) -> Vec<NodeId> {
        self.clocks.keys().cloned().collect()
    }

    /// Get the sum of all clock values (logical timestamp)
    pub fn sum(&self) -> u64 {
        self.clocks.values().sum()
    }
}

impl Default for VectorClock {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for VectorClock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{")?;
        let mut items: Vec<_> = self.clocks.iter().collect();
        items.sort_by_key(|(k, _)| *k);

        for (i, (node, val)) in items.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}: {}", node, val)?;
        }
        write!(f, "}}")
    }
}

/// Ordering relationship between vector clocks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ordering {
    /// First clock is less than second
    Less,
    /// First clock is greater than second
    Greater,
    /// Clocks are equal
    Equal,
    /// Clocks are concurrent (no causal relationship)
    Concurrent,
}

/// Version information for a data object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    /// Vector clock for causality tracking
    pub clock: VectorClock,

    /// Node that created this version
    pub node_id: NodeId,

    /// Timestamp when version was created
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Content hash for integrity verification
    pub content_hash: String,
}

impl Version {
    /// Create a new version
    pub fn new(node_id: NodeId, content_hash: String) -> Self {
        let mut clock = VectorClock::new();
        clock.increment(&node_id);

        Self {
            clock,
            node_id,
            timestamp: chrono::Utc::now(),
            content_hash,
        }
    }

    /// Create a new version based on a parent version
    pub fn from_parent(parent: &Version, node_id: NodeId, content_hash: String) -> Self {
        let mut clock = parent.clock.clone();
        clock.increment(&node_id);

        Self {
            clock,
            node_id,
            timestamp: chrono::Utc::now(),
            content_hash,
        }
    }

    /// Compare versions
    pub fn compare(&self, other: &Version) -> Ordering {
        self.clock.compare(&other.clock)
    }

    /// Check if this version is concurrent with another
    pub fn is_concurrent(&self, other: &Version) -> bool {
        self.clock.is_concurrent(&other.clock)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_clock_increment() {
        let mut clock = VectorClock::new();
        clock.increment(&"node1".to_string());
        assert_eq!(clock.get(&"node1".to_string()), 1);

        clock.increment(&"node1".to_string());
        assert_eq!(clock.get(&"node1".to_string()), 2);
    }

    #[test]
    fn test_vector_clock_merge() {
        let mut clock1 = VectorClock::new();
        clock1.increment(&"node1".to_string());
        clock1.increment(&"node1".to_string());

        let mut clock2 = VectorClock::new();
        clock2.increment(&"node2".to_string());

        clock1.merge(&clock2);
        assert_eq!(clock1.get(&"node1".to_string()), 2);
        assert_eq!(clock1.get(&"node2".to_string()), 1);
    }

    #[test]
    fn test_vector_clock_compare() {
        let mut clock1 = VectorClock::new();
        clock1.increment(&"node1".to_string());

        let mut clock2 = VectorClock::new();
        clock2.increment(&"node1".to_string());
        clock2.increment(&"node1".to_string());

        assert_eq!(clock1.compare(&clock2), Ordering::Less);
        assert_eq!(clock2.compare(&clock1), Ordering::Greater);

        let mut clock3 = VectorClock::new();
        clock3.increment(&"node2".to_string());

        assert_eq!(clock1.compare(&clock3), Ordering::Concurrent);
    }
}
