//! Cuckoo filter for space-efficient lookups.
//!
//! Probabilistic data structure similar to Bloom filter but supports deletions.
//! Uses cuckoo hashing with fingerprints for compact storage.
//!
//! # Complexity
//! - Insert: O(1) average, O(max_kicks) worst case
//! - Query: O(1)
//! - Delete: O(1)
//! - Space: More compact than Bloom filter for same false positive rate

use crate::config::CuckooConfig;
use crate::error::{AlgorithmError, Result};
use parking_lot::RwLock;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// Cuckoo filter bucket.
#[derive(Clone)]
struct Bucket {
    fingerprints: Vec<Option<u16>>,
}

impl Bucket {
    fn new(size: usize) -> Self {
        Self {
            fingerprints: vec![None; size],
        }
    }

    fn is_full(&self) -> bool {
        self.fingerprints.iter().all(|f| f.is_some())
    }

    fn insert(&mut self, fingerprint: u16) -> bool {
        for slot in &mut self.fingerprints {
            if slot.is_none() {
                *slot = Some(fingerprint);
                return true;
            }
        }
        false
    }

    fn remove(&mut self, fingerprint: u16) -> bool {
        for slot in &mut self.fingerprints {
            if *slot == Some(fingerprint) {
                *slot = None;
                return true;
            }
        }
        false
    }

    fn contains(&self, fingerprint: u16) -> bool {
        self.fingerprints.iter().any(|&f| f == Some(fingerprint))
    }

    fn get_random_fingerprint(&self) -> Option<u16> {
        self.fingerprints
            .iter()
            .filter_map(|&f| f)
            .next()
    }
}

/// Cuckoo filter for probabilistic set membership testing with deletions.
///
/// Unlike Bloom filters, Cuckoo filters support deletion and are more
/// space-efficient for target false positive rates below 3%.
pub struct CuckooFilter {
    buckets: Arc<RwLock<Vec<Bucket>>>,
    config: CuckooConfig,
    item_count: Arc<RwLock<usize>>,
}

impl CuckooFilter {
    /// Create a new Cuckoo filter.
    pub fn new(config: CuckooConfig) -> Self {
        let bucket_count = (config.capacity + config.bucket_size - 1) / config.bucket_size;
        let buckets = vec![Bucket::new(config.bucket_size); bucket_count];

        Self {
            buckets: Arc::new(RwLock::new(buckets)),
            config,
            item_count: Arc::new(RwLock::new(0)),
        }
    }

    /// Create with default configuration.
    pub fn default() -> Self {
        Self::new(CuckooConfig::default())
    }

