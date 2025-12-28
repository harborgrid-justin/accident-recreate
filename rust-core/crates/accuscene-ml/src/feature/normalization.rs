//! Feature normalization and scaling

use crate::error::Result;
use ndarray::{Array1, Array2, Axis};
use serde::{Deserialize, Serialize};

/// Standard scaler (z-score normalization)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardScaler {
    /// Feature means
    mean: Option<Array1<f64>>,

    /// Feature standard deviations
    std: Option<Array1<f64>>,

    /// Whether to center the data
    with_mean: bool,

    /// Whether to scale the data
    with_std: bool,
}

impl StandardScaler {
    /// Create a new standard scaler
    pub fn new() -> Self {
        Self {
            mean: None,
            std: None,
            with_mean: true,
            with_std: true,
        }
    }

    /// Set whether to center the data
    pub fn with_mean(mut self, with_mean: bool) -> Self {
        self.with_mean = with_mean;
        self
    }

    /// Set whether to scale the data
    pub fn with_std(mut self, with_std: bool) -> Self {
        self.with_std = with_std;
        self
    }

    /// Fit the scaler to training data
    pub fn fit(&mut self, data: &Array2<f64>) -> Result<()> {
        let n_features = data.ncols();

        if self.with_mean {
            self.mean = Some(data.mean_axis(Axis(0)).unwrap());
        }

        if self.with_std {
            let mut std = Array1::zeros(n_features);
            for i in 0..n_features {
                let col = data.column(i);
                std[i] = col.std(0.0);
                // Avoid division by zero
                if std[i] == 0.0 {
                    std[i] = 1.0;
                }
            }
            self.std = Some(std);
        }

        Ok(())
    }

    /// Transform data using fitted parameters
    pub fn transform(&self, data: &Array2<f64>) -> Result<Array2<f64>> {
        let mut result = data.clone();

        if self.with_mean {
            if let Some(ref mean) = self.mean {
                for i in 0..result.ncols() {
                    let mut col = result.column_mut(i);
                    col -= mean[i];
                }
            }
        }

        if self.with_std {
            if let Some(ref std) = self.std {
                for i in 0..result.ncols() {
                    let mut col = result.column_mut(i);
                    col /= std[i];
                }
            }
        }

        Ok(result)
    }

    /// Fit and transform in one step
    pub fn fit_transform(&mut self, data: &Array2<f64>) -> Result<Array2<f64>> {
        self.fit(data)?;
        self.transform(data)
    }

    /// Inverse transform (denormalize)
    pub fn inverse_transform(&self, data: &Array2<f64>) -> Result<Array2<f64>> {
        let mut result = data.clone();

        if self.with_std {
            if let Some(ref std) = self.std {
                for i in 0..result.ncols() {
                    let mut col = result.column_mut(i);
                    col *= std[i];
                }
            }
        }

        if self.with_mean {
            if let Some(ref mean) = self.mean {
                for i in 0..result.ncols() {
                    let mut col = result.column_mut(i);
                    col += mean[i];
                }
            }
        }

        Ok(result)
    }
}

impl Default for StandardScaler {
    fn default() -> Self {
        Self::new()
    }
}

/// Min-max scaler (normalization to [0, 1] range)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinMaxScaler {
    /// Feature minimums
    min: Option<Array1<f64>>,

    /// Feature maximums
    max: Option<Array1<f64>>,

    /// Target range minimum
    feature_range_min: f64,

    /// Target range maximum
    feature_range_max: f64,
}

impl MinMaxScaler {
    /// Create a new min-max scaler with default range [0, 1]
    pub fn new() -> Self {
        Self {
            min: None,
            max: None,
            feature_range_min: 0.0,
            feature_range_max: 1.0,
        }
    }

    /// Set the target feature range
    pub fn with_feature_range(mut self, min: f64, max: f64) -> Self {
        self.feature_range_min = min;
        self.feature_range_max = max;
        self
    }

    /// Fit the scaler to training data
    pub fn fit(&mut self, data: &Array2<f64>) -> Result<()> {
        let n_features = data.ncols();
        let mut min = Array1::zeros(n_features);
        let mut max = Array1::zeros(n_features);

        for i in 0..n_features {
            let col = data.column(i);
            min[i] = col.iter().copied().fold(f64::INFINITY, f64::min);
            max[i] = col.iter().copied().fold(f64::NEG_INFINITY, f64::max);

            // Handle case where min == max
            if (max[i] - min[i]).abs() < 1e-10 {
                max[i] = min[i] + 1.0;
            }
        }

        self.min = Some(min);
        self.max = Some(max);

        Ok(())
    }

