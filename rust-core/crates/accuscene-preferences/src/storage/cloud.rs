//! Cloud storage backend with local caching

use crate::config::PreferencesConfig;
use crate::error::{PreferencesError, Result};
use crate::schema::types::PreferenceValue;
use crate::storage::local::LocalStorage;
use crate::storage::{PreferenceStorage, StorageMetadata};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Cloud storage backend (pure cloud, no local cache)
pub struct CloudStorage {
    config: PreferencesConfig,
    client: reqwest::Client,
    cache: HashMap<String, PreferenceValue>,
}

/// Hybrid storage backend (cloud with local cache)
pub struct HybridStorage {
    local: LocalStorage,
    cloud: CloudStorage,
    config: PreferencesConfig,
}

/// Cloud API request/response types
#[derive(Debug, Serialize, Deserialize)]
struct CloudPreference {
    key: String,
    value: PreferenceValue,
    version: u64,
    device_id: String,
    user_id: Option<String>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CloudSyncRequest {
    device_id: String,
    user_id: Option<String>,
    preferences: Vec<CloudPreference>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CloudSyncResponse {
    preferences: Vec<CloudPreference>,
    conflicts: Vec<ConflictInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ConflictInfo {
    key: String,
    local_version: u64,
    remote_version: u64,
}

impl CloudStorage {
    /// Create a new cloud storage instance
    pub async fn new(config: PreferencesConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| PreferencesError::NetworkError(e.to_string()))?;

        let mut storage = Self {
            config,
            client,
            cache: HashMap::new(),
        };

        // Load from cloud
        storage.sync_from_cloud().await?;

        Ok(storage)
    }

    /// Get the cloud endpoint URL
    fn get_endpoint(&self, path: &str) -> Result<String> {
        let base = self
            .config
            .sync_endpoint
            .as_ref()
            .ok_or_else(|| PreferencesError::ConfigError("No sync endpoint configured".to_string()))?;

        Ok(format!("{}/{}", base.trim_end_matches('/'), path.trim_start_matches('/')))
    }

    /// Get authorization header
    fn get_auth_header(&self) -> Result<String> {
        let api_key = self
            .config
            .sync_api_key
            .as_ref()
            .ok_or_else(|| PreferencesError::ConfigError("No API key configured".to_string()))?;

        Ok(format!("Bearer {}", api_key))
    }

    /// Sync from cloud
    async fn sync_from_cloud(&mut self) -> Result<()> {
        let url = self.get_endpoint("preferences")?;
        let auth = self.get_auth_header()?;

        let response = self
            .client
            .get(&url)
            .header("Authorization", auth)
            .header("X-Device-ID", &self.config.device_id)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(PreferencesError::NetworkError(format!(
                "Cloud sync failed: {}",
                response.status()
            )));
        }

        let sync_response: CloudSyncResponse = response.json().await?;

        // Update cache with cloud preferences
        for pref in sync_response.preferences {
            self.cache.insert(pref.key, pref.value);
        }

        Ok(())
    }

    /// Push to cloud
    async fn push_to_cloud(&self, key: &str, value: &PreferenceValue) -> Result<()> {
        let url = self.get_endpoint("preferences")?;
        let auth = self.get_auth_header()?;

        let cloud_pref = CloudPreference {
            key: key.to_string(),
            value: value.clone(),
            version: 1,
            device_id: self.config.device_id.clone(),
            user_id: self.config.user_id.clone(),
            updated_at: chrono::Utc::now(),
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", auth)
            .header("X-Device-ID", &self.config.device_id)
            .json(&cloud_pref)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(PreferencesError::NetworkError(format!(
                "Failed to push to cloud: {}",
                response.status()
            )));
        }

        Ok(())
    }

    /// Delete from cloud
    async fn delete_from_cloud(&self, key: &str) -> Result<()> {
        let url = self.get_endpoint(&format!("preferences/{}", key))?;
        let auth = self.get_auth_header()?;

        let response = self
            .client
            .delete(&url)
            .header("Authorization", auth)
            .header("X-Device-ID", &self.config.device_id)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(PreferencesError::NetworkError(format!(
                "Failed to delete from cloud: {}",
                response.status()
            )));
        }

        Ok(())
    }
}

#[async_trait]
impl PreferenceStorage for CloudStorage {
    async fn load_all(&self) -> Result<HashMap<String, PreferenceValue>> {
        Ok(self.cache.clone())
    }

