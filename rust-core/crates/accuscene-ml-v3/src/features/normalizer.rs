//! Feature normalization and standardization

use crate::error::{MlError, Result};
use crate::features::{FeatureTransformer, FeatureVector};
use ndarray::{Array1, Array2};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Normalization method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NormalizationMethod {
    /// Min-Max scaling to [0, 1]
    MinMax,
    /// Z-score standardization (mean=0, std=1)
    ZScore,
    /// Robust scaling using median and IQR
    Robust,
    /// L2 normalization (unit vector)
    L2,
    /// No normalization
    None,
}

/// Feature normalizer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Normalizer {
    method: NormalizationMethod,
    statistics: HashMap<String, FeatureStatistics>,
    fitted: bool,
}

/// Statistics for a single feature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureStatistics {
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub std_dev: f64,
    pub median: f64,
    pub q25: f64,  // 25th percentile
    pub q75: f64,  // 75th percentile
    pub count: usize,
}

impl Normalizer {
    /// Create a new normalizer
    pub fn new(method: NormalizationMethod) -> Self {
        Self {
            method,
            statistics: HashMap::new(),
            fitted: false,
        }
    }

    /// Fit the normalizer to data
    pub fn fit(&mut self, features: &[FeatureVector]) -> Result<()> {
        if features.is_empty() {
            return Err(MlError::InvalidInput(
                "Cannot fit normalizer on empty data".to_string(),
            ));
        }

        // Get feature names from first sample
        let feature_names = &features[0].names;

        // Calculate statistics for each feature
        for (idx, name) in feature_names.iter().enumerate() {
            let values: Vec<f64> = features
                .iter()
                .filter_map(|fv| fv.values.get(idx).copied())
                .filter(|v| v.is_finite())
                .collect();

            if values.is_empty() {
                continue;
            }

            let stats = self.calculate_statistics(&values);
            self.statistics.insert(name.clone(), stats);
        }

        self.fitted = true;
        Ok(())
    }

    /// Calculate statistics from values
    fn calculate_statistics(&self, values: &[f64]) -> FeatureStatistics {
        let count = values.len();

        let mean = values.iter().sum::<f64>() / count as f64;

        let variance = values
            .iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>()
            / count as f64;

        let std_dev = variance.sqrt();

        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let min = sorted[0];
        let max = sorted[count - 1];

        let median = if count % 2 == 0 {
            (sorted[count / 2 - 1] + sorted[count / 2]) / 2.0
        } else {
            sorted[count / 2]
        };

        let q25 = sorted[count / 4];
        let q75 = sorted[3 * count / 4];

        FeatureStatistics {
            min,
            max,
            mean,
            std_dev,
            median,
            q25,
            q75,
            count,
        }
    }

    /// Normalize a single value
    fn normalize_value(&self, value: f64, stats: &FeatureStatistics) -> f64 {
        match self.method {
            NormalizationMethod::MinMax => {
                if (stats.max - stats.min).abs() < 1e-10 {
                    0.5 // Constant feature, return middle value
                } else {
                    (value - stats.min) / (stats.max - stats.min)
                }
            }
            NormalizationMethod::ZScore => {
                if stats.std_dev < 1e-10 {
                    0.0 // Constant feature
                } else {
                    (value - stats.mean) / stats.std_dev
                }
            }
            NormalizationMethod::Robust => {
                let iqr = stats.q75 - stats.q25;
                if iqr < 1e-10 {
                    0.0
                } else {
                    (value - stats.median) / iqr
                }
            }
            NormalizationMethod::L2 => value, // Handled at vector level
            NormalizationMethod::None => value,
        }
    }

    /// Denormalize a single value
    pub fn denormalize_value(&self, normalized: f64, feature_name: &str) -> Result<f64> {
        let stats = self
            .statistics
            .get(feature_name)
            .ok_or_else(|| {
                MlError::InvalidInput(format!("No statistics for feature '{}'", feature_name))
            })?;

        let value = match self.method {
            NormalizationMethod::MinMax => {
                normalized * (stats.max - stats.min) + stats.min
            }
            NormalizationMethod::ZScore => {
                normalized * stats.std_dev + stats.mean
            }
            NormalizationMethod::Robust => {
                let iqr = stats.q75 - stats.q25;
                normalized * iqr + stats.median
            }
            NormalizationMethod::L2 => normalized,
            NormalizationMethod::None => normalized,
        };

        Ok(value)
    }

    /// Normalize array (for L2 normalization)
    fn normalize_array(&self, array: &mut Array1<f64>) {
        if self.method == NormalizationMethod::L2 {
            let norm = array.iter().map(|v| v.powi(2)).sum::<f64>().sqrt();
            if norm > 1e-10 {
                *array = array.mapv(|v| v / norm);
            }
        }
    }

