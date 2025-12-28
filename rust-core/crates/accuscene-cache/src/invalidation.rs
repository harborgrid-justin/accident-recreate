//! Cache invalidation strategies

use crate::backends::CacheBackend;
use crate::error::CacheResult;
use crate::key::CacheKey;
use std::collections::HashSet;
use tracing::debug;

/// Cache invalidation strategy
pub trait InvalidationStrategy: Send + Sync {
    /// Determine if a cache entry should be invalidated
    fn should_invalidate(&self, key: &CacheKey) -> bool;

    /// Strategy name
    fn name(&self) -> &str;
}

/// Invalidate by namespace
#[derive(Debug, Clone)]
pub struct NamespaceInvalidation {
    pub namespaces: HashSet<String>,
}

impl NamespaceInvalidation {
    pub fn new(namespaces: Vec<String>) -> Self {
        Self {
            namespaces: namespaces.into_iter().collect(),
        }
    }

    pub fn single(namespace: String) -> Self {
        let mut set = HashSet::new();
        set.insert(namespace);
        Self { namespaces: set }
    }
}

impl InvalidationStrategy for NamespaceInvalidation {
    fn should_invalidate(&self, key: &CacheKey) -> bool {
        self.namespaces.contains(&key.namespace)
    }

    fn name(&self) -> &str {
        "namespace"
    }
}

/// Invalidate by key pattern
#[derive(Debug, Clone)]
pub struct PatternInvalidation {
    pub pattern: String,
}

impl PatternInvalidation {
    pub fn new(pattern: String) -> Self {
        Self { pattern }
    }
}

impl InvalidationStrategy for PatternInvalidation {
    fn should_invalidate(&self, key: &CacheKey) -> bool {
        let key_str = key.as_string();
        key_str.contains(&self.pattern)
    }

    fn name(&self) -> &str {
        "pattern"
    }
}

/// Invalidate by version
#[derive(Debug, Clone)]
pub struct VersionInvalidation {
    pub min_version: u64,
}

impl VersionInvalidation {
    pub fn new(min_version: u64) -> Self {
        Self { min_version }
    }

    pub fn all() -> Self {
        Self { min_version: 0 }
    }
}

impl InvalidationStrategy for VersionInvalidation {
    fn should_invalidate(&self, key: &CacheKey) -> bool {
        if let Some(version) = key.version {
            version < self.min_version
        } else {
            false
        }
    }

    fn name(&self) -> &str {
        "version"
    }
}

/// Invalidation manager
#[derive(Debug)]
pub struct InvalidationManager<T: Clone + Send + Sync> {
    cache: Box<dyn CacheBackend<Value = T>>,
}

impl<T: Clone + Send + Sync + 'static> InvalidationManager<T> {
    pub fn new(cache: Box<dyn CacheBackend<Value = T>>) -> Self {
        Self { cache }
    }

    /// Invalidate entries matching a strategy
    pub fn invalidate<S: InvalidationStrategy>(&self, strategy: &S) -> CacheResult<usize> {
        let keys = self.cache.keys();
        let mut invalidated = 0;

        for key in keys {
            if strategy.should_invalidate(&key) {
                self.cache.remove(&key)?;
                invalidated += 1;
            }
        }

        debug!(
            "Invalidated {} entries using {} strategy",
            invalidated,
            strategy.name()
        );

        Ok(invalidated)
    }

    /// Invalidate by namespace
    pub fn invalidate_namespace(&self, namespace: &str) -> CacheResult<usize> {
        let strategy = NamespaceInvalidation::single(namespace.to_string());
        self.invalidate(&strategy)
    }

    /// Invalidate by pattern
    pub fn invalidate_pattern(&self, pattern: &str) -> CacheResult<usize> {
        let strategy = PatternInvalidation::new(pattern.to_string());
        self.invalidate(&strategy)
    }

    /// Invalidate expired entries
    pub fn invalidate_expired(&self) -> CacheResult<usize> {
        self.cache.evict_expired()
    }

    /// Invalidate all entries
    pub fn invalidate_all(&self) -> CacheResult<()> {
        self.cache.clear()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backends::memory::MemoryCache;
    use crate::value::CacheValue;

    #[test]
    fn test_namespace_invalidation() {
        let cache = Box::new(MemoryCache::with_capacity(10));
        let manager = InvalidationManager::new(cache);

        // Insert entries in different namespaces
        let _ = manager.cache.insert(
            CacheKey::new("physics", "key1"),
            CacheValue::new("data1"),
        );
        let _ = manager.cache.insert(
            CacheKey::new("physics", "key2"),
            CacheValue::new("data2"),
        );
        let _ = manager.cache.insert(
            CacheKey::new("query", "key3"),
            CacheValue::new("data3"),
        );

        // Invalidate physics namespace
        let count = manager.invalidate_namespace("physics").unwrap();
        assert_eq!(count, 2);

        // Query namespace should still exist
        assert!(manager.cache.contains_key(&CacheKey::new("query", "key3")));
    }

    #[test]
    fn test_pattern_invalidation() {
        let cache = Box::new(MemoryCache::with_capacity(10));
        let manager = InvalidationManager::new(cache);

        let _ = manager.cache.insert(
            CacheKey::new("test", "user_123"),
            CacheValue::new("data1"),
        );
        let _ = manager.cache.insert(
            CacheKey::new("test", "user_456"),
            CacheValue::new("data2"),
        );
        let _ = manager.cache.insert(
            CacheKey::new("test", "session_789"),
            CacheValue::new("data3"),
        );

        // Invalidate all user_ entries
        let count = manager.invalidate_pattern("user_").unwrap();
        assert_eq!(count, 2);

        // Session should still exist
        assert!(manager.cache.contains_key(&CacheKey::new("test", "session_789")));
    }
}
