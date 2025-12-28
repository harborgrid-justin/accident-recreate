//! Regression analysis - linear and polynomial regression

use crate::error::{AnalyticsError, Result};
use ndarray::{s, Array1, Array2};
use serde::{Deserialize, Serialize};

/// Result of a regression analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionResult {
    pub coefficients: Vec<f64>,
    pub r_squared: f64,
    pub adjusted_r_squared: f64,
    pub residuals: Vec<f64>,
    pub predictions: Vec<f64>,
}

impl RegressionResult {
    /// Predict a value using the regression model
    pub fn predict(&self, x: f64) -> f64 {
        self.coefficients[0] + self.coefficients[1] * x
    }

    /// Predict multiple values
    pub fn predict_many(&self, x_values: &[f64]) -> Vec<f64> {
        x_values.iter().map(|&x| self.predict(x)).collect()
    }

    /// Get the mean absolute error
    pub fn mae(&self) -> f64 {
        if self.residuals.is_empty() {
            return 0.0;
        }
        self.residuals.iter().map(|r| r.abs()).sum::<f64>() / self.residuals.len() as f64
    }

    /// Get the root mean squared error
    pub fn rmse(&self) -> f64 {
        if self.residuals.is_empty() {
            return 0.0;
        }
        let mse = self.residuals.iter().map(|r| r.powi(2)).sum::<f64>() / self.residuals.len() as f64;
        mse.sqrt()
    }
}

/// Linear regression (y = a + bx)
pub struct LinearRegression;

impl LinearRegression {
    /// Fit a linear regression model
    pub fn fit(x: &[f64], y: &[f64]) -> Result<RegressionResult> {
        if x.len() != y.len() {
            return Err(AnalyticsError::InvalidData(
                "X and Y must have the same length".to_string(),
            ));
        }

        if x.len() < 2 {
            return Err(AnalyticsError::InsufficientData(
                "Need at least 2 data points for regression".to_string(),
            ));
        }

        let n = x.len() as f64;
        let sum_x: f64 = x.iter().sum();
        let sum_y: f64 = y.iter().sum();
        let sum_xx: f64 = x.iter().map(|xi| xi * xi).sum();
        let sum_xy: f64 = x.iter().zip(y.iter()).map(|(xi, yi)| xi * yi).sum();

        // Calculate slope and intercept
        let denominator = n * sum_xx - sum_x * sum_x;
        if denominator.abs() < 1e-10 {
            return Err(AnalyticsError::Statistical(
                "Cannot fit regression: X values are constant".to_string(),
            ));
        }

        let slope = (n * sum_xy - sum_x * sum_y) / denominator;
        let intercept = (sum_y - slope * sum_x) / n;

        let coefficients = vec![intercept, slope];

        // Calculate predictions and residuals
        let predictions: Vec<f64> = x.iter().map(|xi| intercept + slope * xi).collect();
        let residuals: Vec<f64> = y
            .iter()
            .zip(predictions.iter())
            .map(|(yi, pi)| yi - pi)
            .collect();

        // Calculate R-squared
        let y_mean = sum_y / n;
        let ss_tot: f64 = y.iter().map(|yi| (yi - y_mean).powi(2)).sum();
        let ss_res: f64 = residuals.iter().map(|r| r.powi(2)).sum();

        let r_squared = if ss_tot > 0.0 {
            1.0 - (ss_res / ss_tot)
        } else {
            0.0
        };

        // Calculate adjusted R-squared
        let k = 1.0; // number of predictors
        let adjusted_r_squared = if n > k + 1.0 {
            1.0 - (1.0 - r_squared) * (n - 1.0) / (n - k - 1.0)
        } else {
            r_squared
        };

        Ok(RegressionResult {
            coefficients,
            r_squared,
            adjusted_r_squared,
            residuals,
            predictions,
        })
    }

