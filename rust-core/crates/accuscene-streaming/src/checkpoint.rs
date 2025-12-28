//! Checkpointing for fault tolerance and recovery.

use crate::error::{Result, StreamingError};
use crate::watermark::Watermark;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

/// Checkpoint ID
pub type CheckpointId = u64;

/// Checkpoint metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointMetadata {
    /// Unique checkpoint ID
    pub id: CheckpointId,
    /// When the checkpoint was created
    pub timestamp: DateTime<Utc>,
    /// Watermark at checkpoint time
    pub watermark: Option<Watermark>,
    /// Size in bytes
    pub size_bytes: u64,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl CheckpointMetadata {
    /// Create new checkpoint metadata
    pub fn new(id: CheckpointId) -> Self {
        Self {
            id,
            timestamp: Utc::now(),
            watermark: None,
            size_bytes: 0,
            metadata: HashMap::new(),
        }
    }

    /// Set watermark
    pub fn with_watermark(mut self, watermark: Watermark) -> Self {
        self.watermark = Some(watermark);
        self
    }

    /// Add metadata entry
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Checkpoint data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    /// Metadata
    pub metadata: CheckpointMetadata,
    /// State snapshot
    pub state: HashMap<String, Vec<u8>>,
}

impl Checkpoint {
    /// Create a new checkpoint
    pub fn new(id: CheckpointId) -> Self {
        Self {
            metadata: CheckpointMetadata::new(id),
            state: HashMap::new(),
        }
    }

    /// Add state to checkpoint
    pub fn add_state(&mut self, key: String, data: Vec<u8>) {
        self.metadata.size_bytes += data.len() as u64;
        self.state.insert(key, data);
    }

    /// Get state from checkpoint
    pub fn get_state(&self, key: &str) -> Option<&Vec<u8>> {
        self.state.get(key)
    }

    /// Serialize checkpoint to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self).map_err(|e| {
            StreamingError::Checkpoint(format!("Failed to serialize checkpoint: {}", e))
        })
    }

    /// Deserialize checkpoint from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        serde_json::from_slice(data).map_err(|e| {
            StreamingError::Checkpoint(format!("Failed to deserialize checkpoint: {}", e))
        })
    }
}

/// Checkpoint coordinator
#[derive(Clone)]
pub struct CheckpointCoordinator {
    inner: Arc<RwLock<CheckpointCoordinatorInner>>,
    storage: Arc<dyn CheckpointStorage>,
}

struct CheckpointCoordinatorInner {
    next_id: CheckpointId,
    active_checkpoint: Option<CheckpointId>,
    completed_checkpoints: Vec<CheckpointMetadata>,
    max_retained: usize,
}

impl CheckpointCoordinator {
    /// Create a new checkpoint coordinator
    pub fn new(storage: Arc<dyn CheckpointStorage>, max_retained: usize) -> Self {
        Self {
            inner: Arc::new(RwLock::new(CheckpointCoordinatorInner {
                next_id: 1,
                active_checkpoint: None,
                completed_checkpoints: Vec::new(),
                max_retained,
            })),
            storage,
        }
    }

    /// Trigger a new checkpoint
    pub async fn trigger_checkpoint(&self) -> Result<CheckpointId> {
        let mut inner = self.inner.write().await;

        if inner.active_checkpoint.is_some() {
            return Err(StreamingError::Checkpoint(
                "A checkpoint is already in progress".to_string(),
            ));
        }

        let id = inner.next_id;
        inner.next_id += 1;
        inner.active_checkpoint = Some(id);

        info!("Triggered checkpoint {}", id);
        Ok(id)
    }

    /// Complete a checkpoint
    pub async fn complete_checkpoint(
        &self,
        id: CheckpointId,
        checkpoint: Checkpoint,
    ) -> Result<()> {
        let mut inner = self.inner.write().await;

        if inner.active_checkpoint != Some(id) {
            return Err(StreamingError::Checkpoint(format!(
                "Checkpoint {} is not active",
                id
            )));
        }

        // Store checkpoint
        self.storage.store(&checkpoint).await?;

        // Update metadata
        inner.completed_checkpoints.push(checkpoint.metadata.clone());
        inner.active_checkpoint = None;

        // Cleanup old checkpoints
        if inner.completed_checkpoints.len() > inner.max_retained {
            let to_remove = inner.completed_checkpoints.len() - inner.max_retained;
            for metadata in inner.completed_checkpoints.drain(..to_remove) {
                if let Err(e) = self.storage.delete(metadata.id).await {
                    error!("Failed to delete old checkpoint {}: {}", metadata.id, e);
                }
            }
        }

        info!("Completed checkpoint {}", id);
        Ok(())
    }