    /// Create with specific capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self::new(CuckooConfig {
            capacity,
            ..Default::default()
        })
    }

    /// Insert an item into the filter.
    ///
    /// # Complexity
    /// O(1) average, O(max_kicks) worst case
    ///
    /// # Errors
    /// Returns error if filter is full and cannot relocate items.
    pub fn insert<T: Hash>(&self, item: &T) -> Result<()> {
        let fingerprint = self.fingerprint(item);
        let i1 = self.hash1(item);
        let i2 = self.hash2(i1, fingerprint);

        let mut buckets = self.buckets.write();

        // Try primary bucket
        if buckets[i1].insert(fingerprint) {
            *self.item_count.write() += 1;
            return Ok(());
        }

        // Try secondary bucket
        if buckets[i2].insert(fingerprint) {
            *self.item_count.write() += 1;
            return Ok(());
        }

        // Cuckoo eviction
        let mut current_index = i2;
        let mut current_fp = fingerprint;

        for _ in 0..self.config.max_kicks {
            // Evict random entry from current bucket
            if let Some(evicted_fp) = buckets[current_index].get_random_fingerprint() {
                buckets[current_index].remove(evicted_fp);
                buckets[current_index].insert(current_fp);

                // Find alternative bucket for evicted entry
                let alt_index = self.hash2(current_index, evicted_fp);

                if buckets[alt_index].insert(evicted_fp) {
                    *self.item_count.write() += 1;
                    return Ok(());
                }

                // Continue with evicted entry
                current_index = alt_index;
                current_fp = evicted_fp;
            }
        }

        Err(AlgorithmError::StorageError(
            "Cuckoo filter is full".to_string(),
        ))
    }

    /// Check if an item might be in the set.
    ///
    /// # Complexity
    /// O(1)
    pub fn contains<T: Hash>(&self, item: &T) -> bool {
        let fingerprint = self.fingerprint(item);
        let i1 = self.hash1(item);
        let i2 = self.hash2(i1, fingerprint);

        let buckets = self.buckets.read();
        buckets[i1].contains(fingerprint) || buckets[i2].contains(fingerprint)
    }

    /// Remove an item from the filter.
    ///
    /// # Complexity
    /// O(1)
    ///
    /// Returns `true` if item was found and removed.
    pub fn remove<T: Hash>(&self, item: &T) -> bool {
        let fingerprint = self.fingerprint(item);
        let i1 = self.hash1(item);
        let i2 = self.hash2(i1, fingerprint);

        let mut buckets = self.buckets.write();

        if buckets[i1].remove(fingerprint) {
            *self.item_count.write() -= 1;
            return true;
        }

        if buckets[i2].remove(fingerprint) {
            *self.item_count.write() -= 1;
            return true;
        }

        false
    }

    /// Generate fingerprint for an item.
    fn fingerprint<T: Hash>(&self, item: &T) -> u16 {
        let mut hasher = seahash::SeaHasher::new();
        item.hash(&mut hasher);
        let hash = hasher.finish();

        // Ensure fingerprint is non-zero
        let fp = (hash & 0xFFFF) as u16;
        if fp == 0 {
            1
        } else {
            fp
        }
    }

    /// Primary hash function.
    fn hash1<T: Hash>(&self, item: &T) -> usize {
        let mut hasher = seahash::SeaHasher::with_seeds(1, 2, 3, 4);
        item.hash(&mut hasher);
        (hasher.finish() as usize) % self.buckets.read().len()
    }

    /// Secondary hash function using partial-key cuckoo hashing.
    fn hash2(&self, i1: usize, fingerprint: u16) -> usize {
        let bucket_count = self.buckets.read().len();
        (i1 ^ self.hash_fingerprint(fingerprint)) % bucket_count
    }

    fn hash_fingerprint(&self, fingerprint: u16) -> usize {
        let mut hasher = seahash::SeaHasher::with_seeds(5, 6, 7, 8);
        fingerprint.hash(&mut hasher);
        hasher.finish() as usize
    }

    /// Get load factor (filled slots / total slots).
    pub fn load_factor(&self) -> f64 {
        let buckets = self.buckets.read();
        let filled: usize = buckets
            .iter()
            .map(|b| b.fingerprints.iter().filter(|f| f.is_some()).count())
            .sum();
        let total = buckets.len() * self.config.bucket_size;
        filled as f64 / total as f64
    }

    /// Clear the filter.
    pub fn clear(&self) {
        let mut buckets = self.buckets.write();
        for bucket in buckets.iter_mut() {
            *bucket = Bucket::new(self.config.bucket_size);
        }
        *self.item_count.write() = 0;
    }

    /// Get number of items in the filter.
    pub fn len(&self) -> usize {
        *self.item_count.read()
    }

    /// Check if filter is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get capacity.
    pub fn capacity(&self) -> usize {
        self.buckets.read().len() * self.config.bucket_size
    }
}

impl Clone for CuckooFilter {
    fn clone(&self) -> Self {
        Self {
            buckets: Arc::new(RwLock::new(self.buckets.read().clone())),
            config: self.config.clone(),
            item_count: Arc::new(RwLock::new(*self.item_count.read())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_contains() {
        let filter = CuckooFilter::with_capacity(1000);

        filter.insert(&"hello").unwrap();
        filter.insert(&"world").unwrap();
        filter.insert(&123).unwrap();

        assert!(filter.contains(&"hello"));
        assert!(filter.contains(&"world"));
        assert!(filter.contains(&123));
        assert!(!filter.contains(&"missing"));
    }

    #[test]
    fn test_remove() {
        let filter = CuckooFilter::with_capacity(100);

        filter.insert(&"test").unwrap();
        assert!(filter.contains(&"test"));

        assert!(filter.remove(&"test"));
        assert!(!filter.contains(&"test"));
        assert!(!filter.remove(&"test"));
    }

    #[test]
    fn test_many_items() {
        let filter = CuckooFilter::with_capacity(1000);

        for i in 0..500 {
            filter.insert(&i).unwrap();
        }

        for i in 0..500 {
            assert!(filter.contains(&i));
        }

        assert!(filter.load_factor() > 0.0);
    }

    #[test]
    fn test_clear() {
        let filter = CuckooFilter::with_capacity(100);

        filter.insert(&"test").unwrap();
        assert!(filter.contains(&"test"));

        filter.clear();
        assert!(!filter.contains(&"test"));
        assert_eq!(filter.len(), 0);
    }
}
