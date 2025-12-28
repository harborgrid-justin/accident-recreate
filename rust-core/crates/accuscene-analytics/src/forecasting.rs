//! Time series forecasting algorithms

use crate::error::{AnalyticsError, Result};
use crate::statistics::descriptive::Statistics;
use serde::{Deserialize, Serialize};

/// Forecast result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Forecast {
    pub values: Vec<f64>,
    pub lower_bound: Vec<f64>,
    pub upper_bound: Vec<f64>,
    pub confidence: f64,
}

/// Simple moving average forecaster
pub struct MovingAverageForecaster {
    window_size: usize,
}

impl MovingAverageForecaster {
    pub fn new(window_size: usize) -> Self {
        Self { window_size }
    }

    /// Forecast the next n values
    pub fn forecast(&self, data: &[f64], n: usize) -> Result<Forecast> {
        if data.len() < self.window_size {
            return Err(AnalyticsError::InsufficientData(format!(
                "Need at least {} data points",
                self.window_size
            )));
        }

        // Use the last window_size points to calculate the average
        let window = &data[data.len() - self.window_size..];
        let mean = Statistics::mean(window);
        let std_dev = Statistics::std_dev(window);

        // Forecast is constant (mean of last window)
        let values = vec![mean; n];
        let lower_bound = vec![mean - 1.96 * std_dev; n];
        let upper_bound = vec![mean + 1.96 * std_dev; n];

        Ok(Forecast {
            values,
            lower_bound,
            upper_bound,
            confidence: 0.95,
        })
    }
}

/// Exponential smoothing forecaster
pub struct ExponentialSmoothingForecaster {
    alpha: f64, // Smoothing parameter
}

impl ExponentialSmoothingForecaster {
    pub fn new(alpha: f64) -> Result<Self> {
        if alpha < 0.0 || alpha > 1.0 {
            return Err(AnalyticsError::Configuration(
                "Alpha must be between 0 and 1".to_string(),
            ));
        }

        Ok(Self { alpha })
    }

    /// Forecast the next n values using simple exponential smoothing
    pub fn forecast(&self, data: &[f64], n: usize) -> Result<Forecast> {
        if data.is_empty() {
            return Err(AnalyticsError::InsufficientData(
                "Cannot forecast on empty data".to_string(),
            ));
        }

        // Calculate smoothed value
        let mut smoothed = data[0];
        for &value in data.iter().skip(1) {
            smoothed = self.alpha * value + (1.0 - self.alpha) * smoothed;
        }

        // Calculate residuals for confidence interval
        let mut residuals = Vec::new();
        let mut s = data[0];
        for &value in data.iter().skip(1) {
            residuals.push(value - s);
            s = self.alpha * value + (1.0 - self.alpha) * s;
        }

        let residual_std = if residuals.is_empty() {
            0.0
        } else {
            Statistics::std_dev(&residuals)
        };

        // Forecast is constant
        let values = vec![smoothed; n];
        let lower_bound = vec![smoothed - 1.96 * residual_std; n];
        let upper_bound = vec![smoothed + 1.96 * residual_std; n];

        Ok(Forecast {
            values,
            lower_bound,
            upper_bound,
            confidence: 0.95,
        })
    }
}

/// Holt's linear trend forecaster
pub struct HoltForecaster {
    alpha: f64, // Level smoothing
    beta: f64,  // Trend smoothing
}

impl HoltForecaster {
    pub fn new(alpha: f64, beta: f64) -> Result<Self> {
        if alpha < 0.0 || alpha > 1.0 || beta < 0.0 || beta > 1.0 {
            return Err(AnalyticsError::Configuration(
                "Alpha and beta must be between 0 and 1".to_string(),
            ));
        }

        Ok(Self { alpha, beta })
    }

    /// Forecast using Holt's linear trend method
    pub fn forecast(&self, data: &[f64], n: usize) -> Result<Forecast> {
        if data.len() < 2 {
            return Err(AnalyticsError::InsufficientData(
                "Need at least 2 data points for Holt's method".to_string(),
            ));
        }

        // Initialize level and trend
        let mut level = data[0];
        let mut trend = data[1] - data[0];

        let mut residuals = Vec::new();

        // Update level and trend for each observation
        for i in 1..data.len() {
            let prev_level = level;
            let prev_trend = trend;

            level = self.alpha * data[i] + (1.0 - self.alpha) * (prev_level + prev_trend);
            trend = self.beta * (level - prev_level) + (1.0 - self.beta) * prev_trend;

            // Calculate residual
            let predicted = prev_level + prev_trend;
            residuals.push(data[i] - predicted);
        }

        let residual_std = Statistics::std_dev(&residuals);

        // Generate forecasts
        let mut values = Vec::new();
        let mut lower_bound = Vec::new();
        let mut upper_bound = Vec::new();

        for h in 1..=n {
            let forecast = level + (h as f64) * trend;
            let margin = 1.96 * residual_std * (h as f64).sqrt();

            values.push(forecast);
            lower_bound.push(forecast - margin);
            upper_bound.push(forecast + margin);
        }

        Ok(Forecast {
            values,
            lower_bound,
            upper_bound,
            confidence: 0.95,
        })
    }
}

