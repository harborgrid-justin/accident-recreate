//! Real-time inference engine

use crate::error::Result;
use crate::inference::{InferenceEngine, InferenceResult, HealthStatus, ModelInfo, InferenceMetrics};
use async_trait::async_trait;
use ndarray::{Array1, Array2};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

/// Real-time inference engine with request queue and metrics
pub struct RealtimeInferenceEngine {
    /// Underlying inference engine
    engine: Arc<dyn InferenceEngine>,

    /// Request queue sender
    request_tx: mpsc::Sender<InferenceTask>,

    /// Inference metrics
    metrics: Arc<RwLock<InferenceMetrics>>,

    /// Maximum queue size
    max_queue_size: usize,

    /// Request timeout
    request_timeout: Duration,
}

impl RealtimeInferenceEngine {
    /// Create a new real-time inference engine
    pub fn new(engine: Arc<dyn InferenceEngine>) -> Self {
        let max_queue_size = 1000;
        let (request_tx, request_rx) = mpsc::channel(max_queue_size);

        let engine_clone = Arc::clone(&engine);
        let metrics = Arc::new(RwLock::new(InferenceMetrics::new()));
        let metrics_clone = Arc::clone(&metrics);

        // Spawn worker task
        tokio::spawn(async move {
            Self::worker_loop(engine_clone, request_rx, metrics_clone).await;
        });

        Self {
            engine,
            request_tx,
            metrics,
            max_queue_size,
            request_timeout: Duration::from_secs(30),
        }
    }

    /// Set maximum queue size
    pub fn with_max_queue_size(mut self, size: usize) -> Self {
        self.max_queue_size = size;
        self
    }

    /// Set request timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = timeout;
        self
    }

    /// Process an inference request
    pub async fn infer(&self, request: InferenceRequest) -> Result<InferenceResponse> {
        let request_id = Uuid::new_v4();
        let (response_tx, response_rx) = oneshot::channel();

        let task = InferenceTask {
            request_id,
            features: request.features,
            response_tx,
            submitted_at: Instant::now(),
        };

        // Send task to queue
        self.request_tx
            .send(task)
            .await
            .map_err(|_| crate::error::MLError::inference("Request queue full"))?;

        // Wait for response with timeout
        let result = tokio::time::timeout(self.request_timeout, response_rx)
            .await
            .map_err(|_| crate::error::MLError::inference("Request timeout"))?
            .map_err(|_| crate::error::MLError::inference("Request cancelled"))?;

        Ok(InferenceResponse {
            request_id,
            result: result?,
            metadata: request.metadata,
        })
    }

    /// Get current metrics
    pub fn metrics(&self) -> InferenceMetrics {
        self.metrics.read().clone()
    }

    /// Reset metrics
    pub fn reset_metrics(&self) {
        *self.metrics.write() = InferenceMetrics::new();
    }

    /// Get queue size
    pub fn queue_size(&self) -> usize {
        self.request_tx.capacity() - self.request_tx.max_capacity()
    }

    /// Worker loop for processing inference requests
    async fn worker_loop(
        engine: Arc<dyn InferenceEngine>,
        mut request_rx: mpsc::Receiver<InferenceTask>,
        metrics: Arc<RwLock<InferenceMetrics>>,
    ) {
        while let Some(task) = request_rx.recv().await {
            let start = Instant::now();

            let result = engine.predict(task.features).await;

            let inference_time_ms = start.elapsed().as_secs_f64() * 1000.0;

            // Update metrics
            {
                let mut m = metrics.write();
                match &result {
                    Ok(_) => m.record_prediction(inference_time_ms),
                    Err(_) => m.record_error(),
                }
            }

            // Send response
            let _ = task.response_tx.send(result);
        }
    }
}

#[async_trait]
impl InferenceEngine for RealtimeInferenceEngine {
    async fn predict(&self, features: Array1<f64>) -> Result<InferenceResult> {
        let request = InferenceRequest::new(features);
        let response = self.infer(request).await?;
        Ok(response.result)
    }