    /// Abort a checkpoint
    pub async fn abort_checkpoint(&self, id: CheckpointId) -> Result<()> {
        let mut inner = self.inner.write().await;

        if inner.active_checkpoint == Some(id) {
            inner.active_checkpoint = None;
            info!("Aborted checkpoint {}", id);
            Ok(())
        } else {
            Err(StreamingError::Checkpoint(format!(
                "Checkpoint {} is not active",
                id
            )))
        }
    }

    /// Restore from the latest checkpoint
    pub async fn restore_latest(&self) -> Result<Option<Checkpoint>> {
        let inner = self.inner.read().await;

        if let Some(metadata) = inner.completed_checkpoints.last() {
            let checkpoint = self.storage.load(metadata.id).await?;
            info!("Restored from checkpoint {}", metadata.id);
            Ok(Some(checkpoint))
        } else {
            Ok(None)
        }
    }

    /// Get all checkpoint metadata
    pub async fn list_checkpoints(&self) -> Vec<CheckpointMetadata> {
        self.inner.read().await.completed_checkpoints.clone()
    }

    /// Get the latest checkpoint metadata
    pub async fn get_latest_metadata(&self) -> Option<CheckpointMetadata> {
        self.inner
            .read()
            .await
            .completed_checkpoints
            .last()
            .cloned()
    }
}

/// Trait for checkpoint storage backends
#[async_trait::async_trait]
pub trait CheckpointStorage: Send + Sync {
    /// Store a checkpoint
    async fn store(&self, checkpoint: &Checkpoint) -> Result<()>;

    /// Load a checkpoint
    async fn load(&self, id: CheckpointId) -> Result<Checkpoint>;

    /// Delete a checkpoint
    async fn delete(&self, id: CheckpointId) -> Result<()>;

    /// List all checkpoints
    async fn list(&self) -> Result<Vec<CheckpointMetadata>>;
}

/// File-based checkpoint storage
pub struct FileCheckpointStorage {
    base_path: PathBuf,
}

impl FileCheckpointStorage {
    /// Create a new file checkpoint storage
    pub async fn new<P: AsRef<Path>>(base_path: P) -> Result<Self> {
        let base_path = base_path.as_ref().to_path_buf();

        // Create directory if it doesn't exist
        fs::create_dir_all(&base_path).await.map_err(|e| {
            StreamingError::Checkpoint(format!("Failed to create checkpoint directory: {}", e))
        })?;

        Ok(Self { base_path })
    }

    fn checkpoint_path(&self, id: CheckpointId) -> PathBuf {
        self.base_path.join(format!("checkpoint-{}.json", id))
    }

    fn metadata_path(&self, id: CheckpointId) -> PathBuf {
        self.base_path.join(format!("checkpoint-{}.meta.json", id))
    }
}

#[async_trait::async_trait]
impl CheckpointStorage for FileCheckpointStorage {
    async fn store(&self, checkpoint: &Checkpoint) -> Result<()> {
        let checkpoint_path = self.checkpoint_path(checkpoint.metadata.id);
        let metadata_path = self.metadata_path(checkpoint.metadata.id);

        // Write checkpoint data
        let data = checkpoint.to_bytes()?;
        fs::write(&checkpoint_path, &data).await.map_err(|e| {
            StreamingError::Checkpoint(format!("Failed to write checkpoint: {}", e))
        })?;

        // Write metadata
        let metadata_json = serde_json::to_vec_pretty(&checkpoint.metadata).map_err(|e| {
            StreamingError::Checkpoint(format!("Failed to serialize metadata: {}", e))
        })?;
        fs::write(&metadata_path, metadata_json)
            .await
            .map_err(|e| {
                StreamingError::Checkpoint(format!("Failed to write metadata: {}", e))
            })?;

        debug!(
            "Stored checkpoint {} ({} bytes)",
            checkpoint.metadata.id, data.len()
        );
        Ok(())
    }

    async fn load(&self, id: CheckpointId) -> Result<Checkpoint> {
        let checkpoint_path = self.checkpoint_path(id);

        let data = fs::read(&checkpoint_path).await.map_err(|e| {
            StreamingError::CheckpointRestore(format!("Failed to read checkpoint: {}", e))
        })?;

        Checkpoint::from_bytes(&data)
    }

