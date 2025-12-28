//! Data partitioning strategies.

pub mod consistent_hash;
pub mod range;

pub use consistent_hash::{ConsistentHashRing, RingStats};
pub use range::{RangePartition, RangePartitionMap};

use uuid::Uuid;

/// Partitioning strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartitioningStrategy {
    /// Consistent hashing
    ConsistentHash,
    /// Range-based partitioning
    Range,
}

/// Unified partitioning service.
pub struct PartitioningService {
    /// Partitioning strategy
    strategy: PartitioningStrategy,

    /// Consistent hash ring
    hash_ring: Option<ConsistentHashRing>,

    /// Range partition map
    range_map: Option<RangePartitionMap>,

    /// Replication factor
    replication_factor: usize,
}

impl PartitioningService {
    /// Create a new partitioning service with consistent hashing.
    pub fn with_consistent_hash(virtual_nodes: usize, replication_factor: usize) -> Self {
        Self {
            strategy: PartitioningStrategy::ConsistentHash,
            hash_ring: Some(ConsistentHashRing::new(virtual_nodes)),
            range_map: None,
            replication_factor,
        }
    }

    /// Create a new partitioning service with range partitioning.
    pub fn with_range(replication_factor: usize) -> Self {
        Self {
            strategy: PartitioningStrategy::Range,
            hash_ring: None,
            range_map: Some(RangePartitionMap::new(replication_factor)),
            replication_factor,
        }
    }

    /// Add a node.
    pub fn add_node(&mut self, node_id: Uuid) {
        match self.strategy {
            PartitioningStrategy::ConsistentHash => {
                if let Some(ref mut ring) = self.hash_ring {
                    ring.add_node(node_id);
                }
            }
            PartitioningStrategy::Range => {
                // Range partitions are added explicitly
            }
        }
    }

    /// Remove a node.
    pub fn remove_node(&mut self, node_id: &Uuid) {
        match self.strategy {
            PartitioningStrategy::ConsistentHash => {
                if let Some(ref mut ring) = self.hash_ring {
                    ring.remove_node(node_id);
                }
            }
            PartitioningStrategy::Range => {
                // Range partitions need manual rebalancing
            }
        }
    }

    /// Get node for a key.
    pub fn get_node(&self, key: &str) -> Option<Uuid> {
        match self.strategy {
            PartitioningStrategy::ConsistentHash => {
                self.hash_ring.as_ref().and_then(|r| r.get_node(key))
            }
            PartitioningStrategy::Range => {
                self.range_map.as_ref().and_then(|m| m.get_node(key))
            }
        }
    }

    /// Get nodes for a key (for replication).
    pub fn get_nodes(&self, key: &str) -> Vec<Uuid> {
        match self.strategy {
            PartitioningStrategy::ConsistentHash => self
                .hash_ring
                .as_ref()
                .map(|r| r.get_nodes(key, self.replication_factor))
                .unwrap_or_default(),
            PartitioningStrategy::Range => {
                self.range_map.as_ref().map(|m| m.get_nodes(key)).unwrap_or_default()
            }
        }
    }

    /// Get all nodes.
    pub fn all_nodes(&self) -> Vec<Uuid> {
        match self.strategy {
            PartitioningStrategy::ConsistentHash => self
                .hash_ring
                .as_ref()
                .map(|r| r.nodes().to_vec())
                .unwrap_or_default(),
            PartitioningStrategy::Range => {
                // Collect unique nodes from all partitions
                if let Some(ref map) = self.range_map {
                    let mut nodes: Vec<Uuid> = map
                        .partitions()
                        .iter()
                        .flat_map(|p| p.all_nodes())
                        .collect();
                    nodes.sort();
                    nodes.dedup();
                    nodes
                } else {
                    Vec::new()
                }
            }
        }
    }

    /// Get node count.
    pub fn node_count(&self) -> usize {
        match self.strategy {
            PartitioningStrategy::ConsistentHash => {
                self.hash_ring.as_ref().map(|r| r.node_count()).unwrap_or(0)
            }
            PartitioningStrategy::Range => self.all_nodes().len(),
        }
    }

    /// Get strategy.
    pub fn strategy(&self) -> PartitioningStrategy {
        self.strategy
    }

    /// Get hash ring (if using consistent hashing).
    pub fn hash_ring(&self) -> Option<&ConsistentHashRing> {
        self.hash_ring.as_ref()
    }

    /// Get hash ring (mutable).
    pub fn hash_ring_mut(&mut self) -> Option<&mut ConsistentHashRing> {
        self.hash_ring.as_mut()
    }

    /// Get range map (if using range partitioning).
    pub fn range_map(&self) -> Option<&RangePartitionMap> {
        self.range_map.as_ref()
    }

    /// Get range map (mutable).
    pub fn range_map_mut(&mut self) -> Option<&mut RangePartitionMap> {
        self.range_map.as_mut()
    }
}

impl Default for PartitioningService {
    fn default() -> Self {
        Self::with_consistent_hash(150, 3)
    }
}
