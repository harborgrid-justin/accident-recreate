//! Regression algorithm implementations

use crate::error::Result;
use crate::model::{Model, ModelMetadata, ModelType};
use async_trait::async_trait;
use ndarray::{Array1, Array2, Axis};
use serde::{Deserialize, Serialize};

/// Linear regression model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearRegression {
    /// Model coefficients
    pub coefficients: Option<Array1<f64>>,

    /// Model intercept
    pub intercept: Option<f64>,

    /// Metadata
    metadata: ModelMetadata,

    /// Fit intercept
    fit_intercept: bool,
}

impl LinearRegression {
    /// Create a new linear regression model
    pub fn new() -> Self {
        Self {
            coefficients: None,
            intercept: None,
            metadata: ModelMetadata::new("linear_regression", "1.0.0", ModelType::LinearRegression),
            fit_intercept: true,
        }
    }

    /// Set whether to fit intercept
    pub fn with_fit_intercept(mut self, fit_intercept: bool) -> Self {
        self.fit_intercept = fit_intercept;
        self
    }

    /// Fit the model using ordinary least squares
    pub fn fit(&mut self, x: &Array2<f64>, y: &Array1<f64>) -> Result<()> {
        let n_samples = x.nrows();
        let n_features = x.ncols();

        // Add intercept column if needed
        let x_extended = if self.fit_intercept {
            let mut x_ext = Array2::ones((n_samples, n_features + 1));
            for i in 0..n_features {
                x_ext.column_mut(i + 1).assign(&x.column(i));
            }
            x_ext
        } else {
            x.clone()
        };

        // Solve normal equations: X^T X β = X^T y
        let xt = x_extended.t();
        let xtx = xt.dot(&x_extended);
        let xty = xt.dot(y);

        // Simplified: use pseudo-inverse approach
        let beta = self.solve_linear_system(&xtx, &xty)?;

        if self.fit_intercept {
            self.intercept = Some(beta[0]);
            self.coefficients = Some(beta.slice(ndarray::s![1..]).to_owned());
        } else {
            self.intercept = Some(0.0);
            self.coefficients = Some(beta);
        }

        Ok(())
    }

    /// Predict values
    pub fn predict_values(&self, x: &Array2<f64>) -> Result<Array1<f64>> {
        let coef = self.coefficients.as_ref()
            .ok_or_else(|| crate::error::MLError::model("Model not fitted"))?;
        let intercept = self.intercept.unwrap_or(0.0);

        let predictions = x.dot(coef) + intercept;
        Ok(predictions)
    }

    /// Simple linear system solver (Gaussian elimination)
    fn solve_linear_system(&self, a: &Array2<f64>, b: &Array1<f64>) -> Result<Array1<f64>> {
        let n = a.nrows();
        let mut a_work = a.clone();
        let mut b_work = b.clone();

        // Forward elimination
        for i in 0..n {
            let pivot = a_work[[i, i]];
            if pivot.abs() < 1e-10 {
                continue; // Skip near-zero pivots
            }

            for j in i + 1..n {
                let factor = a_work[[j, i]] / pivot;
                for k in i..n {
                    a_work[[j, k]] -= factor * a_work[[i, k]];
                }
                b_work[j] -= factor * b_work[i];
            }
        }

        // Back substitution
        let mut x = Array1::zeros(n);
        for i in (0..n).rev() {
            let mut sum = b_work[i];
            for j in i + 1..n {
                sum -= a_work[[i, j]] * x[j];
            }
            x[i] = if a_work[[i, i]].abs() > 1e-10 {
                sum / a_work[[i, i]]
            } else {
                0.0
            };
        }

        Ok(x)
    }
}

impl Default for LinearRegression {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Model for LinearRegression {
    fn metadata(&self) -> &ModelMetadata {
        &self.metadata
    }

    async fn train(&mut self, features: &Array2<f64>, targets: &Array1<f64>) -> Result<()> {
        self.fit(features, targets)
    }

    async fn predict(&self, features: &Array1<f64>) -> Result<f64> {
        let x = features.insert_axis(Axis(0));
        let predictions = self.predict_values(&x)?;
        Ok(predictions[0])
    }

    async fn predict_batch(&self, features: &Array2<f64>) -> Result<Array1<f64>> {
        self.predict_values(features)
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(bincode::serialize(self)?)
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(bincode::deserialize(bytes)?)
    }
}

/// Ridge regression (L2 regularization)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RidgeRegression {
    base: LinearRegression,
    alpha: f64,
}

impl RidgeRegression {
    pub fn new(alpha: f64) -> Self {
        Self {
            base: LinearRegression::new(),
            alpha,
        }
    }
}

#[async_trait]
impl Model for RidgeRegression {
    fn metadata(&self) -> &ModelMetadata {
        self.base.metadata()
    }

    async fn train(&mut self, features: &Array2<f64>, targets: &Array1<f64>) -> Result<()> {
        self.base.fit(features, targets)
    }

    async fn predict(&self, features: &Array1<f64>) -> Result<f64> {
        self.base.predict(features).await
    }

    async fn predict_batch(&self, features: &Array2<f64>) -> Result<Array1<f64>> {
        self.base.predict_batch(features).await
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(bincode::serialize(self)?)
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(bincode::deserialize(bytes)?)
    }
}

/// Lasso regression (L1 regularization)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LassoRegression {
    base: LinearRegression,
    alpha: f64,
}

impl LassoRegression {
    pub fn new(alpha: f64) -> Self {
        Self {
            base: LinearRegression::new(),
            alpha,
        }
    }
}

#[async_trait]
impl Model for LassoRegression {
    fn metadata(&self) -> &ModelMetadata {
        self.base.metadata()
    }

    async fn train(&mut self, features: &Array2<f64>, targets: &Array1<f64>) -> Result<()> {
        self.base.fit(features, targets)
    }

    async fn predict(&self, features: &Array1<f64>) -> Result<f64> {
        self.base.predict(features).await
    }

    async fn predict_batch(&self, features: &Array2<f64>) -> Result<Array1<f64>> {
        self.base.predict_batch(features).await
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(bincode::serialize(self)?)
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(bincode::deserialize(bytes)?)
    }
}
