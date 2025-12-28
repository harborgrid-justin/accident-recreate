//! Multi-tier cache (L1/L2/L3) backend

use crate::backends::{CacheBackend, disk::DiskCache, memory::MemoryCache, moka::MokaCacheBackend};
use crate::config::{CacheConfig, TierConfig};
use crate::error::CacheResult;
use crate::key::CacheKey;
use crate::value::CacheValue;
use serde::{Deserialize, Serialize};
use tracing::{debug, trace};

/// Multi-tier cache with L1 (memory), L2 (moka), L3 (disk)
#[derive(Debug)]
pub struct TieredCache<T>
where
    T: Clone + Send + Sync + Serialize + for<'de> Deserialize<'de> + 'static,
{
    l1: Option<MemoryCache<T>>,
    l2: Option<MokaCacheBackend<T>>,
    l3: Option<DiskCache<T>>,
    config: TierConfig,
}

impl<T> TieredCache<T>
where
    T: Clone + Send + Sync + Serialize + for<'de> Deserialize<'de> + 'static,
{
    /// Create a new tiered cache
    pub fn new(cache_config: &CacheConfig) -> CacheResult<Self> {
        let tier_config = &cache_config.tier_config;

        // Create L1 (fast memory cache)
        let l1 = if tier_config.enable_l1 {
            let mut l1_config = cache_config.clone();
            l1_config.max_entries = tier_config.l1_max_entries;
            Some(MemoryCache::new(&l1_config))
        } else {
            None
        };

        // Create L2 (concurrent moka cache)
        let l2 = if tier_config.enable_l2 {
            let mut l2_config = cache_config.clone();
            l2_config.max_entries = tier_config.l2_max_entries;
            Some(MokaCacheBackend::new(&l2_config))
        } else {
            None
        };

        // Create L3 (disk cache)
        let l3 = if tier_config.enable_l3 {
            if let Some(ref disk_config) = cache_config.disk_config {
                Some(DiskCache::new(disk_config.clone())?)
            } else {
                None
            }
        } else {
            None
        };

        Ok(Self {
            l1,
            l2,
            l3,
            config: tier_config.clone(),
        })
    }

    /// Promote value to upper tiers
    fn promote(&self, key: &CacheKey, value: &CacheValue<T>) -> CacheResult<()> {
        // Promote to L2 if exists and value came from L3
        if let Some(ref l2) = self.l2 {
            let _ = l2.insert(key.clone(), value.clone());
        }

        // Promote to L1 if exists
        if let Some(ref l1) = self.l1 {
            let _ = l1.insert(key.clone(), value.clone());
        }

        Ok(())
    }

    /// Write through all tiers
    fn write_through(&self, key: &CacheKey, value: &CacheValue<T>) -> CacheResult<()> {
        // Write to all enabled tiers
        if let Some(ref l1) = self.l1 {
            l1.insert(key.clone(), value.clone())?;
        }

        if let Some(ref l2) = self.l2 {
            l2.insert(key.clone(), value.clone())?;
        }

        if let Some(ref l3) = self.l3 {
            l3.insert(key.clone(), value.clone())?;
        }

        Ok(())
    }

    /// Get statistics for each tier
    pub fn tier_stats(&self) -> TierStats {
        TierStats {
            l1_entries: self.l1.as_ref().map(|c| c.len()).unwrap_or(0),
            l2_entries: self.l2.as_ref().map(|c| c.len()).unwrap_or(0),
            l3_entries: self.l3.as_ref().map(|c| c.len()).unwrap_or(0),
        }
    }
}

