//! Distribution fitting and analysis

use crate::error::{AnalyticsError, Result};
use crate::statistics::descriptive::Statistics;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistributionType {
    Normal,
    Exponential,
    Uniform,
    Poisson,
}

/// Distribution fitter
pub struct DistributionFitter;

impl DistributionFitter {
    /// Fit a normal distribution to data
    pub fn fit_normal(data: &[f64]) -> Result<NormalDistribution> {
        if data.is_empty() {
            return Err(AnalyticsError::InsufficientData(
                "Cannot fit distribution to empty data".to_string(),
            ));
        }

        let mean = Statistics::mean(data);
        let std_dev = Statistics::std_dev(data);

        Ok(NormalDistribution { mean, std_dev })
    }

    /// Fit an exponential distribution to data
    pub fn fit_exponential(data: &[f64]) -> Result<ExponentialDistribution> {
        if data.is_empty() {
            return Err(AnalyticsError::InsufficientData(
                "Cannot fit distribution to empty data".to_string(),
            ));
        }

        let mean = Statistics::mean(data);
        if mean <= 0.0 {
            return Err(AnalyticsError::InvalidData(
                "Exponential distribution requires positive values".to_string(),
            ));
        }

        let lambda = 1.0 / mean;

        Ok(ExponentialDistribution { lambda })
    }

    /// Fit a uniform distribution to data
    pub fn fit_uniform(data: &[f64]) -> Result<UniformDistribution> {
        if data.is_empty() {
            return Err(AnalyticsError::InsufficientData(
                "Cannot fit distribution to empty data".to_string(),
            ));
        }

        let min = data.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        Ok(UniformDistribution { min, max })
    }

    /// Determine the best-fitting distribution using goodness-of-fit tests
    pub fn auto_fit(data: &[f64]) -> Result<BestFitDistribution> {
        if data.is_empty() {
            return Err(AnalyticsError::InsufficientData(
                "Cannot fit distribution to empty data".to_string(),
            ));
        }

        let normal = Self::fit_normal(data)?;
        let exponential = Self::fit_exponential(data).ok();
        let uniform = Self::fit_uniform(data)?;

        // Calculate goodness of fit for each distribution
        let normal_gof = Self::kolmogorov_smirnov(data, &normal);
        let exponential_gof = exponential.as_ref().map(|d| Self::kolmogorov_smirnov_exp(data, d)).unwrap_or(f64::INFINITY);
        let uniform_gof = Self::kolmogorov_smirnov_uniform(data, &uniform);

        // Choose the distribution with the smallest KS statistic
        let (dist_type, gof_statistic) = if normal_gof <= exponential_gof && normal_gof <= uniform_gof {
            (DistributionType::Normal, normal_gof)
        } else if exponential_gof <= uniform_gof {
            (DistributionType::Exponential, exponential_gof)
        } else {
            (DistributionType::Uniform, uniform_gof)
        };

        Ok(BestFitDistribution {
            distribution_type: dist_type,
            gof_statistic,
            normal: Some(normal),
            exponential,
            uniform: Some(uniform),
        })
    }

    /// Kolmogorov-Smirnov test for normal distribution
    fn kolmogorov_smirnov(data: &[f64], dist: &NormalDistribution) -> f64 {
        let mut sorted = data.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let n = sorted.len() as f64;
        let mut max_diff = 0.0;

        for (i, &value) in sorted.iter().enumerate() {
            let empirical_cdf = (i + 1) as f64 / n;
            let theoretical_cdf = dist.cdf(value);
            let diff = (empirical_cdf - theoretical_cdf).abs();
            max_diff = max_diff.max(diff);
        }

        max_diff
    }

    /// KS test for exponential distribution
    fn kolmogorov_smirnov_exp(data: &[f64], dist: &ExponentialDistribution) -> f64 {
        let mut sorted = data.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let n = sorted.len() as f64;
        let mut max_diff = 0.0;

        for (i, &value) in sorted.iter().enumerate() {
            if value < 0.0 {
                continue;
            }
            let empirical_cdf = (i + 1) as f64 / n;
            let theoretical_cdf = dist.cdf(value);
            let diff = (empirical_cdf - theoretical_cdf).abs();
            max_diff = max_diff.max(diff);
        }

        max_diff
    }

