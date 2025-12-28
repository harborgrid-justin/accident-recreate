//! Cache statistics and hit rate tracking

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    /// Total cache hits
    pub hits: u64,
    /// Total cache misses
    pub misses: u64,
    /// Total insertions
    pub insertions: u64,
    /// Total evictions
    pub evictions: u64,
    /// Total removals (explicit)
    pub removals: u64,
    /// Current cache size
    pub current_size: usize,
    /// Maximum cache size
    pub max_size: usize,
    /// Total bytes stored (approximate)
    pub bytes_stored: usize,
}

impl CacheStats {
    pub fn new(max_size: usize) -> Self {
        Self {
            hits: 0,
            misses: 0,
            insertions: 0,
            evictions: 0,
            removals: 0,
            current_size: 0,
            max_size,
            bytes_stored: 0,
        }
    }

    /// Calculate hit rate (0.0 to 1.0)
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    /// Calculate miss rate (0.0 to 1.0)
    pub fn miss_rate(&self) -> f64 {
        1.0 - self.hit_rate()
    }

    /// Calculate cache utilization (0.0 to 1.0)
    pub fn utilization(&self) -> f64 {
        if self.max_size == 0 {
            0.0
        } else {
            self.current_size as f64 / self.max_size as f64
        }
    }

    /// Calculate eviction rate
    pub fn eviction_rate(&self) -> f64 {
        if self.insertions == 0 {
            0.0
        } else {
            self.evictions as f64 / self.insertions as f64
        }
    }

    /// Get total operations
    pub fn total_operations(&self) -> u64 {
        self.hits + self.misses + self.insertions + self.removals
    }
}

/// Thread-safe statistics tracker
#[derive(Debug, Clone)]
pub struct StatsTracker {
    hits: Arc<AtomicU64>,
    misses: Arc<AtomicU64>,
    insertions: Arc<AtomicU64>,
    evictions: Arc<AtomicU64>,
    removals: Arc<AtomicU64>,
    current_size: Arc<AtomicU64>,
    bytes_stored: Arc<AtomicU64>,
    max_size: usize,
}

impl StatsTracker {
    pub fn new(max_size: usize) -> Self {
        Self {
            hits: Arc::new(AtomicU64::new(0)),
            misses: Arc::new(AtomicU64::new(0)),
            insertions: Arc::new(AtomicU64::new(0)),
            evictions: Arc::new(AtomicU64::new(0)),
            removals: Arc::new(AtomicU64::new(0)),
            current_size: Arc::new(AtomicU64::new(0)),
            bytes_stored: Arc::new(AtomicU64::new(0)),
            max_size,
        }
    }

    pub fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_insertion(&self, size_bytes: usize) {
        self.insertions.fetch_add(1, Ordering::Relaxed);
        self.current_size.fetch_add(1, Ordering::Relaxed);
        self.bytes_stored.fetch_add(size_bytes as u64, Ordering::Relaxed);
    }

    pub fn record_eviction(&self, size_bytes: usize) {
        self.evictions.fetch_add(1, Ordering::Relaxed);
        self.current_size.fetch_sub(1, Ordering::Relaxed);
        self.bytes_stored.fetch_sub(size_bytes as u64, Ordering::Relaxed);
    }

    pub fn record_removal(&self, size_bytes: usize) {
        self.removals.fetch_add(1, Ordering::Relaxed);
        self.current_size.fetch_sub(1, Ordering::Relaxed);
        self.bytes_stored.fetch_sub(size_bytes as u64, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> CacheStats {
        CacheStats {
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            insertions: self.insertions.load(Ordering::Relaxed),
            evictions: self.evictions.load(Ordering::Relaxed),
            removals: self.removals.load(Ordering::Relaxed),
            current_size: self.current_size.load(Ordering::Relaxed) as usize,
            max_size: self.max_size,
            bytes_stored: self.bytes_stored.load(Ordering::Relaxed) as usize,
        }
    }

    pub fn reset(&self) {
        self.hits.store(0, Ordering::Relaxed);
        self.misses.store(0, Ordering::Relaxed);
        self.insertions.store(0, Ordering::Relaxed);
        self.evictions.store(0, Ordering::Relaxed);
        self.removals.store(0, Ordering::Relaxed);
        // Don't reset current_size and bytes_stored - they reflect actual state
    }
}

/// Statistics aggregator for multiple caches
#[derive(Debug)]
pub struct StatsAggregator {
    trackers: Arc<RwLock<Vec<StatsTracker>>>,
}

impl StatsAggregator {
    pub fn new() -> Self {
        Self {
            trackers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn add_tracker(&self, tracker: StatsTracker) {
        let mut trackers = self.trackers.write();
        trackers.push(tracker);
    }

    pub fn aggregate(&self) -> CacheStats {
        let trackers = self.trackers.read();
        let mut total = CacheStats::new(0);

        for tracker in trackers.iter() {
            let stats = tracker.snapshot();
            total.hits += stats.hits;
            total.misses += stats.misses;
            total.insertions += stats.insertions;
            total.evictions += stats.evictions;
            total.removals += stats.removals;
            total.current_size += stats.current_size;
            total.max_size += stats.max_size;
            total.bytes_stored += stats.bytes_stored;
        }

        total
    }
}

impl Default for StatsAggregator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_tracker() {
        let tracker = StatsTracker::new(100);

        tracker.record_hit();
        tracker.record_hit();
        tracker.record_miss();
        tracker.record_insertion(1024);

        let stats = tracker.snapshot();
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.insertions, 1);
        assert_eq!(stats.current_size, 1);
        assert_eq!(stats.bytes_stored, 1024);
    }

    #[test]
    fn test_hit_rate() {
        let mut stats = CacheStats::new(100);
        stats.hits = 80;
        stats.misses = 20;

        assert_eq!(stats.hit_rate(), 0.8);
        assert_eq!(stats.miss_rate(), 0.2);
    }

    #[test]
    fn test_utilization() {
        let mut stats = CacheStats::new(100);
        stats.current_size = 75;

        assert_eq!(stats.utilization(), 0.75);
    }

    #[test]
    fn test_stats_aggregator() {
        let aggregator = StatsAggregator::new();

        let tracker1 = StatsTracker::new(100);
        tracker1.record_hit();
        tracker1.record_hit();

        let tracker2 = StatsTracker::new(100);
        tracker2.record_hit();
        tracker2.record_miss();

        aggregator.add_tracker(tracker1);
        aggregator.add_tracker(tracker2);

        let total = aggregator.aggregate();
        assert_eq!(total.hits, 3);
        assert_eq!(total.misses, 1);
    }
}
