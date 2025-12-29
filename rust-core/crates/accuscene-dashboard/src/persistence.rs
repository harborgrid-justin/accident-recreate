//! Dashboard persistence layer
//!
//! Handles saving and loading dashboard states

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tokio::sync::RwLock;

use crate::error::{PersistenceError, PersistenceResult};
use crate::state::DashboardState;

/// Persistence metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceMetadata {
    /// Dashboard ID
    pub dashboard_id: String,

    /// User ID (optional)
    pub user_id: Option<String>,

    /// Created timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Updated timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,

    /// Size in bytes
    pub size_bytes: usize,

    /// Version
    pub version: u64,

    /// Tags
    pub tags: Vec<String>,
}

/// Persisted dashboard record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedDashboard {
    /// Metadata
    pub metadata: PersistenceMetadata,

    /// Dashboard state
    pub state: DashboardState,
}

/// Persistence storage trait
#[async_trait]
pub trait PersistenceStorage: Send + Sync {
    /// Save dashboard state
    async fn save(&self, dashboard_id: &str, state: &DashboardState) -> PersistenceResult<()>;

    /// Load dashboard state
    async fn load(&self, dashboard_id: &str) -> PersistenceResult<DashboardState>;

    /// Delete dashboard state
    async fn delete(&self, dashboard_id: &str) -> PersistenceResult<()>;

    /// Check if dashboard exists
    async fn exists(&self, dashboard_id: &str) -> PersistenceResult<bool>;

    /// List all dashboard IDs
    async fn list(&self) -> PersistenceResult<Vec<String>>;

    /// Get metadata for a dashboard
    async fn get_metadata(&self, dashboard_id: &str) -> PersistenceResult<PersistenceMetadata>;

    /// Update metadata only
    async fn update_metadata(&self, dashboard_id: &str, metadata: PersistenceMetadata) -> PersistenceResult<()>;
}

/// In-memory persistence storage (for testing/development)
pub struct InMemoryStorage {
    dashboards: RwLock<HashMap<String, PersistedDashboard>>,
}

