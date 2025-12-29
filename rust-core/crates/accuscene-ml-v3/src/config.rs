//! Configuration management for the ML engine

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use validator::Validate;

use crate::error::{MlError, Result};

/// Main configuration for the ML engine
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct MlConfig {
    /// Model configuration
    pub models: ModelConfig,

    /// Inference configuration
    pub inference: InferenceConfig,

    /// Training configuration
    pub training: TrainingConfig,

    /// Feature extraction configuration
    pub features: FeatureConfig,

    /// GPU/Hardware configuration
    pub hardware: HardwareConfig,

    /// Logging and monitoring
    pub monitoring: MonitoringConfig,
}

/// Model-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ModelConfig {
    /// Base directory for model files
    pub model_dir: PathBuf,

    /// Model versioning strategy
    pub versioning: VersioningStrategy,

    /// Model cache size (number of models)
    #[validate(range(min = 1, max = 100))]
    pub cache_size: usize,

    /// Auto-reload models on file change
    pub auto_reload: bool,

    /// Model format preference
    pub preferred_format: ModelFormat,
}

/// Inference engine configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct InferenceConfig {
    /// Batch size for inference
    #[validate(range(min = 1, max = 1024))]
    pub batch_size: usize,

    /// Maximum concurrent inference requests
    #[validate(range(min = 1, max = 1000))]
    pub max_concurrent: usize,

    /// Inference timeout in milliseconds
    #[validate(range(min = 100))]
    pub timeout_ms: u64,

    /// Use GPU acceleration if available
    pub use_gpu: bool,

    /// ONNX execution provider
    pub execution_provider: ExecutionProvider,

    /// Enable model warmup
    pub warmup: bool,

    /// Number of warmup iterations
    #[validate(range(min = 1, max = 100))]
    pub warmup_iterations: usize,
}

/// Training configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct TrainingConfig {
    /// Training data directory
    pub data_dir: PathBuf,

    /// Output directory for trained models
    pub output_dir: PathBuf,

    /// Learning rate
    #[validate(range(min = 0.00001, max = 1.0))]
    pub learning_rate: f64,

    /// Number of epochs
    #[validate(range(min = 1, max = 10000))]
    pub epochs: usize,

    /// Batch size
    #[validate(range(min = 1, max = 1024))]
    pub batch_size: usize,

    /// Validation split ratio
    #[validate(range(min = 0.0, max = 0.5))]
    pub validation_split: f32,

    /// Early stopping patience
    #[validate(range(min = 1, max = 100))]
    pub early_stopping_patience: usize,

    /// Cross-validation folds
    #[validate(range(min = 2, max = 20))]
    pub cv_folds: usize,

    /// Random seed for reproducibility
    pub random_seed: Option<u64>,
}

/// Feature extraction configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct FeatureConfig {
    /// Feature normalization strategy
    pub normalization: NormalizationStrategy,

    /// Feature selection method
    pub selection: FeatureSelectionMethod,

    /// Maximum number of features
    #[validate(range(min = 1, max = 10000))]
    pub max_features: usize,

    /// Enable feature caching
    pub cache_features: bool,

    /// Image preprocessing settings
    pub image_preprocessing: ImagePreprocessingConfig,
}

/// Hardware/GPU configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct HardwareConfig {
    /// Enable GPU acceleration
    pub enable_gpu: bool,

    /// GPU device ID (-1 for CPU)
    pub gpu_device_id: i32,

    /// CUDA memory limit in MB
    #[validate(range(min = 128))]
    pub cuda_memory_limit_mb: usize,

    /// Number of CPU threads
    #[validate(range(min = 1, max = 256))]
    pub num_threads: usize,

    /// Enable mixed precision (FP16)
    pub enable_mixed_precision: bool,
}

/// Monitoring and logging configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct MonitoringConfig {
    /// Enable performance metrics
    pub enable_metrics: bool,

    /// Log inference latency
    pub log_latency: bool,

    /// Log prediction confidence
    pub log_confidence: bool,

    /// Metrics export interval in seconds
    #[validate(range(min = 1))]
    pub metrics_interval_secs: u64,

    /// Enable model explainability logging
    pub log_explanations: bool,
}

/// Image preprocessing configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ImagePreprocessingConfig {
    /// Target image width
    #[validate(range(min = 32, max = 4096))]
    pub width: u32,

    /// Target image height
    #[validate(range(min = 32, max = 4096))]
    pub height: u32,

    /// Normalization mean values (RGB)
    pub mean: [f32; 3],

    /// Normalization std values (RGB)
    pub std: [f32; 3],

    /// Enable data augmentation
    pub augmentation: bool,
}

/// Model versioning strategy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VersioningStrategy {
    /// Use latest version
    Latest,
    /// Use specific semantic version
    Semantic(String),
    /// Use git hash
    GitHash(String),
    /// Use timestamp
    Timestamp(i64),
}

