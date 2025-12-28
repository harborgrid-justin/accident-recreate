//! Cache partitioning by type

use crate::backends::CacheBackend;
use crate::config::CacheType;
use crate::error::{CacheError, CacheResult};
use crate::key::CacheKey;
use crate::value::CacheValue;
use dashmap::DashMap;
use std::sync::Arc;
use tracing::debug;

/// Partitioned cache with separate storage per cache type
#[derive(Debug)]
pub struct PartitionedCache<T: Clone + Send + Sync + 'static> {
    partitions: Arc<DashMap<CacheType, Box<dyn CacheBackend<Value = T>>>>,
    default_partition: CacheType,
}

impl<T: Clone + Send + Sync + 'static> PartitionedCache<T> {
    pub fn new(default_partition: CacheType) -> Self {
        Self {
            partitions: Arc::new(DashMap::new()),
            default_partition,
        }
    }

    /// Add a partition for a specific cache type
    pub fn add_partition(
        &self,
        cache_type: CacheType,
        backend: Box<dyn CacheBackend<Value = T>>,
    ) {
        self.partitions.insert(cache_type, backend);
        debug!("Added partition for cache type: {:?}", cache_type);
    }

    /// Get partition for a cache type
    fn get_partition(&self, cache_type: CacheType) -> CacheResult<impl std::ops::Deref<Target = Box<dyn CacheBackend<Value = T>>> + '_> {
        self.partitions
            .get(&cache_type)
            .ok_or_else(|| {
                CacheError::PartitionError(format!("No partition for type: {:?}", cache_type))
            })
    }

    /// Infer cache type from key namespace
    fn infer_cache_type(&self, key: &CacheKey) -> CacheType {
        match key.namespace.as_str() {
            "physics" => CacheType::Physics,
            "query" => CacheType::Query,
            "image" | "render" => CacheType::RenderedImage,
            "session" => CacheType::Session,
            "config" | "configuration" => CacheType::Configuration,
            _ => self.default_partition,
        }
    }

    /// Get value from appropriate partition
    pub fn get(&self, key: &CacheKey) -> CacheResult<Option<CacheValue<T>>> {
        let cache_type = self.infer_cache_type(key);
        let partition = self.get_partition(cache_type)?;
        partition.get(key)
    }

    /// Insert value into appropriate partition
    pub fn insert(&self, key: CacheKey, value: CacheValue<T>) -> CacheResult<()> {
        let cache_type = self.infer_cache_type(&key);
        let partition = self.get_partition(cache_type)?;
        partition.insert(key, value)
    }

    /// Insert with explicit cache type
    pub fn insert_typed(
        &self,
        cache_type: CacheType,
        key: CacheKey,
        value: CacheValue<T>,
    ) -> CacheResult<()> {
        let partition = self.get_partition(cache_type)?;
        partition.insert(key, value)
    }

    /// Remove from appropriate partition
    pub fn remove(&self, key: &CacheKey) -> CacheResult<Option<CacheValue<T>>> {
        let cache_type = self.infer_cache_type(key);
        let partition = self.get_partition(cache_type)?;
        partition.remove(key)
    }

    /// Check if key exists in appropriate partition
    pub fn contains_key(&self, key: &CacheKey) -> bool {
        let cache_type = self.infer_cache_type(key);
        if let Ok(partition) = self.get_partition(cache_type) {
            partition.contains_key(key)
        } else {
            false
        }
    }

    /// Clear a specific partition
    pub fn clear_partition(&self, cache_type: CacheType) -> CacheResult<()> {
        let partition = self.get_partition(cache_type)?;
        partition.clear()?;
        debug!("Cleared partition: {:?}", cache_type);
        Ok(())
    }

    /// Clear all partitions
    pub fn clear_all(&self) -> CacheResult<()> {
        for partition in self.partitions.iter() {
            partition.value().clear()?;
        }
        debug!("Cleared all partitions");
        Ok(())
    }

    /// Get partition statistics
    pub fn partition_stats(&self, cache_type: CacheType) -> CacheResult<PartitionStats> {
        let partition = self.get_partition(cache_type)?;
        Ok(PartitionStats {
            cache_type,
            entries: partition.len(),
            capacity: partition.capacity(),
        })
    }

    /// Get all partition statistics
    pub fn all_partition_stats(&self) -> Vec<PartitionStats> {
        self.partitions
            .iter()
            .map(|entry| {
                let cache_type = *entry.key();
                let partition = entry.value();
                PartitionStats {
                    cache_type,
                    entries: partition.len(),
                    capacity: partition.capacity(),
                }
            })
            .collect()
    }

    /// Get total entries across all partitions
    pub fn total_entries(&self) -> usize {
        self.partitions
            .iter()
            .map(|entry| entry.value().len())
            .sum()
    }

    /// Evict expired entries from all partitions
    pub fn evict_expired_all(&self) -> CacheResult<usize> {
        let mut total = 0;
        for partition in self.partitions.iter() {
            total += partition.value().evict_expired()?;
        }
        if total > 0 {
            debug!("Evicted {} expired entries across all partitions", total);
        }
        Ok(total)
    }
}

/// Statistics for a single partition
#[derive(Debug, Clone)]
pub struct PartitionStats {
    pub cache_type: CacheType,
    pub entries: usize,
    pub capacity: usize,
}

impl PartitionStats {
    pub fn utilization(&self) -> f64 {
        if self.capacity == 0 {
            0.0
        } else {
            self.entries as f64 / self.capacity as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backends::memory::MemoryCache;

    #[test]
    fn test_partitioned_cache() {
        let cache: PartitionedCache<String> = PartitionedCache::new(CacheType::Generic);

        // Add partitions
        cache.add_partition(
            CacheType::Physics,
            Box::new(MemoryCache::with_capacity(10)),
        );
        cache.add_partition(
            CacheType::Query,
            Box::new(MemoryCache::with_capacity(20)),
        );

        // Insert into physics partition (inferred from namespace)
        let physics_key = CacheKey::new("physics", "sim1");
        cache
            .insert(physics_key.clone(), CacheValue::new("physics_data".to_string()))
            .unwrap();

        // Insert into query partition
        let query_key = CacheKey::new("query", "result1");
        cache
            .insert(query_key.clone(), CacheValue::new("query_data".to_string()))
            .unwrap();

        // Verify retrieval
        assert!(cache.get(&physics_key).unwrap().is_some());
        assert!(cache.get(&query_key).unwrap().is_some());

        // Check partition stats
        let physics_stats = cache.partition_stats(CacheType::Physics).unwrap();
        assert_eq!(physics_stats.entries, 1);
        assert_eq!(physics_stats.capacity, 10);
    }

    #[test]
    fn test_partition_clearing() {
        let cache: PartitionedCache<String> = PartitionedCache::new(CacheType::Generic);

        cache.add_partition(
            CacheType::Physics,
            Box::new(MemoryCache::with_capacity(10)),
        );

        let key = CacheKey::new("physics", "sim1");
        cache
            .insert(key.clone(), CacheValue::new("data".to_string()))
            .unwrap();

        assert!(cache.contains_key(&key));

        cache.clear_partition(CacheType::Physics).unwrap();

        assert!(!cache.contains_key(&key));
    }
}
