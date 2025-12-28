//! Gauge metrics - values that can go up or down

use super::{Metric, MetricMetadata};
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Gauge metric that can increase or decrease
#[derive(Debug)]
pub struct Gauge {
    metadata: MetricMetadata,
    value: Arc<RwLock<f64>>,
    min_value: Arc<RwLock<f64>>,
    max_value: Arc<RwLock<f64>>,
    last_updated: Arc<RwLock<DateTime<Utc>>>,
}

impl Gauge {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            metadata: MetricMetadata::new(name),
            value: Arc::new(RwLock::new(0.0)),
            min_value: Arc::new(RwLock::new(f64::INFINITY)),
            max_value: Arc::new(RwLock::new(f64::NEG_INFINITY)),
            last_updated: Arc::new(RwLock::new(Utc::now())),
        }
    }

    pub fn with_metadata(metadata: MetricMetadata) -> Self {
        Self {
            metadata,
            value: Arc::new(RwLock::new(0.0)),
            min_value: Arc::new(RwLock::new(f64::INFINITY)),
            max_value: Arc::new(RwLock::new(f64::NEG_INFINITY)),
            last_updated: Arc::new(RwLock::new(Utc::now())),
        }
    }

    /// Set the gauge to a specific value
    pub fn set(&self, value: f64) {
        *self.value.write() = value;
        self.update_bounds(value);
        *self.last_updated.write() = Utc::now();
    }

    /// Increment the gauge
    pub fn inc(&self) {
        self.add(1.0);
    }

    /// Decrement the gauge
    pub fn dec(&self) {
        self.sub(1.0);
    }

    /// Add to the gauge
    pub fn add(&self, delta: f64) {
        let mut value = self.value.write();
        *value += delta;
        let new_value = *value;
        drop(value);

        self.update_bounds(new_value);
        *self.last_updated.write() = Utc::now();
    }

    /// Subtract from the gauge
    pub fn sub(&self, delta: f64) {
        self.add(-delta);
    }

    /// Get the current value
    pub fn value(&self) -> f64 {
        *self.value.read()
    }

    /// Get the minimum value observed
    pub fn min(&self) -> f64 {
        *self.min_value.read()
    }

    /// Get the maximum value observed
    pub fn max(&self) -> f64 {
        *self.max_value.read()
    }

    /// Get metadata
    pub fn metadata(&self) -> &MetricMetadata {
        &self.metadata
    }

    fn update_bounds(&self, value: f64) {
        let mut min = self.min_value.write();
        if value < *min {
            *min = value;
        }
        drop(min);

        let mut max = self.max_value.write();
        if value > *max {
            *max = value;
        }
    }
}

impl Metric for Gauge {
    fn name(&self) -> &str {
        &self.metadata.name
    }

    fn tags(&self) -> &HashMap<String, String> {
        &self.metadata.tags
    }

    fn reset(&mut self) {
        *self.value.write() = 0.0;
        *self.min_value.write() = f64::INFINITY;
        *self.max_value.write() = f64::NEG_INFINITY;
        *self.last_updated.write() = Utc::now();
    }

    fn last_updated(&self) -> DateTime<Utc> {
        *self.last_updated.read()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaugeSnapshot {
    pub name: String,
    pub value: f64,
    pub min: f64,
    pub max: f64,
    pub timestamp: DateTime<Utc>,
    pub tags: HashMap<String, String>,
}

impl From<&Gauge> for GaugeSnapshot {
    fn from(gauge: &Gauge) -> Self {
        Self {
            name: gauge.name().to_string(),
            value: gauge.value(),
            min: gauge.min(),
            max: gauge.max(),
            timestamp: gauge.last_updated(),
            tags: gauge.tags().clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gauge() {
        let gauge = Gauge::new("test");
        assert_eq!(gauge.value(), 0.0);

        gauge.set(10.0);
        assert_eq!(gauge.value(), 10.0);

        gauge.inc();
        assert_eq!(gauge.value(), 11.0);

        gauge.dec();
        assert_eq!(gauge.value(), 10.0);

        gauge.add(5.0);
        assert_eq!(gauge.value(), 15.0);

        gauge.sub(3.0);
        assert_eq!(gauge.value(), 12.0);
    }

    #[test]
    fn test_gauge_bounds() {
        let gauge = Gauge::new("test");

        gauge.set(10.0);
        gauge.set(20.0);
        gauge.set(5.0);

        assert_eq!(gauge.min(), 5.0);
        assert_eq!(gauge.max(), 20.0);
    }
}
