//! Histogram metrics for distributions

use parking_lot::RwLock;
use std::sync::Arc;

/// A histogram metric for tracking distributions
#[derive(Clone)]
pub struct Histogram {
    name: String,
    description: String,
    data: Arc<RwLock<HistogramData>>,
}

#[derive(Debug, Clone)]
struct HistogramData {
    count: u64,
    sum: f64,
    min: f64,
    max: f64,
    buckets: Vec<Bucket>,
}

#[derive(Debug, Clone)]
struct Bucket {
    upper_bound: f64,
    count: u64,
}

impl Histogram {
    /// Create a new histogram with default buckets
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self::with_buckets(
            name,
            description,
            vec![
                0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
            ],
        )
    }

    /// Create a new histogram with custom buckets
    pub fn with_buckets(
        name: impl Into<String>,
        description: impl Into<String>,
        bucket_bounds: Vec<f64>,
    ) -> Self {
        let mut buckets = bucket_bounds
            .into_iter()
            .map(|upper_bound| Bucket {
                upper_bound,
                count: 0,
            })
            .collect::<Vec<_>>();

        // Add infinity bucket
        buckets.push(Bucket {
            upper_bound: f64::INFINITY,
            count: 0,
        });

        Self {
            name: name.into(),
            description: description.into(),
            data: Arc::new(RwLock::new(HistogramData {
                count: 0,
                sum: 0.0,
                min: f64::INFINITY,
                max: f64::NEG_INFINITY,
                buckets,
            })),
        }
    }

    /// Record a value
    pub fn observe(&self, value: f64) {
        let mut data = self.data.write();

        data.count += 1;
        data.sum += value;

        if value < data.min {
            data.min = value;
        }
        if value > data.max {
            data.max = value;
        }

        // Update buckets
        for bucket in &mut data.buckets {
            if value <= bucket.upper_bound {
                bucket.count += 1;
            }
        }
    }

    /// Get the count of observations
    pub fn count(&self) -> u64 {
        self.data.read().count
    }

    /// Get the sum of all observations
    pub fn sum(&self) -> f64 {
        self.data.read().sum
    }

    /// Get the mean of all observations
    pub fn mean(&self) -> f64 {
        let data = self.data.read();
        if data.count == 0 {
            0.0
        } else {
            data.sum / data.count as f64
        }
    }

    /// Get the minimum value
    pub fn min(&self) -> f64 {
        let min = self.data.read().min;
        if min == f64::INFINITY {
            0.0
        } else {
            min
        }
    }

    /// Get the maximum value
    pub fn max(&self) -> f64 {
        let max = self.data.read().max;
        if max == f64::NEG_INFINITY {
            0.0
        } else {
            max
        }
    }

    /// Get the histogram name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the histogram description
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Reset the histogram
    pub fn reset(&self) {
        let mut data = self.data.write();
        data.count = 0;
        data.sum = 0.0;
        data.min = f64::INFINITY;
        data.max = f64::NEG_INFINITY;
        for bucket in &mut data.buckets {
            bucket.count = 0;
        }
    }

    /// Get bucket counts
    pub fn buckets(&self) -> Vec<(f64, u64)> {
        self.data
            .read()
            .buckets
            .iter()
            .map(|b| (b.upper_bound, b.count))
            .collect()
    }

    /// Get a snapshot of the histogram data
    pub fn snapshot(&self) -> HistogramSnapshot {
        let data = self.data.read();
        HistogramSnapshot {
            count: data.count,
            sum: data.sum,
            min: self.min(),
            max: self.max(),
            mean: self.mean(),
            buckets: data
                .buckets
                .iter()
                .map(|b| (b.upper_bound, b.count))
                .collect(),
        }
    }
}

/// Snapshot of histogram data
#[derive(Debug, Clone)]
pub struct HistogramSnapshot {
    pub count: u64,
    pub sum: f64,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub buckets: Vec<(f64, u64)>,
}

impl std::fmt::Debug for Histogram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Histogram")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("count", &self.count())
            .field("sum", &self.sum())
            .field("mean", &self.mean())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_histogram_observe() {
        let histogram = Histogram::new("test_histogram", "A test histogram");

        histogram.observe(1.0);
        histogram.observe(2.0);
        histogram.observe(3.0);

        assert_eq!(histogram.count(), 3);
        assert_eq!(histogram.sum(), 6.0);
        assert_eq!(histogram.mean(), 2.0);
        assert_eq!(histogram.min(), 1.0);
        assert_eq!(histogram.max(), 3.0);
    }

    #[test]
    fn test_histogram_buckets() {
        let histogram = Histogram::with_buckets("test", "test", vec![1.0, 5.0, 10.0]);

        histogram.observe(0.5);
        histogram.observe(3.0);
        histogram.observe(7.0);
        histogram.observe(15.0);

        let buckets = histogram.buckets();
        assert_eq!(buckets[0].1, 1); // <= 1.0
        assert_eq!(buckets[1].1, 2); // <= 5.0
        assert_eq!(buckets[2].1, 3); // <= 10.0
        assert_eq!(buckets[3].1, 4); // <= inf
    }
}
