//! Feature transformation implementations

use crate::error::Result;
use ndarray::{Array1, Array2};
use serde::{Deserialize, Serialize};

/// Trait for feature transformers
pub trait FeatureTransformer: Send + Sync {
    /// Transform input features
    fn transform(&self, data: &Array2<f64>) -> Result<Array2<f64>>;

    /// Inverse transform (if applicable)
    fn inverse_transform(&self, data: &Array2<f64>) -> Result<Array2<f64>>;

    /// Fit the transformer (if needed)
    fn fit(&mut self, data: &Array2<f64>) -> Result<()> {
        let _ = data;
        Ok(())
    }

    /// Fit and transform in one step
    fn fit_transform(&mut self, data: &Array2<f64>) -> Result<Array2<f64>> {
        self.fit(data)?;
        self.transform(data)
    }
}

/// Logarithmic transformation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogTransform {
    /// Base of logarithm (natural log if None)
    base: Option<f64>,

    /// Offset to add before taking log (to handle zeros)
    offset: f64,
}

impl LogTransform {
    /// Create a new log transform (natural log)
    pub fn new() -> Self {
        Self {
            base: None,
            offset: 1.0,
        }
    }

    /// Create log transform with specific base
    pub fn with_base(base: f64) -> Self {
        Self {
            base: Some(base),
            offset: 1.0,
        }
    }

    /// Set offset value
    pub fn with_offset(mut self, offset: f64) -> Self {
        self.offset = offset;
        self
    }

    /// Create log10 transform
    pub fn log10() -> Self {
        Self::with_base(10.0)
    }

    /// Create log2 transform
    pub fn log2() -> Self {
        Self::with_base(2.0)
    }
}

impl Default for LogTransform {
    fn default() -> Self {
        Self::new()
    }
}

impl FeatureTransformer for LogTransform {
    fn transform(&self, data: &Array2<f64>) -> Result<Array2<f64>> {
        let mut result = data.clone();

        for val in result.iter_mut() {
            let transformed = match self.base {
                None => (*val + self.offset).ln(),
                Some(base) => (*val + self.offset).log(base),
            };
            *val = transformed;
        }

        Ok(result)
    }

    fn inverse_transform(&self, data: &Array2<f64>) -> Result<Array2<f64>> {
        let mut result = data.clone();

        for val in result.iter_mut() {
            let original = match self.base {
                None => val.exp(),
                Some(base) => base.powf(*val),
            };
            *val = original - self.offset;
        }

        Ok(result)
    }
}

/// Power transformation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerTransform {
    /// Power/exponent value
    power: f64,

    /// Fitted lambda value for Box-Cox (if applicable)
    lambda: Option<f64>,

    /// Method: BoxCox or YeoJohnson
    method: PowerMethod,
}

impl PowerTransform {
    /// Create a new power transform with specified power
    pub fn new(power: f64) -> Self {
        Self {
            power,
            lambda: None,
            method: PowerMethod::Simple,
        }
    }

    /// Create Box-Cox transform
    pub fn box_cox() -> Self {
        Self {
            power: 1.0,
            lambda: None,
            method: PowerMethod::BoxCox,
        }
    }

    /// Create Yeo-Johnson transform
    pub fn yeo_johnson() -> Self {
        Self {
            power: 1.0,
            lambda: None,
            method: PowerMethod::YeoJohnson,
        }
    }

    /// Create square root transform
    pub fn sqrt() -> Self {
        Self::new(0.5)
    }

    /// Create square transform
    pub fn square() -> Self {
        Self::new(2.0)
    }
}

impl FeatureTransformer for PowerTransform {
    fn fit(&mut self, data: &Array2<f64>) -> Result<()> {
        match self.method {
            PowerMethod::BoxCox | PowerMethod::YeoJohnson => {
                // Simplified: use fixed lambda
                // In production, estimate optimal lambda
                self.lambda = Some(0.0); // log transform
            }
            PowerMethod::Simple => {}
        }
        Ok(())
    }

