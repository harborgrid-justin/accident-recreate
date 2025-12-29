//! # AccuScene ML v3 - AI/ML Prediction Engine
//!
//! Advanced machine learning engine for accident reconstruction and analysis.
//!
//! ## Features
//!
//! - **Speed Estimation**: Estimate vehicle speed from damage patterns
//! - **Impact Classification**: Classify impact types (frontal, side, rear, rollover)
//! - **Trajectory Prediction**: Predict vehicle trajectories post-impact
//! - **Damage Analysis**: Analyze damage severity from images using computer vision
//! - **Occupant Risk**: Predict occupant injury risk
//! - **ONNX Support**: Load and run ONNX models with GPU acceleration
//! - **Batch Processing**: Efficient batch inference processing
//! - **Model Explainability**: SHAP values and feature importance
//!
//! ## Example
//!
//! ```rust,no_run
//! use accuscene_ml_v3::{MlConfig, models::SpeedEstimator};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = MlConfig::default();
//! let estimator = SpeedEstimator::new(&config)?;
//!
//! let damage_features = vec![0.8, 0.6, 0.3]; // Normalized damage features
//! let prediction = estimator.estimate(&damage_features).await?;
//!
//! println!("Estimated speed: {:.2} mph (confidence: {:.2}%)",
//!     prediction.speed_mph,
//!     prediction.confidence * 100.0
//! );
//! # Ok(())
//! # }
//! ```

#![warn(
    missing_docs,
    missing_debug_implementations,
    rust_2018_idioms,
    unreachable_pub
)]
#![forbid(unsafe_code)]

pub mod config;
pub mod error;
pub mod evaluation;
pub mod explainability;
pub mod features;
pub mod inference;
pub mod models;
pub mod training;

// Re-export commonly used types
pub use config::{MlConfig, InferenceConfig, ModelConfig, TrainingConfig};
pub use error::{MlError, Result};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the ML engine with logging
pub fn init() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    tracing::info!("AccuScene ML v{} initialized", VERSION);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_default_config() {
        let config = MlConfig::default();
        assert!(config.inference.batch_size > 0);
        assert!(config.hardware.num_threads > 0);
    }
}
