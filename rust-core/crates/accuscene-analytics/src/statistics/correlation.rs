//! Correlation analysis

use crate::error::{AnalyticsError, Result};
use crate::statistics::descriptive::Statistics;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CorrelationType {
    Pearson,
    Spearman,
    Kendall,
}

/// Correlation analyzer
pub struct CorrelationAnalyzer;

impl CorrelationAnalyzer {
    /// Calculate Pearson correlation coefficient
    pub fn pearson(x: &[f64], y: &[f64]) -> Result<f64> {
        if x.len() != y.len() {
            return Err(AnalyticsError::InvalidData(
                "Arrays must have the same length".to_string(),
            ));
        }

        if x.len() < 2 {
            return Err(AnalyticsError::InsufficientData(
                "Need at least 2 data points".to_string(),
            ));
        }

        let mean_x = Statistics::mean(x);
        let mean_y = Statistics::mean(y);

        let mut numerator = 0.0;
        let mut sum_x_sq = 0.0;
        let mut sum_y_sq = 0.0;

        for i in 0..x.len() {
            let dx = x[i] - mean_x;
            let dy = y[i] - mean_y;
            numerator += dx * dy;
            sum_x_sq += dx * dx;
            sum_y_sq += dy * dy;
        }

        let denominator = (sum_x_sq * sum_y_sq).sqrt();

        if denominator < 1e-10 {
            return Ok(0.0);
        }

        Ok(numerator / denominator)
    }

    /// Calculate Spearman rank correlation
    pub fn spearman(x: &[f64], y: &[f64]) -> Result<f64> {
        if x.len() != y.len() {
            return Err(AnalyticsError::InvalidData(
                "Arrays must have the same length".to_string(),
            ));
        }

        let rank_x = Self::rank(x);
        let rank_y = Self::rank(y);

        Self::pearson(&rank_x, &rank_y)
    }

    /// Calculate Kendall's tau correlation
    pub fn kendall(x: &[f64], y: &[f64]) -> Result<f64> {
        if x.len() != y.len() {
            return Err(AnalyticsError::InvalidData(
                "Arrays must have the same length".to_string(),
            ));
        }

        let n = x.len();
        if n < 2 {
            return Err(AnalyticsError::InsufficientData(
                "Need at least 2 data points".to_string(),
            ));
        }

        let mut concordant = 0;
        let mut discordant = 0;

        for i in 0..n - 1 {
            for j in i + 1..n {
                let sign_x = (x[j] - x[i]).signum();
                let sign_y = (y[j] - y[i]).signum();

                if sign_x * sign_y > 0.0 {
                    concordant += 1;
                } else if sign_x * sign_y < 0.0 {
                    discordant += 1;
                }
            }
        }

        let n_pairs = (n * (n - 1)) / 2;
        Ok((concordant - discordant) as f64 / n_pairs as f64)
    }

    /// Calculate correlation with specified type
    pub fn correlate(x: &[f64], y: &[f64], cor_type: CorrelationType) -> Result<f64> {
        match cor_type {
            CorrelationType::Pearson => Self::pearson(x, y),
            CorrelationType::Spearman => Self::spearman(x, y),
            CorrelationType::Kendall => Self::kendall(x, y),
        }
    }

    /// Calculate correlation matrix for multiple variables
    pub fn correlation_matrix(data: &[Vec<f64>], cor_type: CorrelationType) -> Result<Vec<Vec<f64>>> {
        let n = data.len();
        let mut matrix = vec![vec![0.0; n]; n];

        for i in 0..n {
            matrix[i][i] = 1.0; // Diagonal is always 1

            for j in i + 1..n {
                let corr = Self::correlate(&data[i], &data[j], cor_type)?;
                matrix[i][j] = corr;
                matrix[j][i] = corr; // Symmetric
            }
        }

        Ok(matrix)
    }

