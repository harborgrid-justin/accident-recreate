//! Confusion matrix analysis

use crate::error::{MlError, Result};
use ndarray::{Array1, Array2};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Confusion matrix for classification evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfusionMatrix {
    /// The matrix itself (rows = true labels, cols = predicted labels)
    pub matrix: Array2<usize>,

    /// Class labels
    pub labels: Vec<String>,

    /// Total number of samples
    pub total_samples: usize,
}

impl ConfusionMatrix {
    /// Create a confusion matrix from predictions
    pub fn from_predictions(
        y_true: &Array1<f64>,
        y_pred: &Array1<f64>,
        class_labels: Option<Vec<String>>,
    ) -> Result<Self> {
        if y_true.len() != y_pred.len() {
            return Err(MlError::Evaluation(
                "True and predicted labels have different lengths".to_string(),
            ));
        }

        // Get unique classes
        let mut classes: Vec<f64> = y_true.iter().chain(y_pred.iter()).copied().collect();
        classes.sort_by(|a, b| a.partial_cmp(b).unwrap());
        classes.dedup();

        let n_classes = classes.len();

        // Create index mapping
        let class_to_idx: HashMap<i32, usize> = classes
            .iter()
            .enumerate()
            .map(|(idx, &class)| (class.round() as i32, idx))
            .collect();

        // Initialize matrix
        let mut matrix = Array2::zeros((n_classes, n_classes));

        // Fill matrix
        for (true_label, pred_label) in y_true.iter().zip(y_pred.iter()) {
            let true_idx = class_to_idx[&(true_label.round() as i32)];
            let pred_idx = class_to_idx[&(pred_label.round() as i32)];
            matrix[[true_idx, pred_idx]] += 1;
        }

        // Generate labels
        let labels = if let Some(labels) = class_labels {
            labels
        } else {
            classes.iter().map(|c| format!("Class {}", c)).collect()
        };

        Ok(Self {
            matrix,
            labels,
            total_samples: y_true.len(),
        })
    }

    /// Get accuracy
    pub fn accuracy(&self) -> f64 {
        let correct: usize = (0..self.matrix.nrows())
            .map(|i| self.matrix[[i, i]])
            .sum();

        correct as f64 / self.total_samples as f64
    }

    /// Get precision for a specific class
    pub fn precision(&self, class_idx: usize) -> f64 {
        if class_idx >= self.matrix.ncols() {
            return 0.0;
        }

        let tp = self.matrix[[class_idx, class_idx]];
        let col_sum: usize = (0..self.matrix.nrows())
            .map(|i| self.matrix[[i, class_idx]])
            .sum();

        if col_sum == 0 {
            return 0.0;
        }

        tp as f64 / col_sum as f64
    }

    /// Get recall for a specific class
    pub fn recall(&self, class_idx: usize) -> f64 {
        if class_idx >= self.matrix.nrows() {
            return 0.0;
        }

        let tp = self.matrix[[class_idx, class_idx]];
        let row_sum: usize = (0..self.matrix.ncols())
            .map(|j| self.matrix[[class_idx, j]])
            .sum();

        if row_sum == 0 {
            return 0.0;
        }

        tp as f64 / row_sum as f64
    }

    /// Get F1 score for a specific class
    pub fn f1_score(&self, class_idx: usize) -> f64 {
        let prec = self.precision(class_idx);
        let rec = self.recall(class_idx);

        if prec + rec == 0.0 {
            return 0.0;
        }

        2.0 * prec * rec / (prec + rec)
    }

    /// Get macro-averaged precision
    pub fn macro_precision(&self) -> f64 {
        let sum: f64 = (0..self.matrix.ncols())
            .map(|i| self.precision(i))
            .sum();

        sum / self.matrix.ncols() as f64
    }

    /// Get macro-averaged recall
    pub fn macro_recall(&self) -> f64 {
        let sum: f64 = (0..self.matrix.nrows())
            .map(|i| self.recall(i))
            .sum();

        sum / self.matrix.nrows() as f64
    }

