//! Model artifact storage and management

use crate::error::{MLError, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Model artifact storage
pub struct ArtifactStore {
    base_path: PathBuf,
}

impl ArtifactStore {
    /// Create a new artifact store
    pub fn new(base_path: impl Into<PathBuf>) -> Result<Self> {
        let base_path = base_path.into();
        fs::create_dir_all(&base_path)?;
        Ok(Self { base_path })
    }

    /// Save a model artifact
    pub fn save(&self, artifact: &ModelArtifact) -> Result<PathBuf> {
        let artifact_path = self.artifact_path(&artifact.id);
        fs::create_dir_all(&artifact_path)?;

        // Save metadata
        let metadata_path = artifact_path.join("metadata.json");
        let metadata_json = serde_json::to_string_pretty(&artifact.metadata)?;
        fs::write(&metadata_path, metadata_json)?;

        // Save model data
        let model_path = artifact_path.join("model.bin");
        fs::write(&model_path, &artifact.data)?;

        // Save additional files
        for (name, content) in &artifact.files {
            let file_path = artifact_path.join(name);
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&file_path, content)?;
        }

        Ok(artifact_path)
    }

    /// Load a model artifact
    pub fn load(&self, id: &Uuid) -> Result<ModelArtifact> {
        let artifact_path = self.artifact_path(id);

        if !artifact_path.exists() {
            return Err(MLError::ModelNotFound(id.to_string()));
        }

        // Load metadata
        let metadata_path = artifact_path.join("metadata.json");
        let metadata_json = fs::read_to_string(&metadata_path)?;
        let metadata = serde_json::from_str(&metadata_json)?;

        // Load model data
        let model_path = artifact_path.join("model.bin");
        let data = fs::read(&model_path)?;

        // Load additional files
        let mut files = std::collections::HashMap::new();
        for entry in fs::read_dir(&artifact_path)? {
            let entry = entry?;
            let path = entry.path();
            let file_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            if file_name != "metadata.json" && file_name != "model.bin" && path.is_file() {
                files.insert(file_name, fs::read(&path)?);
            }
        }

        Ok(ModelArtifact {
            id: *id,
            metadata,
            data,
            files,
        })
    }

    /// Delete a model artifact
    pub fn delete(&self, id: &Uuid) -> Result<()> {
        let artifact_path = self.artifact_path(id);
        if artifact_path.exists() {
            fs::remove_dir_all(&artifact_path)?;
        }
        Ok(())
    }

    /// List all artifacts
    pub fn list(&self) -> Result<Vec<Uuid>> {
        let mut artifacts = Vec::new();

        if !self.base_path.exists() {
            return Ok(artifacts);
        }

        for entry in fs::read_dir(&self.base_path)? {
            let entry = entry?;
            if entry.path().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    if let Ok(id) = Uuid::parse_str(name) {
                        artifacts.push(id);
                    }
                }
            }
        }

        Ok(artifacts)
    }

    /// Check if artifact exists
    pub fn exists(&self, id: &Uuid) -> bool {
        self.artifact_path(id).exists()
    }

    /// Get artifact path
    fn artifact_path(&self, id: &Uuid) -> PathBuf {
        self.base_path.join(id.to_string())
    }

    /// Get artifact size
    pub fn size(&self, id: &Uuid) -> Result<u64> {
        let artifact_path = self.artifact_path(id);
        if !artifact_path.exists() {
            return Err(MLError::ModelNotFound(id.to_string()));
        }

        let mut total_size = 0u64;
        for entry in fs::read_dir(&artifact_path)? {
            let entry = entry?;
            if entry.path().is_file() {
                total_size += entry.metadata()?.len();
            }
        }

        Ok(total_size)
    }
}

/// Model artifact
#[derive(Debug, Clone)]
pub struct ModelArtifact {
    /// Artifact ID
    pub id: Uuid,

    /// Artifact metadata
    pub metadata: ArtifactMetadata,

    /// Model binary data
    pub data: Vec<u8>,

    /// Additional files (e.g., scalers, encoders, config)
    pub files: std::collections::HashMap<String, Vec<u8>>,
}

impl ModelArtifact {
    /// Create a new model artifact
    pub fn new(id: Uuid, data: Vec<u8>) -> Self {
        Self {
            id,
            metadata: ArtifactMetadata::new(),
            data,
            files: std::collections::HashMap::new(),
        }
    }

    /// Add a file to the artifact
    pub fn add_file(&mut self, name: impl Into<String>, content: Vec<u8>) {
        self.files.insert(name.into(), content);
    }

    /// Get a file from the artifact
    pub fn get_file(&self, name: &str) -> Option<&Vec<u8>> {
        self.files.get(name)
    }

    /// Get artifact size in bytes
    pub fn size(&self) -> usize {
        let mut total = self.data.len();
        for content in self.files.values() {
            total += content.len();
        }
        total
    }
}

/// Artifact metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactMetadata {
    /// Format version
    pub format_version: String,

    /// Compression type
    pub compression: CompressionType,

    /// Checksum
    pub checksum: String,

    /// Additional metadata
    pub metadata: std::collections::HashMap<String, String>,
}

impl ArtifactMetadata {
    /// Create new artifact metadata
    pub fn new() -> Self {
        Self {
            format_version: String::from("1.0"),
            compression: CompressionType::None,
            checksum: String::new(),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Set compression type
    pub fn with_compression(mut self, compression: CompressionType) -> Self {
        self.compression = compression;
        self
    }

    /// Set checksum
    pub fn with_checksum(mut self, checksum: impl Into<String>) -> Self {
        self.checksum = checksum.into();
        self
    }
}

impl Default for ArtifactMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// Compression type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CompressionType {
    None,
    Gzip,
    Zstd,
    Lz4,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_artifact_creation() {
        let id = Uuid::new_v4();
        let data = vec![1, 2, 3, 4, 5];
        let artifact = ModelArtifact::new(id, data.clone());

        assert_eq!(artifact.id, id);
        assert_eq!(artifact.data, data);
        assert_eq!(artifact.size(), 5);
    }

    #[test]
    fn test_artifact_store() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let store = ArtifactStore::new(temp_dir.path())?;

        let id = Uuid::new_v4();
        let mut artifact = ModelArtifact::new(id, vec![1, 2, 3]);
        artifact.add_file("config.json", b"{\"test\": true}".to_vec());

        store.save(&artifact)?;

        let loaded = store.load(&id)?;
        assert_eq!(loaded.id, id);
        assert_eq!(loaded.data, vec![1, 2, 3]);
        assert!(loaded.get_file("config.json").is_some());

        Ok(())
    }
}
