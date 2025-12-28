//! Local file system storage backend

use crate::config::PreferencesConfig;
use crate::error::{PreferencesError, Result};
use crate::schema::types::PreferenceValue;
use crate::storage::{PreferenceStorage, StorageMetadata};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;

/// Local file system storage
pub struct LocalStorage {
    config: PreferencesConfig,
    storage_path: PathBuf,
    cache: HashMap<String, PreferenceValue>,
}

/// Storage file format
#[derive(Debug, Serialize, Deserialize)]
struct StorageFile {
    version: String,
    device_id: String,
    user_id: Option<String>,
    last_modified: chrono::DateTime<chrono::Utc>,
    preferences: HashMap<String, PreferenceValue>,
}

impl LocalStorage {
    /// Create a new local storage instance
    pub async fn new(config: PreferencesConfig) -> Result<Self> {
        let storage_path = config.storage_path.clone();

        // Create storage directory if it doesn't exist
        if let Some(parent) = storage_path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                PreferencesError::StorageError(format!("Failed to create storage directory: {}", e))
            })?;
        }

        let mut storage = Self {
            config,
            storage_path,
            cache: HashMap::new(),
        };

        // Load existing preferences
        storage.load_from_disk().await?;

        Ok(storage)
    }

    /// Load preferences from disk
    async fn load_from_disk(&mut self) -> Result<()> {
        if !self.storage_path.exists() {
            // No file yet, start with empty cache
            return Ok(());
        }

        let content = fs::read_to_string(&self.storage_path).await.map_err(|e| {
            PreferencesError::StorageError(format!("Failed to read storage file: {}", e))
        })?;

        let storage_file: StorageFile = serde_json::from_str(&content).map_err(|e| {
            PreferencesError::StorageError(format!("Failed to parse storage file: {}", e))
        })?;

        self.cache = storage_file.preferences;

        Ok(())
    }

    /// Save preferences to disk
    async fn save_to_disk(&self) -> Result<()> {
        let storage_file = StorageFile {
            version: self.config.app_version.clone(),
            device_id: self.config.device_id.clone(),
            user_id: self.config.user_id.clone(),
            last_modified: chrono::Utc::now(),
            preferences: self.cache.clone(),
        };

        let content = serde_json::to_string_pretty(&storage_file).map_err(|e| {
            PreferencesError::StorageError(format!("Failed to serialize preferences: {}", e))
        })?;

        // Write to temporary file first, then rename for atomicity
        let temp_path = self.storage_path.with_extension("tmp");

        let mut file = fs::File::create(&temp_path).await.map_err(|e| {
            PreferencesError::StorageError(format!("Failed to create temp file: {}", e))
        })?;

        file.write_all(content.as_bytes()).await.map_err(|e| {
            PreferencesError::StorageError(format!("Failed to write to temp file: {}", e))
        })?;

        file.sync_all().await.map_err(|e| {
            PreferencesError::StorageError(format!("Failed to sync temp file: {}", e))
        })?;

        drop(file);

        fs::rename(&temp_path, &self.storage_path).await.map_err(|e| {
            PreferencesError::StorageError(format!("Failed to rename temp file: {}", e))
        })?;

        Ok(())
    }

    /// Get the storage file size
    async fn get_file_size(&self) -> usize {
        if let Ok(metadata) = fs::metadata(&self.storage_path).await {
            metadata.len() as usize
        } else {
            0
        }
    }
}

#[async_trait]
impl PreferenceStorage for LocalStorage {
    async fn load_all(&self) -> Result<HashMap<String, PreferenceValue>> {
        Ok(self.cache.clone())
    }

    async fn load(&self, key: &str) -> Result<Option<PreferenceValue>> {
        Ok(self.cache.get(key).cloned())
    }

    async fn save(&mut self, key: &str, value: &PreferenceValue) -> Result<()> {
        self.cache.insert(key.to_string(), value.clone());
        self.save_to_disk().await?;
        Ok(())
    }

    async fn save_all(&mut self, preferences: &HashMap<String, PreferenceValue>) -> Result<()> {
        self.cache.extend(
            preferences
                .iter()
                .map(|(k, v)| (k.clone(), v.clone())),
        );
        self.save_to_disk().await?;
        Ok(())
    }

    async fn delete(&mut self, key: &str) -> Result<()> {
        self.cache.remove(key);
        self.save_to_disk().await?;
        Ok(())
    }

    async fn clear(&mut self) -> Result<()> {
        self.cache.clear();
        self.save_to_disk().await?;
        Ok(())
    }

    async fn is_available(&self) -> bool {
        // Check if we can write to the storage directory
        if let Some(parent) = self.storage_path.parent() {
            parent.exists() || fs::create_dir_all(parent).await.is_ok()
        } else {
            false
        }
    }

    async fn metadata(&self) -> StorageMetadata {
        StorageMetadata {
            backend_type: "local".to_string(),
            total_keys: self.cache.len(),
            total_size_bytes: self.get_file_size().await,
            last_sync: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_local_storage() {
        let dir = tempdir().unwrap();
        let storage_path = dir.path().join("preferences.json");

        let config = PreferencesConfig::default().with_storage_path(storage_path.clone());

        let mut storage = LocalStorage::new(config).await.unwrap();

        // Test save and load
        storage
            .save("test.key", &PreferenceValue::String("value".to_string()))
            .await
            .unwrap();

        let value = storage.load("test.key").await.unwrap();
        assert_eq!(value, Some(PreferenceValue::String("value".to_string())));

        // Test persistence
        let storage2 = LocalStorage::new(PreferencesConfig::default().with_storage_path(storage_path))
            .await
            .unwrap();
        let value = storage2.load("test.key").await.unwrap();
        assert_eq!(value, Some(PreferenceValue::String("value".to_string())));
    }

    #[tokio::test]
    async fn test_delete() {
        let dir = tempdir().unwrap();
        let storage_path = dir.path().join("preferences.json");
        let config = PreferencesConfig::default().with_storage_path(storage_path);

        let mut storage = LocalStorage::new(config).await.unwrap();

        storage
            .save("test.key", &PreferenceValue::String("value".to_string()))
            .await
            .unwrap();

        storage.delete("test.key").await.unwrap();

        let value = storage.load("test.key").await.unwrap();
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_clear() {
        let dir = tempdir().unwrap();
        let storage_path = dir.path().join("preferences.json");
        let config = PreferencesConfig::default().with_storage_path(storage_path);

        let mut storage = LocalStorage::new(config).await.unwrap();

        storage
            .save("key1", &PreferenceValue::String("value1".to_string()))
            .await
            .unwrap();
        storage
            .save("key2", &PreferenceValue::String("value2".to_string()))
            .await
            .unwrap();

        storage.clear().await.unwrap();

        let all = storage.load_all().await.unwrap();
        assert!(all.is_empty());
    }
}