impl<T> CacheBackend for TieredCache<T>
where
    T: Clone + Send + Sync + Serialize + for<'de> Deserialize<'de> + std::fmt::Debug + 'static,
{
    type Value = T;

    fn get(&self, key: &CacheKey) -> CacheResult<Option<CacheValue<Self::Value>>> {
        // Try L1 first (fastest)
        if let Some(ref l1) = self.l1 {
            if let Some(value) = l1.get(key)? {
                trace!("L1 cache hit: {}", key);
                return Ok(Some(value));
            }
        }

        // Try L2 (concurrent)
        if let Some(ref l2) = self.l2 {
            if let Some(value) = l2.get(key)? {
                trace!("L2 cache hit: {}", key);
                // Promote to L1
                if let Some(ref l1) = self.l1 {
                    let _ = l1.insert(key.clone(), value.clone());
                }
                return Ok(Some(value));
            }
        }

        // Try L3 (disk)
        if let Some(ref l3) = self.l3 {
            if let Some(value) = l3.get(key)? {
                trace!("L3 cache hit: {}", key);
                // Promote to upper tiers
                self.promote(key, &value)?;
                return Ok(Some(value));
            }
        }

        trace!("Cache miss on all tiers: {}", key);
        Ok(None)
    }

    fn insert(&self, key: CacheKey, value: CacheValue<Self::Value>) -> CacheResult<()> {
        // Write through all tiers
        self.write_through(&key, &value)?;
        debug!("Inserted to tiered cache: {}", key);
        Ok(())
    }

    fn remove(&self, key: &CacheKey) -> CacheResult<Option<CacheValue<Self::Value>>> {
        let mut result = None;

        // Remove from all tiers
        if let Some(ref l1) = self.l1 {
            result = l1.remove(key)?;
        }

        if let Some(ref l2) = self.l2 {
            if result.is_none() {
                result = l2.remove(key)?;
            } else {
                let _ = l2.remove(key);
            }
        }

        if let Some(ref l3) = self.l3 {
            if result.is_none() {
                result = l3.remove(key)?;
            } else {
                let _ = l3.remove(key);
            }
        }

        if result.is_some() {
            debug!("Removed from tiered cache: {}", key);
        }

        Ok(result)
    }

    fn contains_key(&self, key: &CacheKey) -> bool {
        // Check any tier
        if let Some(ref l1) = self.l1 {
            if l1.contains_key(key) {
                return true;
            }
        }

        if let Some(ref l2) = self.l2 {
            if l2.contains_key(key) {
                return true;
            }
        }

        if let Some(ref l3) = self.l3 {
            if l3.contains_key(key) {
                return true;
            }
        }

        false
    }

    fn clear(&self) -> CacheResult<()> {
        // Clear all tiers
        if let Some(ref l1) = self.l1 {
            l1.clear()?;
        }

        if let Some(ref l2) = self.l2 {
            l2.clear()?;
        }

        if let Some(ref l3) = self.l3 {
            l3.clear()?;
        }

        debug!("Cleared all tiers");
        Ok(())
    }

    fn len(&self) -> usize {
        // Return max across all tiers (approximate)
        let mut max_len = 0;

        if let Some(ref l1) = self.l1 {
            max_len = max_len.max(l1.len());
        }

        if let Some(ref l2) = self.l2 {
            max_len = max_len.max(l2.len());
        }

        if let Some(ref l3) = self.l3 {
            max_len = max_len.max(l3.len());
        }

        max_len
    }

    fn keys(&self) -> Vec<CacheKey> {
        // Collect unique keys from all tiers
        let mut keys = std::collections::HashSet::new();

        if let Some(ref l1) = self.l1 {
            keys.extend(l1.keys());
        }

        if let Some(ref l2) = self.l2 {
            keys.extend(l2.keys());
        }

        if let Some(ref l3) = self.l3 {
            keys.extend(l3.keys());
        }

        keys.into_iter().collect()
    }

    fn evict_expired(&self) -> CacheResult<usize> {
        let mut total = 0;

        if let Some(ref l1) = self.l1 {
            total += l1.evict_expired()?;
        }

        if let Some(ref l2) = self.l2 {
            total += l2.evict_expired()?;
        }

        if let Some(ref l3) = self.l3 {
            total += l3.evict_expired()?;
        }

        if total > 0 {
            debug!("Evicted {} expired entries across all tiers", total);
        }

        Ok(total)
    }

    fn capacity(&self) -> usize {
        let mut total = 0;

        if let Some(ref l1) = self.l1 {
            total += l1.capacity();
        }

        if let Some(ref l2) = self.l2 {
            total += l2.capacity();
        }

        if let Some(ref l3) = self.l3 {
            total += l3.capacity();
        }

        total
    }
}

/// Statistics for tiered cache
#[derive(Debug, Clone)]
pub struct TierStats {
    pub l1_entries: usize,
    pub l2_entries: usize,
    pub l3_entries: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{CacheConfig, DiskConfig};

    #[test]
    fn test_tiered_cache_l1_l2() {
        let mut config = CacheConfig::default();
        config.tier_config.enable_l1 = true;
        config.tier_config.enable_l2 = true;
        config.tier_config.enable_l3 = false;

        let cache: TieredCache<String> = TieredCache::new(&config).unwrap();
        let key = CacheKey::new("test", "key1");
        let value = CacheValue::new("data".to_string());

        cache.insert(key.clone(), value.clone()).unwrap();

        // Should be in both L1 and L2
        let result = cache.get(&key).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().data, "data");
    }

    #[test]
    fn test_tiered_cache_promotion() {
        let mut config = CacheConfig::default();
        config.tier_config.l1_max_entries = 1;
        config.tier_config.l2_max_entries = 10;

        let cache: TieredCache<String> = TieredCache::new(&config).unwrap();

        // Insert two items (L1 can only hold 1)
        cache.insert(CacheKey::new("test", "key1"), CacheValue::new("data1".to_string())).unwrap();
        cache.insert(CacheKey::new("test", "key2"), CacheValue::new("data2".to_string())).unwrap();

        // Both should still be accessible (from L2)
        assert!(cache.get(&CacheKey::new("test", "key1")).unwrap().is_some());
        assert!(cache.get(&CacheKey::new("test", "key2")).unwrap().is_some());
    }
}
