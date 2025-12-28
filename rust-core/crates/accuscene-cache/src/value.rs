//! Cache value wrapper with metadata

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Wrapper for cached values with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheValue<T> {
    /// The actual cached data
    pub data: T,

    /// Metadata about the cached value
    pub metadata: CacheMetadata,
}

impl<T> CacheValue<T> {
    /// Create a new cache value
    pub fn new(data: T) -> Self {
        Self {
            data,
            metadata: CacheMetadata::new(),
        }
    }

    /// Create a cache value with TTL
    pub fn with_ttl(data: T, ttl: Duration) -> Self {
        let mut metadata = CacheMetadata::new();
        metadata.set_ttl(ttl);
        Self { data, metadata }
    }

    /// Create a cache value with tags
    pub fn with_tags(data: T, tags: Vec<String>) -> Self {
        let mut metadata = CacheMetadata::new();
        metadata.tags = tags;
        Self { data, metadata }
    }

    /// Check if value has expired
    pub fn is_expired(&self) -> bool {
        self.metadata.is_expired()
    }

    /// Get time until expiration
    pub fn time_until_expiry(&self) -> Option<Duration> {
        self.metadata.time_until_expiry()
    }

    /// Increment access count
    pub fn record_access(&mut self) {
        self.metadata.record_access();
    }

    /// Get the size estimate of this value in bytes
    pub fn size_estimate(&self) -> usize {
        self.metadata.size_bytes
    }

    /// Check if value has a specific tag
    pub fn has_tag(&self, tag: &str) -> bool {
        self.metadata.tags.iter().any(|t| t == tag)
    }
}

/// Metadata about cached values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetadata {
    /// When the value was created
    pub created_at: DateTime<Utc>,

    /// When the value was last accessed
    pub last_accessed: DateTime<Utc>,

    /// When the value expires (if TTL is set)
    pub expires_at: Option<DateTime<Utc>>,

    /// Number of times this value has been accessed
    pub access_count: u64,

    /// Estimated size in bytes
    pub size_bytes: usize,

    /// Priority for eviction (higher = keep longer)
    pub priority: u8,

    /// Tags for group invalidation
    pub tags: Vec<String>,

    /// Version number for cache busting
    pub version: u64,

    /// Whether this value is pinned (never evict)
    pub pinned: bool,
}

impl CacheMetadata {
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            created_at: now,
            last_accessed: now,
            expires_at: None,
            access_count: 0,
            size_bytes: 0,
            priority: 0,
            tags: Vec::new(),
            version: 1,
            pinned: false,
        }
    }

    /// Set TTL for this value
    pub fn set_ttl(&mut self, ttl: Duration) {
        self.expires_at = Some(Utc::now() + ttl);
    }

    /// Check if value has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    /// Get time until expiration
    pub fn time_until_expiry(&self) -> Option<Duration> {
        self.expires_at.map(|expires_at| {
            let now = Utc::now();
            if expires_at > now {
                expires_at - now
            } else {
                Duration::zero()
            }
        })
    }

    /// Record an access to this value
    pub fn record_access(&mut self) {
        self.last_accessed = Utc::now();
        self.access_count += 1;
    }

    /// Get age of the value
    pub fn age(&self) -> Duration {
        Utc::now() - self.created_at
    }

    /// Get time since last access
    pub fn idle_time(&self) -> Duration {
        Utc::now() - self.last_accessed
    }

    /// Calculate eviction score (lower = evict first)
    /// Based on LRU, LFU, and priority
    pub fn eviction_score(&self) -> f64 {
        let idle_seconds = self.idle_time().num_seconds() as f64;
        let access_frequency = self.access_count as f64;
        let priority_bonus = self.priority as f64 * 1000.0;

        // Higher access count and priority = higher score (keep longer)
        // Higher idle time = lower score (evict sooner)
        let score = (access_frequency + priority_bonus) / (idle_seconds + 1.0);

        if self.pinned {
            f64::MAX
        } else {
            score
        }
    }
}

impl Default for CacheMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe access counter
#[derive(Debug)]
pub struct AccessCounter {
    count: Arc<AtomicU64>,
}

impl AccessCounter {
    pub fn new() -> Self {
        Self {
            count: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn increment(&self) -> u64 {
        self.count.fetch_add(1, Ordering::SeqCst)
    }

    pub fn get(&self) -> u64 {
        self.count.load(Ordering::SeqCst)
    }

    pub fn reset(&self) {
        self.count.store(0, Ordering::SeqCst);
    }
}

impl Clone for AccessCounter {
    fn clone(&self) -> Self {
        Self {
            count: Arc::clone(&self.count),
        }
    }
}

impl Default for AccessCounter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_value_creation() {
        let value = CacheValue::new("test data");
        assert_eq!(value.data, "test data");
        assert!(!value.is_expired());
    }

    #[test]
    fn test_cache_value_with_ttl() {
        let value = CacheValue::with_ttl("data", Duration::seconds(60));
        assert!(!value.is_expired());
        assert!(value.time_until_expiry().is_some());
    }

    #[test]
    fn test_metadata_access_tracking() {
        let mut metadata = CacheMetadata::new();
        assert_eq!(metadata.access_count, 0);

        metadata.record_access();
        assert_eq!(metadata.access_count, 1);

        metadata.record_access();
        assert_eq!(metadata.access_count, 2);
    }

    #[test]
    fn test_eviction_score() {
        let mut metadata = CacheMetadata::new();
        metadata.access_count = 10;
        metadata.priority = 5;

        let score = metadata.eviction_score();
        assert!(score > 0.0);

        // Pinned items have maximum score
        metadata.pinned = true;
        assert_eq!(metadata.eviction_score(), f64::MAX);
    }

    #[test]
    fn test_access_counter() {
        let counter = AccessCounter::new();
        assert_eq!(counter.get(), 0);

        counter.increment();
        assert_eq!(counter.get(), 1);

        counter.reset();
        assert_eq!(counter.get(), 0);
    }
}
