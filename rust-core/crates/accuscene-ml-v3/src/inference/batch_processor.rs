//! Batch inference processing for efficiency

use crate::error::{MlError, Result};
use crate::inference::{InferenceRequest, InferenceResult, OnnxModel};
use ndarray::{Array2, Axis};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock, Semaphore};
use tokio::time::timeout;

/// Batch processor for efficient inference
pub struct BatchProcessor {
    model: Arc<OnnxModel>,
    batch_size: usize,
    max_wait_time: Duration,
    max_concurrent_batches: usize,
    semaphore: Arc<Semaphore>,
    request_buffer: Arc<RwLock<Vec<InferenceRequest>>>,
}

/// Batch processing statistics
#[derive(Debug, Clone)]
pub struct BatchStats {
    /// Total batches processed
    pub total_batches: u64,

    /// Total requests processed
    pub total_requests: u64,

    /// Average batch size
    pub avg_batch_size: f64,

    /// Average batch processing time
    pub avg_batch_time_ms: f64,

    /// Throughput (requests per second)
    pub throughput: f64,

    /// Batch utilization (actual vs max batch size)
    pub batch_utilization: f64,
}

impl BatchProcessor {
    /// Create a new batch processor
    pub fn new(
        model: Arc<OnnxModel>,
        batch_size: usize,
        max_wait_time: Duration,
        max_concurrent_batches: usize,
    ) -> Self {
        Self {
            model,
            batch_size,
            max_wait_time,
            max_concurrent_batches,
            semaphore: Arc::new(Semaphore::new(max_concurrent_batches)),
            request_buffer: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Process a single request (will be batched automatically)
    pub async fn process_request(&self, request: InferenceRequest) -> Result<InferenceResult> {
        // Add request to buffer
        {
            let mut buffer = self.request_buffer.write().await;
            buffer.push(request.clone());
        }

        // Wait for batch to fill or timeout
        let batch_start = Instant::now();

        loop {
            let buffer_len = self.request_buffer.read().await.len();

            if buffer_len >= self.batch_size
                || batch_start.elapsed() >= self.max_wait_time
            {
                break;
            }

            tokio::time::sleep(Duration::from_millis(1)).await;
        }

        // Process batch
        let batch = {
            let mut buffer = self.request_buffer.write().await;
            let batch_end = buffer.len().min(self.batch_size);
            buffer.drain(0..batch_end).collect::<Vec<_>>()
        };

        self.process_batch(batch).await
            .and_then(|results| {
                results
                    .into_iter()
                    .find(|r| r.request_id == request.request_id)
                    .ok_or_else(|| {
                        MlError::BatchProcessing("Request not found in batch results".to_string())
                    })
            })
    }

    /// Process a batch of requests
    pub async fn process_batch(
        &self,
        requests: Vec<InferenceRequest>,
    ) -> Result<Vec<InferenceResult>> {
        if requests.is_empty() {
            return Ok(vec![]);
        }

        // Acquire semaphore permit
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|e| MlError::BatchProcessing(e.to_string()))?;

        let start = Instant::now();
        let batch_size = requests.len();

        tracing::debug!("Processing batch of {} requests", batch_size);

        // Validate all inputs have same shape
        let first_shape = requests[0].input.shape();
        for req in &requests[1..] {
            if req.input.shape() != first_shape {
                return Err(MlError::ShapeMismatch {
                    expected: first_shape.to_vec(),
                    actual: req.input.shape().to_vec(),
                });
            }
        }

        // Concatenate inputs into batch
        let batched_input = self.concatenate_inputs(&requests)?;

        // Run batch inference
        let batched_output = self.model.run(batched_input).await?;

        // Split outputs back into individual results
        let results = self.split_outputs(batched_output, requests, start)?;

        let batch_time = start.elapsed();
        tracing::debug!(
            "Batch of {} completed in {:?} ({:.2} req/sec)",
            batch_size,
            batch_time,
            batch_size as f64 / batch_time.as_secs_f64()
        );

        Ok(results)
    }

    /// Concatenate multiple inputs into a single batch
    fn concatenate_inputs(&self, requests: &[InferenceRequest]) -> Result<Array2<f64>> {
        let mut rows = Vec::new();

        for request in requests {
            rows.push(request.input.clone());
        }

        let concatenated = ndarray::concatenate(
            Axis(0),
            &rows.iter().map(|r| r.view()).collect::<Vec<_>>(),
        )
        .map_err(|e| MlError::BatchProcessing(e.to_string()))?;

        Ok(concatenated)
    }

    /// Split batch output into individual results
    fn split_outputs(
        &self,
        batched_output: Array2<f64>,
        requests: Vec<InferenceRequest>,
        start_time: Instant,
    ) -> Result<Vec<InferenceResult>> {
        let output_dim = batched_output.ncols();
        let mut results = Vec::with_capacity(requests.len());

        for (i, request) in requests.into_iter().enumerate() {
            let row = batched_output
                .row(i)
                .to_owned()
                .into_shape((1, output_dim))
                .map_err(|e| MlError::BatchProcessing(e.to_string()))?;

            results.push(InferenceResult::new(
                row,
                request.request_id,
                start_time.elapsed(),
                self.model.version().to_string(),
                self.model.device(),
            ));
        }

        Ok(results)
    }

    /// Process multiple batches concurrently
    pub async fn process_batches_concurrent(
        &self,
        batches: Vec<Vec<InferenceRequest>>,
    ) -> Result<Vec<Vec<InferenceResult>>> {
        let mut handles = vec![];

        for batch in batches {
            let processor = self.clone_arc();
            let handle = tokio::spawn(async move {
                processor.process_batch(batch).await
            });
            handles.push(handle);
        }

        let mut all_results = Vec::new();

        for handle in handles {
            let results = handle
                .await
                .map_err(|e| MlError::BatchProcessing(e.to_string()))??;
            all_results.push(results);
        }

        Ok(all_results)
    }

    /// Get current buffer size
    pub async fn buffer_size(&self) -> usize {
        self.request_buffer.read().await.len()
    }

    /// Clear the request buffer
    pub async fn clear_buffer(&self) {
        self.request_buffer.write().await.clear();
    }

    /// Clone with Arc for concurrent processing
    fn clone_arc(&self) -> Arc<Self> {
        Arc::new(Self {
            model: Arc::clone(&self.model),
            batch_size: self.batch_size,
            max_wait_time: self.max_wait_time,
            max_concurrent_batches: self.max_concurrent_batches,
            semaphore: Arc::clone(&self.semaphore),
            request_buffer: Arc::clone(&self.request_buffer),
        })
    }
}

/// Adaptive batch processor that dynamically adjusts batch size
pub struct AdaptiveBatchProcessor {
    inner: BatchProcessor,
    min_batch_size: usize,
    max_batch_size: usize,
    current_batch_size: Arc<RwLock<usize>>,
    stats: Arc<RwLock<BatchStats>>,
}

impl AdaptiveBatchProcessor {
    /// Create a new adaptive batch processor
    pub fn new(
        model: Arc<OnnxModel>,
        min_batch_size: usize,
        max_batch_size: usize,
        max_wait_time: Duration,
        max_concurrent_batches: usize,
    ) -> Self {
        let initial_batch_size = (min_batch_size + max_batch_size) / 2;

        Self {
            inner: BatchProcessor::new(
                model,
                initial_batch_size,
                max_wait_time,
                max_concurrent_batches,
            ),
            min_batch_size,
            max_batch_size,
            current_batch_size: Arc::new(RwLock::new(initial_batch_size)),
            stats: Arc::new(RwLock::new(BatchStats::default())),
        }
    }

