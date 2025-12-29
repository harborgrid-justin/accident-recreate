//! Machine learning models for accident reconstruction

pub mod damage_analyzer;
pub mod impact_classifier;
pub mod occupant_risk;
pub mod speed_estimator;
pub mod trajectory_predictor;

use crate::error::{MlError, Result};
use crate::inference::onnx_runtime::OnnxModel;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
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

    /// Model type
    pub model_type: ModelType,

    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Model file path
    pub model_path: PathBuf,

    /// Input feature names
    pub input_features: Vec<String>,

    /// Output feature names
    pub output_features: Vec<String>,

    /// Model performance metrics
    pub metrics: ModelMetrics,

    /// Model description
    pub description: Option<String>,

    /// Training dataset info
    pub training_info: TrainingInfo,
}

/// Type of model
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModelType {
    /// Speed estimation model
    SpeedEstimator,
    /// Impact type classifier
    ImpactClassifier,
    /// Trajectory prediction model
    TrajectoryPredictor,
    /// Damage analysis model
    DamageAnalyzer,
    /// Occupant risk model
    OccupantRisk,
}

/// Model performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetrics {
    /// Accuracy (0.0 - 1.0)
    pub accuracy: f64,

    /// Precision (0.0 - 1.0)
    pub precision: f64,

    /// Recall (0.0 - 1.0)
    pub recall: f64,

    /// F1 score (0.0 - 1.0)
    pub f1_score: f64,

    /// Mean absolute error (for regression)
    pub mae: Option<f64>,

    /// Root mean squared error (for regression)
    pub rmse: Option<f64>,

    /// R² score (for regression)
    pub r2_score: Option<f64>,
}

/// Training information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingInfo {
    /// Number of training samples
    pub num_samples: usize,

    /// Number of epochs trained
    pub epochs: usize,

    /// Learning rate used
    pub learning_rate: f64,

    /// Training duration in seconds
    pub training_duration_secs: u64,

    /// Dataset name/version
    pub dataset: String,

    /// Cross-validation scores
    pub cv_scores: Option<Vec<f64>>,
}

/// Prediction result with confidence interval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prediction<T> {
    /// Predicted value
    pub value: T,

    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,

    /// Lower bound of confidence interval
    pub lower_bound: Option<T>,

    /// Upper bound of confidence interval
    pub upper_bound: Option<T>,

    /// Model version used
    pub model_version: String,

    /// Prediction timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Feature importance (optional)
    pub feature_importance: Option<Vec<(String, f64)>>,
}

/// Base trait for all models
pub trait Model: Send + Sync {
    /// Get model metadata
    fn metadata(&self) -> &ModelMetadata;

    /// Get model type
    fn model_type(&self) -> ModelType {
        self.metadata().model_type.clone()
    }

    /// Get model version
    fn version(&self) -> &str {
        &self.metadata().version
    }

    /// Validate input features
    fn validate_input(&self, features: &[f64]) -> Result<()> {
        let expected = self.metadata().input_features.len();
        let actual = features.len();

        if expected != actual {
            return Err(MlError::ShapeMismatch {
                expected: vec![expected],
                actual: vec![actual],
            });
        }

        // Check for NaN or infinite values
        for (i, &value) in features.iter().enumerate() {
            if !value.is_finite() {
                return Err(MlError::InvalidInput(format!(
                    "Feature {} has invalid value: {}",
                    i, value
                )));
            }
        }

        Ok(())
    }
}

/// Model registry for managing multiple models
#[derive(Debug)]
pub struct ModelRegistry {
    models: std::collections::HashMap<String, Box<dyn std::any::Any + Send + Sync>>,
    base_path: PathBuf,
}

impl ModelRegistry {
    /// Create a new model registry
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        Self {
            models: std::collections::HashMap::new(),
            base_path: base_path.as_ref().to_path_buf(),
        }
    }

    /// Register a model with a name
    pub fn register<T: 'static + Send + Sync>(&mut self, name: String, model: T) {
        self.models.insert(name, Box::new(model));
    }

    /// Get a model by name
    pub fn get<T: 'static>(&self, name: &str) -> Option<&T> {
        self.models
            .get(name)
            .and_then(|m| m.downcast_ref::<T>())
    }

    /// List all registered model names
    pub fn list_models(&self) -> Vec<String> {
        self.models.keys().cloned().collect()
    }

    /// Get the base path for models
    pub fn base_path(&self) -> &Path {
        &self.base_path
    }
}

impl Default for ModelMetrics {
    fn default() -> Self {
        Self {
            accuracy: 0.0,
            precision: 0.0,
            recall: 0.0,
            f1_score: 0.0,
            mae: None,
            rmse: None,
            r2_score: None,
        }
    }
}

impl<T> Prediction<T> {
    /// Create a new prediction
    pub fn new(value: T, confidence: f64, model_version: String) -> Self {
        Self {
            value,
            confidence,
            lower_bound: None,
            upper_bound: None,
            model_version,
            timestamp: chrono::Utc::now(),
            feature_importance: None,
        }
    }

    /// Set confidence interval bounds
    pub fn with_bounds(mut self, lower: T, upper: T) -> Self {
        self.lower_bound = Some(lower);
        self.upper_bound = Some(upper);
        self
    }

    /// Add feature importance scores
    pub fn with_importance(mut self, importance: Vec<(String, f64)>) -> Self {
        self.feature_importance = Some(importance);
        self
    }
}

// Re-export model types
pub use damage_analyzer::DamageAnalyzer;
pub use impact_classifier::ImpactClassifier;
pub use occupant_risk::OccupantRiskPredictor;
pub use speed_estimator::SpeedEstimator;
pub use trajectory_predictor::TrajectoryPredictor;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_registry() {
        let mut registry = ModelRegistry::new("/tmp/models");
        registry.register("test".to_string(), 42i32);

        assert_eq!(registry.get::<i32>("test"), Some(&42));
        assert_eq!(registry.get::<String>("test"), None);
    }

    #[test]
    fn test_prediction() {
        let pred = Prediction::new(50.0, 0.95, "v1.0".to_string())
            .with_bounds(45.0, 55.0);

        assert_eq!(pred.value, 50.0);
        assert_eq!(pred.confidence, 0.95);
        assert_eq!(pred.lower_bound, Some(45.0));
        assert_eq!(pred.upper_bound, Some(55.0));
    }
}
