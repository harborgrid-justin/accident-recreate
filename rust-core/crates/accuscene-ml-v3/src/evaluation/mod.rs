//! Model evaluation metrics and analysis

pub mod confusion_matrix;
pub mod metrics;

use crate::error::{MlError, Result};
use ndarray::Array1;
use serde::{Deserialize, Serialize};

/// Evaluation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResults {
    /// Classification metrics (if applicable)
    pub classification: Option<ClassificationMetrics>,

    /// Regression metrics (if applicable)
    pub regression: Option<RegressionMetrics>,

    /// Confusion matrix (for classification)
    pub confusion_matrix: Option<confusion_matrix::ConfusionMatrix>,

    /// ROC AUC score (for binary classification)
    pub roc_auc: Option<f64>,

    /// Number of samples evaluated
    pub n_samples: usize,
}

/// Classification metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationMetrics {
    /// Accuracy
    pub accuracy: f64,

    /// Precision (macro-averaged)
    pub precision: f64,

    /// Recall (macro-averaged)
    pub recall: f64,

    /// F1 score (macro-averaged)
    pub f1_score: f64,

    /// Per-class metrics
    pub per_class: Vec<PerClassMetrics>,
}

/// Per-class metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerClassMetrics {
    pub class_label: String,
    pub precision: f64,
    pub recall: f64,
    pub f1_score: f64,
    pub support: usize,
}

/// Regression metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionMetrics {
    /// Mean Absolute Error
    pub mae: f64,

    /// Mean Squared Error
    pub mse: f64,

    /// Root Mean Squared Error
    pub rmse: f64,

    /// R² score (coefficient of determination)
    pub r2_score: f64,

    /// Mean Absolute Percentage Error
    pub mape: f64,

    /// Median Absolute Error
    pub median_ae: f64,
}

// Re-export submodules
pub use confusion_matrix::ConfusionMatrix;
pub use metrics::{
    accuracy, f1_score, mae, mape, mse, precision, r2_score, recall, rmse,
};

impl EvaluationResults {
    /// Create new evaluation results for classification
    pub fn new_classification(
        y_true: &Array1<f64>,
        y_pred: &Array1<f64>,
        class_labels: Option<Vec<String>>,
    ) -> Result<Self> {
        let n_samples = y_true.len();

        let accuracy = metrics::accuracy(y_true, y_pred)?;
        let precision = metrics::precision(y_true, y_pred)?;
        let recall = metrics::recall(y_true, y_pred)?;
        let f1 = metrics::f1_score(y_true, y_pred)?;

        let cm = ConfusionMatrix::from_predictions(y_true, y_pred, class_labels.clone())?;

        Ok(Self {
            classification: Some(ClassificationMetrics {
                accuracy,
                precision,
                recall,
                f1_score: f1,
                per_class: vec![], // Would be calculated from confusion matrix
            }),
            regression: None,
            confusion_matrix: Some(cm),
            roc_auc: None,
            n_samples,
        })
    }

    /// Create new evaluation results for regression
    pub fn new_regression(y_true: &Array1<f64>, y_pred: &Array1<f64>) -> Result<Self> {
        let n_samples = y_true.len();

        let mae_val = metrics::mae(y_true, y_pred)?;
        let mse_val = metrics::mse(y_true, y_pred)?;
        let rmse_val = metrics::rmse(y_true, y_pred)?;
        let r2_val = metrics::r2_score(y_true, y_pred)?;
        let mape_val = metrics::mape(y_true, y_pred)?;

        // Calculate median absolute error
        let mut abs_errors: Vec<f64> = y_true
            .iter()
            .zip(y_pred.iter())
            .map(|(t, p)| (t - p).abs())
            .collect();
        abs_errors.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median_ae = abs_errors[abs_errors.len() / 2];

        Ok(Self {
            classification: None,
            regression: Some(RegressionMetrics {
                mae: mae_val,
                mse: mse_val,
                rmse: rmse_val,
                r2_score: r2_val,
                mape: mape_val,
                median_ae,
            }),
            confusion_matrix: None,
            roc_auc: None,
            n_samples,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classification_evaluation() {
        let y_true = Array1::from_vec(vec![0.0, 0.0, 1.0, 1.0, 1.0]);
        let y_pred = Array1::from_vec(vec![0.0, 0.0, 1.0, 0.0, 1.0]);

        let results = EvaluationResults::new_classification(&y_true, &y_pred, None).unwrap();

        assert!(results.classification.is_some());
        let metrics = results.classification.unwrap();
        assert!((metrics.accuracy - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_regression_evaluation() {
        let y_true = Array1::from_vec(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        let y_pred = Array1::from_vec(vec![1.1, 2.1, 2.9, 4.2, 4.8]);

        let results = EvaluationResults::new_regression(&y_true, &y_pred).unwrap();

        assert!(results.regression.is_some());
        let metrics = results.regression.unwrap();
        assert!(metrics.mae < 0.3);
        assert!(metrics.r2_score > 0.95);
    }
}
