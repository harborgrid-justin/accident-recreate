//! Storage backends for preferences

pub mod cloud;
pub mod local;

use crate::config::{PreferencesConfig, StorageBackend as ConfigBackend};
use crate::error::Result;
use crate::schema::types::PreferenceValue;
use async_trait::async_trait;
use std::collections::HashMap;

/// Storage backend trait
#[async_trait]
pub trait PreferenceStorage: Send + Sync {
    /// Load all preferences from storage
    async fn load_all(&self) -> Result<HashMap<String, PreferenceValue>>;

    /// Load a single preference
    async fn load(&self, key: &str) -> Result<Option<PreferenceValue>>;

    /// Save a preference
    async fn save(&mut self, key: &str, value: &PreferenceValue) -> Result<()>;

    /// Save multiple preferences
    async fn save_all(&mut self, preferences: &HashMap<String, PreferenceValue>) -> Result<()> {
        for (key, value) in preferences {
            self.save(key, value).await?;
        }
        Ok(())
    }

    /// Delete a preference
    async fn delete(&mut self, key: &str) -> Result<()>;

    /// Clear all preferences
    async fn clear(&mut self) -> Result<()>;

    /// Check if storage is available
    async fn is_available(&self) -> bool;

    /// Get storage metadata
    async fn metadata(&self) -> StorageMetadata;
}

/// Storage metadata
#[derive(Debug, Clone)]
pub struct StorageMetadata {
    pub backend_type: String,
    pub total_keys: usize,
    pub total_size_bytes: usize,
    pub last_sync: Option<chrono::DateTime<chrono::Utc>>,
}

/// Create a storage backend based on configuration
pub async fn create_storage(config: &PreferencesConfig) -> Result<Box<dyn PreferenceStorage>> {
    match config.storage_backend {
        ConfigBackend::Local => {
            let storage = local::LocalStorage::new(config.clone()).await?;
            Ok(Box::new(storage))
        }
        ConfigBackend::Cloud => {
            let storage = cloud::CloudStorage::new(config.clone()).await?;
            Ok(Box::new(storage))
        }
        ConfigBackend::Hybrid => {
            let storage = cloud::HybridStorage::new(config.clone()).await?;
            Ok(Box::new(storage))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::PreferencesConfig;

    #[tokio::test]
    async fn test_create_local_storage() {
        let config = PreferencesConfig::default();
        let storage = create_storage(&config).await.unwrap();
        assert!(storage.is_available().await);
    }
}
