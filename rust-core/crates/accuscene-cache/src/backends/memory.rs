//! In-memory LRU cache backend

use crate::backends::CacheBackend;
use crate::config::CacheConfig;
use crate::error::{CacheError, CacheResult};
use crate::key::CacheKey;
use crate::value::CacheValue;
use parking_lot::RwLock;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tracing::{debug, trace};

/// In-memory LRU cache backend
#[derive(Debug, Clone)]
pub struct MemoryCache<T: Clone> {
    inner: Arc<RwLock<MemoryCacheInner<T>>>,
    max_entries: usize,
}

#[derive(Debug)]
struct MemoryCacheInner<T: Clone> {
    /// Main storage
    data: HashMap<String, CacheValue<T>>,
    /// LRU tracking (most recent at back)
    lru_queue: VecDeque<String>,
    /// Access order tracking
    access_order: HashMap<String, usize>,
    /// Current access generation
    generation: usize,
}

impl<T: Clone> MemoryCache<T> {
    /// Create a new memory cache
    pub fn new(config: &CacheConfig) -> Self {
        Self {
            inner: Arc::new(RwLock::new(MemoryCacheInner {
                data: HashMap::with_capacity(config.max_entries),
                lru_queue: VecDeque::with_capacity(config.max_entries),
                access_order: HashMap::with_capacity(config.max_entries),
                generation: 0,
            })),
            max_entries: config.max_entries,
        }
    }

    /// Create with specific capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Arc::new(RwLock::new(MemoryCacheInner {
                data: HashMap::with_capacity(capacity),
                lru_queue: VecDeque::with_capacity(capacity),
                access_order: HashMap::with_capacity(capacity),
                generation: 0,
            })),
            max_entries: capacity,
        }
    }

    /// Evict least recently used entry
    fn evict_lru(&self) -> CacheResult<()> {
        let mut inner = self.inner.write();

        if let Some(key) = inner.lru_queue.pop_front() {
            inner.data.remove(&key);
            inner.access_order.remove(&key);
            debug!("Evicted LRU entry: {}", key);
            Ok(())
        } else {
            Err(CacheError::Unknown("No entries to evict".to_string()))
        }
    }

    /// Update LRU tracking
    fn update_lru(&self, key: &str) {
        let mut inner = self.inner.write();

        // Remove from current position
        if let Some(pos) = inner.lru_queue.iter().position(|k| k == key) {
            inner.lru_queue.remove(pos);
        }

        // Add to back (most recent)
        inner.lru_queue.push_back(key.to_string());

        // Update access generation
        inner.generation += 1;
        let gen = inner.generation;
        inner.access_order.insert(key.to_string(), gen);
    }
}

impl<T: Clone + Send + Sync + std::fmt::Debug> CacheBackend for MemoryCache<T> {
    type Value = T;

    fn get(&self, key: &CacheKey) -> CacheResult<Option<CacheValue<Self::Value>>> {
        let key_str = key.as_string();

        let result = {
            let inner = self.inner.read();
            inner.data.get(&key_str).cloned()
        };

        if let Some(mut value) = result {
            // Check expiration
            if value.is_expired() {
                trace!("Cache entry expired: {}", key_str);
                let mut inner = self.inner.write();
                inner.data.remove(&key_str);
                if let Some(pos) = inner.lru_queue.iter().position(|k| k == &key_str) {
                    inner.lru_queue.remove(pos);
                }
                inner.access_order.remove(&key_str);
                return Ok(None);
            }

            // Update access tracking
            value.record_access();
            self.update_lru(&key_str);

            // Update stored value with new access info
            let mut inner = self.inner.write();
            inner.data.insert(key_str.clone(), value.clone());

            trace!("Cache hit: {}", key_str);
            Ok(Some(value))
        } else {
            trace!("Cache miss: {}", key_str);
            Ok(None)
        }
    }

