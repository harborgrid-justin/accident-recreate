//! Telemetry-specific error types

use thiserror::Error;

/// Result type for telemetry operations
pub type Result<T> = std::result::Result<T, TelemetryError>;

/// Errors that can occur in the telemetry system
#[derive(Error, Debug)]
pub enum TelemetryError {
    /// Failed to initialize logging system
    #[error("Failed to initialize logging: {0}")]
    LoggingInit(String),

    /// Failed to initialize metrics system
    #[error("Failed to initialize metrics: {0}")]
    MetricsInit(String),

    /// Failed to export metrics
    #[error("Failed to export metrics: {0}")]
    MetricsExport(String),

    /// Failed to initialize tracing
    #[error("Failed to initialize tracing: {0}")]
    TracingInit(String),

    /// Health check failed
    #[error("Health check failed: {0}")]
    HealthCheckFailed(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Invalid metric name
    #[error("Invalid metric name: {0}")]
    InvalidMetricName(String),

    /// Metric not found
    #[error("Metric not found: {0}")]
    MetricNotFound(String),

    /// Alert threshold error
    #[error("Alert threshold error: {0}")]
    AlertThreshold(String),

    /// Performance profiling error
    #[error("Performance profiling error: {0}")]
    Profiling(String),

    /// File rotation error
    #[error("File rotation error: {0}")]
    FileRotation(String),

    /// Generic error
    #[error("{0}")]
    Other(String),
}

impl TelemetryError {
    /// Create a new config error
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    /// Create a new logging init error
    pub fn logging_init(msg: impl Into<String>) -> Self {
        Self::LoggingInit(msg.into())
    }

    /// Create a new metrics init error
    pub fn metrics_init(msg: impl Into<String>) -> Self {
        Self::MetricsInit(msg.into())
    }

    /// Create a new other error
    pub fn other(msg: impl Into<String>) -> Self {
        Self::Other(msg.into())
    }
}
