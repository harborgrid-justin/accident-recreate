//! Preference synchronization across devices

use crate::config::PreferencesConfig;
use crate::error::{PreferencesError, Result};
use crate::schema::types::PreferenceValue;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

/// Preference sync trait
#[async_trait]
pub trait PreferenceSync: Send + Sync {
    /// Sync a single preference
    async fn sync_preference(&mut self, key: &str, value: &PreferenceValue) -> Result<()>;

    /// Sync all preferences
    async fn sync_all(&mut self, preferences: &HashMap<String, PreferenceValue>) -> Result<()>;

    /// Delete a synced preference
    async fn delete_preference(&mut self, key: &str) -> Result<()>;

    /// Pull updates from remote
    async fn pull(&mut self) -> Result<HashMap<String, PreferenceValue>>;

    /// Push updates to remote
    async fn push(&mut self, preferences: &HashMap<String, PreferenceValue>) -> Result<()>;

    /// Resolve sync conflicts
    async fn resolve_conflicts(&mut self) -> Result<Vec<SyncConflict>>;
}

/// Sync manager for coordinating preference synchronization
pub struct SyncManager {
    config: PreferencesConfig,
    client: reqwest::Client,
    sync_state: Arc<RwLock<SyncState>>,
    auto_sync_handle: Option<tokio::task::JoinHandle<()>>,
}

/// Sync state tracking
#[derive(Debug, Clone)]
struct SyncState {
    last_sync: Option<chrono::DateTime<chrono::Utc>>,
    pending_changes: HashMap<String, PreferenceValue>,
    conflicts: Vec<SyncConflict>,
    is_syncing: bool,
}

/// Sync conflict information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConflict {
    pub key: String,
    pub local_value: PreferenceValue,
    pub remote_value: PreferenceValue,
    pub local_version: u64,
    pub remote_version: u64,
    pub resolution: ConflictResolution,
}

/// Conflict resolution strategy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConflictResolution {
    /// Use local value
    UseLocal,
    /// Use remote value
    UseRemote,
    /// Use the most recent value based on timestamp
    UseMostRecent,
    /// Requires manual resolution
    Manual,
}

impl SyncManager {
    /// Create a new sync manager
    pub async fn new(config: PreferencesConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| PreferencesError::NetworkError(e.to_string()))?;

        let sync_state = Arc::new(RwLock::new(SyncState {
            last_sync: None,
            pending_changes: HashMap::new(),
            conflicts: Vec::new(),
            is_syncing: false,
        }));

        let mut manager = Self {
            config: config.clone(),
            client,
            sync_state: sync_state.clone(),
            auto_sync_handle: None,
        };

        // Start auto-sync if enabled
        if config.auto_sync {
            manager.start_auto_sync(sync_state).await;
        }

