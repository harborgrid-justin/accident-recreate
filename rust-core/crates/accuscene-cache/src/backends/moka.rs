//! Moka concurrent cache backend

use crate::backends::CacheBackend;
use crate::config::CacheConfig;
use crate::error::CacheResult;
use crate::key::CacheKey;
use crate::value::CacheValue;
use moka::sync::Cache as MokaCache;
use std::sync::Arc;
use tracing::{debug, trace};

/// Moka-based concurrent cache backend
#[derive(Debug, Clone)]
pub struct MokaCacheBackend<T: Clone + Send + Sync + 'static> {
    cache: Arc<MokaCache<String, CacheValue<T>>>,
    max_entries: usize,
}

impl<T: Clone + Send + Sync + 'static> MokaCacheBackend<T> {
    /// Create a new Moka cache backend
    pub fn new(config: &CacheConfig) -> Self {
        let cache = MokaCache::builder()
            .max_capacity(config.max_entries as u64)
            .build();

        Self {
            cache: Arc::new(cache),
            max_entries: config.max_entries,
        }
    }

    /// Create with specific capacity
    pub fn with_capacity(capacity: usize) -> Self {
        let cache = MokaCache::builder()
            .max_capacity(capacity as u64)
            .build();

        Self {
            cache: Arc::new(cache),
            max_entries: capacity,
        }
    }

    /// Create with TTL
    pub fn with_ttl(capacity: usize, ttl_seconds: u64) -> Self {
        let cache = MokaCache::builder()
            .max_capacity(capacity as u64)
            .time_to_live(std::time::Duration::from_secs(ttl_seconds))
            .build();

        Self {
            cache: Arc::new(cache),
            max_entries: capacity,
        }
    }

    /// Create with custom builder
    pub fn with_builder(
        builder: moka::sync::CacheBuilder<String, CacheValue<T>, moka::sync::Cache<String, CacheValue<T>>>,
    ) -> Self {
        let cache = builder.build();
        let max_entries = cache.policy().max_capacity().unwrap_or(10_000) as usize;

        Self {
            cache: Arc::new(cache),
            max_entries,
        }
    }

    /// Get cache statistics
    pub fn hit_count(&self) -> u64 {
        self.cache.hit_count()
    }

    pub fn miss_count(&self) -> u64 {
        self.cache.miss_count()
    }

    pub fn hit_rate(&self) -> f64 {
        let hits = self.hit_count();
        let misses = self.miss_count();
        let total = hits + misses;

        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }

    /// Run cache maintenance (eviction, cleanup)
    pub fn run_pending_tasks(&self) {
        self.cache.run_pending_tasks();
    }
}

impl<T: Clone + Send + Sync + std::fmt::Debug + 'static> CacheBackend for MokaCacheBackend<T> {
    type Value = T;

    fn get(&self, key: &CacheKey) -> CacheResult<Option<CacheValue<Self::Value>>> {
        let key_str = key.as_string();

        if let Some(mut value) = self.cache.get(&key_str) {
            // Check expiration
            if value.is_expired() {
                trace!("Cache entry expired: {}", key_str);
                self.cache.invalidate(&key_str);
                return Ok(None);
            }

            // Update access tracking
            value.record_access();

            // Re-insert to update metadata
            self.cache.insert(key_str.clone(), value.clone());

            trace!("Cache hit: {}", key_str);
            Ok(Some(value))
        } else {
            trace!("Cache miss: {}", key_str);
            Ok(None)
        }
    }

    fn insert(&self, key: CacheKey, value: CacheValue<Self::Value>) -> CacheResult<()> {
        let key_str = key.as_string();
        self.cache.insert(key_str.clone(), value);
        debug!("Inserted cache entry: {}", key_str);
        Ok(())
    }

    fn remove(&self, key: &CacheKey) -> CacheResult<Option<CacheValue<Self::Value>>> {
        let key_str = key.as_string();
        let value = self.cache.get(&key_str);
        self.cache.invalidate(&key_str);

        if value.is_some() {
            debug!("Removed cache entry: {}", key_str);
        }

        Ok(value)
    }

    fn contains_key(&self, key: &CacheKey) -> bool {
        self.cache.contains_key(&key.as_string())
    }

    fn clear(&self) -> CacheResult<()> {
        self.cache.invalidate_all();
        debug!("Cleared all cache entries");
        Ok(())
    }

    fn len(&self) -> usize {
        self.cache.entry_count() as usize
    }

    fn keys(&self) -> Vec<CacheKey> {
        // Moka doesn't provide direct key iteration
        // This is a limitation - would need to maintain separate key tracking
        Vec::new()
    }

    fn evict_expired(&self) -> CacheResult<usize> {
        // Moka handles TTL-based eviction automatically
        // Run pending tasks to trigger cleanup
        self.cache.run_pending_tasks();
        Ok(0)
    }

    fn capacity(&self) -> usize {
        self.max_entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_moka_cache_insert_get() {
        let cache = MokaCacheBackend::with_capacity(10);
        let key = CacheKey::new("test", "key1");
        let value = CacheValue::new("data");

        cache.insert(key.clone(), value.clone()).unwrap();
        let result = cache.get(&key).unwrap();

        assert!(result.is_some());
        assert_eq!(result.unwrap().data, "data");
    }

    #[test]
    fn test_moka_cache_remove() {
        let cache = MokaCacheBackend::with_capacity(10);
        let key = CacheKey::new("test", "key1");
        let value = CacheValue::new("data");

        cache.insert(key.clone(), value).unwrap();
        assert!(cache.contains_key(&key));

        cache.remove(&key).unwrap();
        assert!(!cache.contains_key(&key));
    }

    #[test]
    fn test_moka_cache_statistics() {
        let cache = MokaCacheBackend::with_capacity(10);
        let key = CacheKey::new("test", "key1");

        // Miss
        let _ = cache.get(&key);
        assert_eq!(cache.miss_count(), 1);

        // Insert and hit
        cache.insert(key.clone(), CacheValue::new("data")).unwrap();
        let _ = cache.get(&key);
        assert_eq!(cache.hit_count(), 1);

        // Hit rate
        assert!(cache.hit_rate() > 0.0);
    }

    #[test]
    fn test_moka_cache_clear() {
        let cache = MokaCacheBackend::with_capacity(10);
        cache.insert(CacheKey::new("test", "key1"), CacheValue::new("data1")).unwrap();
        cache.insert(CacheKey::new("test", "key2"), CacheValue::new("data2")).unwrap();

        assert_eq!(cache.len(), 2);
        cache.clear().unwrap();
        assert_eq!(cache.len(), 0);
    }
}
