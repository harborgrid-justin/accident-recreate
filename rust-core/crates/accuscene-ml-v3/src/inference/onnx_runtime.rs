//! ONNX Runtime integration for model inference

use crate::config::InferenceConfig;
use crate::error::{MlError, Result};
use crate::inference::{Device, InferenceRequest, InferenceResult};
use ndarray::{Array2, ArrayD};
use ort::{Environment, ExecutionProvider, GraphOptimizationLevel, Session, SessionBuilder, Value};
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// ONNX model wrapper
pub struct OnnxModel {
    session: Arc<Session>,
    input_name: String,
    output_name: String,
    model_version: String,
    device: Device,
}

impl OnnxModel {
    /// Load ONNX model from file
    pub fn from_file<P: AsRef<Path>>(
        path: P,
        config: &InferenceConfig,
    ) -> Result<Self> {
        let environment = Arc::new(
            Environment::builder()
                .with_name("accuscene-ml")
                .build()
                .map_err(|e| MlError::OnnxRuntime(e.to_string()))?,
        );

        let mut builder = SessionBuilder::new(&environment)
            .map_err(|e| MlError::OnnxRuntime(e.to_string()))?;

        // Set optimization level
        builder = builder
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| MlError::OnnxRuntime(e.to_string()))?;

        // Set number of threads
        builder = builder
            .with_intra_threads(config.max_concurrent as i16)
            .map_err(|e| MlError::OnnxRuntime(e.to_string()))?;

        // Configure execution provider based on config
        let device = if config.use_gpu {
            match config.execution_provider {
                crate::config::ExecutionProvider::Cuda => {
                    #[cfg(feature = "gpu")]
                    {
                        builder = builder
                            .with_execution_providers(&[ExecutionProvider::CUDA(Default::default())])
                            .map_err(|e| MlError::OnnxRuntime(e.to_string()))?;
                        Device::Gpu { device_id: 0 }
                    }
                    #[cfg(not(feature = "gpu"))]
                    {
                        tracing::warn!("GPU requested but not available, falling back to CPU");
                        Device::Cpu
                    }
                }
                crate::config::ExecutionProvider::TensorRT => {
                    #[cfg(feature = "gpu")]
                    {
                        builder = builder
                            .with_execution_providers(&[ExecutionProvider::TensorRT(Default::default())])
                            .map_err(|e| MlError::OnnxRuntime(e.to_string()))?;
                        Device::Gpu { device_id: 0 }
                    }
                    #[cfg(not(feature = "gpu"))]
                    {
                        tracing::warn!("TensorRT requested but not available, falling back to CPU");
                        Device::Cpu
                    }
                }
                crate::config::ExecutionProvider::CoreML => {
                    builder = builder
                        .with_execution_providers(&[ExecutionProvider::CoreML(Default::default())])
                        .map_err(|e| MlError::OnnxRuntime(e.to_string()))?;
                    Device::Cpu // CoreML runs on Apple Silicon
                }
                _ => Device::Cpu,
            }
        } else {
            Device::Cpu
        };

        // Load model
        let session = builder
            .with_model_from_file(path.as_ref())
            .map_err(|e| MlError::OnnxRuntime(e.to_string()))?;

        // Get input/output names
        let input_name = session
            .inputs
            .get(0)
            .ok_or_else(|| MlError::Model("No input found in ONNX model".to_string()))?
            .name
            .clone();

        let output_name = session
            .outputs
            .get(0)
            .ok_or_else(|| MlError::Model("No output found in ONNX model".to_string()))?
            .name
            .clone();

        // Extract version from metadata if available
        let model_version = session
            .metadata()
            .ok()
            .and_then(|m| m.version().ok().flatten())
            .unwrap_or_else(|| "1.0.0".to_string());

