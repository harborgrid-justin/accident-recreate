//! Feature importance calculation methods

use crate::error::{MlError, Result};
use crate::explainability::{Explanation, ExplanationMethod, FeatureContribution};
use crate::training::Dataset;
use ndarray::{Array1, Array2};
use rand::seq::SliceRandom;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};

/// Feature importance scores
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureImportance {
    /// Importance score for each feature
    pub scores: Vec<f64>,

    /// Feature names
    pub feature_names: Vec<String>,

    /// Importance method used
    pub method: ImportanceMethod,

    /// Standard deviations (if available)
    pub std_devs: Option<Vec<f64>>,
}

/// Feature importance calculation method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImportanceMethod {
    /// Permutation importance
    Permutation,
    /// Drop-column importance
    DropColumn,
    /// Model coefficients (for linear models)
    Coefficients,
    /// Tree-based importance (e.g., Gini importance)
    TreeBased,
}

/// Feature importance calculator
pub struct ImportanceCalculator {
    method: ImportanceMethod,
    n_repeats: usize,
    random_seed: Option<u64>,
}

impl ImportanceCalculator {
    /// Create a new importance calculator
    pub fn new(method: ImportanceMethod) -> Self {
        Self {
            method,
            n_repeats: 10,
            random_seed: Some(42),
        }
    }

    /// Set number of repeats for permutation importance
    pub fn with_repeats(mut self, n_repeats: usize) -> Self {
        self.n_repeats = n_repeats;
        self
    }

    /// Calculate permutation importance
    pub fn permutation_importance<F>(
        &self,
        dataset: &Dataset,
        score_fn: F,
    ) -> Result<FeatureImportance>
    where
        F: Fn(&Dataset) -> Result<f64>,
    {
        let n_features = dataset.n_features();
        let mut importances = vec![0.0; n_features];
        let mut std_devs = vec![0.0; n_features];

        // Baseline score
        let baseline_score = score_fn(dataset)?;

        tracing::debug!("Baseline score: {:.4}", baseline_score);

        // For each feature, permute it and measure score drop
        for feature_idx in 0..n_features {
            let mut scores = Vec::with_capacity(self.n_repeats);

            for repeat in 0..self.n_repeats {
                // Create dataset with permuted feature
                let permuted = self.permute_feature(dataset, feature_idx, repeat)?;

                // Calculate score on permuted dataset
                let permuted_score = score_fn(&permuted)?;

                // Importance = baseline - permuted (higher is more important)
                let importance = baseline_score - permuted_score;
                scores.push(importance);
            }

            // Calculate mean and std dev
            let mean = scores.iter().sum::<f64>() / scores.len() as f64;
            let variance = scores
                .iter()
                .map(|s| (s - mean).powi(2))
                .sum::<f64>()
                / scores.len() as f64;
            let std_dev = variance.sqrt();

            importances[feature_idx] = mean;
            std_devs[feature_idx] = std_dev;

            tracing::debug!(
                "Feature {}: importance = {:.4} ± {:.4}",
                dataset.feature_names[feature_idx],
                mean,
                std_dev
            );
        }

        Ok(FeatureImportance {
            scores: importances,
            feature_names: dataset.feature_names.clone(),
            method: ImportanceMethod::Permutation,
            std_devs: Some(std_devs),
        })
    }

    /// Permute a single feature column
    fn permute_feature(
        &self,
        dataset: &Dataset,
        feature_idx: usize,
        repeat: usize,
    ) -> Result<Dataset> {
        let mut features = dataset.features.clone();

        // Extract column
        let mut column: Vec<f64> = (0..features.nrows())
            .map(|i| features[[i, feature_idx]])
            .collect();

        // Shuffle column
        let mut rng = if let Some(seed) = self.random_seed {
            rand::rngs::StdRng::seed_from_u64(seed + repeat as u64)
        } else {
            rand::rngs::StdRng::from_entropy()
        };

        column.shuffle(&mut rng);

        // Put shuffled column back
        for (i, &value) in column.iter().enumerate() {
            features[[i, feature_idx]] = value;
        }

        Dataset::new(features, dataset.labels.clone(), dataset.feature_names.clone())
    }

    /// Calculate drop-column importance
    pub fn drop_column_importance<F>(
        &self,
        dataset: &Dataset,
        score_fn: F,
    ) -> Result<FeatureImportance>
    where
        F: Fn(&Dataset) -> Result<f64>,
    {
        let n_features = dataset.n_features();
        let mut importances = vec![0.0; n_features];

        // Baseline score
        let baseline_score = score_fn(dataset)?;

        // For each feature, drop it and measure score change
        for feature_idx in 0..n_features {
            // Create dataset without this feature
            let reduced = self.drop_feature(dataset, feature_idx)?;

            // Calculate score on reduced dataset
            let reduced_score = score_fn(&reduced)?;

            // Importance = baseline - reduced
            let importance = baseline_score - reduced_score;
            importances[feature_idx] = importance;

            tracing::debug!(
                "Feature {}: importance = {:.4}",
                dataset.feature_names[feature_idx],
                importance
            );
        }

        Ok(FeatureImportance {
            scores: importances,
            feature_names: dataset.feature_names.clone(),
            method: ImportanceMethod::DropColumn,
            std_devs: None,
        })
    }

