//! Bloom filter for quick existence checks.
//!
//! Probabilistic data structure that can test set membership.
//! May return false positives but never false negatives.
//!
//! # Complexity
//! - Insert: O(k) where k is number of hash functions
//! - Query: O(k) where k is number of hash functions
//! - Space: O(m) where m is bit array size

use crate::config::BloomConfig;
use crate::error::Result;
use parking_lot::RwLock;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// Bloom filter for probabilistic set membership testing.
///
/// # False Positives
/// The filter may report an item exists when it doesn't (false positive).
/// The false positive rate is configurable and depends on:
/// - Size of the bit array (m)
/// - Number of hash functions (k)
/// - Number of inserted elements (n)
///
/// Optimal: k = (m/n) * ln(2)
pub struct BloomFilter {
    bits: Arc<RwLock<Vec<u64>>>,
    bit_count: usize,
    hash_count: usize,
    item_count: Arc<RwLock<usize>>,
}

impl BloomFilter {
    /// Create a new Bloom filter with configuration.
    pub fn new(config: BloomConfig) -> Self {
        let bit_count = config.optimal_bit_size();
        let hash_count = config.optimal_hash_count();
        let word_count = (bit_count + 63) / 64;

        Self {
            bits: Arc::new(RwLock::new(vec![0u64; word_count])),
            bit_count,
            hash_count,
            item_count: Arc::new(RwLock::new(0)),
        }
    }

    /// Create with default configuration.
    pub fn default() -> Self {
        Self::new(BloomConfig::default())
    }

    /// Create with specific parameters.
    pub fn with_parameters(expected_elements: usize, false_positive_rate: f64) -> Self {
        Self::new(BloomConfig {
            expected_elements,
            false_positive_rate,
        })
    }

    /// Insert an item into the filter.
    ///
    /// # Complexity
    /// O(k) where k is number of hash functions
    pub fn insert<T: Hash>(&self, item: &T) -> Result<()> {
        let hashes = self.get_hashes(item);
        let mut bits = self.bits.write();

        for hash in hashes {
            let bit_index = (hash % self.bit_count as u64) as usize;
            let word_index = bit_index / 64;
            let bit_offset = bit_index % 64;
            bits[word_index] |= 1u64 << bit_offset;
        }

        *self.item_count.write() += 1;
        Ok(())
    }

    /// Check if an item might be in the set.
    ///
    /// Returns:
    /// - `true`: Item might be in the set (could be false positive)
    /// - `false`: Item is definitely not in the set
    ///
    /// # Complexity
    /// O(k) where k is number of hash functions
    pub fn contains<T: Hash>(&self, item: &T) -> bool {
        let hashes = self.get_hashes(item);
        let bits = self.bits.read();

        for hash in hashes {
            let bit_index = (hash % self.bit_count as u64) as usize;
            let word_index = bit_index / 64;
            let bit_offset = bit_index % 64;

            if bits[word_index] & (1u64 << bit_offset) == 0 {
                return false;
            }
        }

        true
    }

    /// Get multiple hash values for an item using double hashing.
    ///
    /// Uses Kirsch-Mitzenmacher optimization: h_i(x) = h1(x) + i * h2(x)
    fn get_hashes<T: Hash>(&self, item: &T) -> Vec<u64> {
        // Compute two independent hashes
        let h1 = self.hash_with_seed(item, 0);
        let h2 = self.hash_with_seed(item, 1);

        // Generate k hashes using double hashing
        (0..self.hash_count)
            .map(|i| h1.wrapping_add((i as u64).wrapping_mul(h2)))
            .collect()
    }

    fn hash_with_seed<T: Hash>(&self, item: &T, seed: u64) -> u64 {
        let mut hasher = seahash::SeaHasher::with_seeds(seed, seed, seed, seed);
        item.hash(&mut hasher);
        hasher.finish()
    }

    /// Get current false positive probability.
    pub fn false_positive_rate(&self) -> f64 {
        let n = *self.item_count.read() as f64;
        let m = self.bit_count as f64;
        let k = self.hash_count as f64;

        // (1 - e^(-kn/m))^k
        (1.0 - (-k * n / m).exp()).powf(k)
    }

    /// Clear all bits in the filter.
    pub fn clear(&self) {
        self.bits.write().fill(0);
        *self.item_count.write() = 0;
    }

    /// Get number of items inserted.
    pub fn len(&self) -> usize {
        *self.item_count.read()
    }

    /// Check if filter is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get number of bits set.
    pub fn bits_set(&self) -> usize {
        self.bits.read().iter().map(|w| w.count_ones() as usize).sum()
    }

    /// Get fill ratio (bits set / total bits).
    pub fn fill_ratio(&self) -> f64 {
        self.bits_set() as f64 / self.bit_count as f64
    }

    /// Get configuration parameters.
    pub fn config(&self) -> (usize, usize) {
        (self.bit_count, self.hash_count)
    }
}

impl Clone for BloomFilter {
    fn clone(&self) -> Self {
        Self {
            bits: Arc::new(RwLock::new(self.bits.read().clone())),
            bit_count: self.bit_count,
            hash_count: self.hash_count,
            item_count: Arc::new(RwLock::new(*self.item_count.read())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_contains() {
        let bloom = BloomFilter::with_parameters(1000, 0.01);

        bloom.insert(&"hello").unwrap();
        bloom.insert(&"world").unwrap();
        bloom.insert(&123).unwrap();

        assert!(bloom.contains(&"hello"));
        assert!(bloom.contains(&"world"));
        assert!(bloom.contains(&123));
        assert!(!bloom.contains(&"missing"));
    }

    #[test]
    fn test_false_positive_rate() {
        let bloom = BloomFilter::with_parameters(100, 0.01);

        // Insert items
        for i in 0..100 {
            bloom.insert(&i).unwrap();
        }

        // Test for false positives
        let mut false_positives = 0;
        for i in 100..1000 {
            if bloom.contains(&i) {
                false_positives += 1;
            }
        }

        let actual_fpr = false_positives as f64 / 900.0;
        assert!(actual_fpr < 0.05); // Should be close to 1%
    }

    #[test]
    fn test_clear() {
        let bloom = BloomFilter::with_parameters(100, 0.01);

        bloom.insert(&"test").unwrap();
        assert!(bloom.contains(&"test"));

        bloom.clear();
        assert!(!bloom.contains(&"test"));
        assert_eq!(bloom.len(), 0);
    }
}
