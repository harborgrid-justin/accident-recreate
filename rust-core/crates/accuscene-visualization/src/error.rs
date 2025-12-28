use thiserror::Error;

/// Errors that can occur during visualization operations
#[derive(Error, Debug, Clone)]
pub enum VisualizationError {
    #[error("Invalid data: {0}")]
    InvalidData(String),

    #[error("Empty dataset provided")]
    EmptyDataset,

    #[error("Insufficient data points: expected at least {expected}, got {actual}")]
    InsufficientData { expected: usize, actual: usize },

    #[error("Invalid parameter: {parameter} = {value}")]
    InvalidParameter { parameter: String, value: String },

    #[error("Data dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },

    #[error("Invalid time range: start {start} is after end {end}")]
    InvalidTimeRange { start: String, end: String },

    #[error("Interpolation error: {0}")]
    InterpolationError(String),

    #[error("Aggregation error: {0}")]
    AggregationError(String),

    #[error("Export error: {0}")]
    ExportError(String),

    #[error("Chart generation error: {0}")]
    ChartError(String),

    #[error("Statistical calculation error: {0}")]
    StatisticalError(String),

    #[error("Numerical error: {0}")]
    NumericalError(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Result type alias for visualization operations
pub type Result<T> = std::result::Result<T, VisualizationError>;

impl From<serde_json::Error> for VisualizationError {
    fn from(err: serde_json::Error) -> Self {
        VisualizationError::SerializationError(err.to_string())
    }
}

impl From<std::io::Error> for VisualizationError {
    fn from(err: std::io::Error) -> Self {
        VisualizationError::IoError(err.to_string())
    }
}