    /// KS test for uniform distribution
    fn kolmogorov_smirnov_uniform(data: &[f64], dist: &UniformDistribution) -> f64 {
        let mut sorted = data.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let n = sorted.len() as f64;
        let mut max_diff = 0.0;

        for (i, &value) in sorted.iter().enumerate() {
            let empirical_cdf = (i + 1) as f64 / n;
            let theoretical_cdf = dist.cdf(value);
            let diff = (empirical_cdf - theoretical_cdf).abs();
            max_diff = max_diff.max(diff);
        }

        max_diff
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalDistribution {
    pub mean: f64,
    pub std_dev: f64,
}

impl NormalDistribution {
    /// Calculate PDF at a given point
    pub fn pdf(&self, x: f64) -> f64 {
        let variance = self.std_dev * self.std_dev;
        let coefficient = 1.0 / (2.0 * std::f64::consts::PI * variance).sqrt();
        let exponent = -((x - self.mean).powi(2)) / (2.0 * variance);
        coefficient * exponent.exp()
    }

    /// Calculate CDF at a given point (approximation)
    pub fn cdf(&self, x: f64) -> f64 {
        let z = (x - self.mean) / self.std_dev;
        Self::standard_normal_cdf(z)
    }

    /// Standard normal CDF approximation
    fn standard_normal_cdf(z: f64) -> f64 {
        // Abramowitz and Stegun approximation
        let t = 1.0 / (1.0 + 0.2316419 * z.abs());
        let d = 0.3989423 * (-z * z / 2.0).exp();
        let prob = d * t * (0.3193815 + t * (-0.3565638 + t * (1.781478 + t * (-1.821256 + t * 1.330274))));

        if z > 0.0 {
            1.0 - prob
        } else {
            prob
        }
    }

    /// Generate a sample from the distribution (Box-Muller transform)
    pub fn sample(&self) -> f64 {
        let u1: f64 = rand::random();
        let u2: f64 = rand::random();

        let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
        self.mean + self.std_dev * z
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExponentialDistribution {
    pub lambda: f64,
}

impl ExponentialDistribution {
    pub fn pdf(&self, x: f64) -> f64 {
        if x < 0.0 {
            0.0
        } else {
            self.lambda * (-self.lambda * x).exp()
        }
    }

    pub fn cdf(&self, x: f64) -> f64 {
        if x < 0.0 {
            0.0
        } else {
            1.0 - (-self.lambda * x).exp()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniformDistribution {
    pub min: f64,
    pub max: f64,
}

impl UniformDistribution {
    pub fn pdf(&self, x: f64) -> f64 {
        if x >= self.min && x <= self.max {
            1.0 / (self.max - self.min)
        } else {
            0.0
        }
    }

    pub fn cdf(&self, x: f64) -> f64 {
        if x < self.min {
            0.0
        } else if x > self.max {
            1.0
        } else {
            (x - self.min) / (self.max - self.min)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BestFitDistribution {
    pub distribution_type: DistributionType,
    pub gof_statistic: f64,
    pub normal: Option<NormalDistribution>,
    pub exponential: Option<ExponentialDistribution>,
    pub uniform: Option<UniformDistribution>,
}

// Simple random number generation workaround
mod rand {
    use std::cell::Cell;

    thread_local! {
        static SEED: Cell<u64> = Cell::new(0x123456789abcdef0);
    }

    pub fn random() -> f64 {
        SEED.with(|seed| {
            let mut s = seed.get();
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            seed.set(s);
            (s as f64) / (u64::MAX as f64)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_distribution() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let dist = DistributionFitter::fit_normal(&data).unwrap();

        assert_eq!(dist.mean, 3.0);
        assert!(dist.std_dev > 0.0);
    }

    #[test]
    fn test_normal_pdf() {
        let dist = NormalDistribution {
            mean: 0.0,
            std_dev: 1.0,
        };

        let pdf_at_mean = dist.pdf(0.0);
        assert!(pdf_at_mean > 0.0);
    }
}
