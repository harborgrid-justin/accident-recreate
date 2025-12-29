//! Error types for the AccuScene ML engine

use thiserror::Error;

/// Result type alias for ML operations
pub type Result<T> = std::result::Result<T, MlError>;

/// Comprehensive error types for ML operations
#[derive(Error, Debug)]
pub enum MlError {
    /// Model-related errors
    #[error("Model error: {0}")]
    Model(String),

    /// Model not found
    #[error("Model not found: {0}")]
    ModelNotFound(String),

    /// Model version mismatch
    #[error("Model version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: String, actual: String },

    /// Inference errors
    #[error("Inference failed: {0}")]
    Inference(String),

    /// ONNX runtime errors
    #[error("ONNX runtime error: {0}")]
    OnnxRuntime(String),

    /// Feature extraction errors
    #[error("Feature extraction failed: {0}")]
    FeatureExtraction(String),

    /// Invalid input
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Shape mismatch
    #[error("Shape mismatch: expected {expected:?}, got {actual:?}")]
    ShapeMismatch {
        expected: Vec<usize>,
        actual: Vec<usize>,
    },

    /// Training errors
    #[error("Training error: {0}")]
    Training(String),

    /// Dataset errors
    #[error("Dataset error: {0}")]
    Dataset(String),

    /// Evaluation errors
    #[error("Evaluation error: {0}")]
    Evaluation(String),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization errors
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// GPU/CUDA errors
    #[error("GPU error: {0}")]
    Gpu(String),

    /// Resource not available
    #[error("Resource unavailable: {0}")]
    ResourceUnavailable(String),

    /// Batch processing error
    #[error("Batch processing error: {0}")]
    BatchProcessing(String),

    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),

    /// Unsupported operation
    #[error("Unsupported operation: {0}")]
    Unsupported(String),

    /// Custom error with context
    #[error("{context}: {source}")]
    WithContext {
        context: String,
        source: Box<MlError>,
    },
}

impl MlError {
    /// Add context to an error
    pub fn context<S: Into<String>>(self, context: S) -> Self {
        MlError::WithContext {
            context: context.into(),
            source: Box::new(self),
        }
    }

    /// Create a model error
    pub fn model<S: Into<String>>(msg: S) -> Self {
        MlError::Model(msg.into())
    }

    /// Create an inference error
    pub fn inference<S: Into<String>>(msg: S) -> Self {
        MlError::Inference(msg.into())
    }

    /// Create a feature extraction error
    pub fn feature<S: Into<String>>(msg: S) -> Self {
        MlError::FeatureExtraction(msg.into())
    }

    /// Create an invalid input error
    pub fn invalid_input<S: Into<String>>(msg: S) -> Self {
        MlError::InvalidInput(msg.into())
    }
}

// Conversion from ort errors
impl From<ort::Error> for MlError {
    fn from(err: ort::Error) -> Self {
        MlError::OnnxRuntime(err.to_string())
    }
}

// Conversion from serde_json errors
impl From<serde_json::Error> for MlError {
    fn from(err: serde_json::Error) -> Self {
        MlError::Serialization(err.to_string())
    }
}

// Conversion from bincode errors
impl From<bincode::Error> for MlError {
    fn from(err: bincode::Error) -> Self {
        MlError::Serialization(err.to_string())
    }
}

// Conversion from image errors
impl From<image::ImageError> for MlError {
    fn from(err: image::ImageError) -> Self {
        MlError::FeatureExtraction(format!("Image processing error: {}", err))
    }
}
