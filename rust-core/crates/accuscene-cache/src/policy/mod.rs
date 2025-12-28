//! Cache eviction policies

pub mod lru;
pub mod lfu;
pub mod ttl;
pub mod adaptive;

use crate::value::CacheMetadata;

/// Trait for eviction policies
pub trait EvictionPolicy: Send + Sync {
    /// Calculate eviction score for a cache entry
    /// Lower score = evict first
    fn score(&self, metadata: &CacheMetadata) -> f64;

    /// Policy name
    fn name(&self) -> &str;

    /// Whether this policy respects TTL
    fn respects_ttl(&self) -> bool {
        true
    }
}

/// Eviction decision
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EvictionDecision {
    /// Keep the entry
    Keep,
    /// Evict the entry
    Evict,
    /// Evict if capacity exceeded
    EvictIfNeeded,
}

/// Compare two cache entries for eviction priority
pub fn compare_for_eviction<P: EvictionPolicy>(
    policy: &P,
    meta1: &CacheMetadata,
    meta2: &CacheMetadata,
) -> std::cmp::Ordering {
    let score1 = policy.score(meta1);
    let score2 = policy.score(meta2);

    // Lower score evicts first
    score1.partial_cmp(&score2).unwrap_or(std::cmp::Ordering::Equal)
}

/// Select entries to evict based on policy
pub fn select_eviction_candidates<P: EvictionPolicy>(
    policy: &P,
    entries: &[(String, CacheMetadata)],
    count: usize,
) -> Vec<String> {
    let mut sorted_entries: Vec<_> = entries
        .iter()
        .map(|(key, meta)| (key.clone(), policy.score(meta)))
        .collect();

    // Sort by score (ascending - lowest scores evicted first)
    sorted_entries.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    sorted_entries
        .into_iter()
        .take(count)
        .filter(|(_, score)| !score.is_infinite()) // Don't evict pinned items
        .map(|(key, _)| key)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::CacheMetadata;

    struct TestPolicy;

    impl EvictionPolicy for TestPolicy {
        fn score(&self, metadata: &CacheMetadata) -> f64 {
            metadata.access_count as f64
        }

        fn name(&self) -> &str {
            "test"
        }
    }

    #[test]
    fn test_select_eviction_candidates() {
        let policy = TestPolicy;

        let mut meta1 = CacheMetadata::new();
        meta1.access_count = 1;

        let mut meta2 = CacheMetadata::new();
        meta2.access_count = 10;

        let mut meta3 = CacheMetadata::new();
        meta3.access_count = 5;

        let entries = vec![
            ("key1".to_string(), meta1),
            ("key2".to_string(), meta2),
            ("key3".to_string(), meta3),
        ];

        let candidates = select_eviction_candidates(&policy, &entries, 2);

        // Should evict key1 and key3 (lowest access counts)
        assert_eq!(candidates.len(), 2);
        assert!(candidates.contains(&"key1".to_string()));
        assert!(candidates.contains(&"key3".to_string()));
    }
}
