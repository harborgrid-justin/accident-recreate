//! Model management module

use crate::error::Result;
use async_trait::async_trait;
use ndarray::{Array1, Array2};
use serde::{Deserialize, Serialize};

pub mod artifact;
pub mod metadata;
pub mod registry;

pub use artifact::{ArtifactStore, ModelArtifact};
pub use metadata::{FeatureInfo, FeatureType, ModelMetadata, ModelStatus, ModelType, TargetInfo, TargetType};
pub use registry::{ModelRegistry, RegistryStats};

/// Trait for machine learning models
#[async_trait]
pub trait Model: Send + Sync {
    /// Get model metadata
    fn metadata(&self) -> &ModelMetadata;

    /// Train the model
    async fn train(&mut self, features: &Array2<f64>, targets: &Array1<f64>) -> Result<()>;

    /// Predict single sample
    async fn predict(&self, features: &Array1<f64>) -> Result<f64>;

    /// Predict batch of samples
    async fn predict_batch(&self, features: &Array2<f64>) -> Result<Array1<f64>>;

    /// Get feature importances (if available)
    fn feature_importances(&self) -> Option<Vec<f64>> {
        None
    }

    /// Serialize model to bytes
    fn to_bytes(&self) -> Result<Vec<u8>>;

    /// Deserialize model from bytes
    fn from_bytes(bytes: &[u8]) -> Result<Self>
    where
        Self: Sized;
}

/// Trait for classification models
#[async_trait]
pub trait Classifier: Model {
    /// Predict class probabilities
    async fn predict_proba(&self, features: &Array1<f64>) -> Result<Vec<f64>>;

    /// Predict class probabilities for batch
    async fn predict_proba_batch(&self, features: &Array2<f64>) -> Result<Array2<f64>>;

    /// Get number of classes
    fn num_classes(&self) -> usize;
}

/// Trait for clustering models
#[async_trait]
pub trait Clusterer: Send + Sync {
    /// Fit the clustering model
    async fn fit(&mut self, features: &Array2<f64>) -> Result<()>;

    /// Predict cluster labels
    async fn predict(&self, features: &Array2<f64>) -> Result<Array1<usize>>;

    /// Get cluster centers (if available)
    fn cluster_centers(&self) -> Option<Array2<f64>> {
        None
    }

    /// Get number of clusters
    fn num_clusters(&self) -> usize;
}

/// Model lifecycle manager
pub struct ModelLifecycle {
    /// Model metadata
    metadata: ModelMetadata,

    /// Lifecycle state
    state: LifecycleState,

    /// Training history
    history: Vec<TrainingEpoch>,
}

impl ModelLifecycle {
    /// Create a new model lifecycle
    pub fn new(metadata: ModelMetadata) -> Self {
        Self {
            metadata,
            state: LifecycleState::Created,
            history: Vec::new(),
        }
    }

    /// Start training
    pub fn start_training(&mut self) {
        self.state = LifecycleState::Training;
        self.metadata.set_status(ModelStatus::Training);
    }

    /// Record training epoch
    pub fn record_epoch(&mut self, epoch: TrainingEpoch) {
        self.history.push(epoch);
    }

    /// Complete training
    pub fn complete_training(&mut self, success: bool) {
        if success {
            self.state = LifecycleState::Trained;
            self.metadata.set_status(ModelStatus::Validating);
        } else {
            self.state = LifecycleState::Failed;
            self.metadata.set_status(ModelStatus::Failed);
        }
    }

    /// Deploy model
    pub fn deploy(&mut self) {
        self.state = LifecycleState::Deployed;
        self.metadata.set_status(ModelStatus::Ready);
    }

    /// Deprecate model
    pub fn deprecate(&mut self) {
        self.state = LifecycleState::Deprecated;
        self.metadata.set_status(ModelStatus::Deprecated);
    }

    /// Get current state
    pub fn state(&self) -> &LifecycleState {
        &self.state
    }

    /// Get metadata
    pub fn metadata(&self) -> &ModelMetadata {
        &self.metadata
    }

    /// Get training history
    pub fn history(&self) -> &[TrainingEpoch] {
        &self.history
    }
}

/// Lifecycle state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifecycleState {
    Created,
    Training,
    Trained,
    Validating,
    Validated,
    Deployed,
    Deprecated,
    Failed,
}

/// Training epoch information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingEpoch {
    /// Epoch number
    pub epoch: usize,

    /// Training loss
    pub train_loss: f64,

    /// Validation loss (if available)
    pub val_loss: Option<f64>,

    /// Additional metrics
    pub metrics: std::collections::HashMap<String, f64>,

    /// Training time (milliseconds)
    pub duration_ms: u64,
}

impl TrainingEpoch {
    /// Create a new training epoch
    pub fn new(epoch: usize, train_loss: f64) -> Self {
        Self {
            epoch,
            train_loss,
            val_loss: None,
            metrics: std::collections::HashMap::new(),
            duration_ms: 0,
        }
    }

    /// Set validation loss
    pub fn with_val_loss(mut self, val_loss: f64) -> Self {
        self.val_loss = Some(val_loss);
        self
    }

    /// Add a metric
    pub fn add_metric(&mut self, name: impl Into<String>, value: f64) {
        self.metrics.insert(name.into(), value);
    }

    /// Set duration
    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = duration_ms;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lifecycle() {
        let metadata = ModelMetadata::new(
            "test_model",
            "1.0.0",
            ModelType::LinearRegression,
        );

        let mut lifecycle = ModelLifecycle::new(metadata);
        assert_eq!(lifecycle.state(), &LifecycleState::Created);

        lifecycle.start_training();
        assert_eq!(lifecycle.state(), &LifecycleState::Training);

        lifecycle.complete_training(true);
        assert_eq!(lifecycle.state(), &LifecycleState::Trained);

        lifecycle.deploy();
        assert_eq!(lifecycle.state(), &LifecycleState::Deployed);
    }

    #[test]
    fn test_training_epoch() {
        let mut epoch = TrainingEpoch::new(1, 0.5)
            .with_val_loss(0.6)
            .with_duration(100);

        epoch.add_metric("accuracy", 0.85);

        assert_eq!(epoch.epoch, 1);
        assert_eq!(epoch.train_loss, 0.5);
        assert_eq!(epoch.val_loss, Some(0.6));
        assert_eq!(epoch.metrics.get("accuracy"), Some(&0.85));
        assert_eq!(epoch.duration_ms, 100);
    }
}
