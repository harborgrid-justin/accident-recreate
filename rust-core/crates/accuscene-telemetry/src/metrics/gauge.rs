//! Gauge metrics

use parking_lot::RwLock;
use std::sync::Arc;

/// A gauge metric that can increase or decrease
#[derive(Clone)]
pub struct Gauge {
    name: String,
    description: String,
    value: Arc<RwLock<f64>>,
}

impl Gauge {
    /// Create a new gauge
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            value: Arc::new(RwLock::new(0.0)),
        }
    }

    /// Set the gauge value
    pub fn set(&self, value: f64) {
        *self.value.write() = value;
    }

    /// Increment the gauge by 1
    pub fn increment(&self) {
        self.increment_by(1.0);
    }

    /// Increment the gauge by a specific value
    pub fn increment_by(&self, value: f64) {
        *self.value.write() += value;
    }

    /// Decrement the gauge by 1
    pub fn decrement(&self) {
        self.decrement_by(1.0);
    }

    /// Decrement the gauge by a specific value
    pub fn decrement_by(&self, value: f64) {
        *self.value.write() -= value;
    }

    /// Get the current value
    pub fn value(&self) -> f64 {
        *self.value.read()
    }

    /// Reset the gauge to zero
    pub fn reset(&self) {
        *self.value.write() = 0.0;
    }

    /// Get the gauge name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the gauge description
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Set to the maximum of current value and the given value
    pub fn set_to_max(&self, value: f64) {
        let mut current = self.value.write();
        if value > *current {
            *current = value;
        }
    }

    /// Set to the minimum of current value and the given value
    pub fn set_to_min(&self, value: f64) {
        let mut current = self.value.write();
        if value < *current {
            *current = value;
        }
    }
}

impl std::fmt::Debug for Gauge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Gauge")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("value", &self.value())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gauge_set() {
        let gauge = Gauge::new("test_gauge", "A test gauge");
        gauge.set(42.0);
        assert_eq!(gauge.value(), 42.0);
    }

    #[test]
    fn test_gauge_increment_decrement() {
        let gauge = Gauge::new("test_gauge", "A test gauge");
        gauge.increment();
        assert_eq!(gauge.value(), 1.0);

        gauge.increment_by(5.0);
        assert_eq!(gauge.value(), 6.0);

        gauge.decrement();
        assert_eq!(gauge.value(), 5.0);

        gauge.decrement_by(3.0);
        assert_eq!(gauge.value(), 2.0);
    }

    #[test]
    fn test_gauge_max_min() {
        let gauge = Gauge::new("test_gauge", "A test gauge");
        gauge.set(10.0);

        gauge.set_to_max(15.0);
        assert_eq!(gauge.value(), 15.0);

        gauge.set_to_max(5.0);
        assert_eq!(gauge.value(), 15.0);

        gauge.set_to_min(12.0);
        assert_eq!(gauge.value(), 12.0);

        gauge.set_to_min(20.0);
        assert_eq!(gauge.value(), 12.0);
    }
}
