//! Adaptive Replacement Cache (ARC) policy

use crate::policy::EvictionPolicy;
use crate::value::CacheMetadata;

/// Adaptive eviction policy
/// Combines LRU and LFU with adaptive weighting based on cache behavior
#[derive(Debug, Clone)]
pub struct AdaptivePolicy {
    /// Current weight for recency (LRU component)
    recency_weight: f64,
    /// Current weight for frequency (LFU component)
    frequency_weight: f64,
    /// Learning rate for adaptation (0.0 to 1.0)
    learning_rate: f64,
}

impl Default for AdaptivePolicy {
    fn default() -> Self {
        Self {
            recency_weight: 0.5,
            frequency_weight: 0.5,
            learning_rate: 0.01,
        }
    }
}

impl AdaptivePolicy {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_weights(recency_weight: f64, frequency_weight: f64) -> Self {
        Self {
            recency_weight,
            frequency_weight,
            learning_rate: 0.01,
        }
    }

    pub fn with_learning_rate(mut self, rate: f64) -> Self {
        self.learning_rate = rate.clamp(0.0, 1.0);
        self
    }

    /// Adapt weights based on cache hit/miss pattern
    /// Call this when a cache miss occurs on an evicted LRU item
    pub fn adapt_for_lru_miss(&mut self) {
        // Increase recency weight when LRU items are missed
        self.recency_weight = (self.recency_weight + self.learning_rate).min(1.0);
        self.frequency_weight = 1.0 - self.recency_weight;
    }

    /// Adapt weights based on cache hit/miss pattern
    /// Call this when a cache miss occurs on an evicted LFU item
    pub fn adapt_for_lfu_miss(&mut self) {
        // Increase frequency weight when LFU items are missed
        self.frequency_weight = (self.frequency_weight + self.learning_rate).min(1.0);
        self.recency_weight = 1.0 - self.frequency_weight;
    }

    /// Reset weights to balanced state
    pub fn reset_weights(&mut self) {
        self.recency_weight = 0.5;
        self.frequency_weight = 0.5;
    }

    /// Get current weights
    pub fn weights(&self) -> (f64, f64) {
        (self.recency_weight, self.frequency_weight)
    }
}

impl EvictionPolicy for AdaptivePolicy {
    fn score(&self, metadata: &CacheMetadata) -> f64 {
        if metadata.pinned {
            return f64::MAX;
        }

        // LRU component (recency)
        let idle_seconds = metadata.idle_time().num_seconds() as f64;
        let recency_score = 1.0 / (idle_seconds + 1.0); // Higher for recent access

        // LFU component (frequency)
        let frequency_score = metadata.access_count as f64;

        // TTL component
        let ttl_score = if let Some(time_remaining) = metadata.time_until_expiry() {
            let remaining_seconds = time_remaining.num_seconds() as f64;
            if remaining_seconds <= 0.0 {
                return f64::MIN; // Expired
            }
            remaining_seconds / 3600.0 // Hours remaining
        } else {
            100.0 // No TTL = high score
        };

        // Priority bonus
        let priority_bonus = metadata.priority as f64 * 100.0;

        // Combine scores with adaptive weights
        let base_score = (recency_score * self.recency_weight * 1000.0)
            + (frequency_score * self.frequency_weight)
            + (ttl_score * 10.0)
            + priority_bonus;

        base_score
    }

    fn name(&self) -> &str {
        "Adaptive"
    }

    fn respects_ttl(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_adaptive_policy() {
        let policy = AdaptivePolicy::new();

        let mut recent_frequent = CacheMetadata::new();
        recent_frequent.access_count = 100;
        recent_frequent.last_accessed = chrono::Utc::now();

        let mut old_rare = CacheMetadata::new();
        old_rare.access_count = 1;
        old_rare.last_accessed = chrono::Utc::now() - Duration::hours(1);

        // Recent and frequent should have higher score
        assert!(policy.score(&recent_frequent) > policy.score(&old_rare));
    }

    #[test]
    fn test_adaptive_weight_adjustment() {
        let mut policy = AdaptivePolicy::new();
        assert_eq!(policy.weights(), (0.5, 0.5));

        // Adapt for LRU miss (increase recency weight)
        policy.adapt_for_lru_miss();
        let (recency, frequency) = policy.weights();
        assert!(recency > 0.5);
        assert!(frequency < 0.5);

        // Reset
        policy.reset_weights();
        assert_eq!(policy.weights(), (0.5, 0.5));

        // Adapt for LFU miss (increase frequency weight)
        policy.adapt_for_lfu_miss();
        let (recency, frequency) = policy.weights();
        assert!(recency < 0.5);
        assert!(frequency > 0.5);
    }

    #[test]
    fn test_adaptive_expired() {
        let policy = AdaptivePolicy::new();

        let mut expired = CacheMetadata::new();
        expired.expires_at = Some(chrono::Utc::now() - Duration::hours(1));

        // Expired entries have minimum score
        assert_eq!(policy.score(&expired), f64::MIN);
    }
}
