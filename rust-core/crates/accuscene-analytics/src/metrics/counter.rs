//! Counter metrics - monotonically increasing values

use super::{Metric, MetricMetadata};
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Atomic counter that only increases
#[derive(Debug)]
pub struct Counter {
    metadata: MetricMetadata,
    value: AtomicU64,
    last_updated: Arc<RwLock<DateTime<Utc>>>,
}

impl Counter {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            metadata: MetricMetadata::new(name),
            value: AtomicU64::new(0),
            last_updated: Arc::new(RwLock::new(Utc::now())),
        }
    }

    pub fn with_metadata(metadata: MetricMetadata) -> Self {
        Self {
            metadata,
            value: AtomicU64::new(0),
            last_updated: Arc::new(RwLock::new(Utc::now())),
        }
    }

    /// Increment the counter by 1
    pub fn inc(&self) {
        self.add(1);
    }

    /// Add a value to the counter
    pub fn add(&self, delta: u64) {
        self.value.fetch_add(delta, Ordering::Relaxed);
        *self.last_updated.write() = Utc::now();
    }

    /// Get the current value
    pub fn value(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }

    /// Get metadata
    pub fn metadata(&self) -> &MetricMetadata {
        &self.metadata
    }
}

impl Metric for Counter {
    fn name(&self) -> &str {
        &self.metadata.name
    }

    fn tags(&self) -> &HashMap<String, String> {
        &self.metadata.tags
    }

    fn reset(&mut self) {
        self.value.store(0, Ordering::Relaxed);
        *self.last_updated.write() = Utc::now();
    }

    fn last_updated(&self) -> DateTime<Utc> {
        *self.last_updated.read()
    }
}

/// Rate counter that tracks events per second
#[derive(Debug)]
pub struct RateCounter {
    metadata: MetricMetadata,
    events: Arc<RwLock<Vec<DateTime<Utc>>>>,
    window_seconds: u64,
    last_updated: Arc<RwLock<DateTime<Utc>>>,
}

impl RateCounter {
    pub fn new(name: impl Into<String>, window_seconds: u64) -> Self {
        Self {
            metadata: MetricMetadata::new(name),
            events: Arc::new(RwLock::new(Vec::new())),
            window_seconds,
            last_updated: Arc::new(RwLock::new(Utc::now())),
        }
    }

    /// Record an event
    pub fn record(&self) {
        let now = Utc::now();
        let mut events = self.events.write();

        // Remove old events outside the window
        let cutoff = now - chrono::Duration::seconds(self.window_seconds as i64);
        events.retain(|t| *t > cutoff);

        events.push(now);
        drop(events);

        *self.last_updated.write() = now;
    }

    /// Get the current rate (events per second)
    pub fn rate(&self) -> f64 {
        let now = Utc::now();
        let cutoff = now - chrono::Duration::seconds(self.window_seconds as i64);

        let events = self.events.read();
        let count = events.iter().filter(|t| **t > cutoff).count();

        count as f64 / self.window_seconds as f64
    }

    /// Get the total event count in the window
    pub fn count(&self) -> usize {
        let now = Utc::now();
        let cutoff = now - chrono::Duration::seconds(self.window_seconds as i64);

        let events = self.events.read();
        events.iter().filter(|t| **t > cutoff).count()
    }

    /// Get metadata
    pub fn metadata(&self) -> &MetricMetadata {
        &self.metadata
    }
}

impl Metric for RateCounter {
    fn name(&self) -> &str {
        &self.metadata.name
    }

    fn tags(&self) -> &HashMap<String, String> {
        &self.metadata.tags
    }

    fn reset(&mut self) {
        self.events.write().clear();
        *self.last_updated.write() = Utc::now();
    }

    fn last_updated(&self) -> DateTime<Utc> {
        *self.last_updated.read()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterSnapshot {
    pub name: String,
    pub value: u64,
    pub timestamp: DateTime<Utc>,
    pub tags: HashMap<String, String>,
}

impl From<&Counter> for CounterSnapshot {
    fn from(counter: &Counter) -> Self {
        Self {
            name: counter.name().to_string(),
            value: counter.value(),
            timestamp: counter.last_updated(),
            tags: counter.tags().clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter() {
        let counter = Counter::new("test");
        assert_eq!(counter.value(), 0);

        counter.inc();
        assert_eq!(counter.value(), 1);

        counter.add(5);
        assert_eq!(counter.value(), 6);
    }

    #[test]
    fn test_rate_counter() {
        let counter = RateCounter::new("test", 1);

        counter.record();
        counter.record();

        assert_eq!(counter.count(), 2);
        assert!(counter.rate() > 0.0);
    }
}
