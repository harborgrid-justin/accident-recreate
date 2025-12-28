//! AccuScene Enterprise Preferences System
//!
//! A comprehensive user preferences and settings management system with:
//! - Cloud synchronization across devices
//! - Preference inheritance and overrides
//! - Real-time updates and notifications
//! - Schema validation and type safety
//! - Migration support for version updates
//! - Export/import functionality

pub mod config;
pub mod error;
pub mod export;
pub mod migration;
pub mod schema;
pub mod storage;
pub mod sync;

pub use config::PreferencesConfig;
pub use error::{PreferencesError, Result};
pub use schema::{PreferenceSchema, PreferenceValue};
pub use storage::{PreferenceStorage, StorageBackend};
pub use sync::PreferenceSync;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Main preferences manager
pub struct PreferencesManager {
    config: PreferencesConfig,
    storage: Box<dyn PreferenceStorage>,
    sync: Option<sync::SyncManager>,
    cache: HashMap<String, PreferenceValue>,
}

impl PreferencesManager {
    /// Create a new preferences manager
    pub async fn new(config: PreferencesConfig) -> Result<Self> {
        let storage = storage::create_storage(&config).await?;

        let sync = if config.enable_sync {
            Some(sync::SyncManager::new(config.clone()).await?)
        } else {
            None
        };

        let mut manager = Self {
            config,
            storage,
            sync,
            cache: HashMap::new(),
        };

        manager.load_preferences().await?;

        Ok(manager)
    }

    /// Load all preferences from storage
    async fn load_preferences(&mut self) -> Result<()> {
        let prefs = self.storage.load_all().await?;
        self.cache = prefs;
        Ok(())
    }

    /// Get a preference value
    pub async fn get(&self, key: &str) -> Result<Option<PreferenceValue>> {
        Ok(self.cache.get(key).cloned())
    }

    /// Get a preference value with a default
    pub async fn get_or_default(&self, key: &str) -> Result<PreferenceValue> {
        if let Some(value) = self.cache.get(key) {
            return Ok(value.clone());
        }

        // Try to get from defaults
        schema::defaults::get_default(key)
            .ok_or_else(|| PreferencesError::KeyNotFound(key.to_string()))
    }

    /// Set a preference value
    pub async fn set(&mut self, key: String, value: PreferenceValue) -> Result<()> {
        // Validate against schema
        schema::validation::validate_preference(&key, &value)?;

        // Store in cache
        self.cache.insert(key.clone(), value.clone());

        // Persist to storage
        self.storage.save(&key, &value).await?;

        // Sync to cloud if enabled
        if let Some(ref mut sync) = self.sync {
            sync.sync_preference(&key, &value).await?;
        }

        Ok(())
    }

    /// Set multiple preferences at once
    pub async fn set_many(&mut self, preferences: HashMap<String, PreferenceValue>) -> Result<()> {
        for (key, value) in preferences {
            self.set(key, value).await?;
        }
        Ok(())
    }

    /// Delete a preference
    pub async fn delete(&mut self, key: &str) -> Result<()> {
        self.cache.remove(key);
        self.storage.delete(key).await?;

        if let Some(ref mut sync) = self.sync {
            sync.delete_preference(key).await?;
        }

        Ok(())
    }

    /// Reset a preference to its default value
    pub async fn reset(&mut self, key: &str) -> Result<()> {
        if let Some(default) = schema::defaults::get_default(key) {
            self.set(key.to_string(), default).await?;
        } else {
            self.delete(key).await?;
        }
        Ok(())
    }

    /// Reset all preferences to defaults
    pub async fn reset_all(&mut self) -> Result<()> {
        let keys: Vec<String> = self.cache.keys().cloned().collect();
        for key in keys {
            self.reset(&key).await?;
        }
        Ok(())
    }

    /// Export all preferences
    pub async fn export(&self) -> Result<export::PreferenceExport> {
        export::export_preferences(&self.cache)
    }

    /// Import preferences
    pub async fn import(&mut self, export: export::PreferenceExport) -> Result<()> {
        let preferences = export::import_preferences(export)?;
        self.set_many(preferences).await?;
        Ok(())
    }

    /// Sync preferences with cloud
    pub async fn sync_now(&mut self) -> Result<()> {
        if let Some(ref mut sync) = self.sync {
            sync.sync_all(&self.cache).await?;
            Ok(())
        } else {
            Err(PreferencesError::SyncDisabled)
        }
    }

    /// Get all preferences
    pub fn get_all(&self) -> &HashMap<String, PreferenceValue> {
        &self.cache
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_preferences_manager() {
        let config = PreferencesConfig::default();
        let mut manager = PreferencesManager::new(config).await.unwrap();

        // Test set and get
        manager.set(
            "test.key".to_string(),
            PreferenceValue::String("test value".to_string())
        ).await.unwrap();

        let value = manager.get("test.key").await.unwrap();
        assert!(value.is_some());
    }
}
