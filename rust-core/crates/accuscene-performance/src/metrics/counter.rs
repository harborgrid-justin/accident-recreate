//! Atomic counters for metrics

use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Atomic counter
#[derive(Clone)]
pub struct Counter {
    name: String,
    value: Arc<AtomicU64>,
}

impl Counter {
    /// Create a new counter
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Increment the counter by 1
    pub fn inc(&self) {
        self.add(1);
    }

    /// Add a value to the counter
    pub fn add(&self, n: u64) {
        self.value.fetch_add(n, Ordering::Relaxed);
    }

    /// Get the current value
    pub fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }

    /// Reset the counter to 0
    pub fn reset(&self) {
        self.value.store(0, Ordering::Relaxed);
    }

    /// Get the counter name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Swap the value
    pub fn swap(&self, value: u64) -> u64 {
        self.value.swap(value, Ordering::Relaxed)
    }
}

/// Counter vector with labels
pub struct CounterVec {
    name: String,
    counters: Arc<Mutex<HashMap<Vec<String>, Counter>>>,
}

impl CounterVec {
    /// Create a new counter vector
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            counters: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get or create a counter with labels
    pub fn with_labels(&self, labels: &[impl AsRef<str>]) -> Counter {
        let label_vec: Vec<String> = labels.iter().map(|s| s.as_ref().to_string()).collect();

        let mut counters = self.counters.lock();

        counters
            .entry(label_vec.clone())
            .or_insert_with(|| {
                let name = format!("{}:{}", self.name, label_vec.join(","));
                Counter::new(name)
            })
            .clone()
    }

    /// Get all counters
    pub fn counters(&self) -> HashMap<Vec<String>, Counter> {
        self.counters.lock().clone()
    }

    /// Reset all counters
    pub fn reset(&self) {
        for counter in self.counters.lock().values() {
            counter.reset();
        }
    }
}

impl Clone for CounterVec {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            counters: self.counters.clone(),
        }
    }
}

/// Rate counter for tracking rates
pub struct RateCounter {
    counter: Counter,
    start_time: Arc<Mutex<std::time::Instant>>,
}

impl RateCounter {
    /// Create a new rate counter
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            counter: Counter::new(name),
            start_time: Arc::new(Mutex::new(std::time::Instant::now())),
        }
    }

    /// Increment the counter
    pub fn inc(&self) {
        self.counter.inc();
    }

    /// Add to the counter
    pub fn add(&self, n: u64) {
        self.counter.add(n);
    }

    /// Get the current rate (items per second)
    pub fn rate(&self) -> f64 {
        let count = self.counter.get() as f64;
        let elapsed = self.start_time.lock().elapsed().as_secs_f64();

        if elapsed > 0.0 {
            count / elapsed
        } else {
            0.0
        }
    }

    /// Get the current count
    pub fn count(&self) -> u64 {
        self.counter.get()
    }

    /// Reset the counter and timer
    pub fn reset(&self) {
        self.counter.reset();
        *self.start_time.lock() = std::time::Instant::now();
    }
}

impl Clone for RateCounter {
    fn clone(&self) -> Self {
        Self {
            counter: self.counter.clone(),
            start_time: self.start_time.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_counter() {
        let counter = Counter::new("test");

        counter.inc();
        counter.inc();
        assert_eq!(counter.get(), 2);

        counter.add(5);
        assert_eq!(counter.get(), 7);

        counter.reset();
        assert_eq!(counter.get(), 0);
    }

    #[test]
    fn test_counter_concurrent() {
        let counter = Counter::new("concurrent");
        let mut handles = vec![];

        for _ in 0..10 {
            let c = counter.clone();
            handles.push(thread::spawn(move || {
                for _ in 0..100 {
                    c.inc();
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(counter.get(), 1000);
    }

    #[test]
    fn test_counter_vec() {
        let vec = CounterVec::new("test_vec");

        let c1 = vec.with_labels(&["label1"]);
        let c2 = vec.with_labels(&["label2"]);

        c1.inc();
        c1.inc();
        c2.inc();

        assert_eq!(c1.get(), 2);
        assert_eq!(c2.get(), 1);
    }

    #[test]
    fn test_rate_counter() {
        let rate = RateCounter::new("rate_test");

        rate.add(100);
        thread::sleep(Duration::from_millis(100));

        let r = rate.rate();
        assert!(r > 0.0);
    }

    #[test]
    fn test_counter_swap() {
        let counter = Counter::new("swap_test");

        counter.add(10);
        let old = counter.swap(5);

        assert_eq!(old, 10);
        assert_eq!(counter.get(), 5);
    }
}