/// ARIMA-like forecaster (simplified AR model)
pub struct ARForecaster {
    order: usize, // AR order (p)
}

impl ARForecaster {
    pub fn new(order: usize) -> Self {
        Self { order }
    }

    /// Forecast using autoregressive model
    pub fn forecast(&self, data: &[f64], n: usize) -> Result<Forecast> {
        if data.len() < self.order + 1 {
            return Err(AnalyticsError::InsufficientData(format!(
                "Need at least {} data points for AR({})",
                self.order + 1,
                self.order
            )));
        }

        // Estimate AR coefficients using Yule-Walker equations
        let coefficients = self.estimate_coefficients(data)?;

        // Generate forecasts
        let mut extended_data = data.to_vec();
        let mut residuals = Vec::new();

        // Calculate in-sample residuals
        for i in self.order..data.len() {
            let mut predicted = 0.0;
            for j in 0..self.order {
                predicted += coefficients[j] * data[i - j - 1];
            }
            residuals.push(data[i] - predicted);
        }

        let residual_std = Statistics::std_dev(&residuals);

        let mut values = Vec::new();
        let mut lower_bound = Vec::new();
        let mut upper_bound = Vec::new();

        // Generate out-of-sample forecasts
        for h in 1..=n {
            let mut forecast = 0.0;
            for j in 0..self.order {
                let idx = extended_data.len() - j - 1;
                forecast += coefficients[j] * extended_data[idx];
            }

            extended_data.push(forecast);

            let margin = 1.96 * residual_std * (h as f64).sqrt();
            values.push(forecast);
            lower_bound.push(forecast - margin);
            upper_bound.push(forecast + margin);
        }

        Ok(Forecast {
            values,
            lower_bound,
            upper_bound,
            confidence: 0.95,
        })
    }

    fn estimate_coefficients(&self, data: &[f64]) -> Result<Vec<f64>> {
        // Simple OLS estimation for AR coefficients
        let n = data.len() - self.order;
        let mut x_matrix = vec![vec![0.0; self.order]; n];
        let mut y_vector = vec![0.0; n];

        for i in 0..n {
            for j in 0..self.order {
                x_matrix[i][j] = data[self.order + i - j - 1];
            }
            y_vector[i] = data[self.order + i];
        }

        // Solve normal equations: (X'X)β = X'y
        let mut xtx = vec![vec![0.0; self.order]; self.order];
        let mut xty = vec![0.0; self.order];

        for i in 0..self.order {
            for j in 0..self.order {
                for k in 0..n {
                    xtx[i][j] += x_matrix[k][i] * x_matrix[k][j];
                }
            }
            for k in 0..n {
                xty[i] += x_matrix[k][i] * y_vector[k];
            }
        }

        // Solve using simple Gaussian elimination
        self.solve_linear_system(&xtx, &xty)
    }

    fn solve_linear_system(&self, a: &[Vec<f64>], b: &[f64]) -> Result<Vec<f64>> {
        let n = a.len();
        let mut a_aug = a.to_vec();
        let mut b_vec = b.to_vec();

        // Gaussian elimination
        for k in 0..n {
            // Find pivot
            let mut max_row = k;
            let mut max_val = a_aug[k][k].abs();

            for i in k + 1..n {
                let val = a_aug[i][k].abs();
                if val > max_val {
                    max_val = val;
                    max_row = i;
                }
            }

            if max_val < 1e-10 {
                return Err(AnalyticsError::Statistical(
                    "Singular matrix in AR estimation".to_string(),
                ));
            }

            // Swap rows
            if max_row != k {
                a_aug.swap(k, max_row);
                b_vec.swap(k, max_row);
            }

            // Eliminate
            for i in k + 1..n {
                let factor = a_aug[i][k] / a_aug[k][k];
                for j in k..n {
                    a_aug[i][j] -= factor * a_aug[k][j];
                }
                b_vec[i] -= factor * b_vec[k];
            }
        }

        // Back substitution
        let mut x = vec![0.0; n];
        for i in (0..n).rev() {
            let mut sum = b_vec[i];
            for j in i + 1..n {
                sum -= a_aug[i][j] * x[j];
            }
            x[i] = sum / a_aug[i][i];
        }

        Ok(x)
    }
}

