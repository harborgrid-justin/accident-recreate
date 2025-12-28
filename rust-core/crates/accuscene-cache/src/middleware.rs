//! Cache middleware pattern for interception and transformation

use crate::backends::CacheBackend;
use crate::error::CacheResult;
use crate::key::CacheKey;
use crate::value::CacheValue;
use std::sync::Arc;
use tracing::trace;

/// Middleware trait for cache operations
pub trait CacheMiddleware<T: Clone + Send + Sync>: Send + Sync {
    /// Called before a get operation
    fn before_get(&self, key: &CacheKey) -> CacheResult<()> {
        let _ = key;
        Ok(())
    }

    /// Called after a successful get operation
    fn after_get(&self, key: &CacheKey, value: &Option<CacheValue<T>>) -> CacheResult<()> {
        let _ = (key, value);
        Ok(())
    }

    /// Called before an insert operation
    fn before_insert(&self, key: &CacheKey, value: &CacheValue<T>) -> CacheResult<()> {
        let _ = (key, value);
        Ok(())
    }

    /// Called after an insert operation
    fn after_insert(&self, key: &CacheKey, value: &CacheValue<T>) -> CacheResult<()> {
        let _ = (key, value);
        Ok(())
    }

    /// Called before a remove operation
    fn before_remove(&self, key: &CacheKey) -> CacheResult<()> {
        let _ = key;
        Ok(())
    }

    /// Called after a remove operation
    fn after_remove(&self, key: &CacheKey, removed: &Option<CacheValue<T>>) -> CacheResult<()> {
        let _ = (key, removed);
        Ok(())
    }
}

/// Cache with middleware chain
pub struct MiddlewareCache<T: Clone + Send + Sync + 'static> {
    cache: Box<dyn CacheBackend<Value = T>>,
    middleware: Vec<Arc<dyn CacheMiddleware<T>>>,
}

impl<T: Clone + Send + Sync + 'static> MiddlewareCache<T> {
    pub fn new(cache: Box<dyn CacheBackend<Value = T>>) -> Self {
        Self {
            cache,
            middleware: Vec::new(),
        }
    }

    /// Add middleware to the chain
    pub fn use_middleware<M: CacheMiddleware<T> + 'static>(&mut self, middleware: M) {
        self.middleware.push(Arc::new(middleware));
    }

    /// Get value with middleware
    pub fn get(&self, key: &CacheKey) -> CacheResult<Option<CacheValue<T>>> {
        // Before hooks
        for mw in &self.middleware {
            mw.before_get(key)?;
        }

        // Actual operation
        let value = self.cache.get(key)?;

        // After hooks
        for mw in &self.middleware {
            mw.after_get(key, &value)?;
        }

        Ok(value)
    }

    /// Insert value with middleware
    pub fn insert(&self, key: CacheKey, value: CacheValue<T>) -> CacheResult<()> {
        // Before hooks
        for mw in &self.middleware {
            mw.before_insert(&key, &value)?;
        }

        // Actual operation
        self.cache.insert(key.clone(), value.clone())?;

        // After hooks
        for mw in &self.middleware {
            mw.after_insert(&key, &value)?;
        }

        Ok(())
    }

    /// Remove value with middleware
    pub fn remove(&self, key: &CacheKey) -> CacheResult<Option<CacheValue<T>>> {
        // Before hooks
        for mw in &self.middleware {
            mw.before_remove(key)?;
        }

        // Actual operation
        let removed = self.cache.remove(key)?;

        // After hooks
        for mw in &self.middleware {
            mw.after_remove(key, &removed)?;
        }

        Ok(removed)
    }

    /// Access underlying cache
    pub fn inner(&self) -> &dyn CacheBackend<Value = T> {
        &*self.cache
    }
}

/// Logging middleware
pub struct LoggingMiddleware {
    prefix: String,
}

impl LoggingMiddleware {
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
        }
    }
}

impl<T: Clone + Send + Sync> CacheMiddleware<T> for LoggingMiddleware {
    fn before_get(&self, key: &CacheKey) -> CacheResult<()> {
        trace!("{} GET: {}", self.prefix, key);
        Ok(())
    }

    fn after_get(&self, key: &CacheKey, value: &Option<CacheValue<T>>) -> CacheResult<()> {
        if value.is_some() {
            trace!("{} GET HIT: {}", self.prefix, key);
        } else {
            trace!("{} GET MISS: {}", self.prefix, key);
        }
        Ok(())
    }

    fn before_insert(&self, key: &CacheKey, _value: &CacheValue<T>) -> CacheResult<()> {
        trace!("{} INSERT: {}", self.prefix, key);
        Ok(())
    }

    fn before_remove(&self, key: &CacheKey) -> CacheResult<()> {
        trace!("{} REMOVE: {}", self.prefix, key);
        Ok(())
    }
}