        Ok(Self {
            session: Arc::new(session),
            input_name,
            output_name,
            model_version,
            device,
        })
    }

    /// Run inference on input data
    pub async fn run(&self, input: Array2<f64>) -> Result<Array2<f64>> {
        let start = Instant::now();

        // Prepare input tensor
        let shape = input.shape();
        let input_array = ArrayD::from_shape_vec(
            vec![shape[0], shape[1]],
            input.into_raw_vec(),
        )
        .map_err(|e| MlError::Inference(e.to_string()))?;

        // Convert to ONNX value
        let input_tensor = Value::from_array(self.session.allocator(), &input_array)
            .map_err(|e| MlError::OnnxRuntime(e.to_string()))?;

        // Run inference
        let outputs = tokio::task::spawn_blocking({
            let session = Arc::clone(&self.session);
            let input_name = self.input_name.clone();
            move || {
                session.run(vec![input_tensor])
            }
        })
        .await
        .map_err(|e| MlError::Inference(format!("Async execution failed: {}", e)))?
        .map_err(|e| MlError::OnnxRuntime(e.to_string()))?;

        // Extract output
        let output_tensor = outputs
            .get(0)
            .ok_or_else(|| MlError::Inference("No output returned".to_string()))?;

        let output_array: ArrayD<f64> = output_tensor
            .try_extract()
            .map_err(|e| MlError::Inference(format!("Failed to extract output: {}", e)))?
            .view()
            .to_owned();

        // Convert to 2D array
        let output_shape = output_array.shape();
        let output = Array2::from_shape_vec(
            (output_shape[0], output_shape[1]),
            output_array.into_raw_vec(),
        )
        .map_err(|e| MlError::Inference(e.to_string()))?;

        let latency = start.elapsed();
        tracing::debug!("Inference completed in {:?}", latency);

        Ok(output)
    }

    /// Run inference with full request/result wrapping
    pub async fn infer(&self, request: InferenceRequest) -> Result<InferenceResult> {
        let start = Instant::now();
        let output = self.run(request.input).await?;
        let latency = start.elapsed();

        Ok(InferenceResult::new(
            output,
            request.request_id,
            latency,
            self.model_version.clone(),
            self.device,
        ))
    }

    /// Get input shape requirements
    pub fn input_shape(&self) -> Vec<i64> {
        self.session
            .inputs
            .get(0)
            .and_then(|input| {
                input
                    .dimensions()
                    .map(|dims| dims.iter().map(|d| d.unwrap_or(1) as i64).collect())
                    .ok()
            })
            .unwrap_or_else(|| vec![-1, -1])
    }

    /// Get output shape
    pub fn output_shape(&self) -> Vec<i64> {
        self.session
            .outputs
            .get(0)
            .and_then(|output| {
                output
                    .dimensions()
                    .map(|dims| dims.iter().map(|d| d.unwrap_or(1) as i64).collect())
                    .ok()
            })
            .unwrap_or_else(|| vec![-1, -1])
    }

    /// Get model version
    pub fn version(&self) -> &str {
        &self.model_version
    }

    /// Get execution device
    pub fn device(&self) -> Device {
        self.device
    }

    /// Warmup the model with dummy input
    pub async fn warmup(&self, input_size: usize, iterations: usize) -> Result<()> {
        tracing::info!("Warming up model with {} iterations", iterations);

        for i in 0..iterations {
            let dummy_input = Array2::zeros((1, input_size));
            self.run(dummy_input).await?;

            if i % 10 == 0 {
                tracing::debug!("Warmup iteration {}/{}", i + 1, iterations);
            }
        }

        tracing::info!("Model warmup completed");
        Ok(())
    }
}

impl std::fmt::Debug for OnnxModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OnnxModel")
            .field("input_name", &self.input_name)
            .field("output_name", &self.output_name)
            .field("model_version", &self.model_version)
            .field("device", &self.device)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::MlConfig;

    #[test]
    fn test_device_types() {
        let cpu = Device::Cpu;
        let gpu = Device::Gpu { device_id: 0 };

        assert_ne!(cpu, gpu);
    }

    // Note: Actual model loading tests require ONNX model files
    #[tokio::test]
    #[ignore] // Requires actual ONNX model file
    async fn test_model_loading() {
        let config = MlConfig::default();
        let model_path = config.models.model_dir.join("test_model.onnx");

        if model_path.exists() {
            let result = OnnxModel::from_file(&model_path, &config.inference);
            assert!(result.is_ok());
        }
    }
}