/// Seasonal decomposition forecaster
pub struct SeasonalForecaster {
    period: usize,
    method: SeasonalMethod,
}

#[derive(Debug, Clone, Copy)]
pub enum SeasonalMethod {
    Additive,
    Multiplicative,
}

impl SeasonalForecaster {
    pub fn new(period: usize, method: SeasonalMethod) -> Self {
        Self { period, method }
    }

    /// Forecast with seasonal decomposition
    pub fn forecast(&self, data: &[f64], n: usize) -> Result<Forecast> {
        if data.len() < 2 * self.period {
            return Err(AnalyticsError::InsufficientData(format!(
                "Need at least {} data points for seasonal forecasting",
                2 * self.period
            )));
        }

        // Decompose into trend and seasonal components
        let (trend, seasonal) = self.decompose(data)?;

        // Forecast trend (simple linear extrapolation)
        let trend_forecast = self.forecast_trend(&trend, n);

        // Repeat seasonal pattern
        let seasonal_forecast = self.forecast_seasonal(&seasonal, n);

        // Combine trend and seasonal
        let values: Vec<f64> = trend_forecast
            .iter()
            .zip(seasonal_forecast.iter())
            .map(|(&t, &s)| match self.method {
                SeasonalMethod::Additive => t + s,
                SeasonalMethod::Multiplicative => t * s,
            })
            .collect();

        // Estimate confidence intervals from residuals
        let residuals = self.calculate_residuals(data, &trend, &seasonal);
        let residual_std = Statistics::std_dev(&residuals);

        let lower_bound: Vec<f64> = values.iter().map(|&v| v - 1.96 * residual_std).collect();
        let upper_bound: Vec<f64> = values.iter().map(|&v| v + 1.96 * residual_std).collect();

        Ok(Forecast {
            values,
            lower_bound,
            upper_bound,
            confidence: 0.95,
        })
    }

    fn decompose(&self, data: &[f64]) -> Result<(Vec<f64>, Vec<f64>)> {
        // Calculate seasonal averages
        let mut seasonal = vec![0.0; self.period];
        let num_cycles = data.len() / self.period;

        for i in 0..self.period {
            let mut sum = 0.0;
            for j in 0..num_cycles {
                sum += data[j * self.period + i];
            }
            seasonal[i] = sum / num_cycles as f64;
        }

        // Deseasonalize
        let deseasonalized: Vec<f64> = data
            .iter()
            .enumerate()
            .map(|(i, &value)| {
                let s = seasonal[i % self.period];
                match self.method {
                    SeasonalMethod::Additive => value - s,
                    SeasonalMethod::Multiplicative => {
                        if s != 0.0 {
                            value / s
                        } else {
                            value
                        }
                    }
                }
            })
            .collect();

        Ok((deseasonalized, seasonal))
    }

    fn forecast_trend(&self, trend: &[f64], n: usize) -> Vec<f64> {
        // Simple linear extrapolation
        let len = trend.len();
        if len < 2 {
            return vec![trend.last().copied().unwrap_or(0.0); n];
        }

        let last = trend[len - 1];
        let slope = (trend[len - 1] - trend[len - 2]) * 0.1; // Dampened

        (1..=n).map(|h| last + slope * h as f64).collect()
    }

    fn forecast_seasonal(&self, seasonal: &[f64], n: usize) -> Vec<f64> {
        (0..n)
            .map(|i| seasonal[i % self.period])
            .collect()
    }

    fn calculate_residuals(&self, data: &[f64], trend: &[f64], seasonal: &[f64]) -> Vec<f64> {
        data.iter()
            .enumerate()
            .take(trend.len())
            .map(|(i, &value)| {
                let t = trend[i];
                let s = seasonal[i % self.period];
                match self.method {
                    SeasonalMethod::Additive => value - t - s,
                    SeasonalMethod::Multiplicative => value - t * s,
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_moving_average_forecaster() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let forecaster = MovingAverageForecaster::new(3);

        let forecast = forecaster.forecast(&data, 3).unwrap();
        assert_eq!(forecast.values.len(), 3);
    }

    #[test]
    fn test_exponential_smoothing() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let forecaster = ExponentialSmoothingForecaster::new(0.3).unwrap();

        let forecast = forecaster.forecast(&data, 3).unwrap();
        assert_eq!(forecast.values.len(), 3);
    }

    #[test]
    fn test_holt_forecaster() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let forecaster = HoltForecaster::new(0.3, 0.1).unwrap();

        let forecast = forecaster.forecast(&data, 3).unwrap();
        assert_eq!(forecast.values.len(), 3);
    }
}