    /// Transform data using fitted parameters
    pub fn transform(&self, data: &Array2<f64>) -> Result<Array2<f64>> {
        let mut result = data.clone();

        if let (Some(ref min), Some(ref max)) = (&self.min, &self.max) {
            let range = self.feature_range_max - self.feature_range_min;

            for i in 0..result.ncols() {
                let mut col = result.column_mut(i);
                let data_range = max[i] - min[i];

                for val in col.iter_mut() {
                    *val = (*val - min[i]) / data_range * range + self.feature_range_min;
                }
            }
        }

        Ok(result)
    }

    /// Fit and transform in one step
    pub fn fit_transform(&mut self, data: &Array2<f64>) -> Result<Array2<f64>> {
        self.fit(data)?;
        self.transform(data)
    }

    /// Inverse transform (denormalize)
    pub fn inverse_transform(&self, data: &Array2<f64>) -> Result<Array2<f64>> {
        let mut result = data.clone();

        if let (Some(ref min), Some(ref max)) = (&self.min, &self.max) {
            let range = self.feature_range_max - self.feature_range_min;

            for i in 0..result.ncols() {
                let mut col = result.column_mut(i);
                let data_range = max[i] - min[i];

                for val in col.iter_mut() {
                    *val = (*val - self.feature_range_min) / range * data_range + min[i];
                }
            }
        }

        Ok(result)
    }
}

impl Default for MinMaxScaler {
    fn default() -> Self {
        Self::new()
    }
}

/// Normalizer (L1, L2, or max normalization)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Normalizer {
    /// Normalization method
    norm: NormType,
}

impl Normalizer {
    /// Create a new normalizer with L2 norm
    pub fn new() -> Self {
        Self {
            norm: NormType::L2,
        }
    }

    /// Create normalizer with specific norm type
    pub fn with_norm(norm: NormType) -> Self {
        Self { norm }
    }

    /// Transform data (row-wise normalization)
    pub fn transform(&self, data: &Array2<f64>) -> Result<Array2<f64>> {
        let mut result = data.clone();

        for i in 0..result.nrows() {
            let row = result.row(i);
            let norm_value = match self.norm {
                NormType::L1 => row.iter().map(|x| x.abs()).sum::<f64>(),
                NormType::L2 => row.iter().map(|x| x * x).sum::<f64>().sqrt(),
                NormType::Max => row.iter().map(|x| x.abs()).fold(0.0f64, f64::max),
            };

            if norm_value > 0.0 {
                let mut row_mut = result.row_mut(i);
                row_mut /= norm_value;
            }
        }

        Ok(result)
    }

    /// Fit is a no-op for Normalizer (included for API consistency)
    pub fn fit(&self, _data: &Array2<f64>) -> Result<()> {
        Ok(())
    }

    /// Fit and transform in one step
    pub fn fit_transform(&self, data: &Array2<f64>) -> Result<Array2<f64>> {
        self.transform(data)
    }
}

impl Default for Normalizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Normalization type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NormType {
    /// L1 norm (sum of absolute values)
    L1,
    /// L2 norm (Euclidean norm)
    L2,
    /// Max norm (maximum absolute value)
    Max,
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::arr2;

    #[test]
    fn test_standard_scaler() -> Result<()> {
        let data = arr2(&[[1.0, 2.0], [3.0, 4.0], [5.0, 6.0]]);

        let mut scaler = StandardScaler::new();
        let transformed = scaler.fit_transform(&data)?;

        // Mean should be ~0
        let mean = transformed.mean_axis(Axis(0)).unwrap();
        assert!(mean[0].abs() < 1e-10);
        assert!(mean[1].abs() < 1e-10);

        // Inverse transform should recover original
        let recovered = scaler.inverse_transform(&transformed)?;
        assert!((recovered[[0, 0]] - data[[0, 0]]).abs() < 1e-10);

        Ok(())
    }

    #[test]
    fn test_minmax_scaler() -> Result<()> {
        let data = arr2(&[[1.0, 2.0], [3.0, 4.0], [5.0, 6.0]]);

        let mut scaler = MinMaxScaler::new();
        let transformed = scaler.fit_transform(&data)?;

        // Values should be in [0, 1]
        for val in transformed.iter() {
            assert!(*val >= 0.0 && *val <= 1.0);
        }

        Ok(())
    }

    #[test]
    fn test_normalizer() -> Result<()> {
        let data = arr2(&[[3.0, 4.0], [1.0, 1.0]]);

        let normalizer = Normalizer::new();
        let transformed = normalizer.transform(&data)?;

        // L2 norm should be 1 for each row
        let row1_norm: f64 = transformed.row(0).iter().map(|x| x * x).sum::<f64>().sqrt();
        assert!((row1_norm - 1.0).abs() < 1e-10);

        Ok(())
    }
}
