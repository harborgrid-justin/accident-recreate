//! High-performance streaming module with backpressure support

pub mod pipeline;
pub mod sink;
pub mod source;
pub mod transform;
pub mod window;

pub use pipeline::{Pipeline, PipelineBuilder, StreamItem};
pub use sink::{Sink, VectorSink, FileSink};
pub use source::{Source, VectorSource, IteratorSource};
pub use transform::{Transform, MapTransform, FilterTransform, FlatMapTransform};
pub use window::{Window, TimeWindow, CountWindow, SlidingWindow};

use std::pin::Pin;

/// Stream trait for async streaming operations
pub trait Stream: Send + Sync {
    /// Item type
    type Item: Send;

    /// Poll the next item from the stream
    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>>;

    /// Get stream size hint
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

/// Backpressure strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackpressureStrategy {
    /// Block when buffer is full
    Block,
    /// Drop oldest items when buffer is full
    DropOldest,
    /// Drop newest items when buffer is full
    DropNewest,
    /// Apply backpressure signal
    Signal,
}

/// Stream statistics
#[derive(Debug, Clone, Default)]
pub struct StreamStats {
    /// Total items processed
    pub items_processed: u64,
    /// Total bytes processed
    pub bytes_processed: u64,
    /// Items dropped due to backpressure
    pub items_dropped: u64,
    /// Current buffer utilization (0.0-1.0)
    pub buffer_utilization: f32,
    /// Average latency in microseconds
    pub avg_latency_us: f64,
    /// Peak latency in microseconds
    pub peak_latency_us: u64,
}

impl StreamStats {
    /// Create new statistics
    pub fn new() -> Self {
        Self::default()
    }

    /// Update statistics with new item
    pub fn record_item(&mut self, bytes: usize, latency_us: u64) {
        self.items_processed += 1;
        self.bytes_processed += bytes as u64;

        // Update average latency using exponential moving average
        let alpha = 0.1;
        self.avg_latency_us = self.avg_latency_us * (1.0 - alpha) + (latency_us as f64) * alpha;

        if latency_us > self.peak_latency_us {
            self.peak_latency_us = latency_us;
        }
    }

    /// Record dropped item
    pub fn record_drop(&mut self) {
        self.items_dropped += 1;
    }

    /// Update buffer utilization
    pub fn update_buffer_utilization(&mut self, used: usize, capacity: usize) {
        self.buffer_utilization = used as f32 / capacity.max(1) as f32;
    }

    /// Get throughput in items per second
    pub fn throughput_items_per_sec(&self, elapsed_secs: f64) -> f64 {
        self.items_processed as f64 / elapsed_secs.max(0.001)
    }

    /// Get throughput in bytes per second
    pub fn throughput_bytes_per_sec(&self, elapsed_secs: f64) -> f64 {
        self.bytes_processed as f64 / elapsed_secs.max(0.001)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_stats() {
        let mut stats = StreamStats::new();
        stats.record_item(1024, 100);
        stats.record_item(2048, 200);

        assert_eq!(stats.items_processed, 2);
        assert_eq!(stats.bytes_processed, 3072);
        assert!(stats.avg_latency_us > 0.0);
    }

    #[test]
    fn test_buffer_utilization() {
        let mut stats = StreamStats::new();
        stats.update_buffer_utilization(80, 100);
        assert!((stats.buffer_utilization - 0.8).abs() < 0.01);
    }
}
