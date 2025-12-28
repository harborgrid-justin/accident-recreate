//! Histogram metrics - distribution of values with percentiles

use super::{Metric, MetricMetadata};
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Histogram with configurable buckets and percentile calculation
#[derive(Debug)]
pub struct Histogram {
    metadata: MetricMetadata,
    buckets: Arc<RwLock<Vec<f64>>>,
    bucket_counts: Arc<RwLock<Vec<u64>>>,
    values: Arc<RwLock<Vec<f64>>>,
    count: Arc<RwLock<u64>>,
    sum: Arc<RwLock<f64>>,
    min: Arc<RwLock<f64>>,
    max: Arc<RwLock<f64>>,
    last_updated: Arc<RwLock<DateTime<Utc>>>,
}

impl Histogram {
    /// Create a new histogram with linear buckets
    pub fn new_linear(name: impl Into<String>, start: f64, width: f64, count: usize) -> Self {
        let buckets: Vec<f64> = (0..count).map(|i| start + (i as f64 * width)).collect();
        Self::new(name, buckets)
    }

    /// Create a new histogram with exponential buckets
    pub fn new_exponential(name: impl Into<String>, start: f64, factor: f64, count: usize) -> Self {
        let buckets: Vec<f64> = (0..count).map(|i| start * factor.powi(i as i32)).collect();
        Self::new(name, buckets)
    }

    /// Create a new histogram with custom buckets
    pub fn new(name: impl Into<String>, buckets: Vec<f64>) -> Self {
        let bucket_count = buckets.len();
        Self {
            metadata: MetricMetadata::new(name),
            buckets: Arc::new(RwLock::new(buckets)),
            bucket_counts: Arc::new(RwLock::new(vec![0; bucket_count])),
            values: Arc::new(RwLock::new(Vec::new())),
            count: Arc::new(RwLock::new(0)),
            sum: Arc::new(RwLock::new(0.0)),
            min: Arc::new(RwLock::new(f64::INFINITY)),
            max: Arc::new(RwLock::new(f64::NEG_INFINITY)),
            last_updated: Arc::new(RwLock::new(Utc::now())),
        }
    }

    /// Record a value in the histogram
    pub fn observe(&self, value: f64) {
        // Update bucket counts
        let buckets = self.buckets.read();
        let mut bucket_counts = self.bucket_counts.write();

        for (i, &bucket) in buckets.iter().enumerate() {
            if value <= bucket {
                bucket_counts[i] += 1;
                break;
            }
        }
        drop(buckets);
        drop(bucket_counts);

        // Update statistics
        let mut values = self.values.write();
        values.push(value);
        drop(values);

        *self.count.write() += 1;
        *self.sum.write() += value;

        let mut min = self.min.write();
        if value < *min {
            *min = value;
        }
        drop(min);

        let mut max = self.max.write();
        if value > *max {
            *max = value;
        }
        drop(max);

        *self.last_updated.write() = Utc::now();
    }

    /// Get the count of observations
    pub fn count(&self) -> u64 {
        *self.count.read()
    }

    /// Get the sum of all observations
    pub fn sum(&self) -> f64 {
        *self.sum.read()
    }

    /// Get the mean of all observations
    pub fn mean(&self) -> f64 {
        let count = *self.count.read();
        if count == 0 {
            return 0.0;
        }
        *self.sum.read() / count as f64
    }

    /// Get the minimum value
    pub fn min(&self) -> f64 {
        *self.min.read()
    }

    /// Get the maximum value
    pub fn max(&self) -> f64 {
        *self.max.read()
    }

    /// Calculate a specific percentile (0.0 - 1.0)
    pub fn percentile(&self, p: f64) -> Option<f64> {
        let mut values = self.values.read().clone();
        if values.is_empty() {
            return None;
        }

        values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let index = ((values.len() as f64 - 1.0) * p) as usize;
        Some(values[index])
    }

    /// Get common percentiles (p50, p90, p95, p99)
    pub fn percentiles(&self) -> HistogramPercentiles {
        HistogramPercentiles {
            p50: self.percentile(0.50),
            p90: self.percentile(0.90),
            p95: self.percentile(0.95),
            p99: self.percentile(0.99),
        }
    }

    /// Get the standard deviation
    pub fn stddev(&self) -> f64 {
        let values = self.values.read();
        if values.is_empty() {
            return 0.0;
        }

        let mean = self.mean();
        let variance: f64 = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
        variance.sqrt()
    }

    /// Get metadata
    pub fn metadata(&self) -> &MetricMetadata {
        &self.metadata
    }

    /// Get a snapshot of the histogram
    pub fn snapshot(&self) -> HistogramSnapshot {
        HistogramSnapshot {
            name: self.name().to_string(),
            count: self.count(),
            sum: self.sum(),
            mean: self.mean(),
            min: self.min(),
            max: self.max(),
            stddev: self.stddev(),
            percentiles: self.percentiles(),
            timestamp: self.last_updated(),
            tags: self.tags().clone(),
        }
    }
}

impl Metric for Histogram {
    fn name(&self) -> &str {
        &self.metadata.name
    }

    fn tags(&self) -> &HashMap<String, String> {
        &self.metadata.tags
    }

    fn reset(&mut self) {
        *self.bucket_counts.write() = vec![0; self.buckets.read().len()];
        self.values.write().clear();
        *self.count.write() = 0;
        *self.sum.write() = 0.0;
        *self.min.write() = f64::INFINITY;
        *self.max.write() = f64::NEG_INFINITY;
        *self.last_updated.write() = Utc::now();
    }

    fn last_updated(&self) -> DateTime<Utc> {
        *self.last_updated.read()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HistogramPercentiles {
    pub p50: Option<f64>,
    pub p90: Option<f64>,
    pub p95: Option<f64>,
    pub p99: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramSnapshot {
    pub name: String,
    pub count: u64,
    pub sum: f64,
    pub mean: f64,
    pub min: f64,
    pub max: f64,
    pub stddev: f64,
    pub percentiles: HistogramPercentiles,
    pub timestamp: DateTime<Utc>,
    pub tags: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_histogram() {
        let hist = Histogram::new_linear("test", 0.0, 10.0, 10);

        for i in 1..=100 {
            hist.observe(i as f64);
        }

        assert_eq!(hist.count(), 100);
        assert_eq!(hist.sum(), 5050.0);
        assert_eq!(hist.mean(), 50.5);
        assert_eq!(hist.min(), 1.0);
        assert_eq!(hist.max(), 100.0);
    }

    #[test]
    fn test_percentiles() {
        let hist = Histogram::new_linear("test", 0.0, 10.0, 10);

        for i in 1..=100 {
            hist.observe(i as f64);
        }

        let percentiles = hist.percentiles();
        assert!(percentiles.p50.is_some());
        assert!(percentiles.p99.is_some());
    }
}
