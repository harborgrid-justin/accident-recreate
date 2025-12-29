//! Data streaming pipeline with backpressure support

use crate::error::{PerformanceError, Result};
use crate::streaming::{BackpressureStrategy, Sink, Source, StreamStats};
use bytes::Bytes;
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;

/// Stream item with metadata
#[derive(Debug, Clone)]
pub struct StreamItem {
    /// Item data
    pub data: Bytes,
    /// Timestamp when item was created
    pub timestamp: Instant,
    /// Item sequence number
    pub sequence: u64,
    /// Item size in bytes
    pub size: usize,
}

impl StreamItem {
    /// Create a new stream item
    pub fn new(data: Bytes, sequence: u64) -> Self {
        let size = data.len();
        Self {
            data,
            timestamp: Instant::now(),
            sequence,
            size,
        }
    }

    /// Get latency in microseconds
    pub fn latency_us(&self) -> u64 {
        self.timestamp.elapsed().as_micros() as u64
    }
}

/// Pipeline configuration
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// Buffer size for pipeline
    pub buffer_size: usize,
    /// Maximum items in flight
    pub max_in_flight: usize,
    /// Backpressure strategy
    pub backpressure_strategy: BackpressureStrategy,
    /// Backpressure threshold (0.0-1.0)
    pub backpressure_threshold: f32,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            buffer_size: 8192,
            max_in_flight: 1000,
            backpressure_strategy: BackpressureStrategy::Block,
            backpressure_threshold: 0.8,
        }
    }
}

/// Data streaming pipeline
pub struct Pipeline {
    /// Pipeline configuration
    config: PipelineConfig,
    /// Pipeline statistics
    stats: Arc<RwLock<StreamStats>>,
    /// Channel sender
    tx: Option<mpsc::Sender<StreamItem>>,
    /// Channel receiver
    rx: Option<mpsc::Receiver<StreamItem>>,
    /// Is pipeline running
    running: Arc<RwLock<bool>>,
}

