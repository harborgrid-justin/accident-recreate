//! Anomaly detection algorithms

use crate::error::{AnalyticsError, Result};
use crate::statistics::descriptive::Statistics;
use serde::{Deserialize, Serialize};

/// Anomaly detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    pub index: usize,
    pub value: f64,
    pub score: f64,
    pub is_anomaly: bool,
}

/// Z-score based anomaly detector
pub struct ZScoreDetector {
    threshold: f64,
}

impl ZScoreDetector {
    pub fn new(threshold: f64) -> Self {
        Self { threshold }
    }

    /// Detect anomalies using Z-score method
    pub fn detect(&self, data: &[f64]) -> Result<Vec<Anomaly>> {
        if data.len() < 3 {
            return Err(AnalyticsError::InsufficientData(
                "Need at least 3 data points for Z-score detection".to_string(),
            ));
        }

        let mean = Statistics::mean(data);
        let std_dev = Statistics::std_dev(data);

        if std_dev == 0.0 {
            return Err(AnalyticsError::InvalidData(
                "Standard deviation is zero".to_string(),
            ));
        }

        let anomalies = data
            .iter()
            .enumerate()
            .map(|(i, &value)| {
                let z_score = ((value - mean) / std_dev).abs();
                Anomaly {
                    index: i,
                    value,
                    score: z_score,
                    is_anomaly: z_score > self.threshold,
                }
            })
            .collect();

        Ok(anomalies)
    }
}

/// IQR (Interquartile Range) based anomaly detector
pub struct IQRDetector {
    multiplier: f64,
}

impl IQRDetector {
    pub fn new(multiplier: f64) -> Self {
        Self { multiplier }
    }

    /// Detect anomalies using IQR method
    pub fn detect(&self, data: &[f64]) -> Result<Vec<Anomaly>> {
        if data.len() < 4 {
            return Err(AnalyticsError::InsufficientData(
                "Need at least 4 data points for IQR detection".to_string(),
            ));
        }

        let mut sorted = data.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let q1 = Statistics::percentile(&sorted, 0.25);
        let q3 = Statistics::percentile(&sorted, 0.75);
        let iqr = q3 - q1;

        let lower_bound = q1 - self.multiplier * iqr;
        let upper_bound = q3 + self.multiplier * iqr;

        let anomalies = data
            .iter()
            .enumerate()
            .map(|(i, &value)| {
                let distance_from_bounds = if value < lower_bound {
                    lower_bound - value
                } else if value > upper_bound {
                    value - upper_bound
                } else {
                    0.0
                };

                Anomaly {
                    index: i,
                    value,
                    score: distance_from_bounds / iqr,
                    is_anomaly: value < lower_bound || value > upper_bound,
                }
            })
            .collect();

        Ok(anomalies)
    }
}

/// Moving average based anomaly detector
pub struct MovingAverageDetector {
    window_size: usize,
    threshold: f64,
}

impl MovingAverageDetector {
    pub fn new(window_size: usize, threshold: f64) -> Self {
        Self {
            window_size,
            threshold,
        }
    }

    /// Detect anomalies by deviation from moving average
    pub fn detect(&self, data: &[f64]) -> Result<Vec<Anomaly>> {
        if data.len() < self.window_size {
            return Err(AnalyticsError::InsufficientData(format!(
                "Need at least {} data points",
                self.window_size
            )));
        }

        let mut anomalies = Vec::new();

        for i in 0..data.len() {
            if i < self.window_size - 1 {
                anomalies.push(Anomaly {
                    index: i,
                    value: data[i],
                    score: 0.0,
                    is_anomaly: false,
                });
                continue;
            }

            let window_start = i.saturating_sub(self.window_size - 1);
            let window = &data[window_start..=i];
            let ma = Statistics::mean(window);
            let std_dev = Statistics::std_dev(window);

            let deviation = if std_dev > 0.0 {
                (data[i] - ma).abs() / std_dev
            } else {
                0.0
            };

            anomalies.push(Anomaly {
                index: i,
                value: data[i],
                score: deviation,
                is_anomaly: deviation > self.threshold,
            });
        }

        Ok(anomalies)
    }
}

/// Isolation Forest-inspired anomaly detector (simplified)
pub struct IsolationForestDetector {
    num_trees: usize,
    sample_size: usize,
    threshold: f64,
}

impl IsolationForestDetector {
    pub fn new(num_trees: usize, sample_size: usize, threshold: f64) -> Self {
        Self {
            num_trees,
            sample_size,
            threshold,
        }
    }

