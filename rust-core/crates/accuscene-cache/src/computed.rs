//! Computed and memoized cache values

use crate::backends::CacheBackend;
use crate::error::CacheResult;
use crate::key::CacheKey;
use crate::value::CacheValue;
use std::sync::Arc;
use tracing::{debug, trace};

/// Trait for computing cache values
pub trait ComputeFn<T>: Send + Sync {
    /// Compute the value for a given key
    fn compute(&self, key: &CacheKey) -> CacheResult<T>;
}

/// Function-based compute implementation
impl<T, F> ComputeFn<T> for F
where
    F: Fn(&CacheKey) -> CacheResult<T> + Send + Sync,
{
    fn compute(&self, key: &CacheKey) -> CacheResult<T> {
        self(key)
    }
}

/// Cache with automatic computation of missing values
pub struct ComputedCache<T: Clone + Send + Sync + 'static> {
    cache: Box<dyn CacheBackend<Value = T>>,
    compute_fn: Arc<dyn ComputeFn<T>>,
}

impl<T: Clone + Send + Sync + std::fmt::Debug + 'static> std::fmt::Debug for ComputedCache<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComputedCache")
            .field("cache", &self.cache)
            .field("compute_fn", &"<function>")
            .finish()
    }
}

impl<T: Clone + Send + Sync + 'static> ComputedCache<T> {
    pub fn new<C: ComputeFn<T> + 'static>(
        cache: Box<dyn CacheBackend<Value = T>>,
        compute_fn: C,
    ) -> Self {
        Self {
            cache,
            compute_fn: Arc::new(compute_fn),
        }
    }

    /// Get value, computing if not in cache
    pub fn get_or_compute(&self, key: &CacheKey) -> CacheResult<T> {
        // Try cache first
        if let Some(cached) = self.cache.get(key)? {
            trace!("Cache hit for computed value: {}", key);
            return Ok(cached.data);
        }

        // Compute and cache
        debug!("Computing value for: {}", key);
        let value = self.compute_fn.compute(key)?;
        let cache_value = CacheValue::new(value.clone());
        self.cache.insert(key.clone(), cache_value)?;

        Ok(value)
    }

    /// Get value with custom TTL if computed
    pub fn get_or_compute_with_ttl(
        &self,
        key: &CacheKey,
        ttl: chrono::Duration,
    ) -> CacheResult<T> {
        // Try cache first
        if let Some(cached) = self.cache.get(key)? {
            trace!("Cache hit for computed value: {}", key);
            return Ok(cached.data);
        }

        // Compute and cache with TTL
        debug!("Computing value with TTL for: {}", key);
        let value = self.compute_fn.compute(key)?;
        let cache_value = CacheValue::with_ttl(value.clone(), ttl);
        self.cache.insert(key.clone(), cache_value)?;

        Ok(value)
    }

    /// Invalidate cached value (will be recomputed on next access)
    pub fn invalidate(&self, key: &CacheKey) -> CacheResult<()> {
        self.cache.remove(key)?;
        debug!("Invalidated computed value: {}", key);
        Ok(())
    }

    /// Pre-compute and cache a value
    pub fn precompute(&self, key: &CacheKey) -> CacheResult<()> {
        let _ = self.get_or_compute(key)?;
        Ok(())
    }

    /// Access underlying cache
    pub fn cache(&self) -> &dyn CacheBackend<Value = T> {
        &*self.cache
    }
}

/// Memoization helper for expensive functions
pub struct Memoizer<K, V>
where
    K: Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    cache: Box<dyn CacheBackend<Value = V>>,
    compute_fn: Arc<dyn Fn(&K) -> CacheResult<V> + Send + Sync>,
    key_builder: Arc<dyn Fn(&K) -> CacheKey + Send + Sync>,
}

