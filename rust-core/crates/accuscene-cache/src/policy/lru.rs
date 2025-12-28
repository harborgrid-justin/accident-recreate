//! Least Recently Used (LRU) eviction policy

use crate::policy::EvictionPolicy;
use crate::value::CacheMetadata;

/// Least Recently Used eviction policy
/// Evicts entries that haven't been accessed recently
#[derive(Debug, Clone, Copy)]
pub struct LruPolicy {
    /// Weight for idle time (higher = more important)
    pub idle_weight: f64,
}

impl Default for LruPolicy {
    fn default() -> Self {
        Self { idle_weight: 1.0 }
    }
}

impl LruPolicy {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_weight(idle_weight: f64) -> Self {
        Self { idle_weight }
    }
}

impl EvictionPolicy for LruPolicy {
    fn score(&self, metadata: &CacheMetadata) -> f64 {
        if metadata.pinned {
            return f64::MAX;
        }

        // Lower score = evict first
        // Longer idle time = lower score
        let idle_seconds = metadata.idle_time().num_seconds() as f64;
        let priority_bonus = metadata.priority as f64 * 1000.0;

        // Invert idle time so longer idle = lower score
        let score = -idle_seconds * self.idle_weight + priority_bonus;

        score
    }

    fn name(&self) -> &str {
        "LRU"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_lru_policy() {
        let policy = LruPolicy::new();

        let mut recent = CacheMetadata::new();
        recent.last_accessed = chrono::Utc::now();

        let mut old = CacheMetadata::new();
        old.last_accessed = chrono::Utc::now() - Duration::hours(1);

        // Recent access should have higher score
        assert!(policy.score(&recent) > policy.score(&old));
    }

    #[test]
    fn test_lru_pinned() {
        let policy = LruPolicy::new();

        let mut pinned = CacheMetadata::new();
        pinned.pinned = true;

        // Pinned items have maximum score
        assert_eq!(policy.score(&pinned), f64::MAX);
    }
}
