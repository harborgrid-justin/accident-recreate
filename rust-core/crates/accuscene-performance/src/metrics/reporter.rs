//! Metrics reporting

use crate::metrics::{Counter, Histogram, MetricsRegistry};
use std::time::Duration;

/// Metrics reporter trait
pub trait MetricsReporter {
    /// Report metrics
    fn report(&self, registry: &MetricsRegistry);

    /// Start reporting at regular intervals
    fn start_reporting(&self, registry: MetricsRegistry, interval: Duration);
}

/// Prometheus-compatible metrics reporter
pub struct PrometheusReporter {
    prefix: String,
}

impl PrometheusReporter {
    /// Create a new Prometheus reporter
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
        }
    }

    /// Format counter for Prometheus
    fn format_counter(&self, name: &str, counter: &Counter) -> String {
        let metric_name = format!("{}_{}", self.prefix, name);
        format!(
            "# TYPE {} counter\n{} {}",
            metric_name,
            metric_name,
            counter.get()
        )
    }

    /// Format histogram for Prometheus
    fn format_histogram(&self, name: &str, histogram: &Histogram) -> String {
        let metric_name = format!("{}_{}", self.prefix, name);
        let stats = histogram.stats();

        format!(
            "# TYPE {} histogram\n\
             {}_count {}\n\
             {}_sum {}\n\
             {}_bucket{{le=\"0.5\"}} {}\n\
             {}_bucket{{le=\"0.95\"}} {}\n\
             {}_bucket{{le=\"0.99\"}} {}\n\
             {}_bucket{{le=\"+Inf\"}} {}",
            metric_name,
            metric_name,
            stats.count,
            metric_name,
            stats.sum,
            metric_name,
            stats.p50,
            metric_name,
            stats.p95,
            metric_name,
            stats.p99,
            metric_name,
            stats.count
        )
    }

    /// Get metrics in Prometheus format
    pub fn to_prometheus(&self, registry: &MetricsRegistry) -> String {
        let mut output = String::new();

        // Add counters
        for (name, counter) in registry.counters() {
            output.push_str(&self.format_counter(&name, &counter));
            output.push('\n');
        }

        // Add histograms
        for (name, histogram) in registry.histograms() {
            output.push_str(&self.format_histogram(&name, &histogram));
            output.push('\n');
        }

        output
    }
}

impl Default for PrometheusReporter {
    fn default() -> Self {
        Self::new("accuscene")
    }
}

impl MetricsReporter for PrometheusReporter {
    fn report(&self, registry: &MetricsRegistry) {
        let metrics = self.to_prometheus(registry);
        println!("{}", metrics);
    }

    fn start_reporting(&self, registry: MetricsRegistry, interval: Duration) {
        let reporter = self.clone();

        std::thread::spawn(move || loop {
            std::thread::sleep(interval);
            reporter.report(&registry);
        });
    }
}

impl Clone for PrometheusReporter {
    fn clone(&self) -> Self {
        Self {
            prefix: self.prefix.clone(),
        }
    }
}

/// Console metrics reporter
pub struct ConsoleReporter;

impl ConsoleReporter {
    /// Create a new console reporter
    pub fn new() -> Self {
        Self
    }
}

impl Default for ConsoleReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsReporter for ConsoleReporter {
    fn report(&self, registry: &MetricsRegistry) {
        registry.print();
    }

    fn start_reporting(&self, registry: MetricsRegistry, interval: Duration) {
        std::thread::spawn(move || loop {
            std::thread::sleep(interval);
            registry.print();
        });
    }
}

/// JSON metrics reporter
pub struct JsonReporter;

impl JsonReporter {
    /// Create a new JSON reporter
    pub fn new() -> Self {
        Self
    }

    /// Convert metrics to JSON
    pub fn to_json(&self, registry: &MetricsRegistry) -> serde_json::Value {
        let mut counters = serde_json::Map::new();
        for (name, counter) in registry.counters() {
            counters.insert(name, serde_json::json!(counter.get()));
        }

        let mut histograms = serde_json::Map::new();
        for (name, histogram) in registry.histograms() {
            let stats = histogram.stats();
            histograms.insert(
                name,
                serde_json::json!({
                    "count": stats.count,
                    "sum": stats.sum,
                    "min": stats.min,
                    "max": stats.max,
                    "avg": stats.avg,
                    "p50": stats.p50,
                    "p95": stats.p95,
                    "p99": stats.p99,
                }),
            );
        }

        serde_json::json!({
            "counters": counters,
            "histograms": histograms,
        })
    }
}

impl Default for JsonReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsReporter for JsonReporter {
    fn report(&self, registry: &MetricsRegistry) {
        let json = self.to_json(registry);
        println!("{}", serde_json::to_string_pretty(&json).unwrap());
    }

    fn start_reporting(&self, registry: MetricsRegistry, interval: Duration) {
        let reporter = JsonReporter::new();

        std::thread::spawn(move || loop {
            std::thread::sleep(interval);
            reporter.report(&registry);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prometheus_reporter() {
        let registry = MetricsRegistry::new();
        let counter = registry.counter("test_counter");
        counter.add(42);

        let reporter = PrometheusReporter::new("test");
        let output = reporter.to_prometheus(&registry);

        assert!(output.contains("test_test_counter 42"));
    }

    #[test]
    fn test_console_reporter() {
        let registry = MetricsRegistry::new();
        let counter = registry.counter("console_test");
        counter.inc();

        let reporter = ConsoleReporter::new();
        reporter.report(&registry);
    }

    #[test]
    fn test_json_reporter() {
        let registry = MetricsRegistry::new();
        let counter = registry.counter("json_test");
        counter.add(10);

        let reporter = JsonReporter::new();
        let json = reporter.to_json(&registry);

        assert_eq!(json["counters"]["json_test"], 10);
    }

    #[test]
    fn test_histogram_reporting() {
        let registry = MetricsRegistry::new();
        let histogram = registry.histogram("request_duration");

        histogram.observe(1.0);
        histogram.observe(2.0);
        histogram.observe(3.0);

        let reporter = PrometheusReporter::new("test");
        let output = reporter.to_prometheus(&registry);

        assert!(output.contains("request_duration_count 3"));
        assert!(output.contains("request_duration_sum 6"));
    }
}
