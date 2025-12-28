//! Feature engineering module

use crate::error::Result;
use ndarray::{Array1, Array2};
use serde::{Deserialize, Serialize};

pub mod encoding;
pub mod extraction;
pub mod normalization;
pub mod store;
pub mod transformation;

pub use encoding::{CategoricalEncoder, EncodingStrategy, LabelEncoder, OneHotEncoder};
pub use extraction::{FeatureExtractor, PolynomialFeatures, StatisticalFeatures};
pub use normalization::{MinMaxScaler, Normalizer, StandardScaler};
pub use store::{FeatureStore, FeatureStoreConfig};
pub use transformation::{FeatureTransformer, LogTransform, PowerTransform};

/// Feature vector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureVector {
    /// Feature names
    pub names: Vec<String>,

    /// Feature values
    pub values: Array1<f64>,

    /// Feature metadata
    pub metadata: std::collections::HashMap<String, String>,
}

impl FeatureVector {
    /// Create a new feature vector
    pub fn new(names: Vec<String>, values: Array1<f64>) -> Self {
        Self {
            names,
            values,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Get number of features
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Get feature by name
    pub fn get(&self, name: &str) -> Option<f64> {
        self.names
            .iter()
            .position(|n| n == name)
            .and_then(|idx| self.values.get(idx).copied())
    }

    /// Add metadata
    pub fn add_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
    }
}

/// Feature set (batch of feature vectors)
#[derive(Debug, Clone)]
pub struct FeatureSet {
    /// Feature names
    pub names: Vec<String>,

    /// Feature matrix (samples x features)
    pub matrix: Array2<f64>,
}

impl FeatureSet {
    /// Create a new feature set
    pub fn new(names: Vec<String>, matrix: Array2<f64>) -> Result<Self> {
        if names.len() != matrix.ncols() {
            return Err(crate::error::MLError::shape_mismatch(
                format!("{} features", names.len()),
                format!("{} columns", matrix.ncols()),
            ));
        }

        Ok(Self { names, matrix })
    }

    /// Get number of samples
    pub fn num_samples(&self) -> usize {
        self.matrix.nrows()
    }

    /// Get number of features
    pub fn num_features(&self) -> usize {
        self.matrix.ncols()
    }

    /// Get feature vector for sample
    pub fn get_sample(&self, idx: usize) -> Option<FeatureVector> {
        if idx >= self.num_samples() {
            return None;
        }

        let values = self.matrix.row(idx).to_owned();
        Some(FeatureVector::new(self.names.clone(), values))
    }

    /// Get column by feature name
    pub fn get_feature(&self, name: &str) -> Option<Array1<f64>> {
        self.names
            .iter()
            .position(|n| n == name)
            .and_then(|idx| Some(self.matrix.column(idx).to_owned()))
    }

    /// Add a feature column
    pub fn add_feature(&mut self, name: String, values: Array1<f64>) -> Result<()> {
        if values.len() != self.num_samples() {
            return Err(crate::error::MLError::shape_mismatch(
                format!("{} samples", self.num_samples()),
                format!("{} values", values.len()),
            ));
        }

        // Stack the new column
        let new_col = values.insert_axis(ndarray::Axis(1));
        self.matrix = ndarray::concatenate![ndarray::Axis(1), self.matrix, new_col];
        self.names.push(name);

        Ok(())
    }

    /// Select features by names
    pub fn select_features(&self, names: &[String]) -> Result<Self> {
        let mut indices = Vec::new();
        for name in names {
            let idx = self.names.iter()
                .position(|n| n == name)
                .ok_or_else(|| crate::error::MLError::feature(format!("Feature not found: {}", name)))?;
            indices.push(idx);
        }

        let mut new_matrix = Array2::zeros((self.num_samples(), indices.len()));
        for (new_idx, &old_idx) in indices.iter().enumerate() {
            new_matrix.column_mut(new_idx).assign(&self.matrix.column(old_idx));
        }

        Ok(Self {
            names: names.to_vec(),
            matrix: new_matrix,
        })
    }
}

/// Feature statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureStats {
    /// Feature name
    pub name: String,

    /// Mean value
    pub mean: f64,

    /// Standard deviation
    pub std: f64,

    /// Minimum value
    pub min: f64,

    /// Maximum value
    pub max: f64,

    /// Median value
    pub median: f64,

    /// Number of missing values
    pub missing_count: usize,

    /// Total count
    pub total_count: usize,
}

impl FeatureStats {
    /// Compute statistics for a feature column
    pub fn compute(name: String, values: &Array1<f64>) -> Self {
        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let total_count = values.len();
        let missing_count = values.iter().filter(|v| v.is_nan()).count();

        let valid_values: Vec<f64> = values.iter().copied().filter(|v| !v.is_nan()).collect();

        let mean = if valid_values.is_empty() {
            0.0
        } else {
            valid_values.iter().sum::<f64>() / valid_values.len() as f64
        };

        let variance = if valid_values.len() > 1 {
            valid_values.iter()
                .map(|v| (v - mean).powi(2))
                .sum::<f64>() / (valid_values.len() - 1) as f64
        } else {
            0.0
        };

        let std = variance.sqrt();

        let min = valid_values.iter().copied().fold(f64::INFINITY, f64::min);
        let max = valid_values.iter().copied().fold(f64::NEG_INFINITY, f64::max);

        let median = if valid_values.is_empty() {
            0.0
        } else {
            let mut sorted_valid = valid_values.clone();
            sorted_valid.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let mid = sorted_valid.len() / 2;
            if sorted_valid.len() % 2 == 0 {
                (sorted_valid[mid - 1] + sorted_valid[mid]) / 2.0
            } else {
                sorted_valid[mid]
            }
        };

        Self {
            name,
            mean,
            std,
            min,
            max,
            median,
            missing_count,
            total_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::arr1;

    #[test]
    fn test_feature_vector() {
        let names = vec!["age".to_string(), "height".to_string()];
        let values = arr1(&[25.0, 175.0]);
        let fv = FeatureVector::new(names, values);

        assert_eq!(fv.len(), 2);
        assert_eq!(fv.get("age"), Some(25.0));
        assert_eq!(fv.get("height"), Some(175.0));
        assert_eq!(fv.get("weight"), None);
    }

    #[test]
    fn test_feature_stats() {
        let values = arr1(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        let stats = FeatureStats::compute("test".to_string(), &values);

        assert_eq!(stats.mean, 3.0);
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 5.0);
        assert_eq!(stats.median, 3.0);
        assert_eq!(stats.missing_count, 0);
    }
}
