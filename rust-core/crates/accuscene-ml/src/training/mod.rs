//! Training framework module

use crate::error::Result;
use ndarray::{Array1, Array2};
use serde::{Deserialize, Serialize};

pub mod cross_validation;
pub mod dataset;
pub mod hyperparameter;
pub mod split;

pub use cross_validation::{CrossValidator, CVResults, KFold, StratifiedKFold};
pub use dataset::{Dataset, DatasetBuilder, DatasetSplit};
pub use hyperparameter::{GridSearch, HyperparameterTuner, ParamGrid, RandomSearch};
pub use split::{TrainTestSplit, ValidationSplit};

/// Training configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingConfig {
    /// Number of epochs
    pub epochs: usize,

    /// Batch size
    pub batch_size: usize,

    /// Learning rate
    pub learning_rate: f64,

    /// Early stopping patience
    pub early_stopping_patience: Option<usize>,

    /// Validation split ratio
    pub validation_split: f64,

    /// Shuffle data
    pub shuffle: bool,

    /// Random seed
    pub random_seed: u64,

    /// Verbose logging
    pub verbose: bool,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            epochs: 100,
            batch_size: 32,
            learning_rate: 0.01,
            early_stopping_patience: Some(10),
            validation_split: 0.2,
            shuffle: true,
            random_seed: 42,
            verbose: true,
        }
    }
}

/// Training history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingHistory {
    /// Training losses per epoch
    pub train_losses: Vec<f64>,

    /// Validation losses per epoch
    pub val_losses: Vec<f64>,

    /// Additional metrics per epoch
    pub metrics: std::collections::HashMap<String, Vec<f64>>,

    /// Best epoch
    pub best_epoch: Option<usize>,

    /// Best validation loss
    pub best_val_loss: Option<f64>,
}

impl TrainingHistory {
    /// Create new training history
    pub fn new() -> Self {
        Self {
            train_losses: Vec::new(),
            val_losses: Vec::new(),
            metrics: std::collections::HashMap::new(),
            best_epoch: None,
            best_val_loss: None,
        }
    }

    /// Add an epoch
    pub fn add_epoch(&mut self, train_loss: f64, val_loss: f64) {
        self.train_losses.push(train_loss);
        self.val_losses.push(val_loss);

        // Update best epoch
        if self.best_val_loss.is_none() || val_loss < self.best_val_loss.unwrap() {
            self.best_val_loss = Some(val_loss);
            self.best_epoch = Some(self.val_losses.len() - 1);
        }
    }

    /// Add a metric value for current epoch
    pub fn add_metric(&mut self, name: impl Into<String>, value: f64) {
        self.metrics
            .entry(name.into())
            .or_insert_with(Vec::new)
            .push(value);
    }

    /// Get number of epochs
    pub fn num_epochs(&self) -> usize {
        self.train_losses.len()
    }
}

impl Default for TrainingHistory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_training_history() {
        let mut history = TrainingHistory::new();

        history.add_epoch(1.0, 0.9);
        history.add_epoch(0.8, 0.75);
        history.add_epoch(0.6, 0.7);

        assert_eq!(history.num_epochs(), 3);
        assert_eq!(history.best_epoch, Some(1));
        assert_eq!(history.best_val_loss, Some(0.75));
    }
}