/// Metrics middleware
pub struct MetricsMiddleware {
    get_count: Arc<std::sync::atomic::AtomicU64>,
    hit_count: Arc<std::sync::atomic::AtomicU64>,
    miss_count: Arc<std::sync::atomic::AtomicU64>,
    insert_count: Arc<std::sync::atomic::AtomicU64>,
    remove_count: Arc<std::sync::atomic::AtomicU64>,
}

impl MetricsMiddleware {
    pub fn new() -> Self {
        Self {
            get_count: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            hit_count: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            miss_count: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            insert_count: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            remove_count: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    pub fn get_metrics(&self) -> MiddlewareMetrics {
        use std::sync::atomic::Ordering;

        MiddlewareMetrics {
            get_count: self.get_count.load(Ordering::Relaxed),
            hit_count: self.hit_count.load(Ordering::Relaxed),
            miss_count: self.miss_count.load(Ordering::Relaxed),
            insert_count: self.insert_count.load(Ordering::Relaxed),
            remove_count: self.remove_count.load(Ordering::Relaxed),
        }
    }
}

impl Default for MetricsMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone + Send + Sync> CacheMiddleware<T> for MetricsMiddleware {
    fn before_get(&self, _key: &CacheKey) -> CacheResult<()> {
        self.get_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    fn after_get(&self, _key: &CacheKey, value: &Option<CacheValue<T>>) -> CacheResult<()> {
        if value.is_some() {
            self.hit_count
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        } else {
            self.miss_count
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        Ok(())
    }

    fn before_insert(&self, _key: &CacheKey, _value: &CacheValue<T>) -> CacheResult<()> {
        self.insert_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    fn before_remove(&self, _key: &CacheKey) -> CacheResult<()> {
        self.remove_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct MiddlewareMetrics {
    pub get_count: u64,
    pub hit_count: u64,
    pub miss_count: u64,
    pub insert_count: u64,
    pub remove_count: u64,
}

impl MiddlewareMetrics {
    pub fn hit_rate(&self) -> f64 {
        if self.get_count == 0 {
            0.0
        } else {
            self.hit_count as f64 / self.get_count as f64
        }
    }
}

/// Validation middleware
pub struct ValidationMiddleware {
    max_key_length: usize,
}

impl ValidationMiddleware {
    pub fn new(max_key_length: usize) -> Self {
        Self { max_key_length }
    }
}

impl<T: Clone + Send + Sync> CacheMiddleware<T> for ValidationMiddleware {
    fn before_insert(&self, key: &CacheKey, _value: &CacheValue<T>) -> CacheResult<()> {
        let key_str = key.as_string();
        if key_str.len() > self.max_key_length {
            return Err(crate::error::CacheError::InvalidConfig(format!(
                "Key too long: {} (max: {})",
                key_str.len(),
                self.max_key_length
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backends::memory::MemoryCache;

    #[test]
    fn test_logging_middleware() {
        let cache = Box::new(MemoryCache::with_capacity(10));
        let mut middleware_cache: MiddlewareCache<String> = MiddlewareCache::new(cache);

        middleware_cache.use_middleware(LoggingMiddleware::new("TEST"));

        let key = CacheKey::new("test", "key1");
        let value = CacheValue::new("data".to_string());

        middleware_cache.insert(key.clone(), value).unwrap();
        let _ = middleware_cache.get(&key);
    }

    #[test]
    fn test_metrics_middleware() {
        let cache = Box::new(MemoryCache::with_capacity(10));
        let mut middleware_cache: MiddlewareCache<String> = MiddlewareCache::new(cache);

        let metrics = Arc::new(MetricsMiddleware::new());
        let metrics_clone = Arc::clone(&metrics);
        middleware_cache.use_middleware((*metrics_clone).clone());

        let key = CacheKey::new("test", "key1");
        let value = CacheValue::new("data".to_string());

        middleware_cache.insert(key.clone(), value).unwrap();
        let _ = middleware_cache.get(&key);
        let _ = middleware_cache.get(&CacheKey::new("test", "missing"));

        let m = metrics.get_metrics();
        assert_eq!(m.insert_count, 1);
        assert_eq!(m.get_count, 2);
        assert_eq!(m.hit_count, 1);
        assert_eq!(m.miss_count, 1);
    }
}

// Make MetricsMiddleware cloneable for testing
impl Clone for MetricsMiddleware {
    fn clone(&self) -> Self {
        Self {
            get_count: Arc::clone(&self.get_count),
            hit_count: Arc::clone(&self.hit_count),
            miss_count: Arc::clone(&self.miss_count),
            insert_count: Arc::clone(&self.insert_count),
            remove_count: Arc::clone(&self.remove_count),
        }
    }
}
