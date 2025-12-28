//! ONNX runtime integration (optional feature)

use crate::error::{MLError, Result};
use crate::inference::{InferenceEngine, InferenceResult, HealthStatus, ModelInfo, OutputType};
use async_trait::async_trait;
use ndarray::{Array1, Array2};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[cfg(feature = "onnx")]
use tract_onnx::prelude::*;

/// ONNX model wrapper
pub struct OnnxModel {
    #[cfg(feature = "onnx")]
    model: SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>,

    /// Model information
    info: ModelInfo,

    /// Input shape
    input_shape: Vec<usize>,

    /// Output shape
    output_shape: Vec<usize>,
}

impl OnnxModel {
    /// Load an ONNX model from file
    #[cfg(feature = "onnx")]
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let model = tract_onnx::onnx()
            .model_for_path(path.as_ref())
            .map_err(|e| MLError::OnnxRuntime(e.to_string()))?
            .into_optimized()
            .map_err(|e| MLError::OnnxRuntime(e.to_string()))?
            .into_runnable()
            .map_err(|e| MLError::OnnxRuntime(e.to_string()))?;

        // Extract input/output shapes
        let input_shape = vec![1]; // Simplified
        let output_shape = vec![1]; // Simplified

        let info = ModelInfo::new("onnx_model", "1.0.0", "onnx")
            .with_output_type(OutputType::Vector);

        Ok(Self {
            model,
            info,
            input_shape,
            output_shape,
        })
    }

    /// Load an ONNX model from file (non-onnx feature version)
    #[cfg(not(feature = "onnx"))]
    pub fn from_file(_path: impl AsRef<Path>) -> Result<Self> {
        Err(MLError::OnnxRuntime(
            "ONNX feature not enabled. Enable 'onnx' feature to use ONNX models".to_string(),
        ))
    }

    /// Load an ONNX model from bytes
    #[cfg(feature = "onnx")]
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let model = tract_onnx::onnx()
            .model_for_read(&mut std::io::Cursor::new(bytes))
            .map_err(|e| MLError::OnnxRuntime(e.to_string()))?
            .into_optimized()
            .map_err(|e| MLError::OnnxRuntime(e.to_string()))?
            .into_runnable()
            .map_err(|e| MLError::OnnxRuntime(e.to_string()))?;

        let input_shape = vec![1];
        let output_shape = vec![1];

        let info = ModelInfo::new("onnx_model", "1.0.0", "onnx")
            .with_output_type(OutputType::Vector);

        Ok(Self {
            model,
            info,
            input_shape,
            output_shape,
        })
    }

    /// Load an ONNX model from bytes (non-onnx feature version)
    #[cfg(not(feature = "onnx"))]
    pub fn from_bytes(_bytes: &[u8]) -> Result<Self> {
        Err(MLError::OnnxRuntime(
            "ONNX feature not enabled. Enable 'onnx' feature to use ONNX models".to_string(),
        ))
    }

    /// Set model information
    pub fn with_info(mut self, info: ModelInfo) -> Self {
        self.info = info;
        self
    }

    /// Run inference on input tensor
    #[cfg(feature = "onnx")]
    fn run_inference(&self, input: Array2<f64>) -> Result<Array2<f64>> {
        // Convert ndarray to tract tensor
        let input_tensor = input.into_dyn();

        let result = self
            .model
            .run(tvec![input_tensor.into()])
            .map_err(|e| MLError::OnnxRuntime(e.to_string()))?;

        // Extract output tensor
        let output_tensor = result[0]
            .to_array_view::<f64>()
            .map_err(|e| MLError::OnnxRuntime(e.to_string()))?;

        // Convert back to Array2
        let shape = output_tensor.shape();
        let output = if shape.len() == 2 {
            Array2::from_shape_vec(
                (shape[0], shape[1]),
                output_tensor.iter().copied().collect(),
            )
            .map_err(|e| MLError::OnnxRuntime(e.to_string()))?
        } else {
            // Reshape if needed
            Array2::from_shape_vec((shape[0], 1), output_tensor.iter().copied().collect())
                .map_err(|e| MLError::OnnxRuntime(e.to_string()))?
        };

        Ok(output)
    }

    /// Run inference on input tensor (non-onnx feature version)
    #[cfg(not(feature = "onnx"))]
    fn run_inference(&self, _input: Array2<f64>) -> Result<Array2<f64>> {
        Err(MLError::OnnxRuntime(
            "ONNX feature not enabled".to_string(),
        ))
    }
}

