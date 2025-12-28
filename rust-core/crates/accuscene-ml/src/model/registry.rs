//! Model registry for managing models and versions

use crate::error::{MLError, Result};
use crate::model::artifact::{ArtifactStore, ModelArtifact};
use crate::model::metadata::{ModelMetadata, ModelStatus};
use dashmap::DashMap;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

/// Model registry for managing models
pub struct ModelRegistry {
    /// In-memory model metadata cache
    metadata_cache: Arc<DashMap<Uuid, ModelMetadata>>,

    /// Model name to ID mapping
    name_to_id: Arc<DashMap<String, Vec<Uuid>>>,

    /// Artifact store
    artifact_store: Arc<ArtifactStore>,

    /// Registry lock for operations
    operation_lock: Arc<RwLock<()>>,
}

impl ModelRegistry {
    /// Create a new model registry
    pub fn new(storage_path: impl Into<PathBuf>) -> Result<Self> {
        let artifact_store = Arc::new(ArtifactStore::new(storage_path)?);

        let registry = Self {
            metadata_cache: Arc::new(DashMap::new()),
            name_to_id: Arc::new(DashMap::new()),
            artifact_store,
            operation_lock: Arc::new(RwLock::new(())),
        };

        // Load existing models
        registry.load_existing_models()?;

        Ok(registry)
    }

    /// Register a new model
    pub fn register(&self, metadata: ModelMetadata, artifact: ModelArtifact) -> Result<Uuid> {
        let _lock = self.operation_lock.write();

        let id = metadata.id;

        // Check if model already exists
        if self.metadata_cache.contains_key(&id) {
            return Err(MLError::ModelAlreadyExists(id.to_string()));
        }

        // Save artifact
        self.artifact_store.save(&artifact)?;

        // Cache metadata
        self.metadata_cache.insert(id, metadata.clone());

        // Update name mapping
        self.name_to_id
            .entry(metadata.name.clone())
            .or_insert_with(Vec::new)
            .push(id);

        Ok(id)
    }

    /// Get model metadata by ID
    pub fn get_metadata(&self, id: &Uuid) -> Result<ModelMetadata> {
        self.metadata_cache
            .get(id)
            .map(|entry| entry.clone())
            .ok_or_else(|| MLError::ModelNotFound(id.to_string()))
    }

    /// Get model artifact by ID
    pub fn get_artifact(&self, id: &Uuid) -> Result<ModelArtifact> {
        if !self.metadata_cache.contains_key(id) {
            return Err(MLError::ModelNotFound(id.to_string()));
        }
        self.artifact_store.load(id)
    }

    /// Update model metadata
    pub fn update_metadata(&self, id: &Uuid, metadata: ModelMetadata) -> Result<()> {
        let _lock = self.operation_lock.write();

        if !self.metadata_cache.contains_key(id) {
            return Err(MLError::ModelNotFound(id.to_string()));
        }

        self.metadata_cache.insert(*id, metadata);
        Ok(())
    }

    /// Update model status
    pub fn update_status(&self, id: &Uuid, status: ModelStatus) -> Result<()> {
        let mut metadata = self.get_metadata(id)?;
        metadata.set_status(status);
        self.update_metadata(id, metadata)
    }

    /// Delete a model
    pub fn delete(&self, id: &Uuid) -> Result<()> {
        let _lock = self.operation_lock.write();

        // Get metadata to update name mapping
        if let Some((_, metadata)) = self.metadata_cache.remove(id) {
            // Remove from name mapping
            if let Some(mut ids) = self.name_to_id.get_mut(&metadata.name) {
                ids.retain(|&model_id| model_id != *id);
            }
        }

        // Delete artifact
        self.artifact_store.delete(id)?;

        Ok(())
    }

    /// List all model IDs
    pub fn list_models(&self) -> Vec<Uuid> {
        self.metadata_cache
            .iter()
            .map(|entry| *entry.key())
            .collect()
    }

    /// List models by name
    pub fn list_by_name(&self, name: &str) -> Vec<Uuid> {
        self.name_to_id
            .get(name)
            .map(|entry| entry.clone())
            .unwrap_or_default()
    }

    /// Get latest model by name
    pub fn get_latest(&self, name: &str) -> Result<(Uuid, ModelMetadata)> {
        let ids = self.list_by_name(name);

        if ids.is_empty() {
            return Err(MLError::ModelNotFound(name.to_string()));
        }

        // Find the most recently updated model
        let mut latest_id = ids[0];
        let mut latest_metadata = self.get_metadata(&latest_id)?;

        for id in ids.iter().skip(1) {
            let metadata = self.get_metadata(id)?;
            if metadata.updated_at > latest_metadata.updated_at {
                latest_id = *id;
                latest_metadata = metadata;
            }
        }

        Ok((latest_id, latest_metadata))
    }

