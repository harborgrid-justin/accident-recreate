//! Batch inference engine

use crate::error::Result;
use crate::inference::{InferenceEngine, InferenceResult, HealthStatus, ModelInfo};
use async_trait::async_trait;
use ndarray::{Array1, Array2};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Semaphore;
use uuid::Uuid;

/// Batch inference engine
pub struct BatchInferenceEngine {
    /// Underlying inference engine
    engine: Arc<dyn InferenceEngine>,

    /// Maximum batch size
    max_batch_size: usize,

    /// Number of parallel workers
    num_workers: usize,

    /// Semaphore for controlling concurrency
    semaphore: Arc<Semaphore>,
}

impl BatchInferenceEngine {
    /// Create a new batch inference engine
    pub fn new(engine: Arc<dyn InferenceEngine>) -> Self {
        let num_workers = num_cpus::get();
        Self {
            engine,
            max_batch_size: 1000,
            num_workers,
            semaphore: Arc::new(Semaphore::new(num_workers)),
        }
    }

    /// Set maximum batch size
    pub fn with_max_batch_size(mut self, size: usize) -> Self {
        self.max_batch_size = size;
        self
    }

    /// Set number of parallel workers
    pub fn with_num_workers(mut self, workers: usize) -> Self {
        self.num_workers = workers;
        self.semaphore = Arc::new(Semaphore::new(workers));
        self
    }

    /// Process a batch request
    pub async fn process_batch(&self, request: BatchRequest) -> Result<BatchResponse> {
        let batch_id = Uuid::new_v4();
        let start_time = std::time::Instant::now();

        // Split into chunks if needed
        let chunks = self.split_into_chunks(&request.features);

        let mut all_results = Vec::new();

        // Process chunks in parallel
        for chunk in chunks {
            let _permit = self.semaphore.acquire().await.unwrap();

            let results = self.engine.predict_batch(chunk).await?;
            all_results.extend(results);
        }

        let processing_time_ms = start_time.elapsed().as_secs_f64() * 1000.0;

        Ok(BatchResponse {
            batch_id,
            results: all_results,
            total_samples: request.features.nrows(),
            processing_time_ms,
            metadata: request.metadata,
        })
    }

    /// Process multiple batch requests concurrently
    pub async fn process_batches(
        &self,
        requests: Vec<BatchRequest>,
    ) -> Result<Vec<BatchResponse>> {
        let mut handles = Vec::new();

        for request in requests {
            let engine = Arc::clone(&self.engine);
            let semaphore = Arc::clone(&self.semaphore);

            let handle = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                engine.predict_batch(request.features).await
            });

            handles.push(handle);
        }

        let mut responses = Vec::new();
        for handle in handles {
            let results = handle.await.map_err(|e| {
                crate::error::MLError::inference(format!("Batch processing failed: {}", e))
            })??;

            responses.push(BatchResponse {
                batch_id: Uuid::new_v4(),
                results,
                total_samples: 0,
                processing_time_ms: 0.0,
                metadata: std::collections::HashMap::new(),
            });
        }

        Ok(responses)
    }

    /// Split features into chunks
    fn split_into_chunks(&self, features: &Array2<f64>) -> Vec<Array2<f64>> {
        let n_samples = features.nrows();
        let mut chunks = Vec::new();

        let mut start = 0;
        while start < n_samples {
            let end = (start + self.max_batch_size).min(n_samples);
            let chunk = features.slice(ndarray::s![start..end, ..]).to_owned();
            chunks.push(chunk);
            start = end;
        }

        chunks
    }
}

#[async_trait]
impl InferenceEngine for BatchInferenceEngine {
    async fn predict(&self, features: Array1<f64>) -> Result<InferenceResult> {
        self.engine.predict(features).await
    }

    async fn predict_batch(&self, features: Array2<f64>) -> Result<Vec<InferenceResult>> {
        // Use parallel processing for large batches
        if features.nrows() > 100 && self.num_workers > 1 {
            self.predict_batch_parallel(features).await
        } else {
            self.engine.predict_batch(features).await
        }
    }

    fn model_info(&self) -> ModelInfo {
        self.engine.model_info()
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        self.engine.health_check().await
    }
}

impl BatchInferenceEngine {
    /// Predict batch using parallel processing
    async fn predict_batch_parallel(&self, features: Array2<f64>) -> Result<Vec<InferenceResult>> {
        let chunks = self.split_into_chunks(&features);
        let mut all_results = Vec::new();

        for chunk in chunks {
            let results = self.engine.predict_batch(chunk).await?;
            all_results.extend(results);
        }

        Ok(all_results)
    }
}

/// Batch inference request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRequest {
    /// Feature matrix
    pub features: Array2<f64>,

    /// Request metadata
    pub metadata: std::collections::HashMap<String, String>,

    /// Priority (higher values = higher priority)
    pub priority: u8,
}

impl BatchRequest {
    /// Create a new batch request
    pub fn new(features: Array2<f64>) -> Self {
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

/// Batch inference response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResponse {
    /// Batch ID
    pub batch_id: Uuid,

    /// Inference results
    pub results: Vec<InferenceResult>,

    /// Total number of samples processed
    pub total_samples: usize,

    /// Total processing time
    pub processing_time_ms: f64,

    /// Response metadata
    pub metadata: std::collections::HashMap<String, String>,
}

impl BatchResponse {
    /// Get average inference time per sample
    pub fn avg_time_per_sample(&self) -> f64 {
        if self.total_samples == 0 {
            return 0.0;
        }
        self.processing_time_ms / self.total_samples as f64
    }

    /// Get throughput (samples per second)
    pub fn throughput(&self) -> f64 {
        if self.processing_time_ms == 0.0 {
            return 0.0;
        }
        self.total_samples as f64 / (self.processing_time_ms / 1000.0)
    }
}

// Helper module for getting CPU count
mod num_cpus {
    pub fn get() -> usize {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_request() {
        use ndarray::arr2;

        let features = arr2(&[[1.0, 2.0], [3.0, 4.0]]);
        let mut request = BatchRequest::new(features).with_priority(5);

        request.add_metadata("source", "test");

        assert_eq!(request.priority, 5);
        assert_eq!(request.metadata.get("source"), Some(&"test".to_string()));
    }

    #[test]
    fn test_batch_response_metrics() {
        let response = BatchResponse {
            batch_id: Uuid::new_v4(),
            results: Vec::new(),
            total_samples: 100,
            processing_time_ms: 1000.0,
            metadata: std::collections::HashMap::new(),
        };

        assert_eq!(response.avg_time_per_sample(), 10.0);
        assert_eq!(response.throughput(), 100.0); // 100 samples per second
    }
}
