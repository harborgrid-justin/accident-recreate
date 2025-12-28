//! Descriptive statistics - mean, median, variance, std dev, etc.

use crate::error::{AnalyticsError, Result};
use serde::{Deserialize, Serialize};

/// Comprehensive descriptive statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DescriptiveStats {
    pub count: usize,
    pub mean: f64,
    pub median: f64,
    pub mode: Option<f64>,
    pub variance: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub range: f64,
    pub q1: f64,
    pub q3: f64,
    pub iqr: f64,
    pub skewness: f64,
    pub kurtosis: f64,
}

impl DescriptiveStats {
    pub fn from_data(data: &[f64]) -> Result<Self> {
        if data.is_empty() {
            return Err(AnalyticsError::InsufficientData(
                "Cannot compute statistics on empty dataset".to_string(),
            ));
        }

        let mut sorted = data.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let count = data.len();
        let mean = Statistics::mean(data);
        let median = Statistics::median(&sorted);
        let mode = Statistics::mode(data);
        let variance = Statistics::variance(data, mean);
        let std_dev = variance.sqrt();
        let min = sorted[0];
        let max = sorted[count - 1];
        let range = max - min;
        let q1 = Statistics::percentile(&sorted, 0.25);
        let q3 = Statistics::percentile(&sorted, 0.75);
        let iqr = q3 - q1;
        let skewness = Statistics::skewness(data, mean, std_dev);
        let kurtosis = Statistics::kurtosis(data, mean, std_dev);

        Ok(Self {
            count,
            mean,
            median,
            mode,
            variance,
            std_dev,
            min,
            max,
            range,
            q1,
            q3,
            iqr,
            skewness,
            kurtosis,
        })
    }

    /// Check if the data is normally distributed (rough heuristic)
    pub fn is_normal(&self) -> bool {
        // Check if skewness and kurtosis are within reasonable bounds
        self.skewness.abs() < 1.0 && (self.kurtosis - 3.0).abs() < 1.0
    }

    /// Identify outliers using IQR method
    pub fn outliers(&self, data: &[f64]) -> Vec<f64> {
        let lower_bound = self.q1 - 1.5 * self.iqr;
        let upper_bound = self.q3 + 1.5 * self.iqr;

        data.iter()
            .copied()
            .filter(|&x| x < lower_bound || x > upper_bound)
            .collect()
    }

    /// Get the coefficient of variation (CV)
    pub fn coefficient_of_variation(&self) -> f64 {
        if self.mean == 0.0 {
            0.0
        } else {
            self.std_dev / self.mean
        }
    }
}

/// Statistical functions
pub struct Statistics;

