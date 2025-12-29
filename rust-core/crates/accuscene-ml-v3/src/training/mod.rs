//! Training utilities for ML models

pub mod cross_validation;
pub mod dataset;
pub mod hyperparameter;

use crate::error::{MlError, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Training configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingConfig {
    /// Learning rate
    pub learning_rate: f64,

    /// Number of epochs
    pub epochs: usize,

    /// Batch size
    pub batch_size: usize,

    /// Validation split
    pub validation_split: f32,

    /// Early stopping patience
    pub early_stopping_patience: usize,

    /// Random seed
    pub random_seed: Option<u64>,
}

/// Training history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingHistory {
    /// Training loss per epoch
    pub train_loss: Vec<f64>,

    /// Validation loss per epoch
    pub val_loss: Vec<f64>,

    /// Training metrics per epoch
    pub train_metrics: Vec<TrainingMetrics>,

    /// Validation metrics per epoch
    pub val_metrics: Vec<TrainingMetrics>,

    /// Total training time
    pub training_duration: Duration,

    /// Best epoch
    pub best_epoch: usize,

    /// Best validation loss
    pub best_val_loss: f64,
}

/// Training metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingMetrics {
    pub accuracy: f64,
    pub precision: f64,
    pub recall: f64,
    pub f1_score: f64,
}

// Re-export submodules
pub use cross_validation::{CrossValidator, KFold};
pub use dataset::{Dataset, DatasetSplit};
pub use hyperparameter::{HyperparameterTuner, ParameterGrid};

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            learning_rate: 0.001,
            epochs: 100,
            batch_size: 32,
            validation_split: 0.2,
            early_stopping_patience: 10,
            random_seed: Some(42),
        }
    }
}

impl TrainingHistory {
    /// Create a new training history
    pub fn new() -> Self {
        Self {
            train_loss: Vec::new(),
            val_loss: Vec::new(),
            train_metrics: Vec::new(),
            val_metrics: Vec::new(),
            training_duration: Duration::from_secs(0),
            best_epoch: 0,
            best_val_loss: f64::INFINITY,
        }
    }

    /// Add epoch results
    pub fn add_epoch(
        &mut self,
        train_loss: f64,
        val_loss: f64,
        train_metrics: TrainingMetrics,
        val_metrics: TrainingMetrics,
    ) {
        self.train_loss.push(train_loss);
        self.val_loss.push(val_loss);
        self.train_metrics.push(train_metrics);
        self.val_metrics.push(val_metrics);

        // Update best epoch if validation loss improved
        if val_loss < self.best_val_loss {
            self.best_val_loss = val_loss;
            self.best_epoch = self.val_loss.len() - 1;
        }
    }

    /// Get number of epochs
    pub fn num_epochs(&self) -> usize {
        self.train_loss.len()
    }

    /// Check if training should stop early
    pub fn should_stop_early(&self, patience: usize) -> bool {
        if self.num_epochs() < patience {
            return false;
        }

        let epochs_since_best = self.num_epochs() - 1 - self.best_epoch;
        epochs_since_best >= patience
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

        history.add_epoch(
            0.5,
            0.6,
            TrainingMetrics {
                accuracy: 0.8,
                precision: 0.8,
                recall: 0.8,
                f1_score: 0.8,
            },
            TrainingMetrics {
                accuracy: 0.75,
                precision: 0.75,
                recall: 0.75,
                f1_score: 0.75,
            },
        );

        assert_eq!(history.num_epochs(), 1);
        assert_eq!(history.best_val_loss, 0.6);
    }

    #[test]
    fn test_early_stopping() {
        let mut history = TrainingHistory::new();

        for i in 0..15 {
            history.add_epoch(
                0.5,
                0.6 + i as f64 * 0.01, // Validation loss getting worse
                TrainingMetrics {
                    accuracy: 0.8,
                    precision: 0.8,
                    recall: 0.8,
                    f1_score: 0.8,
                },
                TrainingMetrics {
                    accuracy: 0.75,
                    precision: 0.75,
                    recall: 0.75,
                    f1_score: 0.75,
                },
            );
        }

        assert!(history.should_stop_early(10));
        assert!(!history.should_stop_early(20));
    }
}
