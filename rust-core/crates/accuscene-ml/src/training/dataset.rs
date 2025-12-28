//! Dataset handling for machine learning

use crate::error::{MLError, Result};
use ndarray::{Array1, Array2};
use rand::prelude::*;
use serde::{Deserialize, Serialize};

/// Machine learning dataset
#[derive(Debug, Clone)]
pub struct Dataset {
    /// Feature matrix (n_samples x n_features)
    pub features: Array2<f64>,

    /// Target vector (n_samples)
    pub targets: Array1<f64>,

    /// Feature names
    pub feature_names: Vec<String>,

    /// Sample weights (optional)
    pub weights: Option<Array1<f64>>,
}

impl Dataset {
    /// Create a new dataset
    pub fn new(features: Array2<f64>, targets: Array1<f64>) -> Result<Self> {
        if features.nrows() != targets.len() {
            return Err(MLError::shape_mismatch(
                format!("{} samples", features.nrows()),
                format!("{} targets", targets.len()),
            ));
        }

        Ok(Self {
            features,
            targets,
            feature_names: Vec::new(),
            weights: None,
        })
    }

    /// Set feature names
    pub fn with_feature_names(mut self, names: Vec<String>) -> Self {
        self.feature_names = names;
        self
    }

    /// Set sample weights
    pub fn with_weights(mut self, weights: Array1<f64>) -> Self {
        self.weights = Some(weights);
        self
    }

    /// Get number of samples
    pub fn num_samples(&self) -> usize {
        self.features.nrows()
    }

    /// Get number of features
    pub fn num_features(&self) -> usize {
        self.features.ncols()
    }

    /// Shuffle the dataset
    pub fn shuffle(&mut self, seed: u64) {
        let mut rng = StdRng::seed_from_u64(seed);
        let mut indices: Vec<usize> = (0..self.num_samples()).collect();
        indices.shuffle(&mut rng);

        let mut new_features = Array2::zeros(self.features.dim());
        let mut new_targets = Array1::zeros(self.targets.len());

        for (new_idx, &old_idx) in indices.iter().enumerate() {
            new_features.row_mut(new_idx).assign(&self.features.row(old_idx));
            new_targets[new_idx] = self.targets[old_idx];
        }

        self.features = new_features;
        self.targets = new_targets;

        if let Some(ref weights) = self.weights {
            let mut new_weights = Array1::zeros(weights.len());
            for (new_idx, &old_idx) in indices.iter().enumerate() {
                new_weights[new_idx] = weights[old_idx];
            }
            self.weights = Some(new_weights);
        }
    }

    /// Get a subset of the dataset
    pub fn subset(&self, indices: &[usize]) -> Result<Self> {
        let mut features = Array2::zeros((indices.len(), self.num_features()));
        let mut targets = Array1::zeros(indices.len());

        for (new_idx, &old_idx) in indices.iter().enumerate() {
            if old_idx >= self.num_samples() {
                return Err(MLError::invalid_input("Index out of bounds"));
            }
            features.row_mut(new_idx).assign(&self.features.row(old_idx));
            targets[new_idx] = self.targets[old_idx];
        }

        let weights = self.weights.as_ref().map(|w| {
            let mut new_weights = Array1::zeros(indices.len());
            for (new_idx, &old_idx) in indices.iter().enumerate() {
                new_weights[new_idx] = w[old_idx];
            }
            new_weights
        });

        Ok(Self {
            features,
            targets,
            feature_names: self.feature_names.clone(),
            weights,
        })
    }
}

/// Dataset builder
pub struct DatasetBuilder {
    features: Option<Array2<f64>>,
    targets: Option<Array1<f64>>,
    feature_names: Vec<String>,
    weights: Option<Array1<f64>>,
}

impl DatasetBuilder {
    /// Create a new dataset builder
    pub fn new() -> Self {
        Self {
            features: None,
            targets: None,
            feature_names: Vec::new(),
            weights: None,
        }
    }

    /// Set features
    pub fn features(mut self, features: Array2<f64>) -> Self {
        self.features = Some(features);
        self
    }

    /// Set targets
    pub fn targets(mut self, targets: Array1<f64>) -> Self {
        self.targets = Some(targets);
        self
    }

    /// Set feature names
    pub fn feature_names(mut self, names: Vec<String>) -> Self {
        self.feature_names = names;
        self
    }

    /// Set weights
    pub fn weights(mut self, weights: Array1<f64>) -> Self {
        self.weights = Some(weights);
        self
    }

    /// Build the dataset
    pub fn build(self) -> Result<Dataset> {
        let features = self.features.ok_or_else(|| MLError::invalid_input("Features not set"))?;
        let targets = self.targets.ok_or_else(|| MLError::invalid_input("Targets not set"))?;

        let mut dataset = Dataset::new(features, targets)?;
        dataset.feature_names = self.feature_names;
        dataset.weights = self.weights;

        Ok(dataset)
    }
}

impl Default for DatasetBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Dataset split (train/test or train/val/test)
#[derive(Debug, Clone)]
pub struct DatasetSplit {
    /// Training dataset
    pub train: Dataset,

    /// Validation dataset (optional)
    pub val: Option<Dataset>,

    /// Test dataset
    pub test: Dataset,
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::{arr1, arr2};

    #[test]
    fn test_dataset_creation() -> Result<()> {
        let features = arr2(&[[1.0, 2.0], [3.0, 4.0]]);
        let targets = arr1(&[0.0, 1.0]);

        let dataset = Dataset::new(features, targets)?;

        assert_eq!(dataset.num_samples(), 2);
        assert_eq!(dataset.num_features(), 2);

        Ok(())
    }

    #[test]
    fn test_dataset_shuffle() -> Result<()> {
        let features = arr2(&[[1.0], [2.0], [3.0]]);
        let targets = arr1(&[1.0, 2.0, 3.0]);

        let mut dataset = Dataset::new(features, targets)?;
        dataset.shuffle(42);

        assert_eq!(dataset.num_samples(), 3);

        Ok(())
    }
}