    /// Drop a single feature from dataset
    fn drop_feature(&self, dataset: &Dataset, feature_idx: usize) -> Result<Dataset> {
        let n_features = dataset.n_features();
        let n_samples = dataset.len();

        // Create new feature matrix without the specified column
        let mut new_features = Vec::with_capacity(n_samples * (n_features - 1));

        for i in 0..n_samples {
            for j in 0..n_features {
                if j != feature_idx {
                    new_features.push(dataset.features[[i, j]]);
                }
            }
        }

        let features_array =
            Array2::from_shape_vec((n_samples, n_features - 1), new_features)
                .map_err(|e| MlError::Dataset(e.to_string()))?;

        // Create new feature names
        let feature_names: Vec<String> = dataset
            .feature_names
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != feature_idx)
            .map(|(_, name)| name.clone())
            .collect();

        Dataset::new(features_array, dataset.labels.clone(), feature_names)
    }
}

impl FeatureImportance {
    /// Get top N most important features
    pub fn top_features(&self, n: usize) -> Vec<(String, f64)> {
        let mut indexed: Vec<(usize, &str, f64)> = self
            .feature_names
            .iter()
            .enumerate()
            .map(|(i, name)| (i, name.as_str(), self.scores[i]))
            .collect();

        indexed.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());

        indexed
            .into_iter()
            .take(n)
            .map(|(_, name, score)| (name.to_string(), score))
            .collect()
    }

    /// Normalize importance scores to sum to 1.0
    pub fn normalize(&mut self) {
        let total: f64 = self.scores.iter().map(|s| s.abs()).sum();

        if total > 1e-10 {
            for score in &mut self.scores {
                *score /= total;
            }

            if let Some(stds) = &mut self.std_devs {
                for std in stds {
                    *std /= total;
                }
            }
        }
    }

    /// Convert to explanation format
    pub fn to_explanation(&self, feature_values: &Array1<f64>, prediction: f64) -> Result<Explanation> {
        if feature_values.len() != self.scores.len() {
            return Err(MlError::InvalidInput(
                "Feature values length mismatch".to_string(),
            ));
        }

        let total_importance: f64 = self.scores.iter().map(|s| s.abs()).sum();

        let contributions: Vec<FeatureContribution> = self
            .feature_names
            .iter()
            .zip(feature_values.iter())
            .zip(self.scores.iter())
            .map(|((name, &value), &importance)| {
                FeatureContribution::new(name.clone(), value, importance, total_importance)
            })
            .collect();

        Ok(Explanation::new(
            contributions,
            0.0, // No base value for feature importance
            prediction,
            ExplanationMethod::Permutation,
        ))
    }

    /// Print feature importance
    pub fn print(&self) {
        println!("\n{:?} Feature Importance", self.method);
        println!("========================");

        let mut indexed: Vec<_> = self
            .feature_names
            .iter()
            .enumerate()
            .map(|(i, name)| (i, name, self.scores[i]))
            .collect();

        indexed.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());

        for (i, (idx, name, score)) in indexed.iter().enumerate() {
            let std_str = if let Some(stds) = &self.std_devs {
                format!(" ± {:.4}", stds[*idx])
            } else {
                String::new()
            };

            println!("{}. {:20} {:.4}{}", i + 1, name, score, std_str);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::Array1;

    fn create_test_dataset() -> Dataset {
        let features = Array2::from_shape_vec(
            (100, 3),
            (0..300).map(|x| x as f64).collect(),
        )
        .unwrap();

        let labels = Array1::from_vec((0..100).map(|x| (x % 2) as f64).collect());
        let feature_names = vec!["a".to_string(), "b".to_string(), "c".to_string()];

        Dataset::new(features, labels, feature_names).unwrap()
    }

    #[test]
    fn test_feature_importance_creation() {
        let importance = FeatureImportance {
            scores: vec![0.5, 0.3, 0.2],
            feature_names: vec!["a".to_string(), "b".to_string(), "c".to_string()],
            method: ImportanceMethod::Permutation,
            std_devs: Some(vec![0.05, 0.03, 0.02]),
        };

        assert_eq!(importance.scores.len(), 3);
        assert_eq!(importance.feature_names.len(), 3);
    }

    #[test]
    fn test_top_features() {
        let importance = FeatureImportance {
            scores: vec![0.5, 0.3, 0.8, 0.2],
            feature_names: vec![
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "d".to_string(),
            ],
            method: ImportanceMethod::Permutation,
            std_devs: None,
        };

        let top_2 = importance.top_features(2);
        assert_eq!(top_2.len(), 2);
        assert_eq!(top_2[0].0, "c");
        assert_eq!(top_2[1].0, "a");
    }

    #[test]
    fn test_normalize() {
        let mut importance = FeatureImportance {
            scores: vec![2.0, 3.0, 5.0],
            feature_names: vec!["a".to_string(), "b".to_string(), "c".to_string()],
            method: ImportanceMethod::Permutation,
            std_devs: None,
        };

        importance.normalize();

        let sum: f64 = importance.scores.iter().sum();
        assert!((sum - 1.0).abs() < 1e-6);
    }
}