#[async_trait]
impl InferenceEngine for OnnxModel {
    async fn predict(&self, features: Array1<f64>) -> Result<InferenceResult> {
        let start = std::time::Instant::now();

        // Reshape to 2D
        let input = features.insert_axis(ndarray::Axis(0));

        let output = self.run_inference(input)?;

        let prediction = output.row(0).to_vec();

        let inference_time_ms = start.elapsed().as_secs_f64() * 1000.0;

        Ok(InferenceResult::new(prediction, self.info.version.clone())
            .with_inference_time(inference_time_ms))
    }

    async fn predict_batch(&self, features: Array2<f64>) -> Result<Vec<InferenceResult>> {
        let start = std::time::Instant::now();

        let output = self.run_inference(features)?;

        let n_samples = output.nrows();
        let inference_time_ms = start.elapsed().as_secs_f64() * 1000.0;
        let time_per_sample = inference_time_ms / n_samples as f64;

        let results = (0..n_samples)
            .map(|i| {
                let prediction = output.row(i).to_vec();
                InferenceResult::new(prediction, self.info.version.clone())
                    .with_inference_time(time_per_sample)
            })
            .collect();

        Ok(results)
    }

    fn model_info(&self) -> ModelInfo {
        self.info.clone()
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        Ok(HealthStatus::Healthy)
    }
}

/// ONNX model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnnxConfig {
    /// Model path
    pub model_path: String,

    /// Optimization level (0-3)
    pub optimization_level: u8,

    /// Number of threads
    pub num_threads: usize,

    /// Enable GPU acceleration
    pub enable_gpu: bool,

    /// Input names
    pub input_names: Vec<String>,

    /// Output names
    pub output_names: Vec<String>,
}

impl Default for OnnxConfig {
    fn default() -> Self {
        Self {
            model_path: String::new(),
            optimization_level: 2,
            num_threads: std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(4),
            enable_gpu: false,
            input_names: vec!["input".to_string()],
            output_names: vec!["output".to_string()],
        }
    }
}

/// ONNX model builder
pub struct OnnxModelBuilder {
    config: OnnxConfig,
    info: Option<ModelInfo>,
}

impl OnnxModelBuilder {
    /// Create a new ONNX model builder
    pub fn new() -> Self {
        Self {
            config: OnnxConfig::default(),
            info: None,
        }
    }

    /// Set model path
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.config.model_path = path.into();
        self
    }

    /// Set optimization level
    pub fn with_optimization_level(mut self, level: u8) -> Self {
        self.config.optimization_level = level;
        self
    }

    /// Set number of threads
    pub fn with_num_threads(mut self, threads: usize) -> Self {
        self.config.num_threads = threads;
        self
    }

    /// Enable GPU acceleration
    pub fn with_gpu(mut self, enable: bool) -> Self {
        self.config.enable_gpu = enable;
        self
    }

    /// Set model info
    pub fn with_info(mut self, info: ModelInfo) -> Self {
        self.info = Some(info);
        self
    }

    /// Build the ONNX model
    pub fn build(self) -> Result<OnnxModel> {
        let mut model = OnnxModel::from_file(&self.config.model_path)?;

        if let Some(info) = self.info {
            model = model.with_info(info);
        }

        Ok(model)
    }
}

impl Default for OnnxModelBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_onnx_config() {
        let config = OnnxConfig::default();
        assert_eq!(config.optimization_level, 2);
        assert!(config.num_threads > 0);
        assert!(!config.enable_gpu);
    }

    #[test]
    fn test_onnx_builder() {
        let builder = OnnxModelBuilder::new()
            .with_path("model.onnx")
            .with_optimization_level(3)
            .with_num_threads(8);

        assert_eq!(builder.config.model_path, "model.onnx");
        assert_eq!(builder.config.optimization_level, 3);
        assert_eq!(builder.config.num_threads, 8);
    }
}
