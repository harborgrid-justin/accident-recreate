//! Error types for the ML system

use thiserror::Error;

/// Result type for ML operations
pub type Result<T> = std::result::Result<T, MLError>;

/// Errors that can occur in the ML system
#[derive(Error, Debug)]
pub enum MLError {
    #[error("Model error: {0}")]
    Model(String),

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Model already exists: {0}")]
    ModelAlreadyExists(String),

    #[error("Inference error: {0}")]
    Inference(String),

    #[error("Feature engineering error: {0}")]
    Feature(String),

    #[error("Training error: {0}")]
    Training(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Data shape mismatch: expected {expected}, got {actual}")]
    ShapeMismatch { expected: String, actual: String },

    #[error("Insufficient data: {0}")]
    InsufficientData(String),

    #[error("Convergence error: {0}")]
    Convergence(String),

    #[error("Algorithm error: {0}")]
    Algorithm(String),

    #[error("ONNX runtime error: {0}")]
    OnnxRuntime(String),

    #[error("Feature store error: {0}")]
    FeatureStore(String),

    #[error("Pipeline error: {0}")]
    Pipeline(String),

    #[error("Evaluation error: {0}")]
    Evaluation(String),

    #[error("Serving error: {0}")]
    Serving(String),

    #[error("Registry error: {0}")]
    Registry(String),

    #[error("Artifact error: {0}")]
    Artifact(String),

    #[error("Metadata error: {0}")]
    Metadata(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl MLError {
    /// Create a model error
    pub fn model(msg: impl Into<String>) -> Self {
        Self::Model(msg.into())
    }

    /// Create an inference error
    pub fn inference(msg: impl Into<String>) -> Self {
        Self::Inference(msg.into())
    }

    /// Create a feature error
    pub fn feature(msg: impl Into<String>) -> Self {
        Self::Feature(msg.into())
    }

    /// Create a training error
    pub fn training(msg: impl Into<String>) -> Self {
        Self::Training(msg.into())
    }

    /// Create a validation error
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    /// Create an invalid input error
    pub fn invalid_input(msg: impl Into<String>) -> Self {
        Self::InvalidInput(msg.into())
    }

    /// Create a shape mismatch error
    pub fn shape_mismatch(expected: impl Into<String>, actual: impl Into<String>) -> Self {
        Self::ShapeMismatch {
            expected: expected.into(),
            actual: actual.into(),
        }
    }
}

impl From<serde_json::Error> for MLError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}

impl From<bincode::Error> for MLError {
    fn from(err: bincode::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}

impl From<csv::Error> for MLError {
    fn from(err: csv::Error) -> Self {
        Self::Io(std::io::Error::new(std::io::ErrorKind::Other, err))
    }
}
