//! Tag-based cache invalidation

use crate::backends::CacheBackend;
use crate::error::CacheResult;
use crate::key::CacheKey;
use crate::value::CacheValue;
use dashmap::DashMap;
use std::collections::HashSet;
use std::sync::Arc;
use tracing::debug;

/// Tag-based cache manager
#[derive(Debug)]
pub struct TaggedCache<T: Clone + Send + Sync + 'static> {
    cache: Box<dyn CacheBackend<Value = T>>,
    tag_index: Arc<DashMap<String, HashSet<String>>>, // tag -> set of keys
    key_tags: Arc<DashMap<String, HashSet<String>>>,  // key -> set of tags
}

impl<T: Clone + Send + Sync + 'static> TaggedCache<T> {
    pub fn new(cache: Box<dyn CacheBackend<Value = T>>) -> Self {
        Self {
            cache,
            tag_index: Arc::new(DashMap::new()),
            key_tags: Arc::new(DashMap::new()),
        }
    }

    /// Insert with tags
    pub fn insert_with_tags(
        &self,
        key: CacheKey,
        value: CacheValue<T>,
        tags: Vec<String>,
    ) -> CacheResult<()> {
        let key_str = key.as_string();

        // Insert into cache
        self.cache.insert(key, value)?;

        // Update tag indices
        for tag in &tags {
            self.tag_index
                .entry(tag.clone())
                .or_insert_with(HashSet::new)
                .insert(key_str.clone());
        }

        // Update key -> tags mapping
        self.key_tags.insert(key_str, tags.into_iter().collect());

        Ok(())
    }

    /// Get value
    pub fn get(&self, key: &CacheKey) -> CacheResult<Option<CacheValue<T>>> {
        self.cache.get(key)
    }

    /// Remove value
    pub fn remove(&self, key: &CacheKey) -> CacheResult<Option<CacheValue<T>>> {
        let key_str = key.as_string();

        // Remove from cache
        let value = self.cache.remove(key)?;

        // Remove from tag indices
        if let Some((_, tags)) = self.key_tags.remove(&key_str) {
            for tag in tags {
                if let Some(mut keys) = self.tag_index.get_mut(&tag) {
                    keys.remove(&key_str);
                }
            }
        }

        Ok(value)
    }

    /// Invalidate all entries with a specific tag
    pub fn invalidate_tag(&self, tag: &str) -> CacheResult<usize> {
        let keys_to_invalidate: Vec<String> = if let Some(keys) = self.tag_index.get(tag) {
            keys.iter().cloned().collect()
        } else {
            return Ok(0);
        };

        let mut count = 0;
        for key_str in keys_to_invalidate {
            let key: CacheKey = key_str.clone().into();
            if self.remove(&key)?.is_some() {
                count += 1;
            }
        }

        // Clean up empty tag entry
        self.tag_index.remove(tag);

        debug!("Invalidated {} entries with tag: {}", count, tag);
        Ok(count)
    }

    /// Invalidate entries matching any of the tags
    pub fn invalidate_any_tags(&self, tags: &[String]) -> CacheResult<usize> {
        let mut total = 0;
        for tag in tags {
            total += self.invalidate_tag(tag)?;
        }
        Ok(total)
    }

    /// Invalidate entries matching all of the tags
    pub fn invalidate_all_tags(&self, tags: &[String]) -> CacheResult<usize> {
        if tags.is_empty() {
            return Ok(0);
        }

        // Find keys that have all tags
        let mut common_keys: Option<HashSet<String>> = None;

        for tag in tags {
            if let Some(keys) = self.tag_index.get(tag) {
                let key_set: HashSet<String> = keys.iter().cloned().collect();
                common_keys = Some(match common_keys {
                    None => key_set,
                    Some(existing) => existing.intersection(&key_set).cloned().collect(),
                });
            } else {
                // Tag doesn't exist, no keys can match all tags
                return Ok(0);
            }
        }

        let keys_to_invalidate = common_keys.unwrap_or_default();
        let mut count = 0;

        for key_str in keys_to_invalidate {
            let key: CacheKey = key_str.into();
            if self.remove(&key)?.is_some() {
                count += 1;
            }
        }

        debug!("Invalidated {} entries with all tags", count);
        Ok(count)
    }

    /// Get all tags for a key
    pub fn get_tags(&self, key: &CacheKey) -> Vec<String> {
        let key_str = key.as_string();
        self.key_tags
            .get(&key_str)
            .map(|tags| tags.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Get all keys with a specific tag
    pub fn get_keys_by_tag(&self, tag: &str) -> Vec<CacheKey> {
        self.tag_index
            .get(tag)
            .map(|keys| keys.iter().map(|k| k.clone().into()).collect())
            .unwrap_or_default()
    }

    /// Get all tags in the cache
    pub fn all_tags(&self) -> Vec<String> {
        self.tag_index
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }

    /// Clear all entries and tags
    pub fn clear(&self) -> CacheResult<()> {
        self.cache.clear()?;
        self.tag_index.clear();
        self.key_tags.clear();
        debug!("Cleared tagged cache");
        Ok(())
    }

    /// Get cache statistics
    pub fn stats(&self) -> TagCacheStats {
        TagCacheStats {
            total_entries: self.cache.len(),
            total_tags: self.tag_index.len(),
            total_tag_associations: self.key_tags.len(),
        }
    }
}

/// Tag cache statistics
#[derive(Debug, Clone)]
pub struct TagCacheStats {
    pub total_entries: usize,
    pub total_tags: usize,
    pub total_tag_associations: usize,
}

/// Helper for building tagged values
pub struct TaggedValueBuilder<T> {
    value: T,
    tags: Vec<String>,
}

impl<T> TaggedValueBuilder<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            tags: Vec::new(),
        }
    }

    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags.extend(tags);
        self
    }

    pub fn build(self) -> (CacheValue<T>, Vec<String>) {
        let cache_value = CacheValue::with_tags(self.value, self.tags.clone());
        (cache_value, self.tags)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backends::memory::MemoryCache;

    #[test]
    fn test_tagged_cache() {
        let cache = Box::new(MemoryCache::with_capacity(10));
        let tagged_cache: TaggedCache<String> = TaggedCache::new(cache);

        let key1 = CacheKey::new("test", "key1");
        let key2 = CacheKey::new("test", "key2");

        // Insert with tags
        tagged_cache
            .insert_with_tags(
                key1.clone(),
                CacheValue::new("value1".to_string()),
                vec!["user".to_string(), "session".to_string()],
            )
            .unwrap();

        tagged_cache
            .insert_with_tags(
                key2.clone(),
                CacheValue::new("value2".to_string()),
                vec!["user".to_string()],
            )
            .unwrap();

        // Verify tags
        let tags1 = tagged_cache.get_tags(&key1);
        assert_eq!(tags1.len(), 2);
        assert!(tags1.contains(&"user".to_string()));
        assert!(tags1.contains(&"session".to_string()));

        // Invalidate by tag
        let count = tagged_cache.invalidate_tag("user").unwrap();
        assert_eq!(count, 2);

        // Both entries should be gone
        assert!(tagged_cache.get(&key1).unwrap().is_none());
        assert!(tagged_cache.get(&key2).unwrap().is_none());
    }

    #[test]
    fn test_invalidate_all_tags() {
        let cache = Box::new(MemoryCache::with_capacity(10));
        let tagged_cache: TaggedCache<String> = TaggedCache::new(cache);

        let key1 = CacheKey::new("test", "key1");
        let key2 = CacheKey::new("test", "key2");
        let key3 = CacheKey::new("test", "key3");

        tagged_cache
            .insert_with_tags(
                key1.clone(),
                CacheValue::new("value1".to_string()),
                vec!["tag1".to_string(), "tag2".to_string()],
            )
            .unwrap();

        tagged_cache
            .insert_with_tags(
                key2.clone(),
                CacheValue::new("value2".to_string()),
                vec!["tag1".to_string()],
            )
            .unwrap();

        tagged_cache
            .insert_with_tags(
                key3.clone(),
                CacheValue::new("value3".to_string()),
                vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()],
            )
            .unwrap();

        // Invalidate entries with both tag1 AND tag2
        let count = tagged_cache
            .invalidate_all_tags(&["tag1".to_string(), "tag2".to_string()])
            .unwrap();
        assert_eq!(count, 2); // key1 and key3

        // key2 should still exist (only has tag1)
        assert!(tagged_cache.get(&key2).unwrap().is_some());
    }
}