    /// Process request with adaptive batching
    pub async fn process_request(&self, request: InferenceRequest) -> Result<InferenceResult> {
        let result = self.inner.process_request(request).await?;

        // Update statistics
        self.update_stats(&result).await;

        // Adjust batch size if needed
        self.adjust_batch_size().await;

        Ok(result)
    }

    /// Update statistics
    async fn update_stats(&self, result: &InferenceResult) {
        let mut stats = self.stats.write().await;
        stats.total_requests += 1;

        let latency_ms = result.latency.as_millis() as f64;
        stats.avg_batch_time_ms =
            (stats.avg_batch_time_ms * (stats.total_batches as f64) + latency_ms)
                / (stats.total_batches + 1) as f64;
    }

    /// Adjust batch size based on performance
    async fn adjust_batch_size(&self) {
        let stats = self.stats.read().await;
        let mut current_size = self.current_batch_size.write().await;

        // Increase batch size if latency is acceptable
        if stats.avg_batch_time_ms < 50.0 && *current_size < self.max_batch_size {
            *current_size = (*current_size + 1).min(self.max_batch_size);
            tracing::debug!("Increasing batch size to {}", *current_size);
        }
        // Decrease batch size if latency is too high
        else if stats.avg_batch_time_ms > 200.0 && *current_size > self.min_batch_size {
            *current_size = (*current_size - 1).max(self.min_batch_size);
            tracing::debug!("Decreasing batch size to {}", *current_size);
        }
    }

    /// Get current statistics
    pub async fn stats(&self) -> BatchStats {
        self.stats.read().await.clone()
    }
}

impl Default for BatchStats {
    fn default() -> Self {
        Self {
            total_batches: 0,
            total_requests: 0,
            avg_batch_size: 0.0,
            avg_batch_time_ms: 0.0,
            throughput: 0.0,
            batch_utilization: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::MlConfig;
    use ndarray::Array2;

    #[test]
    fn test_batch_stats() {
        let stats = BatchStats::default();
        assert_eq!(stats.total_batches, 0);
        assert_eq!(stats.total_requests, 0);
    }

    #[tokio::test]
    #[ignore] // Requires actual ONNX model
    async fn test_batch_concatenation() {
        // Test would go here with actual model
    }
}
