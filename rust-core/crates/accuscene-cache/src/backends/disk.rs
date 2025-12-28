//! Disk-based cache backend

use crate::backends::CacheBackend;
use crate::config::DiskConfig;
use crate::error::{CacheError, CacheResult};
use crate::key::CacheKey;
use crate::value::CacheValue;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{debug, error, trace, warn};

/// Disk-based cache backend
#[derive(Debug)]
pub struct DiskCache<T: Clone + Serialize + for<'de> Deserialize<'de>> {
    config: DiskConfig,
    index: Arc<RwLock<DiskCacheIndex>>,
    _phantom: std::marker::PhantomData<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DiskCacheIndex {
    /// Map of key -> file path
    entries: HashMap<String, DiskCacheEntry>,
    /// Total size on disk
    total_size_bytes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DiskCacheEntry {
    /// Path to the cached file
    file_path: PathBuf,
    /// Size of the file in bytes
    size_bytes: usize,
    /// Last access time (for LRU)
    last_access_secs: i64,
}

impl<T: Clone + Serialize + for<'de> Deserialize<'de>> DiskCache<T> {
    /// Create a new disk cache
    pub fn new(config: DiskConfig) -> CacheResult<Self> {
        // Create cache directory if it doesn't exist
        if !config.cache_dir.exists() {
            fs::create_dir_all(&config.cache_dir).map_err(|e| {
                CacheError::DiskError(format!("Failed to create cache directory: {}", e))
            })?;
        }

        // Load or create index
        let index = Self::load_or_create_index(&config)?;

        Ok(Self {
            config,
            index: Arc::new(RwLock::new(index)),
            _phantom: std::marker::PhantomData,
        })
    }

    /// Load existing index or create new one
    fn load_or_create_index(config: &DiskConfig) -> CacheResult<DiskCacheIndex> {
        let index_path = config.cache_dir.join("cache_index.bin");

        if index_path.exists() {
            match Self::load_index(&index_path) {
                Ok(index) => {
                    debug!("Loaded disk cache index with {} entries", index.entries.len());
                    Ok(index)
                }
                Err(e) => {
                    warn!("Failed to load index, creating new: {}", e);
                    Ok(DiskCacheIndex {
                        entries: HashMap::new(),
                        total_size_bytes: 0,
                    })
                }
            }
        } else {
            Ok(DiskCacheIndex {
                entries: HashMap::new(),
                total_size_bytes: 0,
            })
        }
    }

    /// Load index from disk
    fn load_index(path: &Path) -> CacheResult<DiskCacheIndex> {
        let mut file = File::open(path)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;

        bincode::deserialize(&bytes).map_err(|e| {
            CacheError::DeserializationError(format!("Failed to deserialize index: {}", e))
        })
    }

    /// Save index to disk
    fn save_index(&self) -> CacheResult<()> {
        let index_path = self.config.cache_dir.join("cache_index.bin");
        let index = self.index.read();

        let bytes = bincode::serialize(&*index).map_err(|e| {
            CacheError::SerializationError(format!("Failed to serialize index: {}", e))
        })?;

        let mut file = File::create(&index_path)?;
        file.write_all(&bytes)?;

        debug!("Saved disk cache index");
        Ok(())
    }

    /// Get file path for a cache key
    fn get_file_path(&self, key: &str) -> PathBuf {
        // Use hash to avoid filesystem issues with special characters
        let hash = seahash::hash(key.as_bytes());
        self.config.cache_dir.join(format!("{:x}.cache", hash))
    }

    /// Evict least recently used entries to make space
    fn evict_for_space(&self, required_bytes: usize) -> CacheResult<()> {
        let mut index = self.index.write();

        while index.total_size_bytes + required_bytes > self.config.max_disk_bytes {
            // Find LRU entry
            let lru_key = index
                .entries
                .iter()
                .min_by_key(|(_, entry)| entry.last_access_secs)
                .map(|(key, _)| key.clone());

            if let Some(key) = lru_key {
                if let Some(entry) = index.entries.remove(&key) {
                    // Delete file
                    if let Err(e) = fs::remove_file(&entry.file_path) {
                        warn!("Failed to delete cache file: {}", e);
                    }
                    index.total_size_bytes = index.total_size_bytes.saturating_sub(entry.size_bytes);
                    debug!("Evicted disk cache entry: {}", key);
                }
            } else {
                break;
            }
        }

        Ok(())
    }
}

impl<T: Clone + Serialize + for<'de> Deserialize<'de> + Send + Sync + std::fmt::Debug> CacheBackend for DiskCache<T> {
    type Value = T;

    fn get(&self, key: &CacheKey) -> CacheResult<Option<CacheValue<Self::Value>>> {
        let key_str = key.as_string();

        let entry = {
            let index = self.index.read();
            index.entries.get(&key_str).cloned()
        };

        if let Some(entry) = entry {
            // Read from disk
            match fs::read(&entry.file_path) {
                Ok(bytes) => {
                    match bincode::deserialize::<CacheValue<T>>(&bytes) {
                        Ok(mut value) => {
                            // Check expiration
                            if value.is_expired() {
                                trace!("Disk cache entry expired: {}", key_str);
                                // Remove expired entry
                                let _ = self.remove(key);
                                return Ok(None);
                            }

                            // Update access time
                            value.record_access();

                            // Update index
                            {
                                let mut index = self.index.write();
                                if let Some(entry) = index.entries.get_mut(&key_str) {
                                    entry.last_access_secs = chrono::Utc::now().timestamp();
                                }
                            }

                            trace!("Disk cache hit: {}", key_str);
                            Ok(Some(value))
                        }
                        Err(e) => {
                            error!("Failed to deserialize cache value: {}", e);
                            // Clean up corrupted entry
                            let _ = self.remove(key);
                            Ok(None)
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to read cache file: {}", e);
                    // Clean up missing entry
                    let _ = self.remove(key);
                    Ok(None)
                }
            }
        } else {
            trace!("Disk cache miss: {}", key_str);
            Ok(None)
        }
    }

    fn insert(&self, key: CacheKey, value: CacheValue<Self::Value>) -> CacheResult<()> {
        let key_str = key.as_string();
        let file_path = self.get_file_path(&key_str);

        // Serialize value
        let bytes = bincode::serialize(&value).map_err(|e| {
            CacheError::SerializationError(format!("Failed to serialize value: {}", e))
        })?;

        let size_bytes = bytes.len();

        // Evict if needed
        self.evict_for_space(size_bytes)?;

        // Write to disk
        let mut file = File::create(&file_path)?;
        file.write_all(&bytes)?;
        file.sync_all()?;

        // Update index
        {
            let mut index = self.index.write();
            let entry = DiskCacheEntry {
                file_path: file_path.clone(),
                size_bytes,
                last_access_secs: chrono::Utc::now().timestamp(),
            };

            if let Some(old_entry) = index.entries.insert(key_str.clone(), entry) {
                // Remove old file if it exists
                let _ = fs::remove_file(&old_entry.file_path);
                index.total_size_bytes = index.total_size_bytes.saturating_sub(old_entry.size_bytes);
            }

            index.total_size_bytes += size_bytes;
        }

        // Save index periodically
        let _ = self.save_index();

        debug!("Inserted disk cache entry: {} ({} bytes)", key_str, size_bytes);
        Ok(())
    }

    fn remove(&self, key: &CacheKey) -> CacheResult<Option<CacheValue<Self::Value>>> {
        let key_str = key.as_string();

        let value = self.get(key)?;

        let mut index = self.index.write();
        if let Some(entry) = index.entries.remove(&key_str) {
            // Delete file
            if let Err(e) = fs::remove_file(&entry.file_path) {
                warn!("Failed to delete cache file: {}", e);
            }
            index.total_size_bytes = index.total_size_bytes.saturating_sub(entry.size_bytes);
            debug!("Removed disk cache entry: {}", key_str);
        }

        let _ = self.save_index();
        Ok(value)
    }

    fn contains_key(&self, key: &CacheKey) -> bool {
        let index = self.index.read();
        index.entries.contains_key(&key.as_string())
    }

    fn clear(&self) -> CacheResult<()> {
        let mut index = self.index.write();

        // Delete all cache files
        for (_, entry) in index.entries.drain() {
            let _ = fs::remove_file(&entry.file_path);
        }

        index.total_size_bytes = 0;

        drop(index);
        let _ = self.save_index();

        debug!("Cleared all disk cache entries");
        Ok(())
    }

    fn len(&self) -> usize {
        let index = self.index.read();
        index.entries.len()
    }

    fn keys(&self) -> Vec<CacheKey> {
        let index = self.index.read();
        index
            .entries
            .keys()
            .map(|k| k.clone().into())
            .collect()
    }

    fn evict_expired(&self) -> CacheResult<usize> {
        let keys: Vec<CacheKey> = self.keys();
        let mut count = 0;

        for key in keys {
            if let Ok(Some(value)) = self.get(&key) {
                if value.is_expired() {
                    let _ = self.remove(&key);
                    count += 1;
                }
            }
        }

        if count > 0 {
            debug!("Evicted {} expired disk cache entries", count);
        }

        Ok(count)
    }

    fn capacity(&self) -> usize {
        self.config.max_disk_bytes
    }
}

// Simple hash function for demo (in production, use a proper hash crate)
mod seahash {
    pub fn hash(bytes: &[u8]) -> u64 {
        let mut hash: u64 = 0x16f11fe89b0d677c;
        for &b in bytes {
            hash ^= b as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash
    }
}

impl<T: Clone + Serialize + for<'de> Deserialize<'de>> Drop for DiskCache<T> {
    fn drop(&mut self) {
        // Save index on drop
        let _ = self.save_index();
    }
}