    async fn predict_batch(&self, features: Array2<f64>) -> Result<Vec<InferenceResult>> {
        self.engine.predict_batch(features).await
    }

    fn model_info(&self) -> ModelInfo {
        self.engine.model_info()
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        let metrics = self.metrics.read();

        // Check error rate
        let success_rate = metrics.success_rate();

        if success_rate >= 0.95 {
            Ok(HealthStatus::Healthy)
        } else if success_rate >= 0.80 {
            Ok(HealthStatus::Degraded)
        } else {
            Ok(HealthStatus::Unhealthy)
        }
    }
}

/// Internal inference task
struct InferenceTask {
    request_id: Uuid,
    features: Array1<f64>,
    response_tx: oneshot::Sender<Result<InferenceResult>>,
    submitted_at: Instant,
}

/// Inference request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRequest {
    /// Input features
    pub features: Array1<f64>,

    /// Request metadata
    pub metadata: std::collections::HashMap<String, String>,

    /// Request priority (higher = more important)
    pub priority: u8,
}

impl InferenceRequest {
    /// Create a new inference request
    pub fn new(features: Array1<f64>) -> Self {
        Self {
            features,
            metadata: std::collections::HashMap::new(),
            priority: 0,
        }
    }

    /// Set priority
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    /// Add metadata
    pub fn add_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
    }
}

/// Inference response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResponse {
    /// Request ID
    pub request_id: Uuid,

    /// Inference result
    pub result: InferenceResult,

    /// Response metadata
    pub metadata: std::collections::HashMap<String, String>,
}

/// Streaming inference engine for continuous data
pub struct StreamingInferenceEngine {
    /// Underlying engine
    engine: Arc<dyn InferenceEngine>,

    /// Stream buffer size
    buffer_size: usize,
}

impl StreamingInferenceEngine {
    /// Create a new streaming inference engine
    pub fn new(engine: Arc<dyn InferenceEngine>) -> Self {
        Self {
            engine,
            buffer_size: 100,
        }
    }

    /// Set buffer size
    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// Process a stream of features
    pub async fn process_stream(
        &self,
        mut input: mpsc::Receiver<Array1<f64>>,
        output: mpsc::Sender<InferenceResult>,
    ) -> Result<()> {
        let mut buffer = Vec::with_capacity(self.buffer_size);

        while let Some(features) = input.recv().await {
            buffer.push(features);

            // Process batch when buffer is full
            if buffer.len() >= self.buffer_size {
                self.process_buffer(&mut buffer, &output).await?;
            }
        }

        // Process remaining items
        if !buffer.is_empty() {
            self.process_buffer(&mut buffer, &output).await?;
        }

        Ok(())
    }

    async fn process_buffer(
        &self,
        buffer: &mut Vec<Array1<f64>>,
        output: &mpsc::Sender<InferenceResult>,
    ) -> Result<()> {
        // Convert buffer to matrix
        let n_samples = buffer.len();
        let n_features = buffer[0].len();

        let mut matrix = Array2::zeros((n_samples, n_features));
        for (i, features) in buffer.iter().enumerate() {
            matrix.row_mut(i).assign(features);
        }

        // Run batch inference
        let results = self.engine.predict_batch(matrix).await?;

        // Send results
        for result in results {
            if output.send(result).await.is_err() {
                return Err(crate::error::MLError::inference("Output channel closed"));
            }
        }

        buffer.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inference_request() {
        use ndarray::arr1;

        let features = arr1(&[1.0, 2.0, 3.0]);
        let mut request = InferenceRequest::new(features).with_priority(5);

        request.add_metadata("source", "test");

        assert_eq!(request.priority, 5);
        assert_eq!(request.metadata.get("source"), Some(&"test".to_string()));
    }
}
