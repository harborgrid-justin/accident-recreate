//! Time-To-Live (TTL) based eviction policy

use crate::policy::EvictionPolicy;
use crate::value::CacheMetadata;

/// TTL-based eviction policy
/// Evicts entries based on time remaining until expiration
#[derive(Debug, Clone, Copy)]
pub struct TtlPolicy {
    /// Whether to also consider age (time since creation)
    pub use_age: bool,
}

impl Default for TtlPolicy {
    fn default() -> Self {
        Self { use_age: false }
    }
}

impl TtlPolicy {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_age() -> Self {
        Self { use_age: true }
    }
}

impl EvictionPolicy for TtlPolicy {
    fn score(&self, metadata: &CacheMetadata) -> f64 {
        if metadata.pinned {
            return f64::MAX;
        }

        let priority_bonus = metadata.priority as f64 * 1000.0;

        if let Some(time_remaining) = metadata.time_until_expiry() {
            let remaining_seconds = time_remaining.num_seconds() as f64;

            if remaining_seconds <= 0.0 {
                // Already expired, evict immediately
                return f64::MIN;
            }

            // More time remaining = higher score
            let base_score = remaining_seconds + priority_bonus;

            if self.use_age {
                // Also consider age - older items with same TTL get lower score
                let age_seconds = metadata.age().num_seconds() as f64;
                base_score - (age_seconds / 10.0)
            } else {
                base_score
            }
        } else {
            // No TTL set - use age as fallback
            if self.use_age {
                let age_seconds = metadata.age().num_seconds() as f64;
                priority_bonus - age_seconds
            } else {
                // No TTL and not using age - keep indefinitely
                priority_bonus + 1_000_000.0
            }
        }
    }

    fn name(&self) -> &str {
        if self.use_age {
            "TTL-Age"
        } else {
            "TTL"
        }
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
    fn test_ttl_policy() {
        let policy = TtlPolicy::new();

        let mut expiring_soon = CacheMetadata::new();
        expiring_soon.set_ttl(Duration::minutes(5));

        let mut expiring_later = CacheMetadata::new();
        expiring_later.set_ttl(Duration::hours(1));

        // Entry expiring later should have higher score
        assert!(policy.score(&expiring_later) > policy.score(&expiring_soon));
    }

    #[test]
    fn test_ttl_expired() {
        let policy = TtlPolicy::new();

        let mut expired = CacheMetadata::new();
        expired.expires_at = Some(chrono::Utc::now() - Duration::hours(1));

        // Expired entries have minimum score
        assert_eq!(policy.score(&expired), f64::MIN);
    }

    #[test]
    fn test_ttl_no_expiry() {
        let policy = TtlPolicy::new();

        let no_ttl = CacheMetadata::new();

        // No TTL means high score (keep indefinitely)
        assert!(policy.score(&no_ttl) > 1_000_000.0);
    }
}
