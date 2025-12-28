//! Configuration for the streaming pipeline.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for the streaming pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingConfig {
    /// Buffer configuration
    pub buffer: BufferConfig,
    /// Backpressure configuration
    pub backpressure: BackpressureConfig,
    /// Checkpoint configuration
    pub checkpoint: CheckpointConfig,
    /// Watermark configuration
    pub watermark: WatermarkConfig,
    /// Runtime configuration
    pub runtime: RuntimeConfig,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            buffer: BufferConfig::default(),
            backpressure: BackpressureConfig::default(),
            checkpoint: CheckpointConfig::default(),
            watermark: WatermarkConfig::default(),
            runtime: RuntimeConfig::default(),
        }
    }
}

/// Buffer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferConfig {
    /// Default buffer capacity
    pub default_capacity: usize,
    /// Maximum buffer capacity
    pub max_capacity: usize,
    /// Whether buffers are bounded
    pub bounded: bool,
}

impl Default for BufferConfig {
    fn default() -> Self {
        Self {
            default_capacity: 1000,
            max_capacity: 100_000,
            bounded: true,
        }
    }
}

/// Backpressure configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackpressureConfig {
    /// Backpressure strategy
    pub strategy: BackpressureStrategy,
    /// High watermark threshold (0.0-1.0)
    pub high_watermark: f64,
    /// Low watermark threshold (0.0-1.0)
    pub low_watermark: f64,
    /// Timeout for backpressure operations
    pub timeout: Duration,
}

impl Default for BackpressureConfig {
    fn default() -> Self {
        Self {
            strategy: BackpressureStrategy::Block,
            high_watermark: 0.9,
            low_watermark: 0.5,
            timeout: Duration::from_secs(30),
        }
    }
}

/// Backpressure strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackpressureStrategy {
    /// Block when buffer is full
    Block,
    /// Drop oldest items when buffer is full
    DropOldest,
    /// Drop newest items when buffer is full
    DropNewest,
    /// Fail when buffer is full
    Fail,
}

/// Checkpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointConfig {
    /// Enable checkpointing
    pub enabled: bool,
    /// Checkpoint interval
    pub interval: Duration,
    /// Maximum number of checkpoints to retain
    pub max_retained: usize,
    /// Checkpoint storage path
    pub storage_path: Option<String>,
    /// Checkpoint timeout
    pub timeout: Duration,
}

impl Default for CheckpointConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: Duration::from_secs(60),
            max_retained: 5,
            storage_path: None,
            timeout: Duration::from_secs(30),
        }
    }
}

/// Watermark configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatermarkConfig {
    /// Enable watermark tracking
    pub enabled: bool,
    /// Maximum allowed lateness for events
    pub max_lateness: Duration,
    /// Watermark update interval
    pub update_interval: Duration,
    /// Idle source timeout
    pub idle_timeout: Duration,
}

impl Default for WatermarkConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_lateness: Duration::from_secs(10),
            update_interval: Duration::from_millis(100),
            idle_timeout: Duration::from_secs(60),
        }
    }
}

/// Runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// Number of worker threads
    pub worker_threads: usize,
    /// Enable work stealing
    pub work_stealing: bool,
    /// Task queue capacity
    pub task_queue_capacity: usize,
    /// Shutdown timeout
    pub shutdown_timeout: Duration,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            worker_threads: num_cpus(),
            work_stealing: true,
            task_queue_capacity: 10_000,
            shutdown_timeout: Duration::from_secs(30),
        }
    }
}

fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}

/// Window configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    /// Window size
    pub size: Duration,
    /// Window slide/step
    pub slide: Option<Duration>,
    /// Maximum allowed lateness
    pub allowed_lateness: Duration,
}

impl WindowConfig {
    /// Create a tumbling window configuration
    pub fn tumbling(size: Duration) -> Self {
        Self {
            size,
            slide: None,
            allowed_lateness: Duration::from_secs(0),
        }
    }

    /// Create a sliding window configuration
    pub fn sliding(size: Duration, slide: Duration) -> Self {
        Self {
            size,
            slide: Some(slide),
            allowed_lateness: Duration::from_secs(0),
        }
    }

    /// Set allowed lateness
    pub fn with_allowed_lateness(mut self, lateness: Duration) -> Self {
        self.allowed_lateness = lateness;
        self
    }
}

/// Source configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceConfig {
    /// Source name
    pub name: String,
    /// Buffer capacity
    pub buffer_capacity: usize,
    /// Batch size
    pub batch_size: usize,
    /// Poll interval
    pub poll_interval: Option<Duration>,
}

impl Default for SourceConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            buffer_capacity: 1000,
            batch_size: 100,
            poll_interval: Some(Duration::from_millis(10)),
        }
    }
}

/// Sink configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SinkConfig {
    /// Sink name
    pub name: String,
    /// Buffer capacity
    pub buffer_capacity: usize,
    /// Batch size
    pub batch_size: usize,
    /// Flush interval
    pub flush_interval: Duration,
    /// Retry attempts
    pub retry_attempts: usize,
}

impl Default for SinkConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            buffer_capacity: 1000,
            batch_size: 100,
            flush_interval: Duration::from_secs(5),
            retry_attempts: 3,
        }
    }
}
