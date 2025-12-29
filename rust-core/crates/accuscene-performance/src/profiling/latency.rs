//! Latency histogram tracking

use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// Latency histogram for tracking latency distribution
pub struct LatencyHistogram {
    buckets: Arc<Mutex<Vec<u64>>>,
    bucket_size_us: u64,
    max_latency_us: u64,
    overflow_count: Arc<Mutex<u64>>,
}

impl LatencyHistogram {
    /// Create a new latency histogram
    pub fn new(max_latency_us: u64, num_buckets: usize) -> Self {
        Self {
            buckets: Arc::new(Mutex::new(vec![0; num_buckets])),
            bucket_size_us: max_latency_us / num_buckets as u64,
            max_latency_us,
            overflow_count: Arc::new(Mutex::new(0)),
        }
    }

    /// Record a latency measurement
    pub fn record(&self, latency: Duration) {
        let latency_us = latency.as_micros() as u64;

        if latency_us >= self.max_latency_us {
            *self.overflow_count.lock() += 1;
            return;
        }

        let bucket_idx = (latency_us / self.bucket_size_us) as usize;
        let mut buckets = self.buckets.lock();

        if bucket_idx < buckets.len() {
            buckets[bucket_idx] += 1;
        }
    }

    /// Get the count for a specific bucket
    pub fn bucket(&self, index: usize) -> Option<u64> {
        self.buckets.lock().get(index).copied()
    }

    /// Get all bucket counts
    pub fn buckets(&self) -> Vec<u64> {
        self.buckets.lock().clone()
    }

    /// Get overflow count
    pub fn overflow_count(&self) -> u64 {
        *self.overflow_count.lock()
    }

    /// Get total count
    pub fn total_count(&self) -> u64 {
        self.buckets.lock().iter().sum::<u64>() + *self.overflow_count.lock()
    }

    /// Get percentile
    pub fn percentile(&self, p: f64) -> Option<Duration> {
        let buckets = self.buckets.lock();
        let total: u64 = buckets.iter().sum();

        if total == 0 {
            return None;
        }

        let target_count = (total as f64 * p) as u64;
        let mut cumulative = 0;

        for (i, &count) in buckets.iter().enumerate() {
            cumulative += count;
            if cumulative >= target_count {
                let latency_us = (i as u64 + 1) * self.bucket_size_us;
                return Some(Duration::from_micros(latency_us));
            }
        }

        None
    }

    /// Get p50 (median)
    pub fn p50(&self) -> Option<Duration> {
        self.percentile(0.5)
    }

    /// Get p95
    pub fn p95(&self) -> Option<Duration> {
        self.percentile(0.95)
    }

    /// Get p99
    pub fn p99(&self) -> Option<Duration> {
        self.percentile(0.99)
    }

    /// Reset the histogram
    pub fn reset(&self) {
        self.buckets.lock().fill(0);
        *self.overflow_count.lock() = 0;
    }

    /// Print histogram
    pub fn print(&self) {
        let buckets = self.buckets.lock();
        let total: u64 = buckets.iter().sum();

        println!("\nLatency Histogram:");
        println!("{:<15} {:>10} {:>10}", "Range (µs)", "Count", "Percent");
        println!("{}", "-".repeat(40));

        for (i, &count) in buckets.iter().enumerate() {
            if count > 0 {
                let start = i as u64 * self.bucket_size_us;
                let end = (i as u64 + 1) * self.bucket_size_us;
                let percent = (count as f64 / total as f64) * 100.0;

                println!(
                    "{:7}-{:<7} {:>10} {:>9.2}%",
                    start, end, count, percent
                );
            }
        }

        let overflow = *self.overflow_count.lock();
        if overflow > 0 {
            println!("{:<15} {:>10}", "Overflow", overflow);
        }
    }
}

impl Clone for LatencyHistogram {
    fn clone(&self) -> Self {
        Self {
            buckets: self.buckets.clone(),
            bucket_size_us: self.bucket_size_us,
            max_latency_us: self.max_latency_us,
            overflow_count: self.overflow_count.clone(),
        }
    }
}

