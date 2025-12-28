//! Evaluation metrics

use ndarray::Array1;
use serde::{Deserialize, Serialize};

/// Mean Squared Error
pub fn mse(y_true: &Array1<f64>, y_pred: &Array1<f64>) -> f64 {
    (y_true - y_pred).mapv(|x| x * x).mean().unwrap_or(0.0)
}

/// Root Mean Squared Error
pub fn rmse(y_true: &Array1<f64>, y_pred: &Array1<f64>) -> f64 {
    mse(y_true, y_pred).sqrt()
}

/// Mean Absolute Error
pub fn mae(y_true: &Array1<f64>, y_pred: &Array1<f64>) -> f64 {
    (y_true - y_pred).mapv(|x| x.abs()).mean().unwrap_or(0.0)
}

/// R² Score
pub fn r2_score(y_true: &Array1<f64>, y_pred: &Array1<f64>) -> f64 {
    let mean = y_true.mean().unwrap_or(0.0);
    let ss_tot: f64 = y_true.iter().map(|&y| (y - mean).powi(2)).sum();
    let ss_res: f64 = y_true.iter().zip(y_pred.iter()).map(|(&yt, &yp)| (yt - yp).powi(2)).sum();

    if ss_tot == 0.0 {
        return 0.0;
    }
    1.0 - (ss_res / ss_tot)
}

/// Accuracy score for classification
pub fn accuracy(y_true: &Array1<f64>, y_pred: &Array1<f64>) -> f64 {
    let correct: usize = y_true.iter().zip(y_pred.iter()).filter(|(a, b)| (a - b).abs() < 0.5).count();
    correct as f64 / y_true.len() as f64
}

/// Precision score
pub fn precision(y_true: &Array1<f64>, y_pred: &Array1<f64>) -> f64 {
    let tp = y_true.iter().zip(y_pred.iter())
        .filter(|(&&yt, &&yp)| yt > 0.5 && yp > 0.5).count() as f64;
    let fp = y_true.iter().zip(y_pred.iter())
        .filter(|(&&yt, &&yp)| yt < 0.5 && yp > 0.5).count() as f64;

    if tp + fp == 0.0 { 0.0 } else { tp / (tp + fp) }
}

/// Recall score
pub fn recall(y_true: &Array1<f64>, y_pred: &Array1<f64>) -> f64 {
    let tp = y_true.iter().zip(y_pred.iter())
        .filter(|(&&yt, &&yp)| yt > 0.5 && yp > 0.5).count() as f64;
    let fn_ = y_true.iter().zip(y_pred.iter())
        .filter(|(&&yt, &&yp)| yt > 0.5 && yp < 0.5).count() as f64;

    if tp + fn_ == 0.0 { 0.0 } else { tp / (tp + fn_) }
}

/// F1 score
pub fn f1_score(y_true: &Array1<f64>, y_pred: &Array1<f64>) -> f64 {
    let p = precision(y_true, y_pred);
    let r = recall(y_true, y_pred);

    if p + r == 0.0 { 0.0 } else { 2.0 * (p * r) / (p + r) }
}

/// Regression metrics collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionMetrics {
    pub mse: f64,
    pub rmse: f64,
    pub mae: f64,
    pub r2: f64,
}

impl RegressionMetrics {
    pub fn compute(y_true: &Array1<f64>, y_pred: &Array1<f64>) -> Self {
        Self {
            mse: mse(y_true, y_pred),
            rmse: rmse(y_true, y_pred),
            mae: mae(y_true, y_pred),
            r2: r2_score(y_true, y_pred),
        }
    }
}

/// Classification metrics collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationMetrics {
    pub accuracy: f64,
    pub precision: f64,
    pub recall: f64,
    pub f1: f64,
}

impl ClassificationMetrics {
    pub fn compute(y_true: &Array1<f64>, y_pred: &Array1<f64>) -> Self {
        Self {
            accuracy: accuracy(y_true, y_pred),
            precision: precision(y_true, y_pred),
            recall: recall(y_true, y_pred),
            f1: f1_score(y_true, y_pred),
        }
    }
}
