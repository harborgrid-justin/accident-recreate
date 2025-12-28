//! Cache backend implementations

pub mod memory;
pub mod moka;
pub mod disk;
pub mod tiered;

use crate::error::CacheResult;
use crate::key::CacheKey;
use crate::value::CacheValue;
use std::fmt::Debug;

/// Trait for cache backends
pub trait CacheBackend: Send + Sync + Debug {
    type Value: Clone + Debug;

    /// Get a value from the cache
    fn get(&self, key: &CacheKey) -> CacheResult<Option<CacheValue<Self::Value>>>;

    /// Insert a value into the cache
    fn insert(&self, key: CacheKey, value: CacheValue<Self::Value>) -> CacheResult<()>;

    /// Remove a value from the cache
    fn remove(&self, key: &CacheKey) -> CacheResult<Option<CacheValue<Self::Value>>>;

    /// Check if a key exists in the cache
    fn contains_key(&self, key: &CacheKey) -> bool;

    /// Clear all entries from the cache
    fn clear(&self) -> CacheResult<()>;

    /// Get the number of entries in the cache
    fn len(&self) -> usize;

    /// Check if the cache is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get all keys in the cache
    fn keys(&self) -> Vec<CacheKey>;

    /// Remove expired entries
    fn evict_expired(&self) -> CacheResult<usize>;

    /// Get cache capacity
    fn capacity(&self) -> usize;
}

/// Async cache backend trait
#[cfg(feature = "async")]
#[async_trait::async_trait]
pub trait AsyncCacheBackend: Send + Sync + Debug {
    type Value: Clone + Send + Debug;

    /// Get a value from the cache asynchronously
    async fn get(&self, key: &CacheKey) -> CacheResult<Option<CacheValue<Self::Value>>>;

    /// Insert a value into the cache asynchronously
    async fn insert(&self, key: CacheKey, value: CacheValue<Self::Value>) -> CacheResult<()>;

    /// Remove a value from the cache asynchronously
    async fn remove(&self, key: &CacheKey) -> CacheResult<Option<CacheValue<Self::Value>>>;

    /// Check if a key exists in the cache
    async fn contains_key(&self, key: &CacheKey) -> bool;

    /// Clear all entries from the cache
    async fn clear(&self) -> CacheResult<()>;

    /// Get the number of entries in the cache
    async fn len(&self) -> usize;

    /// Check if the cache is empty
    async fn is_empty(&self) -> bool {
        self.len().await == 0
    }
}