    /// Perform weighted linear regression
    pub fn fit_weighted(x: &[f64], y: &[f64], weights: &[f64]) -> Result<RegressionResult> {
        if x.len() != y.len() || x.len() != weights.len() {
            return Err(AnalyticsError::InvalidData(
                "X, Y, and weights must have the same length".to_string(),
            ));
        }

        let sum_w: f64 = weights.iter().sum();
        let sum_wx: f64 = weights.iter().zip(x.iter()).map(|(w, x)| w * x).sum();
        let sum_wy: f64 = weights.iter().zip(y.iter()).map(|(w, y)| w * y).sum();
        let sum_wxx: f64 = weights.iter().zip(x.iter()).map(|(w, x)| w * x * x).sum();
        let sum_wxy: f64 = weights
            .iter()
            .zip(x.iter())
            .zip(y.iter())
            .map(|((w, x), y)| w * x * y)
            .sum();

        let denominator = sum_w * sum_wxx - sum_wx * sum_wx;
        if denominator.abs() < 1e-10 {
            return Err(AnalyticsError::Statistical(
                "Cannot fit weighted regression".to_string(),
            ));
        }

        let slope = (sum_w * sum_wxy - sum_wx * sum_wy) / denominator;
        let intercept = (sum_wy - slope * sum_wx) / sum_w;

        let coefficients = vec![intercept, slope];

        let predictions: Vec<f64> = x.iter().map(|xi| intercept + slope * xi).collect();
        let residuals: Vec<f64> = y
            .iter()
            .zip(predictions.iter())
            .map(|(yi, pi)| yi - pi)
            .collect();

        // Calculate weighted R-squared
        let y_mean = sum_wy / sum_w;
        let ss_tot: f64 = weights
            .iter()
            .zip(y.iter())
            .map(|(w, yi)| w * (yi - y_mean).powi(2))
            .sum();
        let ss_res: f64 = weights
            .iter()
            .zip(residuals.iter())
            .map(|(w, r)| w * r.powi(2))
            .sum();

        let r_squared = if ss_tot > 0.0 {
            1.0 - (ss_res / ss_tot)
        } else {
            0.0
        };

        Ok(RegressionResult {
            coefficients,
            r_squared,
            adjusted_r_squared: r_squared,
            residuals,
            predictions,
        })
    }
}

/// Polynomial regression
pub struct PolynomialRegression;

impl PolynomialRegression {
    /// Fit a polynomial regression model of given degree
    pub fn fit(x: &[f64], y: &[f64], degree: usize) -> Result<PolynomialResult> {
        if x.len() != y.len() {
            return Err(AnalyticsError::InvalidData(
                "X and Y must have the same length".to_string(),
            ));
        }

        if x.len() < degree + 1 {
            return Err(AnalyticsError::InsufficientData(format!(
                "Need at least {} data points for degree {} polynomial",
                degree + 1,
                degree
            )));
        }

        let n = x.len();

        // Build design matrix X
        let mut x_matrix = Array2::<f64>::zeros((n, degree + 1));
        for i in 0..n {
            for j in 0..=degree {
                x_matrix[[i, j]] = x[i].powi(j as i32);
            }
        }

        let y_vector = Array1::from_vec(y.to_vec());

        // Solve normal equations: (X'X)β = X'y
        let xt = x_matrix.t();
        let xtx = xt.dot(&x_matrix);
        let xty = xt.dot(&y_vector);

        // Simple Gaussian elimination for solving the system
        let coefficients = Self::solve_linear_system(&xtx, &xty)?;

        // Calculate predictions and residuals
        let predictions: Vec<f64> = x
            .iter()
            .map(|&xi| {
                coefficients
                    .iter()
                    .enumerate()
                    .map(|(i, &coef)| coef * xi.powi(i as i32))
                    .sum()
            })
            .collect();

        let residuals: Vec<f64> = y
            .iter()
            .zip(predictions.iter())
            .map(|(yi, pi)| yi - pi)
            .collect();

        // Calculate R-squared
        let y_mean = y.iter().sum::<f64>() / y.len() as f64;
        let ss_tot: f64 = y.iter().map(|yi| (yi - y_mean).powi(2)).sum();
        let ss_res: f64 = residuals.iter().map(|r| r.powi(2)).sum();

        let r_squared = if ss_tot > 0.0 {
            1.0 - (ss_res / ss_tot)
        } else {
            0.0
        };

        let k = degree as f64;
        let n = y.len() as f64;
        let adjusted_r_squared = if n > k + 1.0 {
            1.0 - (1.0 - r_squared) * (n - 1.0) / (n - k - 1.0)
        } else {
            r_squared
        };

        Ok(PolynomialResult {
            degree,
            coefficients,
            r_squared,
            adjusted_r_squared,
            residuals,
            predictions,
        })
    }

