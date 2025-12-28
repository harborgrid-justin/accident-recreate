//! Telemetry data streaming for performance monitoring.

use crate::error::Result;
use crate::stream::DataStream;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Telemetry type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TelemetryType {
    CPU,
    Memory,
    Disk,
    Network,
    GPU,
    FrameRate,
    RenderTime,
    PhysicsTime,
    StreamThroughput,
    Latency,
    Custom(String),
}

/// Telemetry data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryData {
    pub metric_name: String,
    pub telemetry_type: TelemetryType,
    pub timestamp: i64,
    pub value: f64,
    pub unit: String,
    pub source: String,
    pub tags: std::collections::HashMap<String, String>,
}

impl TelemetryData {
    /// Create a new telemetry data point
    pub fn new(
        metric_name: String,
        telemetry_type: TelemetryType,
        value: f64,
        unit: String,
    ) -> Self {
        Self {
            metric_name,
            telemetry_type,
            timestamp: chrono::Utc::now().timestamp_millis(),
            value,
            unit,
            source: "accuscene".to_string(),
            tags: std::collections::HashMap::new(),
        }
    }

    /// Set source
    pub fn with_source(mut self, source: String) -> Self {
        self.source = source;
        self
    }

    /// Add tag
    pub fn with_tag(mut self, key: String, value: String) -> Self {
        self.tags.insert(key, value);
        self
    }

    /// Create CPU usage metric
    pub fn cpu_usage(percentage: f64) -> Self {
        Self::new(
            "cpu_usage".to_string(),
            TelemetryType::CPU,
            percentage,
            "%".to_string(),
        )
    }

    /// Create memory usage metric
    pub fn memory_usage(bytes: f64) -> Self {
        Self::new(
            "memory_usage".to_string(),
            TelemetryType::Memory,
            bytes,
            "bytes".to_string(),
        )
    }

    /// Create frame rate metric
    pub fn frame_rate(fps: f64) -> Self {
        Self::new(
            "frame_rate".to_string(),
            TelemetryType::FrameRate,
            fps,
            "fps".to_string(),
        )
    }

    /// Create latency metric
    pub fn latency(milliseconds: f64) -> Self {
        Self::new(
            "latency".to_string(),
            TelemetryType::Latency,
            milliseconds,
            "ms".to_string(),
        )
    }

    /// Create throughput metric
    pub fn throughput(items_per_second: f64) -> Self {
        Self::new(
            "throughput".to_string(),
            TelemetryType::StreamThroughput,
            items_per_second,
            "items/s".to_string(),
        )
    }
}

/// Telemetry stream for performance metrics
pub struct TelemetryStream<S>
where
    S: DataStream<Item = TelemetryData>,
{
    inner: S,
    filter_types: Option<Vec<TelemetryType>>,
}

impl<S> TelemetryStream<S>
where
    S: DataStream<Item = TelemetryData>,
{
    /// Create a new telemetry stream
    pub fn new(inner: S) -> Self {
        Self {
            inner,
            filter_types: None,
        }
    }

    /// Filter by telemetry types
    pub fn filter_types(mut self, types: Vec<TelemetryType>) -> Self {
        self.filter_types = Some(types);
        self
    }

    fn should_include(&self, data: &TelemetryData) -> bool {
        if let Some(ref types) = self.filter_types {
            types.contains(&data.telemetry_type)
        } else {
            true
        }
    }
}

#[async_trait]
impl<S> DataStream for TelemetryStream<S>
where
    S: DataStream<Item = TelemetryData>,
{
    type Item = TelemetryData;

    async fn next(&mut self) -> Result<Option<Self::Item>> {
        loop {
            match self.inner.next().await? {
                Some(data) => {
                    if self.should_include(&data) {
                        return Ok(Some(data));
                    }
                }
                None => return Ok(None),
            }
        }
    }

    fn is_complete(&self) -> bool {
        self.inner.is_complete()
    }
}

/// Telemetry statistics
#[derive(Debug, Clone)]
pub struct TelemetryStats {
    pub metric_stats: std::collections::HashMap<String, MetricStats>,
}

