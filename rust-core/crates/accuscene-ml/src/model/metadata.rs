//! Model metadata management

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Model metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    /// Unique model ID
    pub id: Uuid,

    /// Model name
    pub name: String,

    /// Model version
    pub version: String,

    /// Model type/algorithm
    pub model_type: ModelType,

    /// Model description
    pub description: String,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,

    /// Model author/creator
    pub author: String,

    /// Training metrics
    pub training_metrics: HashMap<String, f64>,

    /// Hyperparameters
    pub hyperparameters: HashMap<String, serde_json::Value>,

    /// Feature names and types
    pub features: Vec<FeatureInfo>,

    /// Target variable information
    pub target: TargetInfo,

    /// Model status
    pub status: ModelStatus,

    /// Custom tags
    pub tags: Vec<String>,

    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ModelMetadata {
    /// Create new model metadata
    pub fn new(name: impl Into<String>, version: impl Into<String>, model_type: ModelType) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            version: version.into(),
            model_type,
            description: String::new(),
            created_at: now,
            updated_at: now,
            author: String::from("system"),
            training_metrics: HashMap::new(),
            hyperparameters: HashMap::new(),
            features: Vec::new(),
            target: TargetInfo::default(),
            status: ModelStatus::Training,
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Update the model status
    pub fn set_status(&mut self, status: ModelStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }

    /// Add a training metric
    pub fn add_metric(&mut self, name: impl Into<String>, value: f64) {
        self.training_metrics.insert(name.into(), value);
        self.updated_at = Utc::now();
    }

    /// Add a hyperparameter
    pub fn add_hyperparameter(&mut self, name: impl Into<String>, value: serde_json::Value) {
        self.hyperparameters.insert(name.into(), value);
        self.updated_at = Utc::now();
    }

    /// Add a feature
    pub fn add_feature(&mut self, feature: FeatureInfo) {
        self.features.push(feature);
        self.updated_at = Utc::now();
    }

    /// Add a tag
    pub fn add_tag(&mut self, tag: impl Into<String>) {
        self.tags.push(tag.into());
        self.updated_at = Utc::now();
    }

    /// Check if model is ready for inference
    pub fn is_ready(&self) -> bool {
        matches!(self.status, ModelStatus::Ready)
    }
}

/// Model type/algorithm
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModelType {
    LinearRegression,
    RidgeRegression,
    LassoRegression,
    LogisticRegression,
    SVM,
    DecisionTree,
    RandomForest,
    GradientBoosting,
    KMeans,
    DBSCAN,
    NeuralNetwork,
    ONNX,
    Custom(String),
}

/// Feature information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureInfo {
    /// Feature name
    pub name: String,

    /// Feature type
    pub feature_type: FeatureType,

    /// Feature importance (if available)
    pub importance: Option<f64>,

    /// Description
    pub description: String,
}

impl FeatureInfo {
    /// Create new feature info
    pub fn new(name: impl Into<String>, feature_type: FeatureType) -> Self {
        Self {
            name: name.into(),
            feature_type,
            importance: None,
            description: String::new(),
        }
    }

    /// Set feature importance
    pub fn with_importance(mut self, importance: f64) -> Self {
        self.importance = Some(importance);
        self
    }

    /// Set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }
}

/// Feature type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FeatureType {
    Numerical,
    Categorical,
    Binary,
    Text,
    Temporal,
    Custom(String),
}

/// Target variable information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetInfo {
    /// Target name
    pub name: String,

    /// Target type
    pub target_type: TargetType,

    /// Classes (for classification)
    pub classes: Vec<String>,

    /// Description
    pub description: String,
}

impl Default for TargetInfo {
    fn default() -> Self {
        Self {
            name: String::from("target"),
            target_type: TargetType::Regression,
            classes: Vec::new(),
            description: String::new(),
        }
    }
}

/// Target type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TargetType {
    Regression,
    BinaryClassification,
    MultiClassification,
    Clustering,
}

/// Model status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModelStatus {
    Training,
    Validating,
    Ready,
    Deprecated,
    Failed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_metadata_creation() {
        let metadata = ModelMetadata::new(
            "test_model",
            "1.0.0",
            ModelType::RandomForest,
        );

        assert_eq!(metadata.name, "test_model");
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.model_type, ModelType::RandomForest);
        assert_eq!(metadata.status, ModelStatus::Training);
    }

    #[test]
    fn test_add_metrics() {
        let mut metadata = ModelMetadata::new(
            "test_model",
            "1.0.0",
            ModelType::LinearRegression,
        );

        metadata.add_metric("rmse", 0.123);
        metadata.add_metric("r2", 0.95);

        assert_eq!(metadata.training_metrics.get("rmse"), Some(&0.123));
        assert_eq!(metadata.training_metrics.get("r2"), Some(&0.95));
    }

    #[test]
    fn test_feature_info() {
        let feature = FeatureInfo::new("age", FeatureType::Numerical)
            .with_importance(0.8)
            .with_description("User age in years");

        assert_eq!(feature.name, "age");
        assert_eq!(feature.feature_type, FeatureType::Numerical);
        assert_eq!(feature.importance, Some(0.8));
        assert_eq!(feature.description, "User age in years");
    }
}
