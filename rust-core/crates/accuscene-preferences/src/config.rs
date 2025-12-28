//! Preferences configuration

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for the preferences system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferencesConfig {
    /// Enable cloud synchronization
    pub enable_sync: bool,

    /// Cloud sync endpoint URL
    pub sync_endpoint: Option<String>,

    /// API key for cloud sync
    pub sync_api_key: Option<String>,

    /// Storage backend type
    pub storage_backend: StorageBackend,

    /// Local storage path
    pub storage_path: PathBuf,

    /// Enable encryption for sensitive preferences
    pub enable_encryption: bool,

    /// Encryption key (base64 encoded)
    pub encryption_key: Option<String>,

    /// Sync interval in seconds
    pub sync_interval_seconds: u64,

    /// Enable automatic sync
    pub auto_sync: bool,

    /// Maximum cache size (number of preferences)
    pub max_cache_size: usize,

    /// Enable preference inheritance
    pub enable_inheritance: bool,

    /// User ID for multi-user systems
    pub user_id: Option<String>,

    /// Device ID for multi-device sync
    pub device_id: String,

    /// Application version for migration
    pub app_version: String,
}

/// Storage backend type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StorageBackend {
    /// Local file system storage
    Local,
    /// Cloud storage with local cache
    Cloud,
    /// Hybrid: local with cloud sync
    Hybrid,
}

impl Default for PreferencesConfig {
    fn default() -> Self {
        Self {
            enable_sync: false,
            sync_endpoint: None,
            sync_api_key: None,
            storage_backend: StorageBackend::Local,
            storage_path: dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("accuscene")
                .join("preferences"),
            enable_encryption: false,
            encryption_key: None,
            sync_interval_seconds: 300, // 5 minutes
            auto_sync: false,
            max_cache_size: 10000,
            enable_inheritance: true,
            user_id: None,
            device_id: uuid::Uuid::new_v4().to_string(),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

impl PreferencesConfig {
    /// Create a new configuration with cloud sync enabled
    pub fn with_cloud_sync(endpoint: String, api_key: String) -> Self {
        Self {
            enable_sync: true,
            sync_endpoint: Some(endpoint),
            sync_api_key: Some(api_key),
            storage_backend: StorageBackend::Hybrid,
            auto_sync: true,
            ..Default::default()
        }
    }

    /// Create a new configuration with encryption enabled
    pub fn with_encryption(key: String) -> Self {
        Self {
            enable_encryption: true,
            encryption_key: Some(key),
            ..Default::default()
        }
    }

    /// Set the user ID
    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    /// Set the storage path
    pub fn with_storage_path(mut self, path: PathBuf) -> Self {
        self.storage_path = path;
        self
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.enable_sync {
            if self.sync_endpoint.is_none() {
                return Err("Sync endpoint required when sync is enabled".to_string());
            }
            if self.sync_api_key.is_none() {
                return Err("Sync API key required when sync is enabled".to_string());
            }
        }

        if self.enable_encryption && self.encryption_key.is_none() {
            return Err("Encryption key required when encryption is enabled".to_string());
        }

        if self.sync_interval_seconds < 10 {
            return Err("Sync interval must be at least 10 seconds".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = PreferencesConfig::default();
        assert!(!config.enable_sync);
        assert_eq!(config.storage_backend, StorageBackend::Local);
    }

    #[test]
    fn test_cloud_sync_config() {
        let config = PreferencesConfig::with_cloud_sync(
            "https://api.example.com".to_string(),
            "test-key".to_string(),
        );
        assert!(config.enable_sync);
        assert_eq!(config.storage_backend, StorageBackend::Hybrid);
    }

    #[test]
    fn test_validation() {
        let mut config = PreferencesConfig::default();
        assert!(config.validate().is_ok());

        config.enable_sync = true;
        assert!(config.validate().is_err());

        config.sync_endpoint = Some("https://api.example.com".to_string());
        config.sync_api_key = Some("test-key".to_string());
        assert!(config.validate().is_ok());
    }
}
