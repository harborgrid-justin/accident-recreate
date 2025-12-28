//! Consistent hashing for data partitioning.

use std::collections::BTreeMap;
use uuid::Uuid;

/// Consistent hash ring.
pub struct ConsistentHashRing {
    /// Virtual nodes on the ring
    ring: BTreeMap<u64, Uuid>,

    /// Number of virtual nodes per physical node
    virtual_nodes: usize,

    /// Node list for iteration
    nodes: Vec<Uuid>,
}

impl ConsistentHashRing {
    /// Create a new consistent hash ring.
    pub fn new(virtual_nodes: usize) -> Self {
        Self {
            ring: BTreeMap::new(),
            virtual_nodes,
            nodes: Vec::new(),
        }
    }

    /// Add a node to the ring.
    pub fn add_node(&mut self, node_id: Uuid) {
        if !self.nodes.contains(&node_id) {
            self.nodes.push(node_id);
        }

        // Add virtual nodes
        for i in 0..self.virtual_nodes {
            let key = format!("{}:{}", node_id, i);
            let hash = self.hash_key(&key);
            self.ring.insert(hash, node_id);
        }
    }

    /// Remove a node from the ring.
    pub fn remove_node(&mut self, node_id: &Uuid) {
        self.nodes.retain(|n| n != node_id);

        // Remove virtual nodes
        for i in 0..self.virtual_nodes {
            let key = format!("{}:{}", node_id, i);
            let hash = self.hash_key(&key);
            self.ring.remove(&hash);
        }
    }

    /// Get the node responsible for a key.
    pub fn get_node(&self, key: &str) -> Option<Uuid> {
        if self.ring.is_empty() {
            return None;
        }

        let hash = self.hash_key(key);

        // Find first node with hash >= key hash (clockwise)
        self.ring
            .range(hash..)
            .next()
            .or_else(|| self.ring.iter().next())
            .map(|(_, node_id)| *node_id)
    }

    /// Get N nodes for a key (for replication).
    pub fn get_nodes(&self, key: &str, n: usize) -> Vec<Uuid> {
        if self.ring.is_empty() {
            return Vec::new();
        }

        let hash = self.hash_key(key);
        let mut result = Vec::new();
        let mut seen = std::collections::HashSet::new();

        // Start from the first node >= hash
        let mut iter = self.ring.range(hash..).chain(self.ring.iter());

        for (_, node_id) in iter {
            if seen.insert(*node_id) {
                result.push(*node_id);
                if result.len() >= n {
                    break;
                }
            }
        }

        result
    }

    /// Get all nodes.
    pub fn nodes(&self) -> &[Uuid] {
        &self.nodes
    }

    /// Get node count.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Hash a key to position on ring.
    fn hash_key(&self, key: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish()
    }

    /// Get ring statistics.
    pub fn stats(&self) -> RingStats {
        let mut distribution = std::collections::HashMap::new();

        for node_id in self.ring.values() {
            *distribution.entry(*node_id).or_insert(0) += 1;
        }

        RingStats {
            total_virtual_nodes: self.ring.len(),
            physical_nodes: self.nodes.len(),
            virtual_nodes_per_node: self.virtual_nodes,
            distribution,
        }
    }
}

/// Ring statistics.
#[derive(Debug)]
pub struct RingStats {
    pub total_virtual_nodes: usize,
    pub physical_nodes: usize,
    pub virtual_nodes_per_node: usize,
    pub distribution: std::collections::HashMap<Uuid, usize>,
}

impl Default for ConsistentHashRing {
    fn default() -> Self {
        Self::new(150)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consistent_hashing() {
        let mut ring = ConsistentHashRing::new(10);

        let node1 = Uuid::new_v4();
        let node2 = Uuid::new_v4();
        let node3 = Uuid::new_v4();

        ring.add_node(node1);
        ring.add_node(node2);
        ring.add_node(node3);

        assert_eq!(ring.node_count(), 3);

        // Test key assignment
        let key = "test-key";
        let node = ring.get_node(key);
        assert!(node.is_some());

        // Test replication nodes
        let nodes = ring.get_nodes(key, 2);
        assert_eq!(nodes.len(), 2);
        assert_ne!(nodes[0], nodes[1]);

        // Remove node and verify
        ring.remove_node(&node2);
        assert_eq!(ring.node_count(), 2);

        let new_node = ring.get_node(key);
        assert!(new_node.is_some());
    }

    #[test]
    fn test_ring_stats() {
        let mut ring = ConsistentHashRing::new(5);

        let node1 = Uuid::new_v4();
        let node2 = Uuid::new_v4();

        ring.add_node(node1);
        ring.add_node(node2);

        let stats = ring.stats();
        assert_eq!(stats.total_virtual_nodes, 10);
        assert_eq!(stats.physical_nodes, 2);
        assert_eq!(stats.virtual_nodes_per_node, 5);
    }
}