    fn transform(&self, data: &Array2<f64>) -> Result<Array2<f64>> {
        let mut result = data.clone();

        match self.method {
            PowerMethod::Simple => {
                for val in result.iter_mut() {
                    *val = val.powf(self.power);
                }
            }
            PowerMethod::BoxCox => {
                let lambda = self.lambda.unwrap_or(0.0);
                for val in result.iter_mut() {
                    if *val <= 0.0 {
                        *val = 0.0; // Box-Cox requires positive values
                    } else if lambda.abs() < 1e-10 {
                        *val = val.ln();
                    } else {
                        *val = (val.powf(lambda) - 1.0) / lambda;
                    }
                }
            }
            PowerMethod::YeoJohnson => {
                let lambda = self.lambda.unwrap_or(0.0);
                for val in result.iter_mut() {
                    *val = self.yeo_johnson_transform(*val, lambda);
                }
            }
        }

        Ok(result)
    }

    fn inverse_transform(&self, data: &Array2<f64>) -> Result<Array2<f64>> {
        let mut result = data.clone();

        match self.method {
            PowerMethod::Simple => {
                let inv_power = 1.0 / self.power;
                for val in result.iter_mut() {
                    *val = val.powf(inv_power);
                }
            }
            PowerMethod::BoxCox => {
                let lambda = self.lambda.unwrap_or(0.0);
                for val in result.iter_mut() {
                    if lambda.abs() < 1e-10 {
                        *val = val.exp();
                    } else {
                        *val = (lambda * *val + 1.0).powf(1.0 / lambda);
                    }
                }
            }
            PowerMethod::YeoJohnson => {
                let lambda = self.lambda.unwrap_or(0.0);
                for val in result.iter_mut() {
                    *val = self.yeo_johnson_inverse(*val, lambda);
                }
            }
        }

        Ok(result)
    }
}

impl PowerTransform {
    fn yeo_johnson_transform(&self, x: f64, lambda: f64) -> f64 {
        if x >= 0.0 {
            if lambda.abs() < 1e-10 {
                (x + 1.0).ln()
            } else {
                ((x + 1.0).powf(lambda) - 1.0) / lambda
            }
        } else {
            if (2.0 - lambda).abs() < 1e-10 {
                -((-x + 1.0).ln())
            } else {
                -((-x + 1.0).powf(2.0 - lambda) - 1.0) / (2.0 - lambda)
            }
        }
    }

    fn yeo_johnson_inverse(&self, y: f64, lambda: f64) -> f64 {
        if y >= 0.0 {
            if lambda.abs() < 1e-10 {
                y.exp() - 1.0
            } else {
                (lambda * y + 1.0).powf(1.0 / lambda) - 1.0
            }
        } else {
            if (2.0 - lambda).abs() < 1e-10 {
                1.0 - (-y).exp()
            } else {
                1.0 - (-(2.0 - lambda) * y + 1.0).powf(1.0 / (2.0 - lambda))
            }
        }
    }
}

/// Power transformation method
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PowerMethod {
    /// Simple power transformation
    Simple,
    /// Box-Cox transformation (requires positive values)
    BoxCox,
    /// Yeo-Johnson transformation (works with negative values)
    YeoJohnson,
}

/// Binning/discretization transformer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinningTransform {
    /// Number of bins
    n_bins: usize,

    /// Bin edges (computed during fit)
    edges: Option<Vec<Array1<f64>>>,

    /// Binning strategy
    strategy: BinningStrategy,
}

impl BinningTransform {
    /// Create a new binning transform
    pub fn new(n_bins: usize) -> Self {
        Self {
            n_bins,
            edges: None,
            strategy: BinningStrategy::Uniform,
        }
    }

    /// Set binning strategy
    pub fn with_strategy(mut self, strategy: BinningStrategy) -> Self {
        self.strategy = strategy;
        self
    }
}

