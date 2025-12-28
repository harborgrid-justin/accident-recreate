//! Metrics registry

use super::{Counter, Gauge, Histogram};
use std::collections::HashMap;

/// Registry for all metrics
pub struct MetricsRegistry {
    prefix: String,
    counters: HashMap<String, Counter>,
    gauges: HashMap<String, Gauge>,
    histograms: HashMap<String, Histogram>,
}

impl MetricsRegistry {
    /// Create a new metrics registry
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
            counters: HashMap::new(),
            gauges: HashMap::new(),
            histograms: HashMap::new(),
        }
    }

    /// Register or get a counter
    pub fn counter(&mut self, name: &str, description: &str) -> Counter {
        let full_name = self.full_name(name);

        self.counters
            .entry(full_name.clone())
            .or_insert_with(|| Counter::new(&full_name, description))
            .clone()
    }

    /// Register or get a gauge
    pub fn gauge(&mut self, name: &str, description: &str) -> Gauge {
        let full_name = self.full_name(name);

        self.gauges
            .entry(full_name.clone())
            .or_insert_with(|| Gauge::new(&full_name, description))
            .clone()
    }

    /// Register or get a histogram
    pub fn histogram(&mut self, name: &str, description: &str) -> Histogram {
        let full_name = self.full_name(name);

        self.histograms
            .entry(full_name.clone())
            .or_insert_with(|| Histogram::new(&full_name, description))
            .clone()
    }

    /// Get a counter by name
    pub fn get_counter(&self, name: &str) -> Option<&Counter> {
        let full_name = self.full_name(name);
        self.counters.get(&full_name)
    }

    /// Get a gauge by name
    pub fn get_gauge(&self, name: &str) -> Option<&Gauge> {
        let full_name = self.full_name(name);
        self.gauges.get(&full_name)
    }

    /// Get a histogram by name
    pub fn get_histogram(&self, name: &str) -> Option<&Histogram> {
        let full_name = self.full_name(name);
        self.histograms.get(&full_name)
    }

    /// Get all counters
    pub fn counters(&self) -> &HashMap<String, Counter> {
        &self.counters
    }

    /// Get all gauges
    pub fn gauges(&self) -> &HashMap<String, Gauge> {
        &self.gauges
    }

    /// Get all histograms
    pub fn histograms(&self) -> &HashMap<String, Histogram> {
        &self.histograms
    }

    /// Clear all metrics
    pub fn clear(&mut self) {
        self.counters.clear();
        self.gauges.clear();
        self.histograms.clear();
    }

    /// Get metrics count
    pub fn count(&self) -> usize {
        self.counters.len() + self.gauges.len() + self.histograms.len()
    }

    /// Get a snapshot of all metrics
    pub fn snapshot(&self) -> String {
        let mut output = String::new();

        output.push_str("# Counters\n");
        for (name, counter) in &self.counters {
            output.push_str(&format!("{}: {}\n", name, counter.value()));
        }

        output.push_str("\n# Gauges\n");
        for (name, gauge) in &self.gauges {
            output.push_str(&format!("{}: {}\n", name, gauge.value()));
        }

        output.push_str("\n# Histograms\n");
        for (name, histogram) in &self.histograms {
            output.push_str(&format!(
                "{}: count={}, sum={}, mean={}, min={}, max={}\n",
                name,
                histogram.count(),
                histogram.sum(),
                histogram.mean(),
                histogram.min(),
                histogram.max()
            ));
        }

        output
    }

    /// Create Prometheus format output
    pub fn prometheus_format(&self) -> String {
        let mut output = String::new();

        // Export counters
        for (name, counter) in &self.counters {
            let prom_name = self.to_prometheus_name(name);
            output.push_str(&format!("# HELP {} {}\n", prom_name, counter.description()));
            output.push_str(&format!("# TYPE {} counter\n", prom_name));
            output.push_str(&format!("{} {}\n", prom_name, counter.value()));
        }

        // Export gauges
        for (name, gauge) in &self.gauges {
            let prom_name = self.to_prometheus_name(name);
            output.push_str(&format!("# HELP {} {}\n", prom_name, gauge.description()));
            output.push_str(&format!("# TYPE {} gauge\n", prom_name));
            output.push_str(&format!("{} {}\n", prom_name, gauge.value()));
        }

        // Export histograms
        for (name, histogram) in &self.histograms {
            let prom_name = self.to_prometheus_name(name);
            output.push_str(&format!("# HELP {} {}\n", prom_name, histogram.description()));
            output.push_str(&format!("# TYPE {} histogram\n", prom_name));

            for (upper_bound, count) in histogram.buckets() {
                let le = if upper_bound.is_infinite() {
                    "+Inf".to_string()
                } else {
                    upper_bound.to_string()
                };
                output.push_str(&format!("{}_bucket{{le=\"{}\"}} {}\n", prom_name, le, count));
            }

            output.push_str(&format!("{}_sum {}\n", prom_name, histogram.sum()));
            output.push_str(&format!("{}_count {}\n", prom_name, histogram.count()));
        }

        output
    }

    /// Generate full metric name with prefix
    fn full_name(&self, name: &str) -> String {
        if self.prefix.is_empty() {
            name.to_string()
        } else {
            format!("{}_{}", self.prefix, name)
        }
    }

    /// Convert metric name to Prometheus format
    fn to_prometheus_name(&self, name: &str) -> String {
        name.replace('.', "_").replace('-', "_")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_counter() {
        let mut registry = MetricsRegistry::new("test");
        let counter = registry.counter("requests", "Total requests");
        counter.increment();

        assert_eq!(registry.counters().len(), 1);
        assert_eq!(counter.value(), 1.0);
    }

    #[test]
    fn test_registry_gauge() {
        let mut registry = MetricsRegistry::new("test");
        let gauge = registry.gauge("temperature", "Current temperature");
        gauge.set(25.0);

        assert_eq!(registry.gauges().len(), 1);
        assert_eq!(gauge.value(), 25.0);
    }

    #[test]
    fn test_registry_histogram() {
        let mut registry = MetricsRegistry::new("test");
        let histogram = registry.histogram("latency", "Request latency");
        histogram.observe(0.5);

        assert_eq!(registry.histograms().len(), 1);
        assert_eq!(histogram.count(), 1);
    }

    #[test]
    fn test_prometheus_format() {
        let mut registry = MetricsRegistry::new("test");
        let counter = registry.counter("requests", "Total requests");
        counter.increment();

        let output = registry.prometheus_format();
        assert!(output.contains("# HELP"));
        assert!(output.contains("# TYPE"));
        assert!(output.contains("counter"));
    }
}