    async fn load(&self, key: &str) -> Result<Option<PreferenceValue>> {
        Ok(self.cache.get(key).cloned())
    }

    async fn save(&mut self, key: &str, value: &PreferenceValue) -> Result<()> {
        self.cache.insert(key.to_string(), value.clone());
        self.push_to_cloud(key, value).await?;
        Ok(())
    }

    async fn delete(&mut self, key: &str) -> Result<()> {
        self.cache.remove(key);
        self.delete_from_cloud(key).await?;
        Ok(())
    }

    async fn clear(&mut self) -> Result<()> {
        let keys: Vec<String> = self.cache.keys().cloned().collect();
        for key in keys {
            self.delete(&key).await?;
        }
        Ok(())
    }

    async fn is_available(&self) -> bool {
        // Try to ping the cloud endpoint
        if let Ok(url) = self.get_endpoint("health") {
            if let Ok(auth) = self.get_auth_header() {
                if let Ok(response) = self
                    .client
                    .get(&url)
                    .header("Authorization", auth)
                    .timeout(std::time::Duration::from_secs(5))
                    .send()
                    .await
                {
                    return response.status().is_success();
                }
            }
        }
        false
    }

    async fn metadata(&self) -> StorageMetadata {
        StorageMetadata {
            backend_type: "cloud".to_string(),
            total_keys: self.cache.len(),
            total_size_bytes: 0, // Not tracked for cloud
            last_sync: Some(chrono::Utc::now()),
        }
    }
}

impl HybridStorage {
    /// Create a new hybrid storage instance
    pub async fn new(config: PreferencesConfig) -> Result<Self> {
        let local = LocalStorage::new(config.clone()).await?;
        let cloud = CloudStorage::new(config.clone()).await?;

        Ok(Self {
            local,
            cloud,
            config,
        })
    }

    /// Sync local and cloud storage
    pub async fn sync(&mut self) -> Result<()> {
        // Get all local preferences
        let local_prefs = self.local.load_all().await?;

        // Get all cloud preferences
        let cloud_prefs = self.cloud.load_all().await?;

        // Merge: cloud takes precedence for conflicts
        let mut merged = local_prefs.clone();
        merged.extend(cloud_prefs);

        // Update local with merged preferences
        self.local.save_all(&merged).await?;

        Ok(())
    }
}

#[async_trait]
impl PreferenceStorage for HybridStorage {
    async fn load_all(&self) -> Result<HashMap<String, PreferenceValue>> {
        // Always load from local cache
        self.local.load_all().await
    }

    async fn load(&self, key: &str) -> Result<Option<PreferenceValue>> {
        // Load from local cache
        self.local.load(key).await
    }

    async fn save(&mut self, key: &str, value: &PreferenceValue) -> Result<()> {
        // Save to local first (for quick response)
        self.local.save(key, value).await?;

        // Then save to cloud (async, can fail without affecting local)
        if let Err(e) = self.cloud.save(key, value).await {
            tracing::warn!("Failed to sync to cloud: {}", e);
            // Don't fail the operation, just log the error
        }

        Ok(())
    }

    async fn delete(&mut self, key: &str) -> Result<()> {
        // Delete from local first
        self.local.delete(key).await?;

        // Then delete from cloud
        if let Err(e) = self.cloud.delete(key).await {
            tracing::warn!("Failed to delete from cloud: {}", e);
        }

        Ok(())
    }

    async fn clear(&mut self) -> Result<()> {
        self.local.clear().await?;
        if let Err(e) = self.cloud.clear().await {
            tracing::warn!("Failed to clear cloud storage: {}", e);
        }
        Ok(())
    }

    async fn is_available(&self) -> bool {
        // Hybrid is available if local is available
        self.local.is_available().await
    }

    async fn metadata(&self) -> StorageMetadata {
        let local_meta = self.local.metadata().await;
        let cloud_available = self.cloud.is_available().await;

        StorageMetadata {
            backend_type: if cloud_available {
                "hybrid (cloud online)".to_string()
            } else {
                "hybrid (cloud offline)".to_string()
            },
            total_keys: local_meta.total_keys,
            total_size_bytes: local_meta.total_size_bytes,
            last_sync: Some(chrono::Utc::now()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hybrid_storage_creation() {
        // This test requires a mock cloud endpoint
        // In production, you would use a test server or mock
    }
}
