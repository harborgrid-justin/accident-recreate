//! Configuration for the performance crate

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Streaming configuration
    pub streaming: StreamingConfig,

    /// Memory management configuration
    pub memory: MemoryConfig,

    /// Concurrency configuration
    pub concurrency: ConcurrencyConfig,

    /// Profiling configuration
    pub profiling: ProfilingConfig,

    /// Metrics configuration
    pub metrics: MetricsConfig,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            streaming: StreamingConfig::default(),
            memory: MemoryConfig::default(),
            concurrency: ConcurrencyConfig::default(),
            profiling: ProfilingConfig::default(),
            metrics: MetricsConfig::default(),
        }
    }
}

/// Streaming configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingConfig {
    /// Buffer size for streaming operations
    pub buffer_size: usize,

    /// Maximum number of items in flight
    pub max_in_flight: usize,

    /// Backpressure threshold (0.0-1.0)
    pub backpressure_threshold: f32,

    /// Enable zero-copy operations
    pub zero_copy: bool,

    /// Batch size for batch processing
    pub batch_size: usize,

    /// Window size for windowing operations
    pub window_size: usize,

    /// Window duration
    pub window_duration: Duration,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            buffer_size: 8192,
            max_in_flight: 1000,
            backpressure_threshold: 0.8,
            zero_copy: true,
            batch_size: 100,
            window_size: 1000,
            window_duration: Duration::from_secs(1),
        }
    }
}

/// Memory management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Pool size for object pooling
    pub pool_size: usize,

    /// Arena chunk size in bytes
    pub arena_chunk_size: usize,

    /// Slab object size in bytes
    pub slab_object_size: usize,

    /// Number of slabs
    pub slab_count: usize,

    /// Enable memory prefetching
    pub enable_prefetch: bool,

    /// Cache line size in bytes
    pub cache_line_size: usize,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            pool_size: 1024,
            arena_chunk_size: 64 * 1024, // 64KB
            slab_object_size: 4096,
            slab_count: 256,
            enable_prefetch: true,
            cache_line_size: 64,
        }
    }
}

/// Concurrency configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcurrencyConfig {
    /// Number of worker threads
    pub worker_threads: usize,

    /// Work stealing queue size
    pub work_queue_size: usize,

    /// Channel buffer size
    pub channel_buffer_size: usize,

    /// Enable lock-free structures
    pub lock_free: bool,

    /// Batch size for atomic operations
    pub atomic_batch_size: usize,
}

impl Default for ConcurrencyConfig {
    fn default() -> Self {
        Self {
            worker_threads: num_cpus(),
            work_queue_size: 256,
            channel_buffer_size: 1024,
            lock_free: true,
            atomic_batch_size: 64,
        }
    }
}

/// Profiling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilingConfig {
    /// Enable CPU profiling
    pub enable_cpu: bool,

    /// Enable memory profiling
    pub enable_memory: bool,

    /// Enable latency tracking
    pub enable_latency: bool,

    /// Enable flamegraph generation
    pub enable_flamegraph: bool,

    /// Sampling frequency in Hz
    pub sampling_frequency: u64,

    /// Latency histogram precision
    pub histogram_precision: u8,
}

impl Default for ProfilingConfig {
    fn default() -> Self {
        Self {
            enable_cpu: true,
            enable_memory: true,
            enable_latency: true,
            enable_flamegraph: false,
            sampling_frequency: 100,
            histogram_precision: 2,
        }
    }
}

/// Metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Enable metrics collection
    pub enabled: bool,

    /// Metrics reporting interval
    pub report_interval: Duration,

    /// Enable Prometheus export
    pub prometheus_enabled: bool,

    /// Prometheus port
    pub prometheus_port: u16,

    /// Maximum number of histograms
    pub max_histograms: usize,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            report_interval: Duration::from_secs(10),
            prometheus_enabled: true,
            prometheus_port: 9090,
            max_histograms: 100,
        }
    }
}

/// Get the number of CPUs
fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = PerformanceConfig::default();
        assert!(config.streaming.buffer_size > 0);
        assert!(config.memory.pool_size > 0);
        assert!(config.concurrency.worker_threads > 0);
    }

    #[test]
    fn test_serialization() {
        let config = PerformanceConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let decoded: PerformanceConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.streaming.buffer_size, decoded.streaming.buffer_size);
    }
}
