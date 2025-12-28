//! Metrics module for collecting and exporting metrics

pub mod counter;
pub mod gauge;
pub mod histogram;
pub mod registry;
pub mod export;

use crate::{MetricsConfig, Result};
use std::sync::Arc;
use parking_lot::RwLock;

pub use counter::Counter;
pub use gauge::Gauge;
pub use histogram::Histogram;
pub use registry::MetricsRegistry;
pub use export::PrometheusExporter;

/// Metrics system
pub struct MetricsSystem {
    config: MetricsConfig,
    registry: Arc<RwLock<MetricsRegistry>>,
    exporter: Option<Arc<RwLock<PrometheusExporter>>>,
}

impl MetricsSystem {
    /// Create a new metrics system
    pub fn new(config: &MetricsConfig) -> Result<Self> {
        let registry = Arc::new(RwLock::new(MetricsRegistry::new(&config.prefix)));

        let exporter = if config.prometheus {
            Some(Arc::new(RwLock::new(PrometheusExporter::new(
                &config.prometheus_addr,
                Arc::clone(&registry),
            )?)))
        } else {
            None
        };

        Ok(Self {
            config: config.clone(),
            registry,
            exporter,
        })
    }

    /// Get the metrics registry
    pub fn registry(&self) -> Arc<RwLock<MetricsRegistry>> {
        Arc::clone(&self.registry)
    }

    /// Create a new counter
    pub fn counter(&self, name: &str, description: &str) -> Counter {
        self.registry.write().counter(name, description)
    }

    /// Create a new gauge
    pub fn gauge(&self, name: &str, description: &str) -> Gauge {
        self.registry.write().gauge(name, description)
    }

    /// Create a new histogram
    pub fn histogram(&self, name: &str, description: &str) -> Histogram {
        self.registry.write().histogram(name, description)
    }

    /// Start the Prometheus exporter
    pub async fn start_exporter(&self) -> Result<()> {
        if let Some(exporter) = &self.exporter {
            exporter.write().start().await?;
        }
        Ok(())
    }

    /// Stop the Prometheus exporter
    pub async fn stop_exporter(&self) -> Result<()> {
        if let Some(exporter) = &self.exporter {
            exporter.write().stop().await?;
        }
        Ok(())
    }

    /// Get metrics snapshot
    pub fn snapshot(&self) -> String {
        self.registry.read().snapshot()
    }
}