impl FeatureTransformer for BinningTransform {
    fn fit(&mut self, data: &Array2<f64>) -> Result<()> {
        let n_features = data.ncols();
        let mut edges = Vec::with_capacity(n_features);

        for i in 0..n_features {
            let col = data.column(i);
            let col_edges = match self.strategy {
                BinningStrategy::Uniform => self.uniform_bins(&col),
                BinningStrategy::Quantile => self.quantile_bins(&col),
            };
            edges.push(col_edges);
        }

        self.edges = Some(edges);
        Ok(())
    }

    fn transform(&self, data: &Array2<f64>) -> Result<Array2<f64>> {
        let mut result = data.clone();

        if let Some(ref edges) = self.edges {
            for i in 0..result.ncols() {
                let col_edges = &edges[i];
                let mut col = result.column_mut(i);

                for val in col.iter_mut() {
                    *val = self.find_bin(*val, col_edges) as f64;
                }
            }
        }

        Ok(result)
    }

    fn inverse_transform(&self, _data: &Array2<f64>) -> Result<Array2<f64>> {
        // Binning is not reversible
        Err(crate::error::MLError::feature(
            "Binning transformation is not reversible",
        ))
    }
}

impl BinningTransform {
    fn uniform_bins(&self, data: &ndarray::ArrayView1<f64>) -> Array1<f64> {
        let min = data.iter().copied().fold(f64::INFINITY, f64::min);
        let max = data.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let step = (max - min) / self.n_bins as f64;

        Array1::from_iter((0..=self.n_bins).map(|i| min + i as f64 * step))
    }

    fn quantile_bins(&self, data: &ndarray::ArrayView1<f64>) -> Array1<f64> {
        let mut sorted: Vec<f64> = data.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let mut edges = Vec::with_capacity(self.n_bins + 1);
        edges.push(sorted[0]);

        for i in 1..self.n_bins {
            let idx = (i * sorted.len()) / self.n_bins;
            edges.push(sorted[idx.min(sorted.len() - 1)]);
        }

        edges.push(sorted[sorted.len() - 1]);
        Array1::from_vec(edges)
    }

    fn find_bin(&self, value: f64, edges: &Array1<f64>) -> usize {
        for i in 0..edges.len() - 1 {
            if value >= edges[i] && value <= edges[i + 1] {
                return i;
            }
        }
        edges.len() - 2 // Last bin
    }
}

/// Binning strategy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BinningStrategy {
    /// Uniform width bins
    Uniform,
    /// Equal frequency bins (quantiles)
    Quantile,
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::arr2;

    #[test]
    fn test_log_transform() -> Result<()> {
        let data = arr2(&[[1.0, 2.0], [3.0, 4.0]]);

        let transform = LogTransform::new();
        let transformed = transform.transform(&data)?;
        let recovered = transform.inverse_transform(&transformed)?;

        assert!((recovered[[0, 0]] - data[[0, 0]]).abs() < 1e-10);

        Ok(())
    }

    #[test]
    fn test_power_transform() -> Result<()> {
        let data = arr2(&[[1.0, 4.0], [9.0, 16.0]]);

        let transform = PowerTransform::sqrt();
        let transformed = transform.transform(&data)?;

        assert!((transformed[[0, 0]] - 1.0).abs() < 1e-10);
        assert!((transformed[[0, 1]] - 2.0).abs() < 1e-10);

        Ok(())
    }

    #[test]
    fn test_binning_transform() -> Result<()> {
        let data = arr2(&[[1.0], [2.0], [3.0], [4.0], [5.0]]);

        let mut transform = BinningTransform::new(3);
        let transformed = transform.fit_transform(&data)?;

        // Values should be binned into 0, 1, 2
        for val in transformed.iter() {
            assert!(*val >= 0.0 && *val <= 2.0);
        }

        Ok(())
    }
}
