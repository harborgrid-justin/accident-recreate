//! Partitioning strategies for stream distribution.

use crate::error::{Result, StreamingError};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Trait for partitioning strategies
pub trait Partitioner<T>: Send + Sync {
    /// Determine which partition an item belongs to
    fn partition(&self, item: &T, num_partitions: usize) -> usize;
}

/// Round-robin partitioner
pub struct RoundRobinPartitioner {
    counter: std::sync::atomic::AtomicUsize,
}

impl RoundRobinPartitioner {
    /// Create a new round-robin partitioner
    pub fn new() -> Self {
        Self {
            counter: std::sync::atomic::AtomicUsize::new(0),
        }
    }
}

impl Default for RoundRobinPartitioner {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Partitioner<T> for RoundRobinPartitioner {
    fn partition(&self, _item: &T, num_partitions: usize) -> usize {
        let count = self
            .counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        count % num_partitions
    }
}

/// Hash-based partitioner
pub struct HashPartitioner;

impl HashPartitioner {
    /// Create a new hash partitioner
    pub fn new() -> Self {
        Self
    }
}

impl Default for HashPartitioner {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Hash> Partitioner<T> for HashPartitioner {
    fn partition(&self, item: &T, num_partitions: usize) -> usize {
        let mut hasher = DefaultHasher::new();
        item.hash(&mut hasher);
        let hash = hasher.finish();
        (hash % num_partitions as u64) as usize
    }
}

/// Key-based partitioner that extracts a key from an item
pub struct KeyPartitioner<T, K, F>
where
    F: Fn(&T) -> &K + Send + Sync,
    K: Hash,
{
    key_fn: F,
    _phantom: std::marker::PhantomData<(T, K)>,
}

impl<T, K, F> KeyPartitioner<T, K, F>
where
    F: Fn(&T) -> &K + Send + Sync,
    K: Hash,
{
    /// Create a new key-based partitioner
    pub fn new(key_fn: F) -> Self {
        Self {
            key_fn,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, K, F> Partitioner<T> for KeyPartitioner<T, K, F>
where
    F: Fn(&T) -> &K + Send + Sync,
    K: Hash,
{
    fn partition(&self, item: &T, num_partitions: usize) -> usize {
        let key = (self.key_fn)(item);
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash = hasher.finish();
        (hash % num_partitions as u64) as usize
    }
}

/// Random partitioner
pub struct RandomPartitioner;

impl RandomPartitioner {
    /// Create a new random partitioner
    pub fn new() -> Self {
        Self
    }
}

impl Default for RandomPartitioner {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Partitioner<T> for RandomPartitioner {
    fn partition(&self, _item: &T, num_partitions: usize) -> usize {
        use std::collections::hash_map::RandomState;
        use std::hash::BuildHasher;

        let hasher = RandomState::new().build_hasher().finish();
        (hasher % num_partitions as u64) as usize
    }
}

/// Custom partitioner using a closure
pub struct CustomPartitioner<T, F>
where
    F: Fn(&T, usize) -> usize + Send + Sync,
{
    partition_fn: F,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, F> CustomPartitioner<T, F>
where
    F: Fn(&T, usize) -> usize + Send + Sync,
{
    /// Create a new custom partitioner
    pub fn new(partition_fn: F) -> Self {
        Self {
            partition_fn,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, F> Partitioner<T> for CustomPartitioner<T, F>
where
    F: Fn(&T, usize) -> usize + Send + Sync,
{
    fn partition(&self, item: &T, num_partitions: usize) -> usize {
        (self.partition_fn)(item, num_partitions) % num_partitions
    }
}

/// Partition assignment for distributing items across partitions
#[derive(Debug, Clone)]
pub struct PartitionAssignment {
    /// Total number of partitions
    pub num_partitions: usize,
    /// Partitions assigned to this instance
    pub assigned_partitions: Vec<usize>,
}

impl PartitionAssignment {
    /// Create a new partition assignment
    pub fn new(num_partitions: usize, assigned_partitions: Vec<usize>) -> Self {
        Self {
            num_partitions,
            assigned_partitions,
        }
    }

    /// Create an assignment for all partitions
    pub fn all(num_partitions: usize) -> Self {
        Self {
            num_partitions,
            assigned_partitions: (0..num_partitions).collect(),
        }
    }

    /// Check if this instance should handle a given partition
    pub fn owns_partition(&self, partition: usize) -> bool {
        self.assigned_partitions.contains(&partition)
    }

    /// Distribute partitions evenly among instances
    pub fn distribute(num_partitions: usize, instance_id: usize, num_instances: usize) -> Self {
        let mut assigned = Vec::new();
        for partition in 0..num_partitions {
            if partition % num_instances == instance_id {
                assigned.push(partition);
            }
        }

        Self {
            num_partitions,
            assigned_partitions: assigned,
        }
    }
}

/// Partition router that routes items to different channels based on partitioning
pub struct PartitionRouter<T> {
    partitioner: Box<dyn Partitioner<T>>,
    num_partitions: usize,
}

impl<T: Send + 'static> PartitionRouter<T> {
    /// Create a new partition router
    pub fn new(partitioner: Box<dyn Partitioner<T>>, num_partitions: usize) -> Self {
        Self {
            partitioner,
            num_partitions,
        }
    }

    /// Route an item to its partition
    pub fn route(&self, item: &T) -> usize {
        self.partitioner.partition(item, self.num_partitions)
    }

    /// Get the number of partitions
    pub fn num_partitions(&self) -> usize {
        self.num_partitions
    }
}

/// Rebalancer for dynamically adjusting partition assignments
pub struct PartitionRebalancer {
    num_partitions: usize,
    instance_assignments: Vec<Vec<usize>>,
}

impl PartitionRebalancer {
    /// Create a new partition rebalancer
    pub fn new(num_partitions: usize, num_instances: usize) -> Self {
        let mut instance_assignments = vec![Vec::new(); num_instances];

        // Initial even distribution
        for partition in 0..num_partitions {
            let instance = partition % num_instances;
            instance_assignments[instance].push(partition);
        }

        Self {
            num_partitions,
            instance_assignments,
        }
    }

    /// Get assignment for an instance
    pub fn get_assignment(&self, instance_id: usize) -> Result<PartitionAssignment> {
        if instance_id >= self.instance_assignments.len() {
            return Err(StreamingError::Partitioning(format!(
                "Instance {} out of range",
                instance_id
            )));
        }

        Ok(PartitionAssignment {
            num_partitions: self.num_partitions,
            assigned_partitions: self.instance_assignments[instance_id].clone(),
        })
    }

    /// Rebalance partitions when instances change
    pub fn rebalance(&mut self, num_instances: usize) {
        // Collect all partitions
        let all_partitions: Vec<usize> = (0..self.num_partitions).collect();

        // Redistribute
        self.instance_assignments = vec![Vec::new(); num_instances];
        for (i, partition) in all_partitions.iter().enumerate() {
            let instance = i % num_instances;
            self.instance_assignments[instance].push(*partition);
        }
    }

    /// Get total number of instances
    pub fn num_instances(&self) -> usize {
        self.instance_assignments.len()
    }

    /// Get total number of partitions
    pub fn num_partitions(&self) -> usize {
        self.num_partitions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_robin_partitioner() {
        let partitioner = RoundRobinPartitioner::new();

        assert_eq!(partitioner.partition(&"item1", 3), 0);
        assert_eq!(partitioner.partition(&"item2", 3), 1);
        assert_eq!(partitioner.partition(&"item3", 3), 2);
        assert_eq!(partitioner.partition(&"item4", 3), 0);
    }

    #[test]
    fn test_hash_partitioner() {
        let partitioner = HashPartitioner::new();

        // Same item should always go to same partition
        let partition1 = partitioner.partition(&"item1", 3);
        let partition2 = partitioner.partition(&"item1", 3);
        assert_eq!(partition1, partition2);

        // Result should be in valid range
        assert!(partition1 < 3);
    }

    #[test]
    fn test_key_partitioner() {
        #[derive(Hash)]
        struct Item {
            key: String,
            value: i32,
        }

        let partitioner = KeyPartitioner::new(|item: &Item| &item.key);

        let item1 = Item {
            key: "key1".to_string(),
            value: 1,
        };
        let item2 = Item {
            key: "key1".to_string(),
            value: 2,
        };

        // Items with same key should go to same partition
        assert_eq!(
            partitioner.partition(&item1, 3),
            partitioner.partition(&item2, 3)
        );
    }

    #[test]
    fn test_partition_assignment() {
        let assignment = PartitionAssignment::distribute(10, 0, 3);

        // Instance 0 should own partitions 0, 3, 6, 9
        assert!(assignment.owns_partition(0));
        assert!(!assignment.owns_partition(1));
        assert!(!assignment.owns_partition(2));
        assert!(assignment.owns_partition(3));
        assert!(assignment.owns_partition(6));
        assert!(assignment.owns_partition(9));
    }

    #[test]
    fn test_partition_rebalancer() {
        let mut rebalancer = PartitionRebalancer::new(10, 3);

        // Initially 3 instances
        assert_eq!(rebalancer.num_instances(), 3);

        let assignment0 = rebalancer.get_assignment(0).unwrap();
        assert_eq!(assignment0.assigned_partitions.len(), 4); // 0, 3, 6, 9

        // Rebalance to 2 instances
        rebalancer.rebalance(2);
        assert_eq!(rebalancer.num_instances(), 2);

        let assignment0_new = rebalancer.get_assignment(0).unwrap();
        assert_eq!(assignment0_new.assigned_partitions.len(), 5); // 0, 2, 4, 6, 8
    }
}
