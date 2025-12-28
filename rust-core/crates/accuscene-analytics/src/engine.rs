//! Analytics engine core - orchestrates all analytics operations

use crate::config::AnalyticsConfig;
use crate::error::{AnalyticsError, Result};
use crate::metrics::{Counter, Gauge, Histogram, Metric, MetricMetadata, TimeSeries};
use crate::storage::AnalyticsStorage;
use dashmap::DashMap;
use parking_lot::RwLock;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

/// Main analytics engine coordinating all components
pub struct AnalyticsEngine {
    config: Arc<RwLock<AnalyticsConfig>>,
    storage: Arc<AnalyticsStorage>,
    metrics_registry: Arc<MetricsRegistry>,
    pipeline: Arc<AnalyticsPipeline>,
    running: Arc<RwLock<bool>>,
}

impl AnalyticsEngine {
    /// Create a new analytics engine
    pub fn new(config: AnalyticsConfig) -> Self {
        let storage = Arc::new(AnalyticsStorage::new(config.retention_period.as_secs() as i64));

        Self {
            config: Arc::new(RwLock::new(config)),
            storage,
            metrics_registry: Arc::new(MetricsRegistry::new()),
            pipeline: Arc::new(AnalyticsPipeline::new()),
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start the analytics engine
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write();
        if *running {
            return Err(AnalyticsError::Configuration(
                "Engine already running".to_string(),
            ));
        }

        info!("Starting AccuScene Analytics Engine v0.2.0");

        *running = true;
        drop(running);

        // Initialize components
        self.initialize_metrics()?;

        info!("Analytics engine started successfully");
        Ok(())
    }

    /// Stop the analytics engine
    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.write();
        if !*running {
            return Ok(());
        }

        info!("Stopping analytics engine");

        *running = false;

        // Flush any pending data
        self.flush().await?;

        info!("Analytics engine stopped");
        Ok(())
    }

    /// Check if the engine is running
    pub fn is_running(&self) -> bool {
        *self.running.read()
    }

    /// Get the metrics registry
    pub fn metrics(&self) -> Arc<MetricsRegistry> {
        Arc::clone(&self.metrics_registry)
    }

    /// Get the storage backend
    pub fn storage(&self) -> Arc<AnalyticsStorage> {
        Arc::clone(&self.storage)
    }

    /// Get the analytics pipeline
    pub fn pipeline(&self) -> Arc<AnalyticsPipeline> {
        Arc::clone(&self.pipeline)
    }

    /// Update configuration
    pub fn update_config(&self, config: AnalyticsConfig) -> Result<()> {
        *self.config.write() = config;
        info!("Configuration updated");
        Ok(())
    }

    /// Get current configuration
    pub fn config(&self) -> AnalyticsConfig {
        self.config.read().clone()
    }

    /// Flush all pending data
    pub async fn flush(&self) -> Result<()> {
        debug!("Flushing analytics data");

        // Export metrics to storage
        let metrics = self.metrics_registry.snapshot();
        self.storage.put_json("metrics/snapshot", &metrics)?;

        Ok(())
    }

    fn initialize_metrics(&self) -> Result<()> {
        // Register default system metrics
        self.metrics_registry.register_counter("system.queries.total", MetricMetadata::new("system.queries.total"))?;
        self.metrics_registry.register_gauge("system.active_connections", MetricMetadata::new("system.active_connections"))?;
        self.metrics_registry.register_histogram("system.query_latency", MetricMetadata::new("system.query_latency"))?;

        Ok(())
    }
}

/// Metrics registry for managing all metrics
pub struct MetricsRegistry {
    counters: Arc<DashMap<String, Arc<Counter>>>,
    gauges: Arc<DashMap<String, Arc<Gauge>>>,
    histograms: Arc<DashMap<String, Arc<Histogram>>>,
    timeseries: Arc<DashMap<String, Arc<TimeSeries>>>,
}

impl MetricsRegistry {
    pub fn new() -> Self {
        Self {
            counters: Arc::new(DashMap::new()),
            gauges: Arc::new(DashMap::new()),
            histograms: Arc::new(DashMap::new()),
            timeseries: Arc::new(DashMap::new()),
        }
    }

    /// Register a counter metric
    pub fn register_counter(&self, name: impl Into<String>, metadata: MetricMetadata) -> Result<()> {
        let name = name.into();
        if self.counters.contains_key(&name) {
            return Err(AnalyticsError::Metric(format!(
                "Counter '{}' already registered",
                name
            )));
        }

        let counter = Arc::new(Counter::with_metadata(metadata));
        self.counters.insert(name, counter);

        Ok(())
    }

    /// Register a gauge metric
    pub fn register_gauge(&self, name: impl Into<String>, metadata: MetricMetadata) -> Result<()> {
        let name = name.into();
        if self.gauges.contains_key(&name) {
            return Err(AnalyticsError::Metric(format!(
                "Gauge '{}' already registered",
                name
            )));
        }

        let gauge = Arc::new(Gauge::with_metadata(metadata));
        self.gauges.insert(name, gauge);

        Ok(())
    }

