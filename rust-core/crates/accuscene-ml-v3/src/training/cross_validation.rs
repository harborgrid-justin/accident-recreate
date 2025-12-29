//! Cross-validation utilities

use crate::error::{MlError, Result};
use crate::training::Dataset;
use serde::{Deserialize, Serialize};

/// K-Fold cross-validation splitter
#[derive(Debug, Clone)]
pub struct KFold {
    n_splits: usize,
    shuffle: bool,
    random_seed: Option<u64>,
}

/// Cross-validation fold
#[derive(Debug, Clone)]
pub struct Fold {
    pub train_indices: Vec<usize>,
    pub val_indices: Vec<usize>,
    pub fold_number: usize,
}

/// Cross-validation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CVResults {
    /// Scores for each fold
    pub fold_scores: Vec<f64>,

    /// Mean score across folds
    pub mean_score: f64,

    /// Standard deviation of scores
    pub std_score: f64,

    /// Best fold
    pub best_fold: usize,

    /// Worst fold
    pub worst_fold: usize,
}

impl KFold {
    /// Create a new K-Fold cross-validator
    pub fn new(n_splits: usize) -> Result<Self> {
        if n_splits < 2 {
            return Err(MlError::Training("K-Fold requires at least 2 splits".to_string()));
        }

        Ok(Self {
            n_splits,
            shuffle: true,
            random_seed: Some(42),
        })
    }

    /// Set whether to shuffle data
    pub fn with_shuffle(mut self, shuffle: bool) -> Self {
        self.shuffle = shuffle;
        self
    }

    /// Set random seed
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.random_seed = Some(seed);
        self
    }

    /// Generate folds for a dataset
    pub fn split(&self, dataset: &Dataset) -> Result<Vec<Fold>> {
        let n_samples = dataset.len();

        if n_samples < self.n_splits {
            return Err(MlError::Training(format!(
                "Cannot split {} samples into {} folds",
                n_samples, self.n_splits
            )));
        }

        // Create shuffled indices if needed
        let indices: Vec<usize> = if self.shuffle {
            use rand::seq::SliceRandom;
            use rand::SeedableRng;

            let mut indices: Vec<usize> = (0..n_samples).collect();

            if let Some(seed) = self.random_seed {
                let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
                indices.shuffle(&mut rng);
            }

            indices
        } else {
            (0..n_samples).collect()
        };

        // Calculate fold sizes
        let fold_size = n_samples / self.n_splits;
        let remainder = n_samples % self.n_splits;

        let mut folds = Vec::with_capacity(self.n_splits);

        for fold_idx in 0..self.n_splits {
            // Calculate start and end for validation set
            let val_start = fold_idx * fold_size + fold_idx.min(remainder);
            let val_end = val_start + fold_size + if fold_idx < remainder { 1 } else { 0 };

            let val_indices: Vec<usize> = indices[val_start..val_end].to_vec();
            let train_indices: Vec<usize> = indices[..val_start]
                .iter()
                .chain(indices[val_end..].iter())
                .copied()
                .collect();

            folds.push(Fold {
                train_indices,
                val_indices,
                fold_number: fold_idx + 1,
            });
        }

        Ok(folds)
    }

    /// Get number of splits
    pub fn n_splits(&self) -> usize {
        self.n_splits
    }
}

/// Cross-validator for evaluating models
pub struct CrossValidator {
    kfold: KFold,
}

impl CrossValidator {
    /// Create a new cross-validator
    pub fn new(n_splits: usize) -> Result<Self> {
        Ok(Self {
            kfold: KFold::new(n_splits)?,
        })
    }

    /// Validate a model using cross-validation
    pub fn validate<F>(&self, dataset: &Dataset, score_fn: F) -> Result<CVResults>
    where
        F: Fn(&Dataset, &Dataset) -> Result<f64>,
    {
        let folds = self.kfold.split(dataset)?;
        let mut fold_scores = Vec::with_capacity(folds.len());

        for fold in &folds {
            let train_set = dataset.subset(&fold.train_indices)?;
            let val_set = dataset.subset(&fold.val_indices)?;

            let score = score_fn(&train_set, &val_set)?;
            fold_scores.push(score);

            tracing::debug!("Fold {}: score = {:.4}", fold.fold_number, score);
        }

        let mean_score = fold_scores.iter().sum::<f64>() / fold_scores.len() as f64;

        let variance = fold_scores
            .iter()
            .map(|s| (s - mean_score).powi(2))
            .sum::<f64>()
            / fold_scores.len() as f64;

        let std_score = variance.sqrt();

        let best_fold = fold_scores
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(idx, _)| idx)
            .unwrap_or(0);

        let worst_fold = fold_scores
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(idx, _)| idx)
            .unwrap_or(0);

        Ok(CVResults {
            fold_scores,
            mean_score,
            std_score,
            best_fold,
            worst_fold,
        })
    }
}

impl CVResults {
    /// Get the minimum score
    pub fn min_score(&self) -> f64 {
        self.fold_scores
            .iter()
            .copied()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0)
    }

    /// Get the maximum score
    pub fn max_score(&self) -> f64 {
        self.fold_scores
            .iter()
            .copied()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0)
    }

    /// Get confidence interval (mean ± 1.96 * std_dev for 95% CI)
    pub fn confidence_interval(&self) -> (f64, f64) {
        let margin = 1.96 * self.std_score / (self.fold_scores.len() as f64).sqrt();
        (self.mean_score - margin, self.mean_score + margin)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::{Array1, Array2};

    fn create_test_dataset() -> Dataset {
        let features = Array2::from_shape_vec((100, 5), (0..500).map(|x| x as f64).collect()).unwrap();
        let labels = Array1::from_vec((0..100).map(|x| (x % 2) as f64).collect());
        let feature_names = vec!["a".into(), "b".into(), "c".into(), "d".into(), "e".into()];

        Dataset::new(features, labels, feature_names).unwrap()
    }

    #[test]
    fn test_kfold_creation() {
        let kfold = KFold::new(5).unwrap();
        assert_eq!(kfold.n_splits(), 5);
    }

    #[test]
    fn test_kfold_split() {
        let dataset = create_test_dataset();
        let kfold = KFold::new(5).unwrap();
        let folds = kfold.split(&dataset).unwrap();

        assert_eq!(folds.len(), 5);

        // Check that each sample appears exactly once in validation
        let mut all_val_indices = std::collections::HashSet::new();
        for fold in &folds {
            for &idx in &fold.val_indices {
                assert!(!all_val_indices.contains(&idx), "Duplicate index in validation");
                all_val_indices.insert(idx);
            }
        }

        assert_eq!(all_val_indices.len(), dataset.len());
    }

    #[test]
    fn test_cross_validation() {
        let dataset = create_test_dataset();
        let cv = CrossValidator::new(5).unwrap();

        // Simple scoring function that returns constant score
        let results = cv
            .validate(&dataset, |_train, _val| Ok(0.85))
            .unwrap();

        assert_eq!(results.fold_scores.len(), 5);
        assert!((results.mean_score - 0.85).abs() < 1e-6);
    }
}
