//! Feature store for caching and managing features

use crate::error::{MLError, Result};
use crate::feature::{FeatureSet, FeatureVector};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

/// Feature store configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureStoreConfig {
    /// Storage directory
    pub storage_path: PathBuf,

    /// Maximum cache size in bytes
    pub max_cache_size: usize,

    /// Enable persistence
    pub enable_persistence: bool,

    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,
}

impl Default for FeatureStoreConfig {
    fn default() -> Self {
        Self {
            storage_path: PathBuf::from("./feature_store"),
            max_cache_size: 1024 * 1024 * 1024, // 1GB
            enable_persistence: true,
            cache_ttl_seconds: 3600,
        }
    }
}

/// Feature store for managing and caching features
pub struct FeatureStore {
    /// Configuration
    config: FeatureStoreConfig,

    /// In-memory feature cache
    cache: Arc<DashMap<String, CachedFeature>>,

    /// Feature metadata
    metadata: Arc<RwLock<HashMap<String, FeatureMetadata>>>,

    /// Current cache size in bytes
    cache_size: Arc<RwLock<usize>>,
}

impl FeatureStore {
    /// Create a new feature store
    pub fn new(config: FeatureStoreConfig) -> Result<Self> {
        if config.enable_persistence {
            std::fs::create_dir_all(&config.storage_path)?;
        }

        Ok(Self {
            config,
            cache: Arc::new(DashMap::new()),
            metadata: Arc::new(RwLock::new(HashMap::new())),
            cache_size: Arc::new(RwLock::new(0)),
        })
    }

    /// Store a feature set
    pub fn store_feature_set(
        &self,
        key: impl Into<String>,
        feature_set: FeatureSet,
        metadata: Option<FeatureMetadata>,
    ) -> Result<()> {
        let key = key.into();

        // Estimate size
        let size = self.estimate_size(&feature_set);

        // Check if we need to evict
        self.evict_if_needed(size)?;

        // Create cached feature
        let cached = CachedFeature {
            data: FeatureData::Set(feature_set),
            created_at: Utc::now(),
            last_accessed: Utc::now(),
            access_count: 0,
            size_bytes: size,
        };

        // Store in cache
        self.cache.insert(key.clone(), cached);

        // Update cache size
        {
            let mut cache_size = self.cache_size.write();
            *cache_size += size;
        }

        // Store metadata
        if let Some(meta) = metadata {
            self.metadata.write().insert(key.clone(), meta);
        }

        // Persist if enabled
        if self.config.enable_persistence {
            self.persist_feature(&key)?;
        }

        Ok(())
    }

    /// Store a feature vector
    pub fn store_feature_vector(
        &self,
        key: impl Into<String>,
        feature_vector: FeatureVector,
        metadata: Option<FeatureMetadata>,
    ) -> Result<()> {
        let key = key.into();

        let size = feature_vector.values.len() * 8; // Approximate size

        self.evict_if_needed(size)?;

        let cached = CachedFeature {
            data: FeatureData::Vector(feature_vector),
            created_at: Utc::now(),
            last_accessed: Utc::now(),
            access_count: 0,
            size_bytes: size,
        };

        self.cache.insert(key.clone(), cached);

        {
            let mut cache_size = self.cache_size.write();
            *cache_size += size;
        }

        if let Some(meta) = metadata {
            self.metadata.write().insert(key.clone(), meta);
        }

        if self.config.enable_persistence {
            self.persist_feature(&key)?;
        }

        Ok(())
    }

    /// Retrieve a feature set
    pub fn get_feature_set(&self, key: &str) -> Result<FeatureSet> {
        // Try to get from cache
        if let Some(mut cached) = self.cache.get_mut(key) {
            cached.last_accessed = Utc::now();
            cached.access_count += 1;

            if let FeatureData::Set(ref set) = cached.data {
                return Ok(set.clone());
            } else {
                return Err(MLError::FeatureStore("Wrong feature type".to_string()));
            }
        }

        // Try to load from persistence
        if self.config.enable_persistence {
            self.load_feature(key)?;
            return self.get_feature_set(key);
        }

        Err(MLError::FeatureStore(format!("Feature not found: {}", key)))
    }

    /// Retrieve a feature vector
    pub fn get_feature_vector(&self, key: &str) -> Result<FeatureVector> {
        if let Some(mut cached) = self.cache.get_mut(key) {
            cached.last_accessed = Utc::now();
            cached.access_count += 1;

            if let FeatureData::Vector(ref vec) = cached.data {
                return Ok(vec.clone());
            } else {
                return Err(MLError::FeatureStore("Wrong feature type".to_string()));
            }
        }

        if self.config.enable_persistence {
            self.load_feature(key)?;
            return self.get_feature_vector(key);
        }

        Err(MLError::FeatureStore(format!("Feature not found: {}", key)))
    }

    /// Check if a feature exists
    pub fn contains(&self, key: &str) -> bool {
        self.cache.contains_key(key) || self.is_persisted(key)
    }

    /// Delete a feature
    pub fn delete(&self, key: &str) -> Result<()> {
        // Remove from cache
        if let Some((_, cached)) = self.cache.remove(key) {
            let mut cache_size = self.cache_size.write();
            *cache_size = cache_size.saturating_sub(cached.size_bytes);
        }

        // Remove metadata
        self.metadata.write().remove(key);

        // Remove from persistence
        if self.config.enable_persistence {
            let path = self.feature_path(key);
            if path.exists() {
                std::fs::remove_file(path)?;
            }
        }

        Ok(())
    }

