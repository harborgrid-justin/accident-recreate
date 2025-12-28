//! Prometheus metrics exporter

use super::MetricsRegistry;
use crate::{Result, TelemetryError};
use parking_lot::RwLock;
use std::sync::Arc;
use tokio::sync::Notify;

/// Prometheus exporter
pub struct PrometheusExporter {
    addr: String,
    _ registry: Arc<RwLock<MetricsRegistry>>,
    shutdown: Arc<Notify>,
    running: Arc<RwLock<bool>>,
}

impl PrometheusExporter {
    /// Create a new Prometheus exporter
    pub fn new(addr: impl Into<String>, _ registry: Arc<RwLock<MetricsRegistry>>) -> Result<Self> {
        Ok(Self {
            addr: addr.into(),
            registry,
            shutdown: Arc::new(Notify::new()),
            running: Arc::new(RwLock::new(false)),
        })
    }

    /// Start the Prometheus exporter
    pub async fn start(&mut self) -> Result<()> {
        if *self.running.read() {
            return Ok(());
        }

        *self.running.write() = true;

        let addr = self.addr.clone();
        let registry = Arc::clone(&self.registry);
        let shutdown = Arc::clone(&self.shutdown);
        let running = Arc::clone(&self.running);

        tokio::spawn(async move {
            tracing::info!("Starting Prometheus exporter on {}", addr);

            // Note: In a real implementation, you would set up an HTTP server here
            // using something like axum or warp to serve metrics at /metrics endpoint
            // For now, we'll just simulate the exporter running

            loop {
                tokio::select! {
                    _ = shutdown.notified() => {
                        tracing::info!("Prometheus exporter shutting down");
                        *running.write() = false;
                        break;
                    }
                    _ = tokio::time::sleep(tokio::time::Duration::from_secs(1)) => {
                        // In real implementation, this would handle HTTP requests
                        // and serve the metrics from registry.read().prometheus_format()
                    }
                }
            }
        });

        Ok(())
    }

    /// Stop the Prometheus exporter
    pub async fn stop(&mut self) -> Result<()> {
        if !*self.running.read() {
            return Ok(());
        }

        self.shutdown.notify_waiters();

        // Wait for the exporter to stop
        while *self.running.read() {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        tracing::info!("Prometheus exporter stopped");
        Ok(())
    }

    /// Check if the exporter is running
    pub fn is_running(&self) -> bool {
        *self.running.read()
    }

    /// Get the exporter address
    pub fn addr(&self) -> &str {
        &self.addr
    }

    /// Get metrics in Prometheus format
    pub fn metrics(&self) -> String {
        self.registry.read().prometheus_format()
    }

    /// Handle a metrics request (for manual testing)
    pub fn handle_metrics_request(&self) -> String {
        let metrics = self.metrics();

        // Add metadata
        let mut response = String::new();
        response.push_str(&format!(
            "# AccuScene Telemetry Metrics\n# Generated: {}\n\n",
            chrono::Utc::now().to_rfc3339()
        ));
        response.push_str(&metrics);

        response
    }
}

impl Drop for PrometheusExporter {
    fn drop(&mut self) {
        self.shutdown.notify_waiters();
    }
}

/// Helper function to create an HTTP endpoint for metrics (example)
pub fn metrics_endpoint_handler(_ registry: Arc<RwLock<MetricsRegistry>>) -> String {
    registry.read().prometheus_format()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_exporter_lifecycle() {
        let registry = Arc::new(RwLock::new(MetricsRegistry::new("test")));
        let mut exporter = PrometheusExporter::new("0.0.0.0:9090", registry).unwrap();

        assert!(!exporter.is_running());

        exporter.start().await.unwrap();
        assert!(exporter.is_running());

        exporter.stop().await.unwrap();
        assert!(!exporter.is_running());
    }

    #[test]
    fn test_metrics_format() {
        let registry = Arc::new(RwLock::new(MetricsRegistry::new("test")));
        {
            let mut reg = registry.write();
            let counter = reg.counter("requests", "Total requests");
            counter.increment();
        }

        let exporter = PrometheusExporter::new("0.0.0.0:9090", registry).unwrap();
        let metrics = exporter.metrics();

        assert!(metrics.contains("test_requests"));
        assert!(metrics.contains("counter"));
    }
}