/// Latency tracker for multiple operations
pub struct LatencyTracker {
    histograms: Arc<Mutex<HashMap<String, LatencyHistogram>>>,
    default_max_us: u64,
    default_buckets: usize,
}

impl LatencyTracker {
    /// Create a new latency tracker
    pub fn new(default_max_us: u64, default_buckets: usize) -> Self {
        Self {
            histograms: Arc::new(Mutex::new(HashMap::new())),
            default_max_us,
            default_buckets,
        }
    }

    /// Record a latency for a named operation
    pub fn record(&self, name: impl Into<String>, latency: Duration) {
        let name = name.into();
        let mut histograms = self.histograms.lock();

        let histogram = histograms.entry(name).or_insert_with(|| {
            LatencyHistogram::new(self.default_max_us, self.default_buckets)
        });

        histogram.record(latency);
    }

    /// Get histogram for a named operation
    pub fn histogram(&self, name: &str) -> Option<LatencyHistogram> {
        self.histograms.lock().get(name).cloned()
    }

    /// Get all operation names
    pub fn operations(&self) -> Vec<String> {
        self.histograms.lock().keys().cloned().collect()
    }

    /// Reset all histograms
    pub fn reset(&self) {
        for histogram in self.histograms.lock().values() {
            histogram.reset();
        }
    }

    /// Print all histograms
    pub fn print_all(&self) {
        let histograms = self.histograms.lock();

        for (name, histogram) in histograms.iter() {
            println!("\nOperation: {}", name);
            histogram.print();

            if let Some(p50) = histogram.p50() {
                println!("P50: {:?}", p50);
            }
            if let Some(p95) = histogram.p95() {
                println!("P95: {:?}", p95);
            }
            if let Some(p99) = histogram.p99() {
                println!("P99: {:?}", p99);
            }
        }
    }
}

impl Default for LatencyTracker {
    fn default() -> Self {
        Self::new(10_000_000, 100) // 10s max, 100 buckets
    }
}

impl Clone for LatencyTracker {
    fn clone(&self) -> Self {
        Self {
            histograms: self.histograms.clone(),
            default_max_us: self.default_max_us,
            default_buckets: self.default_buckets,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_latency_histogram() {
        let histogram = LatencyHistogram::new(1000, 10);

        histogram.record(Duration::from_micros(50));
        histogram.record(Duration::from_micros(150));
        histogram.record(Duration::from_micros(250));

        assert_eq!(histogram.total_count(), 3);
    }

    #[test]
    fn test_histogram_percentiles() {
        let histogram = LatencyHistogram::new(1000, 10);

        for i in 0..100 {
            histogram.record(Duration::from_micros(i * 10));
        }

        assert!(histogram.p50().is_some());
        assert!(histogram.p95().is_some());
        assert!(histogram.p99().is_some());
    }

    #[test]
    fn test_histogram_overflow() {
        let histogram = LatencyHistogram::new(1000, 10);

        histogram.record(Duration::from_micros(500));
        histogram.record(Duration::from_micros(1500)); // Overflow

        assert_eq!(histogram.overflow_count(), 1);
    }

    #[test]
    fn test_latency_tracker() {
        let tracker = LatencyTracker::new(10000, 10);

        tracker.record("operation1", Duration::from_micros(100));
        tracker.record("operation1", Duration::from_micros(200));
        tracker.record("operation2", Duration::from_micros(150));

        assert_eq!(tracker.operations().len(), 2);

        let hist1 = tracker.histogram("operation1").unwrap();
        assert_eq!(hist1.total_count(), 2);

        let hist2 = tracker.histogram("operation2").unwrap();
        assert_eq!(hist2.total_count(), 1);
    }

    #[test]
    fn test_histogram_reset() {
        let histogram = LatencyHistogram::new(1000, 10);

        histogram.record(Duration::from_micros(100));
        assert_eq!(histogram.total_count(), 1);

        histogram.reset();
        assert_eq!(histogram.total_count(), 0);
    }
}
