//! Least Frequently Used (LFU) eviction policy

use crate::policy::EvictionPolicy;
use crate::value::CacheMetadata;

/// Least Frequently Used eviction policy
/// Evicts entries with lowest access count
#[derive(Debug, Clone, Copy)]
pub struct LfuPolicy {
    /// Weight for access frequency (higher = more important)
    pub frequency_weight: f64,
    /// Whether to consider recency as well
    pub consider_recency: bool,
}

impl Default for LfuPolicy {
    fn default() -> Self {
        Self {
            frequency_weight: 1.0,
            consider_recency: false,
        }
    }
}

impl LfuPolicy {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_weight(frequency_weight: f64) -> Self {
        Self {
            frequency_weight,
            consider_recency: false,
        }
    }

    pub fn with_recency(frequency_weight: f64) -> Self {
        Self {
            frequency_weight,
            consider_recency: true,
        }
    }
}

impl EvictionPolicy for LfuPolicy {
    fn score(&self, metadata: &CacheMetadata) -> f64 {
        if metadata.pinned {
            return f64::MAX;
        }

        let frequency = metadata.access_count as f64;
        let priority_bonus = metadata.priority as f64 * 1000.0;

        let base_score = frequency * self.frequency_weight + priority_bonus;

        if self.consider_recency {
            // Penalize old entries even if frequently accessed
            let idle_seconds = metadata.idle_time().num_seconds() as f64;
            let recency_penalty = idle_seconds / 3600.0; // Hours idle
            base_score - recency_penalty
        } else {
            base_score
        }
    }

    fn name(&self) -> &str {
        if self.consider_recency {
            "LFU-Recency"
        } else {
            "LFU"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lfu_policy() {
        let policy = LfuPolicy::new();

        let mut frequent = CacheMetadata::new();
        frequent.access_count = 100;

        let mut rare = CacheMetadata::new();
        rare.access_count = 5;

        // Frequently accessed should have higher score
        assert!(policy.score(&frequent) > policy.score(&rare));
    }

    #[test]
    fn test_lfu_with_recency() {
        let policy = LfuPolicy::with_recency(1.0);

        let mut old_frequent = CacheMetadata::new();
        old_frequent.access_count = 100;
        old_frequent.last_accessed = chrono::Utc::now() - chrono::Duration::hours(24);

        let mut recent_rare = CacheMetadata::new();
        recent_rare.access_count = 10;
        recent_rare.last_accessed = chrono::Utc::now();

        // With recency, recent items can have higher scores even with lower frequency
        // (depends on the balance)
        let old_score = policy.score(&old_frequent);
        let recent_score = policy.score(&recent_rare);

        // Old item gets penalized for idle time
        assert!(old_score < old_frequent.access_count as f64);
    }
}
