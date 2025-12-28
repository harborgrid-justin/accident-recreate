//! Counter metrics

use parking_lot::RwLock;
use std::sync::Arc;

/// A counter metric that can only increase
#[derive(Clone)]
pub struct Counter {
    name: String,
    description: String,
    value: Arc<RwLock<f64>>,
}

impl Counter {
    /// Create a new counter
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            value: Arc::new(RwLock::new(0.0)),
        }
    }

    /// Increment the counter by 1
    pub fn increment(&self) {
        self.increment_by(1.0);
    }

    /// Increment the counter by a specific value
    pub fn increment_by(&self, value: f64) {
        if value >= 0.0 {
            *self.value.write() += value;
        }
    }

    /// Get the current value
    pub fn value(&self) -> f64 {
        *self.value.read()
    }

    /// Reset the counter to zero
    pub fn reset(&self) {
        *self.value.write() = 0.0;
    }

    /// Get the counter name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the counter description
    pub fn description(&self) -> &str {
        &self.description
    }
}

impl std::fmt::Debug for Counter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Counter")
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
    fn test_counter_increment() {
        let counter = Counter::new("test_counter", "A test counter");
        assert_eq!(counter.value(), 0.0);

        counter.increment();
        assert_eq!(counter.value(), 1.0);

        counter.increment_by(5.0);
        assert_eq!(counter.value(), 6.0);
    }

    #[test]
    fn test_counter_reset() {
        let counter = Counter::new("test_counter", "A test counter");
        counter.increment_by(10.0);
        assert_eq!(counter.value(), 10.0);

        counter.reset();
        assert_eq!(counter.value(), 0.0);
    }

    #[test]
    fn test_counter_negative_increment() {
        let counter = Counter::new("test_counter", "A test counter");
        counter.increment_by(-5.0);
        assert_eq!(counter.value(), 0.0); // Should not decrement
    }
}
