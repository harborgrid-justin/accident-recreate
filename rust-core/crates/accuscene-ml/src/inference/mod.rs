//! Inference engine module

use crate::error::Result;
use crate::model::Model;
use async_trait::async_trait;
use ndarray::{Array1, Array2};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub mod batch;
pub mod realtime;

#[cfg(feature = "onnx")]
pub mod onnx;

pub use batch::{BatchInferenceEngine, BatchRequest, BatchResponse};
pub use realtime::{RealtimeInferenceEngine, InferenceRequest, InferenceResponse};

/// Inference engine trait
#[async_trait]
pub trait InferenceEngine: Send + Sync {
    /// Predict single sample
    async fn predict(&self, features: Array1<f64>) -> Result<InferenceResult>;

    /// Predict batch of samples
    async fn predict_batch(&self, features: Array2<f64>) -> Result<Vec<InferenceResult>>;

    /// Get model information
    fn model_info(&self) -> ModelInfo;

    /// Health check
    async fn health_check(&self) -> Result<HealthStatus>;
}

/// Inference result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResult {
    /// Prediction value(s)
    pub prediction: Vec<f64>,

    /// Confidence scores (if available)
    pub confidence: Option<Vec<f64>>,

    /// Inference time in milliseconds
    pub inference_time_ms: f64,

    /// Model version used
    pub model_version: String,

    /// Additional metadata
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl InferenceResult {
    /// Create a new inference result
    pub fn new(prediction: Vec<f64>, model_version: impl Into<String>) -> Self {
        Self {
            prediction,
            confidence: None,
            inference_time_ms: 0.0,
            model_version: model_version.into(),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Set confidence scores
    pub fn with_confidence(mut self, confidence: Vec<f64>) -> Self {
        self.confidence = Some(confidence);
        self
    }

    /// Set inference time
    pub fn with_inference_time(mut self, time_ms: f64) -> Self {
        self.inference_time_ms = time_ms;
        self
    }

    /// Add metadata
    pub fn add_metadata(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.metadata.insert(key.into(), value);
    }

    /// Get primary prediction
    pub fn primary_prediction(&self) -> Option<f64> {
        self.prediction.first().copied()
    }
}

/// Model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Model name
    pub name: String,

    /// Model version
    pub version: String,

    /// Model type
    pub model_type: String,

    /// Input features
    pub input_features: Vec<String>,

    /// Output type
    pub output_type: OutputType,

    /// Additional information
    pub metadata: std::collections::HashMap<String, String>,
}

impl ModelInfo {
    /// Create new model info
    pub fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        model_type: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            model_type: model_type.into(),
            input_features: Vec::new(),
            output_type: OutputType::Scalar,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Set input features
    pub fn with_input_features(mut self, features: Vec<String>) -> Self {
        self.input_features = features;
        self
    }

    /// Set output type
    pub fn with_output_type(mut self, output_type: OutputType) -> Self {
        self.output_type = output_type;
        self
    }
}

/// Output type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OutputType {
    /// Single value
    Scalar,

    /// Multiple values (regression)
    Vector,

    /// Class probabilities
    Probabilities,

    /// Class label
    Classification,

    /// Cluster assignment
    Clustering,
}

/// Health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Standard inference engine implementation
pub struct StandardInferenceEngine {
    /// Model
    model: Arc<dyn Model>,

    /// Model info
    info: ModelInfo,
}

impl StandardInferenceEngine {
    /// Create a new standard inference engine
    pub fn new(model: Arc<dyn Model>, info: ModelInfo) -> Self {
        Self { model, info }
    }
}

#[async_trait]
impl InferenceEngine for StandardInferenceEngine {
    async fn predict(&self, features: Array1<f64>) -> Result<InferenceResult> {
        let start = std::time::Instant::now();

        let prediction = self.model.predict(&features).await?;

        let inference_time_ms = start.elapsed().as_secs_f64() * 1000.0;

        Ok(InferenceResult::new(
            vec![prediction],
            self.info.version.clone(),
        )
        .with_inference_time(inference_time_ms))
    }

    async fn predict_batch(&self, features: Array2<f64>) -> Result<Vec<InferenceResult>> {
        let start = std::time::Instant::now();

        let predictions = self.model.predict_batch(&features).await?;

        let inference_time_ms = start.elapsed().as_secs_f64() * 1000.0;
        let time_per_sample = inference_time_ms / predictions.len() as f64;

        let results = predictions
            .iter()
            .map(|&pred| {
                InferenceResult::new(vec![pred], self.info.version.clone())
                    .with_inference_time(time_per_sample)
            })
            .collect();

        Ok(results)
    }

    fn model_info(&self) -> ModelInfo {
        self.info.clone()
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        // Simple health check - could be enhanced
        Ok(HealthStatus::Healthy)
    }
}

/// Inference metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InferenceMetrics {
    /// Total predictions made
    pub total_predictions: u64,

    /// Total inference time (milliseconds)
    pub total_inference_time_ms: f64,

    /// Average inference time (milliseconds)
    pub avg_inference_time_ms: f64,

    /// Min inference time (milliseconds)
    pub min_inference_time_ms: f64,

    /// Max inference time (milliseconds)
    pub max_inference_time_ms: f64,

    /// Error count
    pub error_count: u64,
}

impl InferenceMetrics {
    /// Create new metrics
    pub fn new() -> Self {
        Self {
            total_predictions: 0,
            total_inference_time_ms: 0.0,
            avg_inference_time_ms: 0.0,
            min_inference_time_ms: f64::MAX,
            max_inference_time_ms: 0.0,
            error_count: 0,
        }
    }

    /// Record a prediction
    pub fn record_prediction(&mut self, inference_time_ms: f64) {
        self.total_predictions += 1;
        self.total_inference_time_ms += inference_time_ms;
        self.avg_inference_time_ms = self.total_inference_time_ms / self.total_predictions as f64;
        self.min_inference_time_ms = self.min_inference_time_ms.min(inference_time_ms);
        self.max_inference_time_ms = self.max_inference_time_ms.max(inference_time_ms);
    }

    /// Record an error
    pub fn record_error(&mut self) {
        self.error_count += 1;
    }

    /// Get success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_predictions + self.error_count == 0 {
            return 1.0;
        }
        self.total_predictions as f64 / (self.total_predictions + self.error_count) as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inference_result() {
        let result = InferenceResult::new(vec![1.5], "1.0.0")
            .with_confidence(vec![0.95])
            .with_inference_time(10.5);

        assert_eq!(result.prediction, vec![1.5]);
        assert_eq!(result.confidence, Some(vec![0.95]));
        assert_eq!(result.inference_time_ms, 10.5);
        assert_eq!(result.primary_prediction(), Some(1.5));
    }

    #[test]
    fn test_inference_metrics() {
        let mut metrics = InferenceMetrics::new();

        metrics.record_prediction(10.0);
        metrics.record_prediction(20.0);
        metrics.record_error();

        assert_eq!(metrics.total_predictions, 2);
        assert_eq!(metrics.avg_inference_time_ms, 15.0);
        assert_eq!(metrics.error_count, 1);
        assert!((metrics.success_rate() - 0.666).abs() < 0.01);
    }
}
