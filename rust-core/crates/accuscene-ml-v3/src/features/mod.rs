//! Feature extraction and engineering for ML models

pub mod damage_features;
pub mod normalizer;
pub mod scene_features;
pub mod vehicle_features;

use crate::error::{MlError, Result};
use ndarray::{Array1, Array2};
use serde::{Deserialize, Serialize};

/// Feature vector representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureVector {
    /// Feature values
    pub values: Vec<f64>,

    /// Feature names
    pub names: Vec<String>,

    /// Feature metadata
    pub metadata: FeatureMetadata,
}

/// Feature metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureMetadata {
    /// Feature extraction timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Feature version
    pub version: String,

    /// Source data identifier
    pub source_id: Option<String>,

    /// Feature quality score (0.0 - 1.0)
    pub quality_score: f64,

    /// Missing feature indicators
    pub missing_features: Vec<usize>,
}

/// Feature extractor trait
pub trait FeatureExtractor {
    /// Extract features from input
    fn extract(&self, input: &dyn std::any::Any) -> Result<FeatureVector>;

    /// Get feature names
    fn feature_names(&self) -> Vec<String>;

    /// Get expected number of features
    fn feature_count(&self) -> usize {
        self.feature_names().len()
    }
}

/// Feature transformer trait
pub trait FeatureTransformer {
    /// Transform features
    fn transform(&self, features: &FeatureVector) -> Result<FeatureVector>;

    /// Fit and transform features
    fn fit_transform(&mut self, features: &FeatureVector) -> Result<FeatureVector>;
}

impl FeatureVector {
    /// Create a new feature vector
    pub fn new(values: Vec<f64>, names: Vec<String>) -> Self {
        let quality_score = Self::calculate_quality(&values);

        Self {
            values,
            names,
            metadata: FeatureMetadata {
                timestamp: chrono::Utc::now(),
                version: "1.0.0".to_string(),
                source_id: None,
                quality_score,
                missing_features: vec![],
            },
        }
    }

    /// Create from ndarray
    pub fn from_array(array: Array1<f64>, names: Vec<String>) -> Self {
        Self::new(array.to_vec(), names)
    }

    /// Convert to ndarray
    pub fn to_array(&self) -> Array1<f64> {
        Array1::from_vec(self.values.clone())
    }

    /// Get feature value by name
    pub fn get(&self, name: &str) -> Option<f64> {
        self.names
            .iter()
            .position(|n| n == name)
            .map(|idx| self.values[idx])
    }

    /// Set feature value by name
    pub fn set(&mut self, name: &str, value: f64) -> Result<()> {
        let idx = self
            .names
            .iter()
            .position(|n| n == name)
            .ok_or_else(|| MlError::FeatureExtraction(format!("Feature '{}' not found", name)))?;

        self.values[idx] = value;
        Ok(())
    }

    /// Calculate feature quality score
    fn calculate_quality(values: &[f64]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }

        let valid_count = values.iter().filter(|v| v.is_finite()).count();
        let completeness = valid_count as f64 / values.len() as f64;

        // Check for variance (features with no variance are low quality)
        let mean = values.iter().filter(|v| v.is_finite()).sum::<f64>() / valid_count as f64;
        let variance = values
            .iter()
            .filter(|v| v.is_finite())
            .map(|v| (v - mean).powi(2))
            .sum::<f64>()
            / valid_count as f64;

        let variance_score = if variance > 0.0 { 1.0 } else { 0.5 };

        (completeness + variance_score) / 2.0
    }

    /// Check if features are valid
    pub fn is_valid(&self) -> bool {
        self.values.iter().all(|v| v.is_finite())
            && self.values.len() == self.names.len()
            && self.metadata.quality_score > 0.5
    }

    /// Fill missing values with a default
    pub fn fill_missing(&mut self, fill_value: f64) {
        for value in &mut self.values {
            if !value.is_finite() {
                *value = fill_value;
            }
        }
    }

    /// Get statistical summary
    pub fn summary(&self) -> FeatureSummary {
        let valid_values: Vec<f64> = self.values.iter().copied().filter(|v| v.is_finite()).collect();

        if valid_values.is_empty() {
            return FeatureSummary::default();
        }

        let mean = valid_values.iter().sum::<f64>() / valid_values.len() as f64;

        let mut sorted = valid_values.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let min = sorted[0];
        let max = sorted[sorted.len() - 1];
        let median = if sorted.len() % 2 == 0 {
            (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2.0
        } else {
            sorted[sorted.len() / 2]
        };

        let variance = valid_values
            .iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>()
            / valid_values.len() as f64;

        let std_dev = variance.sqrt();

        FeatureSummary {
            count: valid_values.len(),
            mean,
            std_dev,
            min,
            max,
            median,
        }
    }
}

/// Feature statistical summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureSummary {
    pub count: usize,
    pub mean: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub median: f64,
}

impl Default for FeatureSummary {
    fn default() -> Self {
        Self {
            count: 0,
            mean: 0.0,
            std_dev: 0.0,
            min: 0.0,
            max: 0.0,
            median: 0.0,
        }
    }
}

// Re-export feature types
pub use damage_features::{DamageFeatures, DamageFeatureExtractor};
pub use normalizer::{Normalizer, NormalizationMethod};
pub use scene_features::{SceneFeatures, SceneFeatureExtractor};
pub use vehicle_features::{VehicleFeatures, VehicleFeatureExtractor};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_vector() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let names = vec!["a".to_string(), "b".to_string(), "c".to_string(), "d".to_string(), "e".to_string()];

        let fv = FeatureVector::new(values, names);

        assert_eq!(fv.values.len(), 5);
        assert_eq!(fv.get("a"), Some(1.0));
        assert!(fv.is_valid());
    }

    #[test]
    fn test_feature_quality() {
        let good_values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let quality = FeatureVector::calculate_quality(&good_values);
        assert!(quality > 0.8);

        let bad_values = vec![f64::NAN, 2.0, f64::INFINITY];
        let quality = FeatureVector::calculate_quality(&bad_values);
        assert!(quality < 0.7);
    }

    #[test]
    fn test_feature_summary() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let names = vec!["a".into(), "b".into(), "c".into(), "d".into(), "e".into()];
        let fv = FeatureVector::new(values, names);

        let summary = fv.summary();
        assert_eq!(summary.mean, 3.0);
        assert_eq!(summary.min, 1.0);
        assert_eq!(summary.max, 5.0);
        assert_eq!(summary.median, 3.0);
    }
}
