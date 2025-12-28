//! Cross-validation utilities

use crate::error::Result;
use crate::training::Dataset;
use rand::prelude::*;
use serde::{Deserialize, Serialize};

/// K-Fold cross-validator
pub struct KFold {
    /// Number of folds
    n_folds: usize,

    /// Random seed
    random_seed: u64,

    /// Shuffle before splitting
    shuffle: bool,
}

impl KFold {
    /// Create a new K-Fold cross-validator
    pub fn new(n_folds: usize) -> Self {
        Self {
            n_folds,
            random_seed: 42,
            shuffle: true,
        }
    }

    /// Set random seed
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.random_seed = seed;
        self
    }

    /// Set shuffle
    pub fn with_shuffle(mut self, shuffle: bool) -> Self {
        self.shuffle = shuffle;
        self
    }

    /// Generate fold indices
    pub fn split(&self, dataset: &Dataset) -> Vec<(Vec<usize>, Vec<usize>)> {
        let n_samples = dataset.num_samples();
        let mut indices: Vec<usize> = (0..n_samples).collect();

        if self.shuffle {
            let mut rng = StdRng::seed_from_u64(self.random_seed);
            indices.shuffle(&mut rng);
        }

        let fold_size = n_samples / self.n_folds;
        let mut folds = Vec::new();

        for i in 0..self.n_folds {
            let test_start = i * fold_size;
            let test_end = if i == self.n_folds - 1 {
                n_samples
            } else {
                (i + 1) * fold_size
            };

            let test_indices: Vec<usize> = indices[test_start..test_end].to_vec();
            let train_indices: Vec<usize> = indices[..test_start]
                .iter()
                .chain(indices[test_end..].iter())
                .copied()
                .collect();

            folds.push((train_indices, test_indices));
        }

        folds
    }
}

/// Stratified K-Fold cross-validator (for classification)
pub struct StratifiedKFold {
    /// Number of folds
    n_folds: usize,

    /// Random seed
    random_seed: u64,

    /// Shuffle before splitting
    shuffle: bool,
}

impl StratifiedKFold {
    /// Create a new stratified K-Fold cross-validator
    pub fn new(n_folds: usize) -> Self {
        Self {
            n_folds,
            random_seed: 42,
            shuffle: true,
        }
    }

    /// Set random seed
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.random_seed = seed;
        self
    }

    /// Set shuffle
    pub fn with_shuffle(mut self, shuffle: bool) -> Self {
        self.shuffle = shuffle;
        self
    }

    /// Generate stratified fold indices
    pub fn split(&self, dataset: &Dataset) -> Vec<(Vec<usize>, Vec<usize>)> {
        // Simplified: falls back to regular K-Fold
        // In production, implement proper stratification
        KFold::new(self.n_folds)
            .with_seed(self.random_seed)
            .with_shuffle(self.shuffle)
            .split(dataset)
    }
}

/// Cross-validator trait
pub trait CrossValidator {
    /// Split dataset into folds
    fn split(&self, dataset: &Dataset) -> Vec<(Vec<usize>, Vec<usize>)>;

    /// Get number of folds
    fn num_folds(&self) -> usize;
}

impl CrossValidator for KFold {
    fn split(&self, dataset: &Dataset) -> Vec<(Vec<usize>, Vec<usize>)> {
        self.split(dataset)
    }

    fn num_folds(&self) -> usize {
        self.n_folds
    }
}

impl CrossValidator for StratifiedKFold {
    fn split(&self, dataset: &Dataset) -> Vec<(Vec<usize>, Vec<usize>)> {
        self.split(dataset)
    }

    fn num_folds(&self) -> usize {
        self.n_folds
    }
}

/// Cross-validation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CVResults {
    /// Scores for each fold
    pub fold_scores: Vec<f64>,

    /// Mean score
    pub mean_score: f64,

    /// Standard deviation
    pub std_score: f64,

    /// Best fold index
    pub best_fold: usize,

    /// Worst fold index
    pub worst_fold: usize,
}

impl CVResults {
    /// Create CV results from fold scores
    pub fn from_scores(fold_scores: Vec<f64>) -> Self {
        let n = fold_scores.len() as f64;
        let mean_score = fold_scores.iter().sum::<f64>() / n;

        let variance = fold_scores
            .iter()
            .map(|score| (score - mean_score).powi(2))
            .sum::<f64>()
            / n;

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

        Self {
            fold_scores,
            mean_score,
            std_score,
            best_fold,
            worst_fold,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::{arr1, arr2};

    #[test]
    fn test_kfold() -> Result<()> {
        let features = arr2(&[[1.0], [2.0], [3.0], [4.0], [5.0]]);
        let targets = arr1(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        let dataset = Dataset::new(features, targets)?;

        let kfold = KFold::new(5);
        let folds = kfold.split(&dataset);

        assert_eq!(folds.len(), 5);

        Ok(())
    }

    #[test]
    fn test_cv_results() {
        let scores = vec![0.8, 0.85, 0.82, 0.88, 0.79];
        let results = CVResults::from_scores(scores);

        assert!(results.mean_score > 0.8);
        assert!(results.std_score > 0.0);
    }
}