impl<K, V> Memoizer<K, V>
where
    K: Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    pub fn new<F, KB>(cache: Box<dyn CacheBackend<Value = V>>, compute_fn: F, key_builder: KB) -> Self
    where
        F: Fn(&K) -> CacheResult<V> + Send + Sync + 'static,
        KB: Fn(&K) -> CacheKey + Send + Sync + 'static,
    {
        Self {
            cache,
            compute_fn: Arc::new(compute_fn),
            key_builder: Arc::new(key_builder),
        }
    }

    /// Get memoized value
    pub fn get(&self, input: &K) -> CacheResult<V> {
        let key = (self.key_builder)(input);

        // Try cache
        if let Some(cached) = self.cache.get(&key)? {
            trace!("Memoization cache hit");
            return Ok(cached.data);
        }

        // Compute and cache
        debug!("Computing memoized value");
        let value = (self.compute_fn)(input)?;
        let cache_value = CacheValue::new(value.clone());
        self.cache.insert(key, cache_value)?;

        Ok(value)
    }

    /// Clear memoization cache
    pub fn clear(&self) -> CacheResult<()> {
        self.cache.clear()
    }
}

/// Builder for memoized functions
pub struct MemoizerBuilder<K, V>
where
    K: Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    cache: Option<Box<dyn CacheBackend<Value = V>>>,
    ttl: Option<chrono::Duration>,
    namespace: String,
}

impl<K, V> MemoizerBuilder<K, V>
where
    K: Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    pub fn new(namespace: impl Into<String>) -> Self {
        Self {
            cache: None,
            ttl: None,
            namespace: namespace.into(),
        }
    }

    pub fn cache(mut self, cache: Box<dyn CacheBackend<Value = V>>) -> Self {
        self.cache = Some(cache);
        self
    }

    pub fn ttl(mut self, ttl: chrono::Duration) -> Self {
        self.ttl = Some(ttl);
        self
    }

    pub fn build<F, KB>(self, compute_fn: F, key_builder: KB) -> Memoizer<K, V>
    where
        F: Fn(&K) -> CacheResult<V> + Send + Sync + 'static,
        KB: Fn(&K) -> CacheKey + Send + Sync + 'static,
    {
        let cache = self.cache.expect("Cache backend required");
        Memoizer::new(cache, compute_fn, key_builder)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backends::memory::MemoryCache;
    use std::sync::atomic::{AtomicU64, Ordering};

    #[test]
    fn test_computed_cache() {
        let cache = Box::new(MemoryCache::with_capacity(10));
        let compute_count = Arc::new(AtomicU64::new(0));
        let compute_count_clone = compute_count.clone();

        let compute_fn = move |key: &CacheKey| -> CacheResult<String> {
            compute_count_clone.fetch_add(1, Ordering::SeqCst);
            Ok(format!("computed_{}", key.identifier))
        };

        let computed_cache = ComputedCache::new(cache, compute_fn);

        let key = CacheKey::new("test", "key1");

        // First access - should compute
        let value1 = computed_cache.get_or_compute(&key).unwrap();
        assert_eq!(value1, "computed_key1");
        assert_eq!(compute_count.load(Ordering::SeqCst), 1);

        // Second access - should use cache
        let value2 = computed_cache.get_or_compute(&key).unwrap();
        assert_eq!(value2, "computed_key1");
        assert_eq!(compute_count.load(Ordering::SeqCst), 1); // Not incremented
    }

    #[test]
    fn test_memoizer() {
        let cache = Box::new(MemoryCache::with_capacity(10));
        let compute_count = Arc::new(AtomicU64::new(0));
        let compute_count_clone = compute_count.clone();

        let compute_fn = move |input: &u64| -> CacheResult<u64> {
            compute_count_clone.fetch_add(1, Ordering::SeqCst);
            Ok(input * input)
        };

        let key_builder = |input: &u64| CacheKey::new("memoize", input.to_string());

        let memoizer = Memoizer::new(cache, compute_fn, key_builder);

        // First call - computes
        let result1 = memoizer.get(&5).unwrap();
        assert_eq!(result1, 25);
        assert_eq!(compute_count.load(Ordering::SeqCst), 1);

        // Second call - memoized
        let result2 = memoizer.get(&5).unwrap();
        assert_eq!(result2, 25);
        assert_eq!(compute_count.load(Ordering::SeqCst), 1);

        // Different input - computes
        let result3 = memoizer.get(&10).unwrap();
        assert_eq!(result3, 100);
        assert_eq!(compute_count.load(Ordering::SeqCst), 2);
    }
}