impl InMemoryStorage {
    /// Create a new in-memory storage
    pub fn new() -> Self {
        Self {
            dashboards: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PersistenceStorage for InMemoryStorage {
    async fn save(&self, dashboard_id: &str, state: &DashboardState) -> PersistenceResult<()> {
        let mut dashboards = self.dashboards.write().await;

        let now = chrono::Utc::now();
        let state_json = serde_json::to_string(state)
            .map_err(|e| PersistenceError::Database(format!("Serialization failed: {}", e)))?;

        let metadata = if let Some(existing) = dashboards.get(dashboard_id) {
            // Update existing
            if existing.metadata.version != state.version - 1 {
                return Err(PersistenceError::VersionConflict {
                    expected: existing.metadata.version + 1,
                    actual: state.version,
                });
            }

            PersistenceMetadata {
                dashboard_id: dashboard_id.to_string(),
                user_id: existing.metadata.user_id.clone(),
                created_at: existing.metadata.created_at,
                updated_at: now,
                size_bytes: state_json.len(),
                version: state.version,
                tags: existing.metadata.tags.clone(),
            }
        } else {
            // Create new
            PersistenceMetadata {
                dashboard_id: dashboard_id.to_string(),
                user_id: None,
                created_at: now,
                updated_at: now,
                size_bytes: state_json.len(),
                version: state.version,
                tags: Vec::new(),
            }
        };

        dashboards.insert(
            dashboard_id.to_string(),
            PersistedDashboard {
                metadata,
                state: state.clone(),
            },
        );

        Ok(())
    }

    async fn load(&self, dashboard_id: &str) -> PersistenceResult<DashboardState> {
        let dashboards = self.dashboards.read().await;
        dashboards
            .get(dashboard_id)
            .map(|p| p.state.clone())
            .ok_or_else(|| PersistenceError::Database(format!("Dashboard not found: {}", dashboard_id)))
    }

    async fn delete(&self, dashboard_id: &str) -> PersistenceResult<()> {
        let mut dashboards = self.dashboards.write().await;
        dashboards
            .remove(dashboard_id)
            .ok_or_else(|| PersistenceError::Database(format!("Dashboard not found: {}", dashboard_id)))?;
        Ok(())
    }

    async fn exists(&self, dashboard_id: &str) -> PersistenceResult<bool> {
        let dashboards = self.dashboards.read().await;
        Ok(dashboards.contains_key(dashboard_id))
    }

    async fn list(&self) -> PersistenceResult<Vec<String>> {
        let dashboards = self.dashboards.read().await;
        Ok(dashboards.keys().cloned().collect())
    }

    async fn get_metadata(&self, dashboard_id: &str) -> PersistenceResult<PersistenceMetadata> {
        let dashboards = self.dashboards.read().await;
        dashboards
            .get(dashboard_id)
            .map(|p| p.metadata.clone())
            .ok_or_else(|| PersistenceError::Database(format!("Dashboard not found: {}", dashboard_id)))
    }

    async fn update_metadata(&self, dashboard_id: &str, metadata: PersistenceMetadata) -> PersistenceResult<()> {
        let mut dashboards = self.dashboards.write().await;
        let persisted = dashboards
            .get_mut(dashboard_id)
            .ok_or_else(|| PersistenceError::Database(format!("Dashboard not found: {}", dashboard_id)))?;

        persisted.metadata = metadata;
        Ok(())
    }
}

/// File-based persistence storage
pub struct FileStorage {
    base_path: PathBuf,
}

impl FileStorage {
    /// Create a new file storage
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    /// Get file path for a dashboard
    fn get_path(&self, dashboard_id: &str) -> PathBuf {
        self.base_path.join(format!("{}.json", dashboard_id))
    }

    /// Get metadata file path
    fn get_metadata_path(&self, dashboard_id: &str) -> PathBuf {
        self.base_path.join(format!("{}.meta.json", dashboard_id))
    }

    /// Ensure base directory exists
    async fn ensure_directory(&self) -> PersistenceResult<()> {
        if !self.base_path.exists() {
            fs::create_dir_all(&self.base_path)
                .await
                .map_err(|e| PersistenceError::Database(format!("Failed to create directory: {}", e)))?;
        }
        Ok(())
    }
}

#[async_trait]
impl PersistenceStorage for FileStorage {
    async fn save(&self, dashboard_id: &str, state: &DashboardState) -> PersistenceResult<()> {
        self.ensure_directory().await?;

        let path = self.get_path(dashboard_id);
        let meta_path = self.get_metadata_path(dashboard_id);

        // Check for version conflict if file exists
        if path.exists() {
            let existing_meta = self.get_metadata(dashboard_id).await?;
            if existing_meta.version != state.version - 1 {
                return Err(PersistenceError::VersionConflict {
                    expected: existing_meta.version + 1,
                    actual: state.version,
                });
            }
        }

        // Serialize state
        let state_json = serde_json::to_string_pretty(state)
            .map_err(|e| PersistenceError::Database(format!("Serialization failed: {}", e)))?;

        // Write state file
        fs::write(&path, &state_json)
            .await
            .map_err(|e| PersistenceError::Database(format!("Failed to write state: {}", e)))?;

        // Create/update metadata
        let now = chrono::Utc::now();
        let metadata = if meta_path.exists() {
            let existing_meta = self.get_metadata(dashboard_id).await?;
            PersistenceMetadata {
                dashboard_id: dashboard_id.to_string(),
                user_id: existing_meta.user_id,
                created_at: existing_meta.created_at,
                updated_at: now,
                size_bytes: state_json.len(),
                version: state.version,
                tags: existing_meta.tags,
            }
        } else {
            PersistenceMetadata {
                dashboard_id: dashboard_id.to_string(),
                user_id: None,
                created_at: now,
                updated_at: now,
                size_bytes: state_json.len(),
                version: state.version,
                tags: Vec::new(),
            }
        };

        // Write metadata file
        let meta_json = serde_json::to_string_pretty(&metadata)
            .map_err(|e| PersistenceError::Database(format!("Metadata serialization failed: {}", e)))?;

        fs::write(&meta_path, meta_json)
            .await
            .map_err(|e| PersistenceError::Database(format!("Failed to write metadata: {}", e)))?;

        Ok(())
    }

    async fn load(&self, dashboard_id: &str) -> PersistenceResult<DashboardState> {
        let path = self.get_path(dashboard_id);

        if !path.exists() {
            return Err(PersistenceError::Database(format!("Dashboard not found: {}", dashboard_id)));
        }

        let content = fs::read_to_string(&path)
            .await
            .map_err(|e| PersistenceError::Database(format!("Failed to read state: {}", e)))?;

        let state: DashboardState = serde_json::from_str(&content)
            .map_err(|e| PersistenceError::Corruption(format!("Failed to deserialize state: {}", e)))?;

        Ok(state)
    }

    async fn delete(&self, dashboard_id: &str) -> PersistenceResult<()> {
        let path = self.get_path(dashboard_id);
        let meta_path = self.get_metadata_path(dashboard_id);

        if path.exists() {
            fs::remove_file(&path)
                .await
                .map_err(|e| PersistenceError::Database(format!("Failed to delete state: {}", e)))?;
        }

        if meta_path.exists() {
            fs::remove_file(&meta_path)
                .await
                .map_err(|e| PersistenceError::Database(format!("Failed to delete metadata: {}", e)))?;
        }

        Ok(())
    }

    async fn exists(&self, dashboard_id: &str) -> PersistenceResult<bool> {
        Ok(self.get_path(dashboard_id).exists())
    }

    async fn list(&self) -> PersistenceResult<Vec<String>> {
        self.ensure_directory().await?;

        let mut entries = fs::read_dir(&self.base_path)
            .await
            .map_err(|e| PersistenceError::Database(format!("Failed to read directory: {}", e)))?;

        let mut dashboards = Vec::new();

        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            PersistenceError::Database(format!("Failed to read directory entry: {}", e))
        })? {
            let path = entry.path();
            if let Some(file_name) = path.file_name() {
                let name = file_name.to_string_lossy();
                if name.ends_with(".json") && !name.ends_with(".meta.json") {
                    let dashboard_id = name.trim_end_matches(".json").to_string();
                    dashboards.push(dashboard_id);
                }
            }
        }

        Ok(dashboards)
    }

    async fn get_metadata(&self, dashboard_id: &str) -> PersistenceResult<PersistenceMetadata> {
        let meta_path = self.get_metadata_path(dashboard_id);

        if !meta_path.exists() {
            return Err(PersistenceError::Database(format!("Metadata not found: {}", dashboard_id)));
        }

        let content = fs::read_to_string(&meta_path)
            .await
            .map_err(|e| PersistenceError::Database(format!("Failed to read metadata: {}", e)))?;

        let metadata: PersistenceMetadata = serde_json::from_str(&content)
            .map_err(|e| PersistenceError::Corruption(format!("Failed to deserialize metadata: {}", e)))?;

        Ok(metadata)
    }

    async fn update_metadata(&self, dashboard_id: &str, metadata: PersistenceMetadata) -> PersistenceResult<()> {
        let meta_path = self.get_metadata_path(dashboard_id);

        let meta_json = serde_json::to_string_pretty(&metadata)
            .map_err(|e| PersistenceError::Database(format!("Metadata serialization failed: {}", e)))?;

        fs::write(&meta_path, meta_json)
            .await
            .map_err(|e| PersistenceError::Database(format!("Failed to write metadata: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DashboardConfig;

    #[tokio::test]
    async fn test_in_memory_storage() {
        let storage = InMemoryStorage::new();
        let config = DashboardConfig::default();
        let state = DashboardState::new(config);
        let dashboard_id = state.config.id.clone();

        // Save
        storage.save(&dashboard_id, &state).await.unwrap();

        // Check exists
        assert!(storage.exists(&dashboard_id).await.unwrap());

        // Load
        let loaded = storage.load(&dashboard_id).await.unwrap();
        assert_eq!(loaded.config.id, dashboard_id);

        // List
        let list = storage.list().await.unwrap();
        assert_eq!(list.len(), 1);

        // Delete
        storage.delete(&dashboard_id).await.unwrap();
        assert!(!storage.exists(&dashboard_id).await.unwrap());
    }

    #[tokio::test]
    async fn test_version_conflict() {
        let storage = InMemoryStorage::new();
        let config = DashboardConfig::default();
        let mut state = DashboardState::new(config);
        let dashboard_id = state.config.id.clone();

        storage.save(&dashboard_id, &state).await.unwrap();

        // Try to save with wrong version
        state.version = 10; // Skip versions
        let result = storage.save(&dashboard_id, &state).await;
        assert!(result.is_err());
    }
}
