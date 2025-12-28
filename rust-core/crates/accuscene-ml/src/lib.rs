//! AccuScene Enterprise Machine Learning Integration System
//!
//! This crate provides a comprehensive machine learning framework for accident reconstruction
//! and analysis, including:
//!
//! - Model management and versioning
//! - Feature engineering and transformation
//! - Training framework with cross-validation
//! - Multiple ML algorithms (regression, classification, clustering, ensemble)
//! - Inference engines (batch and real-time)
//! - Model evaluation metrics
//! - ML pipelines
//! - Model serving infrastructure
//! - Domain-specific models for accident analysis

pub mod algorithms;
pub mod config;
pub mod domain;
pub mod error;
pub mod evaluation;
pub mod feature;
pub mod inference;
pub mod model;
pub mod pipeline;
pub mod serving;
pub mod training;

// Re-export commonly used types
pub use config::MLConfig;
pub use error::{MLError, Result};

// Model management
pub use model::{
    artifact::{ArtifactStore, ModelArtifact},
    metadata::{FeatureInfo, FeatureType, ModelMetadata, ModelStatus, ModelType},
    registry::{ModelRegistry, RegistryStats},
    Classifier, Clusterer, Model, ModelLifecycle,
};

// Feature engineering
pub use feature::{
    encoding::{CategoricalEncoder, LabelEncoder, OneHotEncoder},
    extraction::{FeatureExtractor, PolynomialFeatures, StatisticalFeatures},
    normalization::{MinMaxScaler, Normalizer, StandardScaler},
    store::{FeatureStore, FeatureStoreConfig},
    transformation::{FeatureTransformer, LogTransform, PowerTransform},
    FeatureSet, FeatureStats, FeatureVector,
};

// Inference
pub use inference::{
    batch::{BatchInferenceEngine, BatchRequest, BatchResponse},
    realtime::{InferenceRequest, InferenceResponse, RealtimeInferenceEngine},
    HealthStatus, InferenceEngine, InferenceMetrics, InferenceResult, ModelInfo, OutputType,
};

// Training
pub use training::{
    cross_validation::{CrossValidator, CVResults, KFold, StratifiedKFold},
    dataset::{Dataset, DatasetBuilder, DatasetSplit},
    hyperparameter::{GridSearch, HyperparameterTuner, ParamGrid, RandomSearch, TuningResults},
    split::{TrainTestSplit, ValidationSplit},
    TrainingConfig, TrainingHistory,
};

// Algorithms
pub use algorithms::{
    classification::{DecisionTreeClassifier, LogisticRegression, SVMClassifier},
    clustering::{DBSCANClusterer, KMeansClusterer},
    ensemble::{GradientBoostingRegressor, RandomForestRegressor},
    regression::{LassoRegression, LinearRegression, RidgeRegression},
};

// Evaluation
pub use evaluation::{
    confusion::ConfusionMatrix,
    metrics::{
        accuracy, f1_score, mae, mse, precision, r2_score, recall, rmse, ClassificationMetrics,
        RegressionMetrics,
    },
    EvaluationResults,
};

// Pipeline and serving
pub use pipeline::{Pipeline, PipelineBuilder, PipelineConfig};
pub use serving::{DeploymentConfig, ServingConfig, ServingServer};

// Domain-specific models
pub use domain::{
    collision_predictor::{CollisionPredictor, CollisionScenario, CollisionSeverity},
    damage_estimator::{DamageEstimate, DamageEstimator, DamageParameters, DamageSeverity},
    fault_analyzer::{AccidentEvidence, FaultAnalysis, FaultAnalyzer, FaultParty},
    trajectory_classifier::{Trajectory, TrajectoryClassifier, TrajectoryType},
};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Library initialization
pub fn init() {
    // Initialize logging or other global state if needed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_init() {
        init();
    }
}