#[derive(Debug, Clone)]
pub struct MetricStats {
    pub count: u64,
    pub sum: f64,
    pub min: f64,
    pub max: f64,
    pub avg: f64,
}

impl MetricStats {
    pub fn new() -> Self {
        Self {
            count: 0,
            sum: 0.0,
            min: f64::MAX,
            max: f64::MIN,
            avg: 0.0,
        }
    }

    pub fn update(&mut self, value: f64) {
        self.count += 1;
        self.sum += value;
        self.min = self.min.min(value);
        self.max = self.max.max(value);
        self.avg = self.sum / self.count as f64;
    }
}

impl Default for MetricStats {
    fn default() -> Self {
        Self::new()
    }
}

impl TelemetryStats {
    pub fn new() -> Self {
        Self {
            metric_stats: std::collections::HashMap::new(),
        }
    }

    pub fn update(&mut self, data: &TelemetryData) {
        let stats = self
            .metric_stats
            .entry(data.metric_name.clone())
            .or_insert_with(MetricStats::new);
        stats.update(data.value);
    }

    pub fn get_metric(&self, metric_name: &str) -> Option<&MetricStats> {
        self.metric_stats.get(metric_name)
    }
}

impl Default for TelemetryStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Telemetry aggregator for time-series data
pub struct TelemetryAggregator {
    window_duration: std::time::Duration,
    data_points: Vec<TelemetryData>,
    stats: TelemetryStats,
}

impl TelemetryAggregator {
    pub fn new(window_duration: std::time::Duration) -> Self {
        Self {
            window_duration,
            data_points: Vec::new(),
            stats: TelemetryStats::new(),
        }
    }

    pub fn add(&mut self, data: TelemetryData) {
        self.stats.update(&data);
        self.data_points.push(data);
    }

    pub fn get_stats(&self) -> &TelemetryStats {
        &self.stats
    }

    pub fn clear(&mut self) {
        self.data_points.clear();
        self.stats = TelemetryStats::new();
    }
}

/// Performance monitor that collects system metrics
pub struct PerformanceMonitor {
    enabled: bool,
    collect_interval: std::time::Duration,
}

impl PerformanceMonitor {
    pub fn new(collect_interval: std::time::Duration) -> Self {
        Self {
            enabled: true,
            collect_interval,
        }
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Collect current metrics
    pub fn collect_metrics(&self) -> Vec<TelemetryData> {
        if !self.enabled {
            return Vec::new();
        }

        // In a real implementation, this would collect actual system metrics
        vec![
            TelemetryData::cpu_usage(45.5),
            TelemetryData::memory_usage(1024.0 * 1024.0 * 512.0), // 512 MB
            TelemetryData::frame_rate(60.0),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::iterator::IteratorSource;

    #[tokio::test]
    async fn test_telemetry_data() {
        let cpu_metric = TelemetryData::cpu_usage(75.5)
            .with_source("system_monitor".to_string())
            .with_tag("host".to_string(), "server1".to_string());

        assert_eq!(cpu_metric.telemetry_type, TelemetryType::CPU);
        assert_eq!(cpu_metric.value, 75.5);
        assert_eq!(cpu_metric.unit, "%");
    }

    #[tokio::test]
    async fn test_telemetry_stats() {
        let mut stats = TelemetryStats::new();

        stats.update(&TelemetryData::cpu_usage(50.0));
        stats.update(&TelemetryData::cpu_usage(75.0));
        stats.update(&TelemetryData::cpu_usage(60.0));

        let cpu_stats = stats.get_metric("cpu_usage").unwrap();
        assert_eq!(cpu_stats.count, 3);
        assert_eq!(cpu_stats.avg, 61.666666666666664);
        assert_eq!(cpu_stats.min, 50.0);
        assert_eq!(cpu_stats.max, 75.0);
    }

    #[tokio::test]
    async fn test_telemetry_stream() {
        let metrics = vec![
            TelemetryData::cpu_usage(50.0),
            TelemetryData::memory_usage(1024.0),
            TelemetryData::frame_rate(60.0),
        ];

        let source = IteratorSource::new(metrics.into_iter());
        let stream = TelemetryStream::new(source).filter_types(vec![TelemetryType::CPU]);

        // Would filter to only CPU metrics
    }
}