    /// Clear all features
    pub fn clear(&self) -> Result<()> {
        self.cache.clear();
        self.metadata.write().clear();
        *self.cache_size.write() = 0;

        if self.config.enable_persistence {
            if self.config.storage_path.exists() {
                std::fs::remove_dir_all(&self.config.storage_path)?;
                std::fs::create_dir_all(&self.config.storage_path)?;
            }
        }

        Ok(())
    }

    /// Get feature metadata
    pub fn get_metadata(&self, key: &str) -> Option<FeatureMetadata> {
        self.metadata.read().get(key).cloned()
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let total_features = self.cache.len();
        let total_size = *self.cache_size.read();

        let mut total_accesses = 0u64;
        for entry in self.cache.iter() {
            total_accesses += entry.access_count;
        }

        CacheStats {
            total_features,
            total_size_bytes: total_size,
            max_size_bytes: self.config.max_cache_size,
            total_accesses,
        }
    }

    /// Estimate size of a feature set
    fn estimate_size(&self, feature_set: &FeatureSet) -> usize {
        // Rough estimate: number of elements * size of f64
        feature_set.matrix.len() * std::mem::size_of::<f64>()
    }

    /// Evict features if needed to make room
    fn evict_if_needed(&self, required_size: usize) -> Result<()> {
        let mut current_size = *self.cache_size.read();

        if current_size + required_size <= self.config.max_cache_size {
            return Ok(());
        }

        // Collect features sorted by last access time
        let mut features: Vec<(String, DateTime<Utc>, usize)> = self
            .cache
            .iter()
            .map(|entry| {
                (
                    entry.key().clone(),
                    entry.value().last_accessed,
                    entry.value().size_bytes,
                )
            })
            .collect();

        features.sort_by(|a, b| a.1.cmp(&b.1)); // Sort by last accessed (oldest first)

        // Evict oldest features until we have enough space
        for (key, _, size) in features {
            if current_size + required_size <= self.config.max_cache_size {
                break;
            }

            self.cache.remove(&key);
            current_size = current_size.saturating_sub(size);
        }

        *self.cache_size.write() = current_size;

        Ok(())
    }

    /// Persist a feature to disk
    fn persist_feature(&self, key: &str) -> Result<()> {
        if let Some(cached) = self.cache.get(key) {
            let path = self.feature_path(key);
            let serialized = bincode::serialize(&cached.data)?;
            std::fs::write(path, serialized)?;
        }
        Ok(())
    }

    /// Load a feature from disk
    fn load_feature(&self, key: &str) -> Result<()> {
        let path = self.feature_path(key);
        if !path.exists() {
            return Err(MLError::FeatureStore(format!("Feature not found: {}", key)));
        }

        let serialized = std::fs::read(path)?;
        let data: FeatureData = bincode::deserialize(&serialized)?;

        let size = match &data {
            FeatureData::Set(set) => self.estimate_size(set),
            FeatureData::Vector(vec) => vec.values.len() * 8,
        };

        self.evict_if_needed(size)?;

        let cached = CachedFeature {
            data,
            created_at: Utc::now(),
            last_accessed: Utc::now(),
            access_count: 0,
            size_bytes: size,
        };

        self.cache.insert(key.to_string(), cached);

        {
            let mut cache_size = self.cache_size.write();
            *cache_size += size;
        }

        Ok(())
    }

    /// Check if feature is persisted
    fn is_persisted(&self, key: &str) -> bool {
        if !self.config.enable_persistence {
            return false;
        }
        self.feature_path(key).exists()
    }

    /// Get feature file path
    fn feature_path(&self, key: &str) -> PathBuf {
        self.config.storage_path.join(format!("{}.bin", key))
    }
}

/// Cached feature entry
#[derive(Debug, Clone)]
struct CachedFeature {
    /// Feature data
    data: FeatureData,

    /// Creation time
    created_at: DateTime<Utc>,

    /// Last access time
    last_accessed: DateTime<Utc>,

    /// Access count
    access_count: u64,

    /// Size in bytes
    size_bytes: usize,
}

/// Feature data (either set or vector)
#[derive(Debug, Clone, Serialize, Deserialize)]
enum FeatureData {
    Set(FeatureSet),
    Vector(FeatureVector),
}

/// Feature metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureMetadata {
    /// Feature description
    pub description: String,

    /// Feature version
    pub version: String,

    /// Feature source
    pub source: String,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Custom metadata
    pub custom: HashMap<String, String>,
}

impl FeatureMetadata {
    /// Create new feature metadata
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            version: String::from("1.0"),
            source: String::from("unknown"),
            created_at: Utc::now(),
            custom: HashMap::new(),
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Total number of features
    pub total_features: usize,

    /// Total size in bytes
    pub total_size_bytes: usize,

    /// Maximum size in bytes
    pub max_size_bytes: usize,

    /// Total access count
    pub total_accesses: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::FeatureVector;
    use ndarray::{arr1, arr2};
    use tempfile::TempDir;

    #[test]
    fn test_feature_store() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let config = FeatureStoreConfig {
            storage_path: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let store = FeatureStore::new(config)?;

        // Store a feature vector
        let fv = FeatureVector::new(
            vec!["a".to_string(), "b".to_string()],
            arr1(&[1.0, 2.0]),
        );

        store.store_feature_vector("test", fv.clone(), None)?;

        assert!(store.contains("test"));

        // Retrieve it
        let retrieved = store.get_feature_vector("test")?;
        assert_eq!(retrieved.values, fv.values);

        Ok(())
    }
}