        Ok(manager)
    }

    /// Start automatic synchronization
    async fn start_auto_sync(&mut self, sync_state: Arc<RwLock<SyncState>>) {
        let config = self.config.clone();
        let client = self.client.clone();
        let sync_interval = Duration::from_secs(self.config.sync_interval_seconds);

        let handle = tokio::spawn(async move {
            let mut ticker = interval(sync_interval);
            loop {
                ticker.tick().await;

                let mut state = sync_state.write().await;
                if !state.is_syncing && !state.pending_changes.is_empty() {
                    state.is_syncing = true;
                    drop(state);

                    // Perform sync
                    if let Err(e) = Self::perform_sync(&config, &client, &sync_state).await {
                        tracing::error!("Auto-sync failed: {}", e);
                    }

                    let mut state = sync_state.write().await;
                    state.is_syncing = false;
                }
            }
        });

        self.auto_sync_handle = Some(handle);
    }

    /// Perform synchronization
    async fn perform_sync(
        config: &PreferencesConfig,
        client: &reqwest::Client,
        sync_state: &Arc<RwLock<SyncState>>,
    ) -> Result<()> {
        let endpoint = config
            .sync_endpoint
            .as_ref()
            .ok_or_else(|| PreferencesError::ConfigError("No sync endpoint".to_string()))?;

        let api_key = config
            .sync_api_key
            .as_ref()
            .ok_or_else(|| PreferencesError::ConfigError("No API key".to_string()))?;

        // Get pending changes
        let pending = {
            let state = sync_state.read().await;
            state.pending_changes.clone()
        };

        if pending.is_empty() {
            return Ok(());
        }

        // Push to remote
        let url = format!("{}/sync", endpoint);
        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("X-Device-ID", &config.device_id)
            .json(&pending)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(PreferencesError::SyncError(format!(
                "Sync failed: {}",
                response.status()
            )));
        }

        // Clear pending changes
        let mut state = sync_state.write().await;
        state.pending_changes.clear();
        state.last_sync = Some(chrono::Utc::now());

        Ok(())
    }

    /// Get the sync endpoint URL
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

    /// Get sync status
    pub async fn get_status(&self) -> SyncStatus {
        let state = self.sync_state.read().await;
        SyncStatus {
            is_syncing: state.is_syncing,
            last_sync: state.last_sync,
            pending_count: state.pending_changes.len(),
            conflict_count: state.conflicts.len(),
        }
    }

    /// Stop auto-sync
    pub async fn stop(&mut self) {
        if let Some(handle) = self.auto_sync_handle.take() {
            handle.abort();
        }
    }
}

#[async_trait]
impl PreferenceSync for SyncManager {
    async fn sync_preference(&mut self, key: &str, value: &PreferenceValue) -> Result<()> {
        let mut state = self.sync_state.write().await;
        state.pending_changes.insert(key.to_string(), value.clone());
        Ok(())
    }

    async fn sync_all(&mut self, preferences: &HashMap<String, PreferenceValue>) -> Result<()> {
        let mut state = self.sync_state.write().await;
        state.pending_changes.extend(
            preferences
                .iter()
                .map(|(k, v)| (k.clone(), v.clone())),
        );
        Ok(())
    }

    async fn delete_preference(&mut self, key: &str) -> Result<()> {
        let mut state = self.sync_state.write().await;
        state.pending_changes.remove(key);

        // Push delete to remote
        let url = self.get_endpoint(&format!("preferences/{}", key))?;
        let auth = self.get_auth_header()?;

        self.client
            .delete(&url)
            .header("Authorization", auth)
            .header("X-Device-ID", &self.config.device_id)
            .send()
            .await?;

        Ok(())
    }

    async fn pull(&mut self) -> Result<HashMap<String, PreferenceValue>> {
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
            return Err(PreferencesError::SyncError(format!(
                "Pull failed: {}",
                response.status()
            )));
        }

        let preferences: HashMap<String, PreferenceValue> = response.json().await?;

        let mut state = self.sync_state.write().await;
        state.last_sync = Some(chrono::Utc::now());

        Ok(preferences)
    }

    async fn push(&mut self, preferences: &HashMap<String, PreferenceValue>) -> Result<()> {
        let url = self.get_endpoint("sync")?;
        let auth = self.get_auth_header()?;

        let response = self
            .client
            .post(&url)
            .header("Authorization", auth)
            .header("X-Device-ID", &self.config.device_id)
            .json(preferences)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(PreferencesError::SyncError(format!(
                "Push failed: {}",
                response.status()
            )));
        }

        let mut state = self.sync_state.write().await;
        state.last_sync = Some(chrono::Utc::now());

        Ok(())
    }

    async fn resolve_conflicts(&mut self) -> Result<Vec<SyncConflict>> {
        let state = self.sync_state.read().await;
        Ok(state.conflicts.clone())
    }
}

/// Sync status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    pub is_syncing: bool,
    pub last_sync: Option<chrono::DateTime<chrono::Utc>>,
    pub pending_count: usize,
    pub conflict_count: usize,
}

impl Drop for SyncManager {
    fn drop(&mut self) {
        if let Some(handle) = self.auto_sync_handle.take() {
            handle.abort();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sync_manager_creation() {
        // This requires a mock endpoint in production
    }
}
