//! Range-based data partitioning.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

/// Range partition definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangePartition {
    /// Partition ID
    pub id: Uuid,

    /// Start key (inclusive)
    pub start: String,

    /// End key (exclusive)
    pub end: String,

    /// Node responsible for this partition
    pub node_id: Uuid,

    /// Replica nodes
    pub replicas: Vec<Uuid>,
}

impl RangePartition {
    /// Create a new range partition.
    pub fn new(start: String, end: String, node_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            start,
            end,
            node_id,
            replicas: Vec::new(),
        }
    }

    /// Check if a key belongs to this partition.
    pub fn contains(&self, key: &str) -> bool {
        key >= self.start.as_str() && key < self.end.as_str()
    }

    /// Add a replica.
    pub fn add_replica(&mut self, node_id: Uuid) {
        if !self.replicas.contains(&node_id) && node_id != self.node_id {
            self.replicas.push(node_id);
        }
    }

    /// Remove a replica.
    pub fn remove_replica(&mut self, node_id: &Uuid) {
        self.replicas.retain(|n| n != node_id);
    }

    /// Get all nodes (primary + replicas).
    pub fn all_nodes(&self) -> Vec<Uuid> {
        let mut nodes = vec![self.node_id];
        nodes.extend_from_slice(&self.replicas);
        nodes
    }
}

/// Range partition map.
pub struct RangePartitionMap {
    /// Partitions sorted by start key
    partitions: BTreeMap<String, RangePartition>,

    /// Replication factor
    replication_factor: usize,
}

impl RangePartitionMap {
    /// Create a new range partition map.
    pub fn new(replication_factor: usize) -> Self {
        Self {
            partitions: BTreeMap::new(),
            replication_factor,
        }
    }

    /// Add a partition.
    pub fn add_partition(&mut self, partition: RangePartition) {
        self.partitions.insert(partition.start.clone(), partition);
    }

    /// Remove a partition.
    pub fn remove_partition(&mut self, start_key: &str) -> Option<RangePartition> {
        self.partitions.remove(start_key)
    }

    /// Find partition for a key.
    pub fn find_partition(&self, key: &str) -> Option<&RangePartition> {
        // Find the partition whose start <= key
        self.partitions
            .range(..=key.to_string())
            .next_back()
            .and_then(|(_, partition)| {
                if partition.contains(key) {
                    Some(partition)
                } else {
                    None
                }
            })
    }

    /// Find partition for a key (mutable).
    pub fn find_partition_mut(&mut self, key: &str) -> Option<&mut RangePartition> {
        // Find start key first
        let start_key = self
            .partitions
            .range(..=key.to_string())
            .next_back()
            .map(|(k, _)| k.clone());

        if let Some(start) = start_key {
            self.partitions.get_mut(&start).filter(|p| p.contains(key))
        } else {
            None
        }
    }

    /// Get node for a key.
    pub fn get_node(&self, key: &str) -> Option<Uuid> {
        self.find_partition(key).map(|p| p.node_id)
    }

    /// Get all nodes for a key (including replicas).
    pub fn get_nodes(&self, key: &str) -> Vec<Uuid> {
        self.find_partition(key)
            .map(|p| p.all_nodes())
            .unwrap_or_default()
    }

    /// Split a partition at a key.
    pub fn split_partition(&mut self, split_key: &str, new_node_id: Uuid) -> Option<Uuid> {
        // Find partition containing split key
        let start_key = self
            .partitions
            .range(..=split_key.to_string())
            .next_back()
            .map(|(k, _)| k.clone());

        if let Some(start) = start_key {
            if let Some(mut partition) = self.partitions.remove(&start) {
                if partition.contains(split_key) && split_key > partition.start.as_str() {
                    // Create new partition for upper range
                    let new_partition = RangePartition::new(
                        split_key.to_string(),
                        partition.end.clone(),
                        new_node_id,
                    );

                    // Save the new partition ID before moving
                    let new_partition_id = new_partition.id;

                    // Update existing partition's end
                    partition.end = split_key.to_string();

                    // Re-insert both partitions
                    self.partitions.insert(partition.start.clone(), partition);
                    self.partitions
                        .insert(new_partition.start.clone(), new_partition);

                    return Some(new_partition_id);
                } else {
                    // Restore partition if split failed
                    self.partitions.insert(start, partition);
                }
            }
        }

        None
    }

    /// Merge two adjacent partitions.
    pub fn merge_partitions(&mut self, start_key: &str) -> bool {
        // Get current partition
        let current = match self.partitions.remove(start_key) {
            Some(p) => p,
            None => return false,
        };

        // Get next partition
        let next_start = current.end.clone();
        if let Some(mut next) = self.partitions.remove(&next_start) {
            // Merge
            let merged = RangePartition {
                id: current.id,
                start: current.start.clone(),
                end: next.end,
                node_id: current.node_id,
                replicas: {
                    let mut replicas = current.replicas;
                    replicas.extend(next.replicas);
                    replicas.sort();
                    replicas.dedup();
                    replicas
                },
            };

            self.partitions.insert(merged.start.clone(), merged);
            true
        } else {
            // Restore if merge failed
            self.partitions.insert(current.start.clone(), current);
            false
        }
    }

    /// Get all partitions.
    pub fn partitions(&self) -> Vec<&RangePartition> {
        self.partitions.values().collect()
    }

    /// Get partition count.
    pub fn partition_count(&self) -> usize {
        self.partitions.len()
    }

    /// Rebalance partitions across nodes.
    pub fn rebalance(&mut self, nodes: &[Uuid]) {
        if nodes.is_empty() {
            return;
        }

        let partitions: Vec<_> = self.partitions.values_mut().collect();
        let partitions_per_node = (partitions.len() + nodes.len() - 1) / nodes.len();

        for (i, partition) in partitions.into_iter().enumerate() {
            let node_idx = i / partitions_per_node;
            partition.node_id = nodes[node_idx.min(nodes.len() - 1)];

            // Assign replicas
            partition.replicas.clear();
            for j in 1..=self.replication_factor.min(nodes.len() - 1) {
                let replica_idx = (node_idx + j) % nodes.len();
                partition.add_replica(nodes[replica_idx]);
            }
        }
    }
}

impl Default for RangePartitionMap {
    fn default() -> Self {
        Self::new(3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_range_partition() {
        let node_id = Uuid::new_v4();
        let partition = RangePartition::new("a".to_string(), "m".to_string(), node_id);

        assert!(partition.contains("b"));
        assert!(partition.contains("l"));
        assert!(!partition.contains("m"));
        assert!(!partition.contains("z"));
    }

    #[test]
    fn test_range_partition_map() {
        let mut map = RangePartitionMap::new(2);

        let node1 = Uuid::new_v4();
        let node2 = Uuid::new_v4();

        map.add_partition(RangePartition::new("a".to_string(), "m".to_string(), node1));
        map.add_partition(RangePartition::new("m".to_string(), "z".to_string(), node2));

        assert_eq!(map.get_node("a"), Some(node1));
        assert_eq!(map.get_node("b"), Some(node1));
        assert_eq!(map.get_node("m"), Some(node2));
        assert_eq!(map.get_node("z"), None);

        // Test split
        let node3 = Uuid::new_v4();
        let new_id = map.split_partition("f", node3);
        assert!(new_id.is_some());
        assert_eq!(map.partition_count(), 3);
    }
}