    fn insert(&self, key: CacheKey, value: CacheValue<Self::Value>) -> CacheResult<()> {
        let key_str = key.as_string();

        // Evict if at capacity and key doesn't exist
        {
            let inner = self.inner.read();
            if inner.data.len() >= self.max_entries && !inner.data.contains_key(&key_str) {
                drop(inner);
                self.evict_lru()?;
            }
        }

        // Insert value
        {
            let mut inner = self.inner.write();
            inner.data.insert(key_str.clone(), value);
        }

        // Update LRU
        self.update_lru(&key_str);

        debug!("Inserted cache entry: {}", key_str);
        Ok(())
    }

    fn remove(&self, key: &CacheKey) -> CacheResult<Option<CacheValue<Self::Value>>> {
        let key_str = key.as_string();
        let mut inner = self.inner.write();

        let value = inner.data.remove(&key_str);

        if value.is_some() {
            if let Some(pos) = inner.lru_queue.iter().position(|k| k == &key_str) {
                inner.lru_queue.remove(pos);
            }
            inner.access_order.remove(&key_str);
            debug!("Removed cache entry: {}", key_str);
        }

        Ok(value)
    }

    fn contains_key(&self, key: &CacheKey) -> bool {
        let inner = self.inner.read();
        inner.data.contains_key(&key.as_string())
    }

    fn clear(&self) -> CacheResult<()> {
        let mut inner = self.inner.write();
        inner.data.clear();
        inner.lru_queue.clear();
        inner.access_order.clear();
        inner.generation = 0;
        debug!("Cleared all cache entries");
        Ok(())
    }

    fn len(&self) -> usize {
        let inner = self.inner.read();
        inner.data.len()
    }

    fn keys(&self) -> Vec<CacheKey> {
        let inner = self.inner.read();
        inner
            .data
            .keys()
            .map(|k| k.clone().into())
            .collect()
    }

    fn evict_expired(&self) -> CacheResult<usize> {
        let mut inner = self.inner.write();
        let mut expired_keys = Vec::new();

        for (key, value) in inner.data.iter() {
            if value.is_expired() {
                expired_keys.push(key.clone());
            }
        }

        let count = expired_keys.len();
        for key in expired_keys {
            inner.data.remove(&key);
            if let Some(pos) = inner.lru_queue.iter().position(|k| k == &key) {
                inner.lru_queue.remove(pos);
            }
            inner.access_order.remove(&key);
        }

        if count > 0 {
            debug!("Evicted {} expired entries", count);
        }

        Ok(count)
    }

    fn capacity(&self) -> usize {
        self.max_entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::CacheConfig;

    #[test]
    fn test_memory_cache_insert_get() {
        let cache = MemoryCache::with_capacity(10);
        let key = CacheKey::new("test", "key1");
        let value = CacheValue::new("data");

        cache.insert(key.clone(), value.clone()).unwrap();
        let result = cache.get(&key).unwrap();

        assert!(result.is_some());
        assert_eq!(result.unwrap().data, "data");
    }

    #[test]
    fn test_memory_cache_lru_eviction() {
        let cache = MemoryCache::with_capacity(2);

        cache.insert(CacheKey::new("test", "key1"), CacheValue::new("data1")).unwrap();
        cache.insert(CacheKey::new("test", "key2"), CacheValue::new("data2")).unwrap();
        cache.insert(CacheKey::new("test", "key3"), CacheValue::new("data3")).unwrap();

        // key1 should be evicted
        assert!(!cache.contains_key(&CacheKey::new("test", "key1")));
        assert!(cache.contains_key(&CacheKey::new("test", "key2")));
        assert!(cache.contains_key(&CacheKey::new("test", "key3")));
    }

    #[test]
    fn test_memory_cache_clear() {
        let cache = MemoryCache::with_capacity(10);
        cache.insert(CacheKey::new("test", "key1"), CacheValue::new("data1")).unwrap();
        cache.insert(CacheKey::new("test", "key2"), CacheValue::new("data2")).unwrap();

        assert_eq!(cache.len(), 2);
        cache.clear().unwrap();
        assert_eq!(cache.len(), 0);
    }
}
