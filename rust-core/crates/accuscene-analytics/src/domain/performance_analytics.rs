//! System performance metrics and monitoring

use crate::error::Result;
use crate::metrics::{Counter, Gauge, Histogram, TimeSeries};
use crate::statistics::descriptive::{DescriptiveStats, Statistics};
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Performance metrics for the analytics system
pub struct PerformanceAnalytics {
    query_latencies: Arc<RwLock<Vec<f64>>>,
    throughput_counter: Arc<Counter>,
    active_queries: Arc<Gauge>,
    error_counter: Arc<Counter>,
    cache_hits: Arc<Counter>,
    cache_misses: Arc<Counter>,
    processing_time: Arc<Histogram>,
    memory_usage: Arc<TimeSeries>,
    cpu_usage: Arc<TimeSeries>,
}

impl PerformanceAnalytics {
    pub fn new() -> Self {
        Self {
            query_latencies: Arc::new(RwLock::new(Vec::new())),
            throughput_counter: Arc::new(Counter::new("throughput")),
            active_queries: Arc::new(Gauge::new("active_queries")),
            error_counter: Arc::new(Counter::new("errors")),
            cache_hits: Arc::new(Counter::new("cache_hits")),
            cache_misses: Arc::new(Counter::new("cache_misses")),
            processing_time: Arc::new(Histogram::new_linear("processing_time", 0.0, 10.0, 100)),
            memory_usage: Arc::new(TimeSeries::new("memory_usage", 1000, 3600)),
            cpu_usage: Arc::new(TimeSeries::new("cpu_usage", 1000, 3600)),
        }
    }

    /// Record a query execution
    pub fn record_query(&self, latency_ms: f64, success: bool) {
        self.query_latencies.write().push(latency_ms);
        self.throughput_counter.inc();
        self.processing_time.observe(latency_ms);

        if !success {
            self.error_counter.inc();
        }
    }

    /// Record query start
    pub fn query_start(&self) {
        self.active_queries.inc();
    }

    /// Record query end
    pub fn query_end(&self) {
        self.active_queries.dec();
    }

    /// Record cache hit
    pub fn cache_hit(&self) {
        self.cache_hits.inc();
    }

    /// Record cache miss
    pub fn cache_miss(&self) {
        self.cache_misses.inc();
    }

    /// Record memory usage
    pub fn record_memory(&self, bytes: f64) {
        self.memory_usage.add(bytes);
    }

    /// Record CPU usage
    pub fn record_cpu(&self, percent: f64) {
        self.cpu_usage.add(percent);
    }

    /// Get current performance snapshot
    pub fn snapshot(&self) -> PerformanceSnapshot {
        let latencies = self.query_latencies.read().clone();

        let latency_stats = if !latencies.is_empty() {
            DescriptiveStats::from_data(&latencies).ok()
        } else {
            None
        };

        let cache_hit_rate = {
            let hits = self.cache_hits.value();
            let misses = self.cache_misses.value();
            let total = hits + misses;

            if total > 0 {
                (hits as f64 / total as f64) * 100.0
            } else {
                0.0
            }
        };

        PerformanceSnapshot {
            timestamp: Utc::now(),
            total_queries: self.throughput_counter.value(),
            active_queries: self.active_queries.value() as u64,
            error_count: self.error_counter.value(),
            latency_stats,
            cache_hit_rate,
            memory_usage_mb: self.memory_usage.latest().map(|p| p.value).unwrap_or(0.0) / 1_048_576.0,
            cpu_usage_pct: self.cpu_usage.latest().map(|p| p.value).unwrap_or(0.0),
        }
    }

    /// Get performance report
    pub fn report(&self) -> PerformanceReport {
        let snapshot = self.snapshot();
        let processing_time_snapshot = self.processing_time.snapshot();

        PerformanceReport {
            snapshot,
            processing_time_p50: processing_time_snapshot.percentiles.p50,
            processing_time_p95: processing_time_snapshot.percentiles.p95,
            processing_time_p99: processing_time_snapshot.percentiles.p99,
            memory_trend: self.memory_usage.trend(),
            cpu_trend: self.cpu_usage.trend(),
        }
    }