    fn solve_linear_system(a: &Array2<f64>, b: &Array1<f64>) -> Result<Vec<f64>> {
        let n = a.nrows();
        let mut a_aug = a.clone();
        let mut b_vec = b.clone();

        // Gaussian elimination with partial pivoting
        for k in 0..n {
            // Find pivot
            let mut max_row = k;
            let mut max_val = a_aug[[k, k]].abs();

            for i in k + 1..n {
                let val = a_aug[[i, k]].abs();
                if val > max_val {
                    max_val = val;
                    max_row = i;
                }
            }

            if max_val < 1e-10 {
                return Err(AnalyticsError::Statistical(
                    "Singular matrix in polynomial regression".to_string(),
                ));
            }

            // Swap rows
            if max_row != k {
                for j in 0..n {
                    let temp = a_aug[[k, j]];
                    a_aug[[k, j]] = a_aug[[max_row, j]];
                    a_aug[[max_row, j]] = temp;
                }
                let temp = b_vec[k];
                b_vec[k] = b_vec[max_row];
                b_vec[max_row] = temp;
            }

            // Eliminate
            for i in k + 1..n {
                let factor = a_aug[[i, k]] / a_aug[[k, k]];
                for j in k..n {
                    a_aug[[i, j]] -= factor * a_aug[[k, j]];
                }
                b_vec[i] -= factor * b_vec[k];
            }
        }

        // Back substitution
        let mut x = vec![0.0; n];
        for i in (0..n).rev() {
            let mut sum = b_vec[i];
            for j in i + 1..n {
                sum -= a_aug[[i, j]] * x[j];
            }
            x[i] = sum / a_aug[[i, i]];
        }

        Ok(x)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolynomialResult {
    pub degree: usize,
    pub coefficients: Vec<f64>,
    pub r_squared: f64,
    pub adjusted_r_squared: f64,
    pub residuals: Vec<f64>,
    pub predictions: Vec<f64>,
}

impl PolynomialResult {
    /// Predict a value using the polynomial model
    pub fn predict(&self, x: f64) -> f64 {
        self.coefficients
            .iter()
            .enumerate()
            .map(|(i, &coef)| coef * x.powi(i as i32))
            .sum()
    }

    /// Predict multiple values
    pub fn predict_many(&self, x_values: &[f64]) -> Vec<f64> {
        x_values.iter().map(|&x| self.predict(x)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_regression() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];

        let result = LinearRegression::fit(&x, &y).unwrap();

        assert!((result.coefficients[0] - 0.0).abs() < 0.1); // intercept ≈ 0
        assert!((result.coefficients[1] - 2.0).abs() < 0.1); // slope ≈ 2
        assert!(result.r_squared > 0.99);
    }

    #[test]
    fn test_polynomial_regression() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![1.0, 4.0, 9.0, 16.0, 25.0]; // y = x^2

        let result = PolynomialRegression::fit(&x, &y, 2).unwrap();

        assert_eq!(result.degree, 2);
        assert!(result.r_squared > 0.99);
    }
}