impl Pipeline {
    /// Create a new pipeline
    pub fn new(config: PipelineConfig) -> Self {
        let (tx, rx) = mpsc::channel(config.buffer_size);
        Self {
            config,
            stats: Arc::new(RwLock::new(StreamStats::new())),
            tx: Some(tx),
            rx: Some(rx),
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Create a pipeline builder
    pub fn builder() -> PipelineBuilder {
        PipelineBuilder::new()
    }

    /// Send an item to the pipeline
    pub async fn send(&self, item: StreamItem) -> Result<()> {
        let tx = self.tx.as_ref().ok_or(PerformanceError::StreamClosed)?;

        // Check buffer utilization for backpressure
        let utilization = tx.capacity() as f32 / self.config.buffer_size as f32;
        let mut stats = self.stats.write();
        stats.update_buffer_utilization(
            (utilization * self.config.buffer_size as f32) as usize,
            self.config.buffer_size,
        );

        if utilization > self.config.backpressure_threshold {
            match self.config.backpressure_strategy {
                BackpressureStrategy::Block => {
                    // Block until space is available
                    tx.send(item).await.map_err(|_| PerformanceError::ChannelSend)?;
                }
                BackpressureStrategy::DropOldest | BackpressureStrategy::DropNewest => {
                    // Try to send, drop if full
                    if tx.try_send(item).is_err() {
                        stats.record_drop();
                    }
                }
                BackpressureStrategy::Signal => {
                    // Signal backpressure but still try to send
                    tx.send(item).await.map_err(|_| PerformanceError::ChannelSend)?;
                }
            }
        } else {
            tx.send(item).await.map_err(|_| PerformanceError::ChannelSend)?;
        }

        Ok(())
    }

    /// Receive an item from the pipeline
    pub async fn recv(&mut self) -> Result<StreamItem> {
        let rx = self.rx.as_mut().ok_or(PerformanceError::StreamClosed)?;
        let item = rx.recv().await.ok_or(PerformanceError::StreamClosed)?;

        // Record statistics
        let latency_us = item.latency_us();
        let mut stats = self.stats.write();
        stats.record_item(item.size, latency_us);

        Ok(item)
    }

    /// Get pipeline statistics
    pub fn stats(&self) -> StreamStats {
        self.stats.read().clone()
    }

    /// Check if pipeline is running
    pub fn is_running(&self) -> bool {
        *self.running.read()
    }

    /// Start the pipeline
    pub fn start(&self) {
        *self.running.write() = true;
    }

    /// Stop the pipeline
    pub fn stop(&self) {
        *self.running.write() = false;
    }

    /// Run the pipeline with a source and sink
    pub async fn run<S, K>(
        &mut self,
        mut source: S,
        mut sink: K,
    ) -> Result<()>
    where
        S: Source,
        K: Sink,
    {
        self.start();
        let start_time = Instant::now();

        while self.is_running() {
            match source.next().await {
                Ok(Some(data)) => {
                    let sequence = self.stats.read().items_processed;
                    let item = StreamItem::new(data, sequence);
                    self.send(item.clone()).await?;

                    // Forward to sink
                    sink.send(item).await?;
                }
                Ok(None) => {
                    // Source exhausted
                    break;
                }
                Err(e) => {
                    eprintln!("Pipeline error: {}", e);
                    break;
                }
            }
        }

        self.stop();
        sink.flush().await?;

        let elapsed = start_time.elapsed().as_secs_f64();
        let stats = self.stats();
        println!(
            "Pipeline completed: {} items, {:.2} items/sec, {:.2} MB/sec",
            stats.items_processed,
            stats.throughput_items_per_sec(elapsed),
            stats.throughput_bytes_per_sec(elapsed) / 1_000_000.0
        );

        Ok(())
    }
}

/// Pipeline builder
pub struct PipelineBuilder {
    config: PipelineConfig,
}

impl PipelineBuilder {
    /// Create a new pipeline builder
    pub fn new() -> Self {
        Self {
            config: PipelineConfig::default(),
        }
    }

    /// Set buffer size
    pub fn buffer_size(mut self, size: usize) -> Self {
        self.config.buffer_size = size;
        self
    }

    /// Set max in-flight items
    pub fn max_in_flight(mut self, max: usize) -> Self {
        self.config.max_in_flight = max;
        self
    }

    /// Set backpressure strategy
    pub fn backpressure_strategy(mut self, strategy: BackpressureStrategy) -> Self {
        self.config.backpressure_strategy = strategy;
        self
    }

    /// Set backpressure threshold
    pub fn backpressure_threshold(mut self, threshold: f32) -> Self {
        self.config.backpressure_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Build the pipeline
    pub fn build(self) -> Pipeline {
        Pipeline::new(self.config)
    }
}

impl Default for PipelineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pipeline_creation() {
        let pipeline = Pipeline::builder()
            .buffer_size(1024)
            .max_in_flight(100)
            .build();

        assert_eq!(pipeline.config.buffer_size, 1024);
        assert_eq!(pipeline.config.max_in_flight, 100);
    }

    #[tokio::test]
    async fn test_pipeline_send_recv() {
        let mut pipeline = Pipeline::builder().buffer_size(10).build();

        let data = Bytes::from("test data");
        let item = StreamItem::new(data, 0);

        pipeline.send(item).await.unwrap();
        let received = pipeline.recv().await.unwrap();

        assert_eq!(received.sequence, 0);
    }

    #[tokio::test]
    async fn test_pipeline_stats() {
        let mut pipeline = Pipeline::builder().build();

        let data = Bytes::from("test");
        let item = StreamItem::new(data, 0);

        pipeline.send(item).await.unwrap();
        pipeline.recv().await.unwrap();

        let stats = pipeline.stats();
        assert_eq!(stats.items_processed, 1);
        assert!(stats.bytes_processed > 0);
    }
}
