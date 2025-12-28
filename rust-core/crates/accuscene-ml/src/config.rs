//! Configuration for the ML system

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// ML system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MLConfig {
    /// Model storage directory
    pub model_storage_path: PathBuf,

    /// Feature store configuration
    pub feature_store: FeatureStoreConfig,

    /// Inference configuration
    pub inference: InferenceConfig,

    /// Training configuration
    pub training: TrainingConfig,

    /// Serving configuration
    pub serving: ServingConfig,
}

impl Default for MLConfig {
    fn default() -> Self {
        Self {
            model_storage_path: PathBuf::from("./models"),
            feature_store: FeatureStoreConfig::default(),
            inference: InferenceConfig::default(),
            training: TrainingConfig::default(),
            serving: ServingConfig::default(),
        }
    }
}

/// Feature store configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureStoreConfig {
    /// Storage path
    pub storage_path: PathBuf,

    /// Cache size (in MB)
    pub cache_size_mb: usize,

    /// Enable compression
    pub enable_compression: bool,
}

impl Default for FeatureStoreConfig {
    fn default() -> Self {
        Self {
            storage_path: PathBuf::from("./features"),
            cache_size_mb: 1024,
            enable_compression: true,
        }
    }
}

/// Inference configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceConfig {
    /// Batch size for batch inference
    pub batch_size: usize,

    /// Number of worker threads
    pub num_workers: usize,

    /// Timeout for inference (in milliseconds)
    pub timeout_ms: u64,

    /// Enable ONNX runtime
    pub enable_onnx: bool,

    /// ONNX optimization level
    pub onnx_optimization_level: u8,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            batch_size: 32,
            num_workers: num_cpus::get(),
            timeout_ms: 5000,
            enable_onnx: false,
            onnx_optimization_level: 2,
        }
    }
}

/// Training configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingConfig {
    /// Default training/test split ratio
    pub train_test_split: f64,

    /// Number of cross-validation folds
    pub cv_folds: usize,

    /// Random seed for reproducibility
    pub random_seed: u64,

    /// Maximum training iterations
    pub max_iterations: usize,

    /// Early stopping patience
    pub early_stopping_patience: usize,

    /// Learning rate
    pub learning_rate: f64,

    /// Enable parallel training
    pub enable_parallel: bool,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            train_test_split: 0.8,
            cv_folds: 5,
            random_seed: 42,
            max_iterations: 1000,
            early_stopping_patience: 10,
            learning_rate: 0.01,
            enable_parallel: true,
        }
    }
}

/// Model serving configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServingConfig {
    /// Maximum concurrent requests
    pub max_concurrent_requests: usize,

    /// Request queue size
    pub queue_size: usize,

    /// Model warm-up on startup
    pub warm_up_enabled: bool,

    /// Health check interval (in seconds)
    pub health_check_interval_secs: u64,

    /// Enable metrics collection
    pub enable_metrics: bool,
}

impl Default for ServingConfig {
    fn default() -> Self {
        Self {
            max_concurrent_requests: 100,
            queue_size: 1000,
            warm_up_enabled: true,
            health_check_interval_secs: 60,
            enable_metrics: true,
        }
    }
}

// Add num_cpus as a minimal implementation
mod num_cpus {
    pub fn get() -> usize {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
    }
}
