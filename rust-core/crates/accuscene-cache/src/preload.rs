//! Cache preloading and warming

use crate::backends::CacheBackend;
use crate::error::CacheResult;
use crate::key::CacheKey;
use crate::value::CacheValue;
use std::collections::HashMap;
use tracing::{debug, info};

/// Cache preloading strategy
pub trait PreloadStrategy<T>: Send + Sync {
    /// Load entries to preload
    fn load_entries(&self) -> CacheResult<Vec<(CacheKey, T)>>;

    /// Strategy name
    fn name(&self) -> &str;
}

/// Preload from a collection
pub struct CollectionPreload<T> {
    entries: Vec<(CacheKey, T)>,
}

impl<T> CollectionPreload<T> {
    pub fn new(entries: Vec<(CacheKey, T)>) -> Self {
        Self { entries }
    }
}

impl<T: Clone + Send + Sync> PreloadStrategy<T> for CollectionPreload<T> {
    fn load_entries(&self) -> CacheResult<Vec<(CacheKey, T)>> {
        Ok(self.entries.clone())
    }

    fn name(&self) -> &str {
        "collection"
    }
}

/// Preload from a function
pub struct FunctionPreload<T, F>
where
    F: Fn() -> CacheResult<Vec<(CacheKey, T)>> + Send + Sync,
{
    loader: F,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, F> FunctionPreload<T, F>
where
    F: Fn() -> CacheResult<Vec<(CacheKey, T)>> + Send + Sync,
{
    pub fn new(loader: F) -> Self {
        Self {
            loader,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: Clone + Send + Sync, F> PreloadStrategy<T> for FunctionPreload<T, F>
where
    F: Fn() -> CacheResult<Vec<(CacheKey, T)>> + Send + Sync,
{
    fn load_entries(&self) -> CacheResult<Vec<(CacheKey, T)>> {
        (self.loader)()
    }

    fn name(&self) -> &str {
        "function"
    }
}

/// Cache preloader/warmer
pub struct CachePreloader<T: Clone + Send + Sync> {
    cache: Box<dyn CacheBackend<Value = T>>,
}

impl<T: Clone + Send + Sync + 'static> CachePreloader<T> {
    pub fn new(cache: Box<dyn CacheBackend<Value = T>>) -> Self {
        Self { cache }
    }

    /// Preload cache using a strategy
    pub fn preload<S: PreloadStrategy<T>>(&self, strategy: &S) -> CacheResult<usize> {
        info!("Starting cache preload with {} strategy", strategy.name());

        let entries = strategy.load_entries()?;
        let count = entries.len();

        for (key, value) in entries {
            let cache_value = CacheValue::new(value);
            self.cache.insert(key, cache_value)?;
        }

        info!("Preloaded {} entries into cache", count);
        Ok(count)
    }

    /// Warm cache with frequently accessed keys
    pub fn warm_frequently_accessed(&self, keys: Vec<CacheKey>) -> CacheResult<usize> {
        let mut warmed = 0;

        for key in keys {
            // Check if already in cache
            if !self.cache.contains_key(&key) {
                // Would typically load from database/source here
                // For now, just count the keys
                debug!("Would warm key: {}", key);
            } else {
                warmed += 1;
            }
        }

        debug!("Warmed {} cache entries", warmed);
        Ok(warmed)
    }

    /// Preload with custom TTL
    pub fn preload_with_ttl<S: PreloadStrategy<T>>(
        &self,
        strategy: &S,
        ttl: chrono::Duration,
    ) -> CacheResult<usize> {
        info!("Starting cache preload with TTL: {:?}", ttl);

        let entries = strategy.load_entries()?;
        let count = entries.len();

        for (key, value) in entries {
            let cache_value = CacheValue::with_ttl(value, ttl);
            self.cache.insert(key, cache_value)?;
        }

        info!("Preloaded {} entries with TTL into cache", count);
        Ok(count)
    }
}

/// Preload configuration
#[derive(Debug, Clone)]
pub struct PreloadConfig {
    /// Enable preloading on startup
    pub enabled: bool,
    /// Maximum entries to preload
    pub max_entries: usize,
    /// Whether to preload in background
    pub background: bool,
    /// Preload timeout in seconds
    pub timeout_secs: u64,
}

impl Default for PreloadConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_entries: 1000,
            background: true,
            timeout_secs: 30,
        }
    }
}

/// Simple in-memory preload data store
pub struct PreloadDataStore<T> {
    data: HashMap<String, T>,
}

impl<T: Clone> PreloadDataStore<T> {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn add(&mut self, namespace: &str, id: &str, value: T) {
        let key = format!("{}:{}", namespace, id);
        self.data.insert(key, value);
    }

    pub fn to_preload_entries(&self) -> Vec<(CacheKey, T)> {
        self.data
            .iter()
            .map(|(key_str, value)| {
                let key: CacheKey = key_str.clone().into();
                (key, value.clone())
            })
            .collect()
    }
}

impl<T: Clone> Default for PreloadDataStore<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backends::memory::MemoryCache;

    #[test]
    fn test_collection_preload() {
        let cache = Box::new(MemoryCache::with_capacity(10));
        let preloader = CachePreloader::new(cache);

        let entries = vec![
            (CacheKey::new("test", "key1"), "value1".to_string()),
            (CacheKey::new("test", "key2"), "value2".to_string()),
        ];

        let strategy = CollectionPreload::new(entries);
        let count = preloader.preload(&strategy).unwrap();

        assert_eq!(count, 2);
        assert!(preloader.cache.contains_key(&CacheKey::new("test", "key1")));
        assert!(preloader.cache.contains_key(&CacheKey::new("test", "key2")));
    }

    #[test]
    fn test_function_preload() {
        let cache = Box::new(MemoryCache::with_capacity(10));
        let preloader = CachePreloader::new(cache);

        let loader = || {
            Ok(vec![
                (CacheKey::new("test", "key1"), "value1".to_string()),
                (CacheKey::new("test", "key2"), "value2".to_string()),
            ])
        };

        let strategy = FunctionPreload::new(loader);
        let count = preloader.preload(&strategy).unwrap();

        assert_eq!(count, 2);
    }

    #[test]
    fn test_preload_data_store() {
        let mut store = PreloadDataStore::new();
        store.add("test", "key1", "value1");
        store.add("test", "key2", "value2");

        let entries = store.to_preload_entries();
        assert_eq!(entries.len(), 2);
    }
}