    /// Get SLA compliance metrics
    pub fn sla_metrics(&self, target_latency_ms: f64) -> SLAMetrics {
        let latencies = self.query_latencies.read();

        let total = latencies.len();
        let within_sla = latencies.iter().filter(|&&l| l <= target_latency_ms).count();

        let compliance_rate = if total > 0 {
            (within_sla as f64 / total as f64) * 100.0
        } else {
            100.0
        };

        SLAMetrics {
            target_latency_ms,
            total_queries: total,
            within_sla_count: within_sla,
            compliance_rate,
            breaches: total - within_sla,
        }
    }

    /// Reset all metrics
    pub fn reset(&self) {
        self.query_latencies.write().clear();
        // Counters and gauges can't be easily reset without mutable access
        // In a real implementation, you'd want to handle this better
    }
}

impl Default for PerformanceAnalytics {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSnapshot {
    pub timestamp: DateTime<Utc>,
    pub total_queries: u64,
    pub active_queries: u64,
    pub error_count: u64,
    pub latency_stats: Option<DescriptiveStats>,
    pub cache_hit_rate: f64,
    pub memory_usage_mb: f64,
    pub cpu_usage_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub snapshot: PerformanceSnapshot,
    pub processing_time_p50: Option<f64>,
    pub processing_time_p95: Option<f64>,
    pub processing_time_p99: Option<f64>,
    pub memory_trend: crate::metrics::timeseries::Trend,
    pub cpu_trend: crate::metrics::timeseries::Trend,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SLAMetrics {
    pub target_latency_ms: f64,
    pub total_queries: usize,
    pub within_sla_count: usize,
    pub compliance_rate: f64,
    pub breaches: usize,
}

/// Resource usage tracker
pub struct ResourceTracker {
    measurements: Arc<RwLock<HashMap<String, Vec<ResourceMeasurement>>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMeasurement {
    pub timestamp: DateTime<Utc>,
    pub cpu_percent: f64,
    pub memory_bytes: u64,
    pub disk_io_bytes: u64,
    pub network_bytes: u64,
}

impl ResourceTracker {
    pub fn new() -> Self {
        Self {
            measurements: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn record(&self, component: impl Into<String>, measurement: ResourceMeasurement) {
        let component = component.into();
        self.measurements
            .write()
            .entry(component)
            .or_insert_with(Vec::new)
            .push(measurement);
    }

    pub fn get_stats(&self, component: &str) -> Option<ResourceStats> {
        let measurements = self.measurements.read();
        let data = measurements.get(component)?;

        if data.is_empty() {
            return None;
        }

        let cpu_values: Vec<f64> = data.iter().map(|m| m.cpu_percent).collect();
        let memory_values: Vec<f64> = data.iter().map(|m| m.memory_bytes as f64).collect();

        Some(ResourceStats {
            component: component.to_string(),
            avg_cpu: Statistics::mean(&cpu_values),
            max_cpu: cpu_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
            avg_memory_bytes: Statistics::mean(&memory_values) as u64,
            max_memory_bytes: memory_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max) as u64,
            sample_count: data.len(),
        })
    }

    pub fn clear(&self, component: &str) {
        self.measurements.write().remove(component);
    }
}

impl Default for ResourceTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceStats {
    pub component: String,
    pub avg_cpu: f64,
    pub max_cpu: f64,
    pub avg_memory_bytes: u64,
    pub max_memory_bytes: u64,
    pub sample_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_analytics() {
        let perf = PerformanceAnalytics::new();

        perf.query_start();
        perf.record_query(10.5, true);
        perf.query_end();

        perf.cache_hit();
        perf.cache_miss();

        let snapshot = perf.snapshot();
        assert_eq!(snapshot.total_queries, 1);
        assert!(snapshot.cache_hit_rate > 0.0);
    }

    #[test]
    fn test_sla_metrics() {
        let perf = PerformanceAnalytics::new();

        perf.record_query(50.0, true);
        perf.record_query(150.0, true);
        perf.record_query(75.0, true);

        let sla = perf.sla_metrics(100.0);
        assert_eq!(sla.total_queries, 3);
        assert_eq!(sla.within_sla_count, 2);
    }

    #[test]
    fn test_resource_tracker() {
        let tracker = ResourceTracker::new();

        tracker.record(
            "analytics_engine",
            ResourceMeasurement {
                timestamp: Utc::now(),
                cpu_percent: 45.0,
                memory_bytes: 1024 * 1024 * 100,
                disk_io_bytes: 0,
                network_bytes: 0,
            },
        );

        let stats = tracker.get_stats("analytics_engine").unwrap();
        assert_eq!(stats.sample_count, 1);
        assert!(stats.avg_cpu > 0.0);
    }
}
