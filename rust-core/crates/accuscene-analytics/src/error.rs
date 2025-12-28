//! Error types for the analytics engine

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AnalyticsError {
    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Metric error: {0}")]
    Metric(String),

    #[error("Aggregation error: {0}")]
    Aggregation(String),

    #[error("Statistical analysis error: {0}")]
    Statistical(String),

    #[error("Invalid window specification: {0}")]
    InvalidWindow(String),

    #[error("Query error: {0}")]
    Query(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Export error: {0}")]
    Export(String),

    #[error("Anomaly detection error: {0}")]
    AnomalyDetection(String),

    #[error("Forecasting error: {0}")]
    Forecasting(String),

    #[error("Insufficient data: {0}")]
    InsufficientData(String),

    #[error("Invalid data: {0}")]
    InvalidData(String),

    #[error("Concurrency error: {0}")]
    Concurrency(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, AnalyticsError>;