/// Model file format
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModelFormat {
    /// ONNX format
    Onnx,
    /// Native Rust format (bincode)
    Native,
    /// JSON format
    Json,
}

/// ONNX execution provider
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExecutionProvider {
    /// CPU execution
    Cpu,
    /// CUDA (NVIDIA GPU)
    Cuda,
    /// TensorRT (NVIDIA optimized)
    TensorRT,
    /// CoreML (Apple Silicon)
    CoreML,
    /// DirectML (Windows GPU)
    DirectML,
}

/// Feature normalization strategy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NormalizationStrategy {
    /// No normalization
    None,
    /// Min-Max scaling [0, 1]
    MinMax,
    /// Z-score standardization (mean=0, std=1)
    ZScore,
    /// Robust scaling using median and IQR
    Robust,
    /// L2 normalization
    L2,
}

/// Feature selection method
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FeatureSelectionMethod {
    /// No feature selection
    None,
    /// Select top K features by variance
    VarianceThreshold,
    /// Select top K features by correlation
    Correlation,
    /// Recursive feature elimination
    RFE,
    /// LASSO-based selection
    Lasso,
}

impl Default for MlConfig {
    fn default() -> Self {
        Self {
            models: ModelConfig::default(),
            inference: InferenceConfig::default(),
            training: TrainingConfig::default(),
            features: FeatureConfig::default(),
            hardware: HardwareConfig::default(),
            monitoring: MonitoringConfig::default(),
        }
    }
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            model_dir: PathBuf::from("./models"),
            versioning: VersioningStrategy::Latest,
            cache_size: 10,
            auto_reload: true,
            preferred_format: ModelFormat::Onnx,
        }
    }
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            batch_size: 32,
            max_concurrent: 100,
            timeout_ms: 5000,
            use_gpu: true,
            execution_provider: ExecutionProvider::Cuda,
            warmup: true,
            warmup_iterations: 10,
        }
    }
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from("./data"),
            output_dir: PathBuf::from("./models/trained"),
            learning_rate: 0.001,
            epochs: 100,
            batch_size: 32,
            validation_split: 0.2,
            early_stopping_patience: 10,
            cv_folds: 5,
            random_seed: Some(42),
        }
    }
}

impl Default for FeatureConfig {
    fn default() -> Self {
        Self {
            normalization: NormalizationStrategy::ZScore,
            selection: FeatureSelectionMethod::None,
            max_features: 1000,
            cache_features: true,
            image_preprocessing: ImagePreprocessingConfig::default(),
        }
    }
}

impl Default for HardwareConfig {
    fn default() -> Self {
        Self {
            enable_gpu: cfg!(feature = "gpu"),
            gpu_device_id: 0,
            cuda_memory_limit_mb: 4096,
            num_threads: num_cpus::get(),
            enable_mixed_precision: false,
        }
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enable_metrics: true,
            log_latency: true,
            log_confidence: true,
            metrics_interval_secs: 60,
            log_explanations: false,
        }
    }
}

impl Default for ImagePreprocessingConfig {
    fn default() -> Self {
        Self {
            width: 224,
            height: 224,
            mean: [0.485, 0.456, 0.406], // ImageNet mean
            std: [0.229, 0.224, 0.225],  // ImageNet std
            augmentation: false,
        }
    }
}

impl MlConfig {
    /// Load configuration from a file
    pub fn from_file(path: &std::path::Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| MlError::Config(format!("Failed to read config file: {}", e)))?;

        let config: Self = serde_json::from_str(&content)
            .map_err(|e| MlError::Config(format!("Failed to parse config: {}", e)))?;

        config.validate()
            .map_err(|e| MlError::Config(format!("Invalid configuration: {}", e)))?;

        Ok(config)
    }

    /// Save configuration to a file
    pub fn save_to_file(&self, path: &std::path::Path) -> Result<()> {
        self.validate()
            .map_err(|e| MlError::Config(format!("Invalid configuration: {}", e)))?;

        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)
            .map_err(|e| MlError::Config(format!("Failed to write config file: {}", e)))?;

        Ok(())
    }

    /// Create a new configuration with sensible defaults for production
    pub fn production() -> Self {
        let mut config = Self::default();
        config.inference.use_gpu = true;
        config.inference.batch_size = 64;
        config.monitoring.enable_metrics = true;
        config.hardware.enable_mixed_precision = true;
        config
    }

    /// Create a new configuration optimized for development
    pub fn development() -> Self {
        let mut config = Self::default();
        config.inference.use_gpu = false;
        config.inference.batch_size = 8;
        config.monitoring.log_explanations = true;
        config
    }
}

// Helper function to get number of CPUs
mod num_cpus {
    pub fn get() -> usize {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
    }
}
