//! Confusion matrix for classification

use ndarray::{Array1, Array2};
use serde::{Deserialize, Serialize};

/// Confusion matrix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfusionMatrix {
    /// Matrix values
    pub matrix: Array2<usize>,

    /// Class labels
    pub labels: Vec<String>,
}

impl ConfusionMatrix {
    /// Compute confusion matrix from predictions
    pub fn compute(y_true: &Array1<f64>, y_pred: &Array1<f64>, n_classes: usize) -> Self {
        let mut matrix = Array2::zeros((n_classes, n_classes));

        for (yt, yp) in y_true.iter().zip(y_pred.iter()) {
            let true_class = (*yt as usize).min(n_classes - 1);
            let pred_class = (*yp as usize).min(n_classes - 1);
            matrix[[true_class, pred_class]] += 1;
        }

        let labels = (0..n_classes).map(|i| i.to_string()).collect();

        Self { matrix, labels }
    }

    /// Get true positives for a class
    pub fn true_positives(&self, class: usize) -> usize {
        self.matrix[[class, class]]
    }

    /// Get false positives for a class
    pub fn false_positives(&self, class: usize) -> usize {
        self.matrix.column(class).sum() - self.matrix[[class, class]]
    }

    /// Get false negatives for a class
    pub fn false_negatives(&self, class: usize) -> usize {
        self.matrix.row(class).sum() - self.matrix[[class, class]]
    }

    /// Get true negatives for a class
    pub fn true_negatives(&self, class: usize) -> usize {
        self.matrix.sum() - self.true_positives(class) - self.false_positives(class) - self.false_negatives(class)
    }

    /// Get overall accuracy
    pub fn accuracy(&self) -> f64 {
        let total: usize = self.matrix.sum();
        if total == 0 {
            return 0.0;
        }

        let correct: usize = (0..self.matrix.nrows())
            .map(|i| self.matrix[[i, i]])
            .sum();

        correct as f64 / total as f64
    }
}