    async fn delete(&self, id: CheckpointId) -> Result<()> {
        let checkpoint_path = self.checkpoint_path(id);
        let metadata_path = self.metadata_path(id);

        // Delete files, ignore errors if they don't exist
        let _ = fs::remove_file(&checkpoint_path).await;
        let _ = fs::remove_file(&metadata_path).await;

        debug!("Deleted checkpoint {}", id);
        Ok(())
    }

    async fn list(&self) -> Result<Vec<CheckpointMetadata>> {
        let mut entries = fs::read_dir(&self.base_path).await.map_err(|e| {
            StreamingError::Checkpoint(format!("Failed to read checkpoint directory: {}", e))
        })?;

        let mut checkpoints = Vec::new();

        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            StreamingError::Checkpoint(format!("Failed to read directory entry: {}", e))
        })? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json")
                && path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.starts_with("checkpoint-"))
                    .unwrap_or(false)
            {
                let data = fs::read(&path).await.map_err(|e| {
                    StreamingError::Checkpoint(format!("Failed to read checkpoint: {}", e))
                })?;

                if let Ok(checkpoint) = Checkpoint::from_bytes(&data) {
                    checkpoints.push(checkpoint.metadata);
                }
            }
        }

        checkpoints.sort_by_key(|m| m.id);
        Ok(checkpoints)
    }
}

/// In-memory checkpoint storage (for testing)
pub struct MemoryCheckpointStorage {
    checkpoints: Arc<RwLock<HashMap<CheckpointId, Checkpoint>>>,
}

impl MemoryCheckpointStorage {
    /// Create a new memory checkpoint storage
    pub fn new() -> Self {
        Self {
            checkpoints: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for MemoryCheckpointStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl CheckpointStorage for MemoryCheckpointStorage {
    async fn store(&self, checkpoint: &Checkpoint) -> Result<()> {
        let mut checkpoints = self.checkpoints.write().await;
        checkpoints.insert(checkpoint.metadata.id, checkpoint.clone());
        Ok(())
    }

    async fn load(&self, id: CheckpointId) -> Result<Checkpoint> {
        let checkpoints = self.checkpoints.read().await;
        checkpoints
            .get(&id)
            .cloned()
            .ok_or_else(|| StreamingError::CheckpointRestore(format!("Checkpoint {} not found", id)))
    }

    async fn delete(&self, id: CheckpointId) -> Result<()> {
        let mut checkpoints = self.checkpoints.write().await;
        checkpoints.remove(&id);
        Ok(())
    }

    async fn list(&self) -> Result<Vec<CheckpointMetadata>> {
        let checkpoints = self.checkpoints.read().await;
        let mut metadata: Vec<_> = checkpoints
            .values()
            .map(|c| c.metadata.clone())
            .collect();
        metadata.sort_by_key(|m| m.id);
        Ok(metadata)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_checkpoint_coordinator() {
        let storage = Arc::new(MemoryCheckpointStorage::new());
        let coordinator = CheckpointCoordinator::new(storage, 3);

        // Trigger checkpoint
        let id1 = coordinator.trigger_checkpoint().await.unwrap();
        assert_eq!(id1, 1);

        // Complete checkpoint
        let mut checkpoint = Checkpoint::new(id1);
        checkpoint.add_state("key1".to_string(), vec![1, 2, 3]);
        coordinator.complete_checkpoint(id1, checkpoint).await.unwrap();

        // Restore latest
        let restored = coordinator.restore_latest().await.unwrap().unwrap();
        assert_eq!(restored.metadata.id, 1);
        assert_eq!(restored.get_state("key1").unwrap(), &vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn test_checkpoint_retention() {
        let storage = Arc::new(MemoryCheckpointStorage::new());
        let coordinator = CheckpointCoordinator::new(storage.clone(), 2);

        // Create 3 checkpoints
        for i in 1..=3 {
            let id = coordinator.trigger_checkpoint().await.unwrap();
            let checkpoint = Checkpoint::new(id);
            coordinator.complete_checkpoint(id, checkpoint).await.unwrap();
        }

        // Only last 2 should be retained
        let checkpoints = coordinator.list_checkpoints().await;
        assert_eq!(checkpoints.len(), 2);
        assert_eq!(checkpoints[0].id, 2);
        assert_eq!(checkpoints[1].id, 3);
    }
}