    /// Test correlation significance (returns p-value approximation)
    pub fn test_significance(r: f64, n: usize) -> f64 {
        if n < 3 {
            return 1.0;
        }

        // t-statistic
        let t = r * ((n - 2) as f64).sqrt() / (1.0 - r * r).sqrt();

        // Approximate p-value using t-distribution
        // This is a simplified approximation
        let p = 2.0 * (1.0 - Self::student_t_cdf(t.abs(), n - 2));

        p.max(0.0).min(1.0)
    }

    /// Convert ranks to values
    fn rank(data: &[f64]) -> Vec<f64> {
        let mut indexed: Vec<(usize, f64)> = data.iter().copied().enumerate().collect();
        indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        let mut ranks = vec![0.0; data.len()];
        for (rank, (idx, _)) in indexed.iter().enumerate() {
            ranks[*idx] = (rank + 1) as f64;
        }

        ranks
    }

    /// Simplified Student's t CDF approximation
    fn student_t_cdf(t: f64, df: usize) -> f64 {
        // Very rough approximation for demonstration
        // In production, use a proper statistical library
        let x = df as f64 / (df as f64 + t * t);
        0.5 + 0.5 * (1.0 - x.powf(df as f64 / 2.0)) * t.signum()
    }
}

/// Autocorrelation analyzer
pub struct AutocorrelationAnalyzer;

impl AutocorrelationAnalyzer {
    /// Calculate autocorrelation at a given lag
    pub fn acf(data: &[f64], lag: usize) -> Result<f64> {
        if lag >= data.len() {
            return Err(AnalyticsError::InvalidData(
                "Lag must be less than data length".to_string(),
            ));
        }

        let mean = Statistics::mean(data);
        let n = data.len();

        let mut numerator = 0.0;
        let mut denominator = 0.0;

        for i in 0..n - lag {
            numerator += (data[i] - mean) * (data[i + lag] - mean);
        }

        for i in 0..n {
            denominator += (data[i] - mean).powi(2);
        }

        if denominator < 1e-10 {
            return Ok(0.0);
        }

        Ok(numerator / denominator)
    }

    /// Calculate autocorrelation function for multiple lags
    pub fn acf_series(data: &[f64], max_lag: usize) -> Result<Vec<f64>> {
        (0..=max_lag)
            .map(|lag| Self::acf(data, lag))
            .collect()
    }

    /// Calculate partial autocorrelation at a given lag
    pub fn pacf(data: &[f64], lag: usize) -> Result<f64> {
        if lag == 0 {
            return Ok(1.0);
        }

        if lag >= data.len() {
            return Err(AnalyticsError::InvalidData(
                "Lag must be less than data length".to_string(),
            ));
        }

        // Simplified PACF calculation using Durbin-Levinson
        let mut phi = vec![0.0; lag + 1];
        let acf_values: Vec<f64> = (0..=lag).map(|k| Self::acf(data, k).unwrap_or(0.0)).collect();

        for k in 1..=lag {
            let mut numerator = acf_values[k];
            let mut denominator = 1.0;

            for j in 1..k {
                numerator -= phi[j] * acf_values[k - j];
            }

            for j in 1..k {
                denominator -= phi[j] * acf_values[j];
            }

            phi[k] = if denominator.abs() > 1e-10 {
                numerator / denominator
            } else {
                0.0
            };

            // Update coefficients
            if k > 1 {
                let temp: Vec<f64> = phi.clone();
                for j in 1..k {
                    phi[j] = temp[j] - phi[k] * temp[k - j];
                }
            }
        }

        Ok(phi[lag])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pearson_correlation() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];

        let corr = CorrelationAnalyzer::pearson(&x, &y).unwrap();
        assert!((corr - 1.0).abs() < 0.01); // Perfect positive correlation
    }

    #[test]
    fn test_autocorrelation() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 4.0, 3.0, 2.0, 1.0];
        let acf = AutocorrelationAnalyzer::acf(&data, 1).unwrap();
        assert!(acf.abs() <= 1.0);
    }
}