impl Statistics {
    /// Calculate mean
    pub fn mean(data: &[f64]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }
        data.iter().sum::<f64>() / data.len() as f64
    }

    /// Calculate median (data must be sorted)
    pub fn median(sorted_data: &[f64]) -> f64 {
        if sorted_data.is_empty() {
            return 0.0;
        }

        let n = sorted_data.len();
        if n % 2 == 0 {
            (sorted_data[n / 2 - 1] + sorted_data[n / 2]) / 2.0
        } else {
            sorted_data[n / 2]
        }
    }

    /// Calculate mode (most frequent value)
    pub fn mode(data: &[f64]) -> Option<f64> {
        if data.is_empty() {
            return None;
        }

        use std::collections::HashMap;
        let mut counts = HashMap::new();

        for &value in data {
            *counts.entry(value.to_bits()).or_insert(0) += 1;
        }

        counts
            .into_iter()
            .max_by_key(|&(_, count)| count)
            .map(|(bits, _)| f64::from_bits(bits))
    }

    /// Calculate variance
    pub fn variance(data: &[f64], mean: f64) -> f64 {
        if data.len() <= 1 {
            return 0.0;
        }

        let sum_squared_diff: f64 = data.iter().map(|x| (x - mean).powi(2)).sum();
        sum_squared_diff / (data.len() - 1) as f64
    }

    /// Calculate standard deviation
    pub fn std_dev(data: &[f64]) -> f64 {
        let mean = Self::mean(data);
        Self::variance(data, mean).sqrt()
    }

    /// Calculate percentile (data must be sorted)
    pub fn percentile(sorted_data: &[f64], p: f64) -> f64 {
        if sorted_data.is_empty() {
            return 0.0;
        }

        let index = (sorted_data.len() as f64 - 1.0) * p;
        let lower = index.floor() as usize;
        let upper = index.ceil() as usize;

        if lower == upper {
            sorted_data[lower]
        } else {
            let weight = index - lower as f64;
            sorted_data[lower] * (1.0 - weight) + sorted_data[upper] * weight
        }
    }

    /// Calculate skewness
    pub fn skewness(data: &[f64], mean: f64, std_dev: f64) -> f64 {
        if data.len() < 3 || std_dev == 0.0 {
            return 0.0;
        }

        let n = data.len() as f64;
        let sum_cubed: f64 = data.iter().map(|x| ((x - mean) / std_dev).powi(3)).sum();

        (n / ((n - 1.0) * (n - 2.0))) * sum_cubed
    }

    /// Calculate kurtosis
    pub fn kurtosis(data: &[f64], mean: f64, std_dev: f64) -> f64 {
        if data.len() < 4 || std_dev == 0.0 {
            return 0.0;
        }

        let n = data.len() as f64;
        let sum_fourth: f64 = data.iter().map(|x| ((x - mean) / std_dev).powi(4)).sum();

        let term1 = (n * (n + 1.0)) / ((n - 1.0) * (n - 2.0) * (n - 3.0));
        let term2 = (3.0 * (n - 1.0).powi(2)) / ((n - 2.0) * (n - 3.0));

        term1 * sum_fourth - term2
    }

    /// Calculate covariance between two datasets
    pub fn covariance(x: &[f64], y: &[f64]) -> Result<f64> {
        if x.len() != y.len() {
            return Err(AnalyticsError::InvalidData(
                "Datasets must have equal length".to_string(),
            ));
        }

        if x.is_empty() {
            return Err(AnalyticsError::InsufficientData(
                "Cannot compute covariance on empty dataset".to_string(),
            ));
        }

        let mean_x = Self::mean(x);
        let mean_y = Self::mean(y);

        let sum: f64 = x
            .iter()
            .zip(y.iter())
            .map(|(xi, yi)| (xi - mean_x) * (yi - mean_y))
            .sum();

        Ok(sum / (x.len() - 1) as f64)
    }

    /// Calculate z-scores
    pub fn z_scores(data: &[f64]) -> Vec<f64> {
        let mean = Self::mean(data);
        let std_dev = Self::std_dev(data);

        if std_dev == 0.0 {
            return vec![0.0; data.len()];
        }

        data.iter().map(|x| (x - mean) / std_dev).collect()
    }

    /// Normalize data to [0, 1] range
    pub fn normalize(data: &[f64]) -> Vec<f64> {
        if data.is_empty() {
            return Vec::new();
        }

        let min = data.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        if min == max {
            return vec![0.5; data.len()];
        }

        data.iter().map(|x| (x - min) / (max - min)).collect()
    }

    /// Standardize data (mean=0, std=1)
    pub fn standardize(data: &[f64]) -> Vec<f64> {
        let mean = Self::mean(data);
        let std_dev = Self::std_dev(data);

        if std_dev == 0.0 {
            return vec![0.0; data.len()];
        }

        data.iter().map(|x| (x - mean) / std_dev).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mean() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(Statistics::mean(&data), 3.0);
    }

    #[test]
    fn test_median() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(Statistics::median(&data), 3.0);

        let data = vec![1.0, 2.0, 3.0, 4.0];
        assert_eq!(Statistics::median(&data), 2.5);
    }

    #[test]
    fn test_variance() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let mean = Statistics::mean(&data);
        let variance = Statistics::variance(&data, mean);
        assert!(variance > 0.0);
    }

    #[test]
    fn test_descriptive_stats() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let stats = DescriptiveStats::from_data(&data).unwrap();

        assert_eq!(stats.count, 10);
        assert_eq!(stats.mean, 5.5);
        assert_eq!(stats.median, 5.5);
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 10.0);
    }
}