    /// Detect anomalies using isolation forest approach
    pub fn detect(&self, data: &[f64]) -> Result<Vec<Anomaly>> {
        if data.is_empty() {
            return Err(AnalyticsError::InsufficientData(
                "Cannot detect anomalies on empty data".to_string(),
            ));
        }

        let mut scores = vec![0.0; data.len()];

        // Build multiple isolation trees
        for _ in 0..self.num_trees {
            let tree_scores = self.build_tree(data);
            for (i, score) in tree_scores.iter().enumerate() {
                scores[i] += score;
            }
        }

        // Average scores across trees
        for score in scores.iter_mut() {
            *score /= self.num_trees as f64;
        }

        // Normalize scores to [0, 1]
        let max_score = scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        if max_score > 0.0 {
            for score in scores.iter_mut() {
                *score /= max_score;
            }
        }

        let anomalies = data
            .iter()
            .enumerate()
            .zip(scores.iter())
            .map(|((i, &value), &score)| Anomaly {
                index: i,
                value,
                score,
                is_anomaly: score > self.threshold,
            })
            .collect();

        Ok(anomalies)
    }

    fn build_tree(&self, data: &[f64]) -> Vec<f64> {
        let mut scores = vec![0.0; data.len()];

        // Sample data
        let sample_size = self.sample_size.min(data.len());

        for (i, &value) in data.iter().enumerate() {
            // Calculate path length for isolation
            let path_length = self.path_length(value, data, sample_size, 0, 10);
            scores[i] = path_length;
        }

        scores
    }

    fn path_length(&self, value: f64, data: &[f64], sample_size: usize, depth: usize, max_depth: usize) -> f64 {
        if depth >= max_depth || sample_size <= 1 {
            return depth as f64 + self.average_path_length(sample_size);
        }

        // Find min and max in sample
        let min = data.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        if max - min < 1e-10 {
            return depth as f64;
        }

        // Random split point
        let split = min + (max - min) * 0.5;

        if value < split {
            let left_size = data.iter().filter(|&&x| x < split).count();
            self.path_length(value, data, left_size, depth + 1, max_depth)
        } else {
            let right_size = data.iter().filter(|&&x| x >= split).count();
            self.path_length(value, data, right_size, depth + 1, max_depth)
        }
    }

    fn average_path_length(&self, n: usize) -> f64 {
        if n <= 1 {
            return 0.0;
        }

        2.0 * ((n as f64).ln() + 0.5772156649) - 2.0 * (n - 1) as f64 / n as f64
    }
}

/// DBSCAN-based density anomaly detector
pub struct DensityDetector {
    epsilon: f64,
    min_points: usize,
}

impl DensityDetector {
    pub fn new(epsilon: f64, min_points: usize) -> Self {
        Self { epsilon, min_points }
    }

    /// Detect anomalies based on density
    pub fn detect(&self, data: &[f64]) -> Result<Vec<Anomaly>> {
        if data.is_empty() {
            return Err(AnalyticsError::InsufficientData(
                "Cannot detect anomalies on empty data".to_string(),
            ));
        }

        let mut anomalies = Vec::new();

        for (i, &value) in data.iter().enumerate() {
            // Count neighbors within epsilon
            let neighbors = data
                .iter()
                .filter(|&&x| (x - value).abs() <= self.epsilon)
                .count();

            let is_anomaly = neighbors < self.min_points;
            let score = if self.min_points > 0 {
                1.0 - (neighbors as f64 / self.min_points as f64).min(1.0)
            } else {
                0.0
            };

            anomalies.push(Anomaly {
                index: i,
                value,
                score,
                is_anomaly,
            });
        }

        Ok(anomalies)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zscore_detector() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 100.0]; // 100 is anomaly
        let detector = ZScoreDetector::new(3.0);

        let anomalies = detector.detect(&data).unwrap();
        let anomaly_count = anomalies.iter().filter(|a| a.is_anomaly).count();

        assert!(anomaly_count > 0);
    }

    #[test]
    fn test_iqr_detector() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 100.0];
        let detector = IQRDetector::new(1.5);

        let anomalies = detector.detect(&data).unwrap();
        let anomaly_count = anomalies.iter().filter(|a| a.is_anomaly).count();

        assert!(anomaly_count > 0);
    }

    #[test]
    fn test_moving_average_detector() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 100.0];
        let detector = MovingAverageDetector::new(3, 3.0);

        let anomalies = detector.detect(&data).unwrap();
        assert_eq!(anomalies.len(), data.len());
    }
}
