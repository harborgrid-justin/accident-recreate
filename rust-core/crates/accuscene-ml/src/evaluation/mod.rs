//! Model evaluation module

use ndarray::{Array1, Array2};
use serde::{Deserialize, Serialize};

pub mod confusion;
pub mod metrics;

pub use confusion::ConfusionMatrix;
pub use metrics::{
    accuracy, f1_score, mae, mse, precision, r2_score, recall, rmse, ClassificationMetrics,
    RegressionMetrics,
};

/// Evaluation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResults {
    pub metrics: std::collections::HashMap<String, f64>,
    pub confusion_matrix: Option<ConfusionMatrix>,
}

impl EvaluationResults {
    pub fn new() -> Self {
        Self {
            metrics: std::collections::HashMap::new(),
            confusion_matrix: None,
        }
    }

    pub fn add_metric(&mut self, name: impl Into<String>, value: f64) {
        self.metrics.insert(name.into(), value);
    }
}

impl Default for EvaluationResults {
    fn default() -> Self {
        Self::new()
    }
}