    /// Register a histogram metric
    pub fn register_histogram(&self, name: impl Into<String>, metadata: MetricMetadata) -> Result<()> {
        let name = name.into();
        if self.histograms.contains_key(&name) {
            return Err(AnalyticsError::Metric(format!(
                "Histogram '{}' already registered",
                name
            )));
        }

        let histogram = Arc::new(Histogram::new_linear(name.clone(), 0.0, 10.0, 100));
        self.histograms.insert(name, histogram);

        Ok(())
    }

    /// Get a counter by name
    pub fn counter(&self, name: &str) -> Option<Arc<Counter>> {
        self.counters.get(name).map(|c| Arc::clone(c.value()))
    }

    /// Get a gauge by name
    pub fn gauge(&self, name: &str) -> Option<Arc<Gauge>> {
        self.gauges.get(name).map(|g| Arc::clone(g.value()))
    }

    /// Get a histogram by name
    pub fn histogram(&self, name: &str) -> Option<Arc<Histogram>> {
        self.histograms.get(name).map(|h| Arc::clone(h.value()))
    }

    /// Get all metric names
    pub fn metric_names(&self) -> Vec<String> {
        let mut names = Vec::new();

        names.extend(self.counters.iter().map(|e| e.key().clone()));
        names.extend(self.gauges.iter().map(|e| e.key().clone()));
        names.extend(self.histograms.iter().map(|e| e.key().clone()));
        names.extend(self.timeseries.iter().map(|e| e.key().clone()));

        names.sort();
        names
    }

    /// Get a snapshot of all metrics
    pub fn snapshot(&self) -> MetricsSnapshot {
        use crate::metrics::counter::CounterSnapshot;
        use crate::metrics::gauge::GaugeSnapshot;

        let counters: Vec<CounterSnapshot> = self
            .counters
            .iter()
            .map(|e| CounterSnapshot::from(e.value().as_ref()))
            .collect();

        let gauges: Vec<GaugeSnapshot> = self
            .gauges
            .iter()
            .map(|e| GaugeSnapshot::from(e.value().as_ref()))
            .collect();

        let histograms = self
            .histograms
            .iter()
            .map(|e| e.value().snapshot())
            .collect();

        MetricsSnapshot {
            timestamp: chrono::Utc::now(),
            counters,
            gauges,
            histograms,
        }
    }

    /// Clear all metrics
    pub fn clear(&self) {
        self.counters.clear();
        self.gauges.clear();
        self.histograms.clear();
        self.timeseries.clear();
    }
}

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MetricsSnapshot {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub counters: Vec<crate::metrics::counter::CounterSnapshot>,
    pub gauges: Vec<crate::metrics::gauge::GaugeSnapshot>,
    pub histograms: Vec<crate::metrics::histogram::HistogramSnapshot>,
}

/// Analytics pipeline for processing events
pub struct AnalyticsPipeline {
    processors: Arc<RwLock<Vec<Box<dyn EventProcessor>>>>,
    event_tx: Arc<RwLock<Option<mpsc::UnboundedSender<AnalyticsEvent>>>>,
}

impl AnalyticsPipeline {
    pub fn new() -> Self {
        Self {
            processors: Arc::new(RwLock::new(Vec::new())),
            event_tx: Arc::new(RwLock::new(None)),
        }
    }

    /// Add an event processor to the pipeline
    pub fn add_processor(&self, processor: Box<dyn EventProcessor>) {
        self.processors.write().push(processor);
    }

    /// Submit an event to the pipeline
    pub fn submit(&self, event: AnalyticsEvent) -> Result<()> {
        if let Some(tx) = self.event_tx.read().as_ref() {
            tx.send(event).map_err(|e| {
                AnalyticsError::Unknown(format!("Failed to submit event: {}", e))
            })?;
        } else {
            warn!("Pipeline not started, dropping event");
        }

        Ok(())
    }
}

impl Default for AnalyticsPipeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for processing analytics events
pub trait EventProcessor: Send + Sync {
    fn process(&self, event: &AnalyticsEvent) -> Result<()>;
    fn name(&self) -> &str;
}

/// Generic analytics event
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AnalyticsEvent {
    pub id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event_type: String,
    pub data: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_engine_lifecycle() {
        let config = AnalyticsConfig::default();
        let engine = AnalyticsEngine::new(config);

        assert!(!engine.is_running());

        engine.start().await.unwrap();
        assert!(engine.is_running());

        engine.stop().await.unwrap();
        assert!(!engine.is_running());
    }

    #[test]
    fn test_metrics_registry() {
        let registry = MetricsRegistry::new();

        registry
            .register_counter("test.counter", MetricMetadata::new("test.counter"))
            .unwrap();

        let counter = registry.counter("test.counter").unwrap();
        counter.inc();

        assert_eq!(counter.value(), 1);
    }
}
