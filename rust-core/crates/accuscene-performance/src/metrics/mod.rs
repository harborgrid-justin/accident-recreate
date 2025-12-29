//! Metrics collection and reporting

pub mod counter;
pub mod histogram;
pub mod reporter;

pub use counter::{Counter, CounterVec};
pub use histogram::{Histogram, HistogramVec};
pub use reporter::{MetricsReporter, PrometheusReporter};

use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

/// Metrics registry
pub struct MetricsRegistry {
    counters: Arc<Mutex<HashMap<String, Counter>>>,
    histograms: Arc<Mutex<HashMap<String, Histogram>>>,
}

impl MetricsRegistry {
    /// Create a new metrics registry
    pub fn new() -> Self {
        Self {
            counters: Arc::new(Mutex::new(HashMap::new())),
            histograms: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register or get a counter
    pub fn counter(&self, name: impl Into<String>) -> Counter {
        let name = name.into();
        let mut counters = self.counters.lock();

        counters
            .entry(name.clone())
            .or_insert_with(|| Counter::new(name))
            .clone()
    }

    /// Register or get a histogram
    pub fn histogram(&self, name: impl Into<String>) -> Histogram {
        let name = name.into();
        let mut histograms = self.histograms.lock();

        histograms
            .entry(name.clone())
            .or_insert_with(|| Histogram::new(name))
            .clone()
    }

    /// Get all counters
    pub fn counters(&self) -> HashMap<String, Counter> {
        self.counters.lock().clone()
    }

    /// Get all histograms
    pub fn histograms(&self) -> HashMap<String, Histogram> {
        self.histograms.lock().clone()
    }

    /// Reset all metrics
    pub fn reset(&self) {
        for counter in self.counters.lock().values() {
            counter.reset();
        }
        for histogram in self.histograms.lock().values() {
            histogram.reset();
        }
    }

    /// Print all metrics
    pub fn print(&self) {
        println!("\nMetrics:");
        println!("{}", "=".repeat(60));

        println!("\nCounters:");
        for (name, counter) in self.counters.lock().iter() {
            println!("  {}: {}", name, counter.get());
        }

        println!("\nHistograms:");
        for (name, histogram) in self.histograms.lock().iter() {
            let stats = histogram.stats();
            println!("  {}:", name);
            println!("    count: {}", stats.count);
            println!("    sum:   {}", stats.sum);
            println!("    min:   {}", stats.min);
            println!("    max:   {}", stats.max);
            println!("    avg:   {:.2}", stats.avg);
        }
    }
}

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for MetricsRegistry {
    fn clone(&self) -> Self {
        Self {
            counters: self.counters.clone(),
            histograms: self.histograms.clone(),
        }
    }
}

/// Global metrics registry
static GLOBAL_REGISTRY: once_cell::sync::Lazy<MetricsRegistry> =
    once_cell::sync::Lazy::new(MetricsRegistry::new);

/// Get the global metrics registry
pub fn global_registry() -> &'static MetricsRegistry {
    &GLOBAL_REGISTRY
}

/// Register a global counter
pub fn counter(name: impl Into<String>) -> Counter {
    global_registry().counter(name)
}

/// Register a global histogram
pub fn histogram(name: impl Into<String>) -> Histogram {
    global_registry().histogram(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_registry() {
        let registry = MetricsRegistry::new();

        let counter = registry.counter("test_counter");
        counter.inc();
        counter.inc();

        assert_eq!(counter.get(), 2);
    }

    #[test]
    fn test_global_registry() {
        let counter = counter("global_test");
        counter.inc();

        assert!(counter.get() > 0);
    }

    #[test]
    fn test_registry_reset() {
        let registry = MetricsRegistry::new();

        let counter = registry.counter("reset_test");
        counter.add(10);

        registry.reset();
        assert_eq!(counter.get(), 0);
    }
}
