//! Inference engine for ML models

pub mod batch_processor;
pub mod gpu_accelerator;
pub mod onnx_runtime;

use crate::error::{MlError, Result};
use ndarray::Array2;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Inference request
#[derive(Debug, Clone)]
pub struct InferenceRequest {
    /// Input features
    pub input: Array2<f64>,

    /// Request ID for tracking
    pub request_id: String,

    /// Priority (higher = more urgent)
    pub priority: u8,

    /// Timeout duration
    pub timeout: Duration,
}

/// Inference result
#[derive(Debug, Clone)]
pub struct InferenceResult {
    /// Output predictions
    pub output: Array2<f64>,

    /// Request ID
    pub request_id: String,

    /// Inference latency
    pub latency: Duration,

    /// Model version used
    pub model_version: String,

    /// Execution device (CPU/GPU)
    pub device: Device,
}

/// Execution device
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Device {
    /// CPU execution
    Cpu,
    /// GPU execution
    Gpu { device_id: u32 },
}

/// Inference statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceStats {
    /// Total number of inferences
    pub total_inferences: u64,

    /// Average latency
    pub avg_latency_ms: f64,

    /// P50 latency
    pub p50_latency_ms: f64,

    /// P95 latency
    pub p95_latency_ms: f64,

    /// P99 latency
    pub p99_latency_ms: f64,

    /// Throughput (inferences per second)
    pub throughput: f64,

    /// Error rate
    pub error_rate: f64,

    /// Device utilization
    pub device_utilization: f64,
}

impl Default for InferenceRequest {
    fn default() -> Self {
        Self {
            input: Array2::zeros((1, 1)),
            request_id: uuid::Uuid::new_v4().to_string(),
            priority: 5,
            timeout: Duration::from_secs(5),
        }
    }
}

impl InferenceRequest {
    /// Create a new inference request
    pub fn new(input: Array2<f64>) -> Self {
        Self {
            input,
            request_id: uuid::Uuid::new_v4().to_string(),
            priority: 5,
            timeout: Duration::from_secs(5),
        }
    }

    /// Set request priority
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    /// Set request timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

impl InferenceResult {
    /// Create a new inference result
    pub fn new(
        output: Array2<f64>,
        request_id: String,
        latency: Duration,
        model_version: String,
        device: Device,
    ) -> Self {
        Self {
            output,
            request_id,
            latency,
            model_version,
            device,
        }
    }
}

impl Default for InferenceStats {
    fn default() -> Self {
        Self {
            total_inferences: 0,
            avg_latency_ms: 0.0,
            p50_latency_ms: 0.0,
            p95_latency_ms: 0.0,
            p99_latency_ms: 0.0,
            throughput: 0.0,
            error_rate: 0.0,
            device_utilization: 0.0,
        }
    }
}

// Re-export commonly used types
pub use batch_processor::BatchProcessor;
pub use gpu_accelerator::GpuAccelerator;
pub use onnx_runtime::OnnxModel;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inference_request() {
        let input = Array2::zeros((1, 10));
        let req = InferenceRequest::new(input);

        assert_eq!(req.priority, 5);
        assert_eq!(req.timeout, Duration::from_secs(5));
    }

    #[test]
    fn test_device() {
        let cpu = Device::Cpu;
        let gpu = Device::Gpu { device_id: 0 };

        assert_ne!(cpu, gpu);
    }
}
