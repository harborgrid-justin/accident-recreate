//! Configuration for the analytics engine

use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsConfig {
    /// Maximum number of metrics to track
    pub max_metrics: usize,

    /// Default retention period for metrics
    pub retention_period: Duration,

    /// Number of histogram buckets
    pub histogram_buckets: usize,

    /// Time series window size
    pub timeseries_window_size: usize,

    /// Enable parallel processing
    pub enable_parallel: bool,

    /// Number of worker threads
    pub worker_threads: usize,

    /// Anomaly detection sensitivity (0.0 - 1.0)
    pub anomaly_sensitivity: f64,

    /// Storage configuration
    pub storage: StorageConfig,

    /// Aggregation configuration
    pub aggregation: AggregationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Enable persistent storage
    pub enabled: bool,

    /// Storage path
    pub path: String,

    /// Flush interval
    pub flush_interval: Duration,

    /// Compression enabled
    pub compression: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationConfig {
    /// Temporal aggregation intervals
    pub temporal_intervals: Vec<TemporalInterval>,

    /// Spatial grid resolution (meters)
    pub spatial_resolution: f64,

    /// Maximum dimensions for multi-dimensional rollups
    pub max_dimensions: usize,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TemporalInterval {
    Minute,
    FiveMinutes,
    FifteenMinutes,
    Hour,
    Day,
    Week,
    Month,
}

impl Default for AnalyticsConfig {
    fn default() -> Self {
        Self {
            max_metrics: 10000,
            retention_period: Duration::from_secs(86400 * 30), // 30 days
            histogram_buckets: 100,
            timeseries_window_size: 1000,
            enable_parallel: true,
            worker_threads: num_cpus::get(),
            anomaly_sensitivity: 0.95,
            storage: StorageConfig::default(),
            aggregation: AggregationConfig::default(),
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            path: "/var/lib/accuscene/analytics".to_string(),
            flush_interval: Duration::from_secs(60),
            compression: true,
        }
    }
}

impl Default for AggregationConfig {
    fn default() -> Self {
        Self {
            temporal_intervals: vec![
                TemporalInterval::Minute,
                TemporalInterval::Hour,
                TemporalInterval::Day,
            ],
            spatial_resolution: 10.0, // 10 meters
            max_dimensions: 5,
        }
    }
}

// Add num_cpus as a dev dependency workaround
mod num_cpus {
    pub fn get() -> usize {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
    }
}