    /// Get macro-averaged F1 score
    pub fn macro_f1(&self) -> f64 {
        let sum: f64 = (0..self.matrix.nrows())
            .map(|i| self.f1_score(i))
            .sum();

        sum / self.matrix.nrows() as f64
    }

    /// Get support (number of samples) for each class
    pub fn support(&self) -> Vec<usize> {
        (0..self.matrix.nrows())
            .map(|i| {
                (0..self.matrix.ncols())
                    .map(|j| self.matrix[[i, j]])
                    .sum()
            })
            .collect()
    }

    /// Normalize confusion matrix by row (true label)
    pub fn normalize(&self) -> Array2<f64> {
        let mut normalized = Array2::zeros((self.matrix.nrows(), self.matrix.ncols()));

        for i in 0..self.matrix.nrows() {
            let row_sum: usize = (0..self.matrix.ncols())
                .map(|j| self.matrix[[i, j]])
                .sum();

            if row_sum > 0 {
                for j in 0..self.matrix.ncols() {
                    normalized[[i, j]] = self.matrix[[i, j]] as f64 / row_sum as f64;
                }
            }
        }

        normalized
    }

    /// Print confusion matrix in a readable format
    pub fn print(&self) {
        println!("\nConfusion Matrix:");
        println!("==================");

        // Print header
        print!("        ");
        for label in &self.labels {
            print!("{:>8} ", label);
        }
        println!();

        // Print matrix
        for (i, true_label) in self.labels.iter().enumerate() {
            print!("{:>8} ", true_label);
            for j in 0..self.matrix.ncols() {
                print!("{:>8} ", self.matrix[[i, j]]);
            }
            println!();
        }

        println!("\nMetrics:");
        println!("--------");
        println!("Accuracy:          {:.4}", self.accuracy());
        println!("Macro Precision:   {:.4}", self.macro_precision());
        println!("Macro Recall:      {:.4}", self.macro_recall());
        println!("Macro F1:          {:.4}", self.macro_f1());

        println!("\nPer-class Metrics:");
        println!("------------------");
        for (i, label) in self.labels.iter().enumerate() {
            println!(
                "{}: Precision={:.4}, Recall={:.4}, F1={:.4}, Support={}",
                label,
                self.precision(i),
                self.recall(i),
                self.f1_score(i),
                self.support()[i]
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confusion_matrix_binary() {
        let y_true = Array1::from_vec(vec![0.0, 0.0, 1.0, 1.0, 1.0]);
        let y_pred = Array1::from_vec(vec![0.0, 0.0, 1.0, 0.0, 1.0]);

        let cm = ConfusionMatrix::from_predictions(&y_true, &y_pred, None).unwrap();

        assert_eq!(cm.matrix[[0, 0]], 2); // TN
        assert_eq!(cm.matrix[[0, 1]], 0); // FP
        assert_eq!(cm.matrix[[1, 0]], 1); // FN
        assert_eq!(cm.matrix[[1, 1]], 2); // TP

        assert!((cm.accuracy() - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_confusion_matrix_multiclass() {
        let y_true = Array1::from_vec(vec![0.0, 1.0, 2.0, 1.0, 2.0, 0.0]);
        let y_pred = Array1::from_vec(vec![0.0, 1.0, 2.0, 2.0, 2.0, 1.0]);

        let cm = ConfusionMatrix::from_predictions(&y_true, &y_pred, None).unwrap();

        assert_eq!(cm.matrix.nrows(), 3);
        assert_eq!(cm.matrix.ncols(), 3);
        assert_eq!(cm.total_samples, 6);
    }

    #[test]
    fn test_metrics() {
        let y_true = Array1::from_vec(vec![0.0, 0.0, 1.0, 1.0, 1.0]);
        let y_pred = Array1::from_vec(vec![0.0, 0.0, 1.0, 0.0, 1.0]);

        let cm = ConfusionMatrix::from_predictions(&y_true, &y_pred, None).unwrap();

        let precision_1 = cm.precision(1);
        let recall_1 = cm.recall(1);
        let f1_1 = cm.f1_score(1);

        assert!(precision_1 > 0.0);
        assert!(recall_1 > 0.0);
        assert!(f1_1 > 0.0);
    }
}
