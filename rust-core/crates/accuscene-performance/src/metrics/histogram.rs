//! HDR histogram for latency metrics

use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

/// Histogram for tracking value distributions
#[derive(Clone)]
pub struct Histogram {
    name: String,
    values: Arc<Mutex<Vec<f64>>>,
    sum: Arc<Mutex<f64>>,
    count: Arc<Mutex<u64>>,
}

impl Histogram {
    /// Create a new histogram
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            values: Arc::new(Mutex::new(Vec::new())),
            sum: Arc::new(Mutex::new(0.0)),
            count: Arc::new(Mutex::new(0)),
        }
    }

    /// Observe a value
    pub fn observe(&self, value: f64) {
        self.values.lock().push(value);
        *self.sum.lock() += value;
        *self.count.lock() += 1;
    }

    /// Get statistics
    pub fn stats(&self) -> HistogramStats {
        let values = self.values.lock();
        let mut sorted = values.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let count = *self.count.lock();
        let sum = *self.sum.lock();

        let min = sorted.first().copied().unwrap_or(0.0);
        let max = sorted.last().copied().unwrap_or(0.0);
        let avg = if count > 0 { sum / count as f64 } else { 0.0 };

        HistogramStats {
            count,
            sum,
            min,
            max,
            avg,
            p50: Self::percentile(&sorted, 0.5),
            p95: Self::percentile(&sorted, 0.95),
            p99: Self::percentile(&sorted, 0.99),
        }
    }

    /// Calculate percentile
    fn percentile(sorted: &[f64], p: f64) -> f64 {
        if sorted.is_empty() {
            return 0.0;
        }

        let index = ((sorted.len() as f64 - 1.0) * p) as usize;
        sorted[index]
    }

    /// Reset the histogram
    pub fn reset(&self) {
        self.values.lock().clear();
        *self.sum.lock() = 0.0;
        *self.count.lock() = 0;
    }

    /// Get the histogram name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get count
    pub fn count(&self) -> u64 {
        *self.count.lock()
    }

    /// Get sum
    pub fn sum(&self) -> f64 {
        *self.sum.lock()
    }
}

/// Histogram statistics
#[derive(Debug, Clone)]
pub struct HistogramStats {
    pub count: u64,
    pub sum: f64,
    pub min: f64,
    pub max: f64,
    pub avg: f64,
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
}

/// Histogram vector with labels
pub struct HistogramVec {
    name: String,
    histograms: Arc<Mutex<HashMap<Vec<String>, Histogram>>>,
}

impl HistogramVec {
    /// Create a new histogram vector
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            histograms: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get or create a histogram with labels
    pub fn with_labels(&self, labels: &[impl AsRef<str>]) -> Histogram {
        let label_vec: Vec<String> = labels.iter().map(|s| s.as_ref().to_string()).collect();

        let mut histograms = self.histograms.lock();

        histograms
            .entry(label_vec.clone())
            .or_insert_with(|| {
                let name = format!("{}:{}", self.name, label_vec.join(","));
                Histogram::new(name)
            })
            .clone()
    }

    /// Get all histograms
    pub fn histograms(&self) -> HashMap<Vec<String>, Histogram> {
        self.histograms.lock().clone()
    }

    /// Reset all histograms
    pub fn reset(&self) {
        for histogram in self.histograms.lock().values() {
            histogram.reset();
        }
    }
}

impl Clone for HistogramVec {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            histograms: self.histograms.clone(),
        }
    }
}

/// Timer for histogram observations
pub struct HistogramTimer {
    histogram: Histogram,
    start: std::time::Instant,
}

impl HistogramTimer {
    /// Create a new timer
    pub fn new(histogram: Histogram) -> Self {
        Self {
            histogram,
            start: std::time::Instant::now(),
        }
    }

    /// Stop the timer and record the duration
    pub fn stop(self) {
        let duration = self.start.elapsed().as_secs_f64();
        self.histogram.observe(duration);
    }

    /// Observe and restart
    pub fn observe_and_restart(&mut self) {
        let duration = self.start.elapsed().as_secs_f64();
        self.histogram.observe(duration);
        self.start = std::time::Instant::now();
    }
}

impl Drop for HistogramTimer {
    fn drop(&mut self) {
        let duration = self.start.elapsed().as_secs_f64();
        self.histogram.observe(duration);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_histogram() {
        let histogram = Histogram::new("test");

        histogram.observe(1.0);
        histogram.observe(2.0);
        histogram.observe(3.0);

        let stats = histogram.stats();
        assert_eq!(stats.count, 3);
        assert_eq!(stats.sum, 6.0);
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 3.0);
        assert_eq!(stats.avg, 2.0);
    }

    #[test]
    fn test_histogram_percentiles() {
        let histogram = Histogram::new("percentiles");

        for i in 1..=100 {
            histogram.observe(i as f64);
        }

        let stats = histogram.stats();
        assert!(stats.p50 >= 49.0 && stats.p50 <= 51.0);
        assert!(stats.p95 >= 94.0 && stats.p95 <= 96.0);
        assert!(stats.p99 >= 98.0 && stats.p99 <= 100.0);
    }

    #[test]
    fn test_histogram_reset() {
        let histogram = Histogram::new("reset");

        histogram.observe(10.0);
        assert_eq!(histogram.count(), 1);

        histogram.reset();
        assert_eq!(histogram.count(), 0);
    }

    #[test]
    fn test_histogram_vec() {
        let vec = HistogramVec::new("test_vec");

        let h1 = vec.with_labels(&["method:GET"]);
        let h2 = vec.with_labels(&["method:POST"]);

        h1.observe(1.0);
        h1.observe(2.0);
        h2.observe(3.0);

        assert_eq!(h1.count(), 2);
        assert_eq!(h2.count(), 1);
    }

    #[test]
    fn test_histogram_timer() {
        let histogram = Histogram::new("timer_test");

        {
            let _timer = HistogramTimer::new(histogram.clone());
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        assert_eq!(histogram.count(), 1);
        assert!(histogram.sum() > 0.0);
    }
}