    /// Check if normalizer is fitted
    pub fn is_fitted(&self) -> bool {
        self.fitted
    }

    /// Get statistics for a feature
    pub fn get_statistics(&self, feature_name: &str) -> Option<&FeatureStatistics> {
        self.statistics.get(feature_name)
    }

    /// Save normalizer to file
    pub fn save<P: AsRef<std::path::Path>>(&self, path: P) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load normalizer from file
    pub fn load<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let normalizer = serde_json::from_str(&json)?;
        Ok(normalizer)
    }
}

impl FeatureTransformer for Normalizer {
    fn transform(&self, features: &FeatureVector) -> Result<FeatureVector> {
        if !self.fitted {
            return Err(MlError::InvalidInput(
                "Normalizer must be fitted before transform".to_string(),
            ));
        }

        let mut normalized_values = Vec::with_capacity(features.values.len());

        for (name, &value) in features.names.iter().zip(features.values.iter()) {
            if let Some(stats) = self.statistics.get(name) {
                let normalized = self.normalize_value(value, stats);
                normalized_values.push(normalized);
            } else {
                // Feature not seen during fitting, pass through
                normalized_values.push(value);
            }
        }

        // Apply L2 normalization if needed
        if self.method == NormalizationMethod::L2 {
            let mut array = Array1::from_vec(normalized_values);
            self.normalize_array(&mut array);
            normalized_values = array.to_vec();
        }

        Ok(FeatureVector::new(
            normalized_values,
            features.names.clone(),
        ))
    }

    fn fit_transform(&mut self, features: &FeatureVector) -> Result<FeatureVector> {
        self.fit(&[features.clone()])?;
        self.transform(features)
    }
}

/// Batch normalizer for transforming multiple feature vectors
pub struct BatchNormalizer {
    normalizer: Normalizer,
}

impl BatchNormalizer {
    /// Create a new batch normalizer
    pub fn new(method: NormalizationMethod) -> Self {
        Self {
            normalizer: Normalizer::new(method),
        }
    }

    /// Fit on batch of features
    pub fn fit(&mut self, features_batch: &[FeatureVector]) -> Result<()> {
        self.normalizer.fit(features_batch)
    }

    /// Transform batch of features
    pub fn transform_batch(&self, features_batch: &[FeatureVector]) -> Result<Vec<FeatureVector>> {
        features_batch
            .iter()
            .map(|fv| self.normalizer.transform(fv))
            .collect()
    }

    /// Fit and transform batch
    pub fn fit_transform_batch(
        &mut self,
        features_batch: &[FeatureVector],
    ) -> Result<Vec<FeatureVector>> {
        self.fit(features_batch)?;
        self.transform_batch(features_batch)
    }

    /// Get underlying normalizer
    pub fn normalizer(&self) -> &Normalizer {
        &self.normalizer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_features() -> Vec<FeatureVector> {
        vec![
            FeatureVector::new(
                vec![1.0, 10.0, 100.0],
                vec!["a".to_string(), "b".to_string(), "c".to_string()],
            ),
            FeatureVector::new(
                vec![2.0, 20.0, 200.0],
                vec!["a".to_string(), "b".to_string(), "c".to_string()],
            ),
            FeatureVector::new(
                vec![3.0, 30.0, 300.0],
                vec!["a".to_string(), "b".to_string(), "c".to_string()],
            ),
        ]
    }

    #[test]
    fn test_minmax_normalization() {
        let mut normalizer = Normalizer::new(NormalizationMethod::MinMax);
        let features = create_test_features();

        normalizer.fit(&features).unwrap();

        let transformed = normalizer.transform(&features[0]).unwrap();
        assert!(transformed.values[0] >= 0.0 && transformed.values[0] <= 1.0);
        assert_eq!(transformed.values[0], 0.0); // Min value should be 0
    }

    #[test]
    fn test_zscore_normalization() {
        let mut normalizer = Normalizer::new(NormalizationMethod::ZScore);
        let features = create_test_features();

        normalizer.fit(&features).unwrap();

        let transformed = normalizer.transform(&features[1]).unwrap();
        // Middle value should be close to 0
        assert!(transformed.values[0].abs() < 0.1);
    }

    #[test]
    fn test_batch_normalization() {
        let mut batch_normalizer = BatchNormalizer::new(NormalizationMethod::MinMax);
        let features = create_test_features();

        let normalized = batch_normalizer.fit_transform_batch(&features).unwrap();
        assert_eq!(normalized.len(), 3);

        // Check that all values are in [0, 1]
        for fv in &normalized {
            for &value in &fv.values {
                assert!(value >= 0.0 && value <= 1.0);
            }
        }
    }
}