    /// Get model by name and version
    pub fn get_by_version(&self, name: &str, version: &str) -> Result<(Uuid, ModelMetadata)> {
        let ids = self.list_by_name(name);

        for id in ids {
            let metadata = self.get_metadata(&id)?;
            if metadata.version == version {
                return Ok((id, metadata));
            }
        }

        Err(MLError::ModelNotFound(format!("{} v{}", name, version)))
    }

    /// List ready models
    pub fn list_ready(&self) -> Vec<(Uuid, ModelMetadata)> {
        self.metadata_cache
            .iter()
            .filter(|entry| entry.value().status == ModelStatus::Ready)
            .map(|entry| (*entry.key(), entry.value().clone()))
            .collect()
    }

    /// Search models by tags
    pub fn search_by_tags(&self, tags: &[String]) -> Vec<(Uuid, ModelMetadata)> {
        self.metadata_cache
            .iter()
            .filter(|entry| {
                tags.iter().all(|tag| entry.value().tags.contains(tag))
            })
            .map(|entry| (*entry.key(), entry.value().clone()))
            .collect()
    }

    /// Get registry statistics
    pub fn stats(&self) -> RegistryStats {
        let total_models = self.metadata_cache.len();
        let mut status_counts: HashMap<String, usize> = HashMap::new();
        let mut type_counts: HashMap<String, usize> = HashMap::new();

        for entry in self.metadata_cache.iter() {
            let metadata = entry.value();

            // Count by status
            let status_key = format!("{:?}", metadata.status);
            *status_counts.entry(status_key).or_insert(0) += 1;

            // Count by type
            let type_key = format!("{:?}", metadata.model_type);
            *type_counts.entry(type_key).or_insert(0) += 1;
        }

        RegistryStats {
            total_models,
            status_counts,
            type_counts,
        }
    }

    /// Load existing models from storage
    fn load_existing_models(&self) -> Result<()> {
        let artifact_ids = self.artifact_store.list()?;

        for id in artifact_ids {
            if let Ok(artifact) = self.artifact_store.load(&id) {
                // Try to reconstruct metadata from artifact
                // In a real implementation, metadata would be stored separately
                // For now, we'll skip loading if metadata is not available
                continue;
            }
        }

        Ok(())
    }
}

/// Registry statistics
#[derive(Debug, Clone)]
pub struct RegistryStats {
    /// Total number of models
    pub total_models: usize,

    /// Count by status
    pub status_counts: HashMap<String, usize>,

    /// Count by type
    pub type_counts: HashMap<String, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::metadata::ModelType;
    use tempfile::TempDir;

    #[test]
    fn test_registry_creation() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let registry = ModelRegistry::new(temp_dir.path())?;
        assert_eq!(registry.list_models().len(), 0);
        Ok(())
    }

    #[test]
    fn test_register_model() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let registry = ModelRegistry::new(temp_dir.path())?;

        let metadata = ModelMetadata::new(
            "test_model",
            "1.0.0",
            ModelType::RandomForest,
        );
        let id = metadata.id;
        let artifact = ModelArtifact::new(id, vec![1, 2, 3]);

        registry.register(metadata, artifact)?;

        assert_eq!(registry.list_models().len(), 1);
        assert!(registry.get_metadata(&id).is_ok());

        Ok(())
    }

    #[test]
    fn test_get_latest() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let registry = ModelRegistry::new(temp_dir.path())?;

        // Register v1
        let metadata1 = ModelMetadata::new(
            "test_model",
            "1.0.0",
            ModelType::RandomForest,
        );
        let id1 = metadata1.id;
        let artifact1 = ModelArtifact::new(id1, vec![1]);
        registry.register(metadata1, artifact1)?;

        std::thread::sleep(std::time::Duration::from_millis(10));

        // Register v2
        let metadata2 = ModelMetadata::new(
            "test_model",
            "2.0.0",
            ModelType::RandomForest,
        );
        let id2 = metadata2.id;
        let artifact2 = ModelArtifact::new(id2, vec![2]);
        registry.register(metadata2, artifact2)?;

        let (latest_id, latest_metadata) = registry.get_latest("test_model")?;
        assert_eq!(latest_id, id2);
        assert_eq!(latest_metadata.version, "2.0.0");

        Ok(())
    }
}
