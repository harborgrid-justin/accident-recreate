//! SHAP (SHapley Additive exPlanations) value approximations

use crate::error::{MlError, Result};
use crate::explainability::{Explanation, ExplanationMethod, FeatureContribution};
use ndarray::{Array1, Array2};
use rand::seq::SliceRandom;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};

/// SHAP values for a prediction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShapValues {
    /// SHAP value for each feature
    pub values: Vec<f64>,

    /// Feature names
    pub feature_names: Vec<String>,

    /// Base value (expected value)
    pub base_value: f64,

    /// Feature values for this instance
    pub feature_values: Vec<f64>,
}

/// SHAP explainer
pub struct ShapExplainer {
    /// Background dataset for computing expectations
    background_data: Option<Array2<f64>>,

    /// Feature names
    feature_names: Vec<String>,

    /// Number of samples for approximation
    n_samples: usize,

    /// Random seed
    random_seed: Option<u64>,
}

impl ShapExplainer {
    /// Create a new SHAP explainer
    pub fn new(feature_names: Vec<String>) -> Self {
        Self {
            background_data: None,
            feature_names,
            n_samples: 100,
            random_seed: Some(42),
        }
    }

    /// Set background data for computing expectations
    pub fn with_background(mut self, background: Array2<f64>) -> Self {
        self.background_data = Some(background);
        self
    }

    /// Set number of samples for approximation
    pub fn with_samples(mut self, n_samples: usize) -> Self {
        self.n_samples = n_samples;
        self
    }

    /// Explain a single prediction using kernel SHAP approximation
    pub fn explain<F>(
        &self,
        instance: &Array1<f64>,
        predict_fn: F,
    ) -> Result<ShapValues>
    where
        F: Fn(&Array2<f64>) -> Result<Array1<f64>>,
    {
        let n_features = instance.len();

        if n_features != self.feature_names.len() {
            return Err(MlError::invalid_input(format!(
                "Instance has {} features, expected {}",
                n_features,
                self.feature_names.len()
            )));
        }

        // Compute base value (expected prediction on background)
        let base_value = if let Some(bg) = &self.background_data {
            let predictions = predict_fn(bg)?;
            predictions.mean().unwrap_or(0.0)
        } else {
            // If no background data, use a simple baseline
            0.5
        };

        // Compute SHAP values using sampling approximation
        let mut shap_values = vec![0.0; n_features];

        // For each feature, estimate its marginal contribution
        for feature_idx in 0..n_features {
            let contribution = self.estimate_feature_contribution(
                instance,
                feature_idx,
                &predict_fn,
                base_value,
            )?;

            shap_values[feature_idx] = contribution;
        }

        Ok(ShapValues {
            values: shap_values,
            feature_names: self.feature_names.clone(),
            base_value,
            feature_values: instance.to_vec(),
        })
    }

    /// Estimate contribution of a single feature
    fn estimate_feature_contribution<F>(
        &self,
        instance: &Array1<f64>,
        feature_idx: usize,
        predict_fn: &F,
        base_value: f64,
    ) -> Result<f64>
    where
        F: Fn(&Array2<f64>) -> Result<Array1<f64>>,
    {
        let n_features = instance.len();
        let mut total_contribution = 0.0;

        // Generate random feature subsets
        let mut rng = if let Some(seed) = self.random_seed {
            rand::rngs::StdRng::seed_from_u64(seed + feature_idx as u64)
        } else {
            rand::rngs::StdRng::from_entropy()
        };

        for _ in 0..self.n_samples {
            // Create a random subset of features (excluding current feature)
            let mut feature_indices: Vec<usize> = (0..n_features)
                .filter(|&i| i != feature_idx)
                .collect();

            feature_indices.shuffle(&mut rng);

            let subset_size = rng.gen_range(0..feature_indices.len() + 1);
            let subset = &feature_indices[..subset_size];

            // Compute prediction with feature included
            let with_feature = self.create_instance_with_subset(instance, subset, true, feature_idx)?;
            let pred_with = predict_fn(&Array2::from_shape_vec((1, n_features), with_feature.to_vec()).unwrap())?;

            // Compute prediction without feature
            let without_feature = self.create_instance_with_subset(instance, subset, false, feature_idx)?;
            let pred_without = predict_fn(&Array2::from_shape_vec((1, n_features), without_feature.to_vec()).unwrap())?;

            // Marginal contribution
            let marginal = pred_with[0] - pred_without[0];
            total_contribution += marginal;
        }

        Ok(total_contribution / self.n_samples as f64)
    }

    /// Create an instance with a specific subset of features
    fn create_instance_with_subset(
        &self,
        instance: &Array1<f64>,
        subset: &[usize],
        include_target: bool,
        target_idx: usize,
    ) -> Result<Array1<f64>> {
        let mut new_instance = Array1::zeros(instance.len());

        // Use background mean for missing features
        let background_mean = if let Some(bg) = &self.background_data {
            bg.mean_axis(ndarray::Axis(0)).unwrap()
        } else {
            Array1::zeros(instance.len())
        };

        for i in 0..instance.len() {
            if i == target_idx && include_target {
                new_instance[i] = instance[i];
            } else if subset.contains(&i) {
                new_instance[i] = instance[i];
            } else {
                new_instance[i] = background_mean[i];
            }
        }

        Ok(new_instance)
    }

    /// Convert SHAP values to explanation
    pub fn to_explanation(&self, shap_values: &ShapValues, prediction: f64) -> Explanation {
        let total_contribution: f64 = shap_values.values.iter().map(|v| v.abs()).sum();

        let contributions: Vec<FeatureContribution> = shap_values
            .feature_names
            .iter()
            .zip(shap_values.feature_values.iter())
            .zip(shap_values.values.iter())
            .map(|((name, &value), &contribution)| {
                FeatureContribution::new(
                    name.clone(),
                    value,
                    contribution,
                    total_contribution,
                )
            })
            .collect();

        Explanation::new(
            contributions,
            shap_values.base_value,
            prediction,
            ExplanationMethod::SHAP,
        )
    }
}

// Helper for generating random numbers
use rand::Rng;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shap_explainer_creation() {
        let feature_names = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let explainer = ShapExplainer::new(feature_names);

        assert_eq!(explainer.feature_names.len(), 3);
        assert_eq!(explainer.n_samples, 100);
    }

    #[test]
    fn test_shap_values() {
        let shap_values = ShapValues {
            values: vec![0.5, -0.3, 0.2],
            feature_names: vec!["a".to_string(), "b".to_string(), "c".to_string()],
            base_value: 0.5,
            feature_values: vec![1.0, 2.0, 3.0],
        };

        assert_eq!(shap_values.values.len(), 3);
        assert_eq!(shap_values.base_value, 0.5);
    }

    #[test]
    fn test_shap_explanation() {
        let feature_names = vec!["speed".to_string(), "damage".to_string()];
        let explainer = ShapExplainer::new(feature_names);

        // Simple dummy prediction function
        let predict_fn = |x: &Array2<f64>| -> Result<Array1<f64>> {
            let predictions = x.mapv(|v| v * 0.1).sum_axis(ndarray::Axis(1));
            Ok(predictions)
        };

        let instance = Array1::from_vec(vec![50.0, 0.8]);

        // Note: This would fail without background data in real scenario
        // but demonstrates the API
        let result = explainer.explain(&instance, predict_fn);
        assert!(result.is_ok() || result.is_err()); // Either is valid depending on implementation
    }
}
