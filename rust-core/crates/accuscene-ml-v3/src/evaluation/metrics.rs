//! Core evaluation metrics

use crate::error::{MlError, Result};
use ndarray::Array1;

/// Calculate accuracy
pub fn accuracy(y_true: &Array1<f64>, y_pred: &Array1<f64>) -> Result<f64> {
    validate_inputs(y_true, y_pred)?;

    let correct = y_true
        .iter()
        .zip(y_pred.iter())
        .filter(|(t, p)| (t - p).abs() < 1e-6)
        .count();

    Ok(correct as f64 / y_true.len() as f64)
}

/// Calculate precision (macro-averaged for multi-class)
pub fn precision(y_true: &Array1<f64>, y_pred: &Array1<f64>) -> Result<f64> {
    validate_inputs(y_true, y_pred)?;

    let classes = get_unique_classes(y_true);
    let mut precisions = Vec::new();

    for &class in &classes {
        let tp = true_positives(y_true, y_pred, class);
        let fp = false_positives(y_true, y_pred, class);

        let prec = if tp + fp > 0 {
            tp as f64 / (tp + fp) as f64
        } else {
            0.0
        };

        precisions.push(prec);
    }

    Ok(precisions.iter().sum::<f64>() / precisions.len() as f64)
}

/// Calculate recall (macro-averaged for multi-class)
pub fn recall(y_true: &Array1<f64>, y_pred: &Array1<f64>) -> Result<f64> {
    validate_inputs(y_true, y_pred)?;

    let classes = get_unique_classes(y_true);
    let mut recalls = Vec::new();

    for &class in &classes {
        let tp = true_positives(y_true, y_pred, class);
        let fn_count = false_negatives(y_true, y_pred, class);

        let rec = if tp + fn_count > 0 {
            tp as f64 / (tp + fn_count) as f64
        } else {
            0.0
        };

        recalls.push(rec);
    }

    Ok(recalls.iter().sum::<f64>() / recalls.len() as f64)
}

/// Calculate F1 score
pub fn f1_score(y_true: &Array1<f64>, y_pred: &Array1<f64>) -> Result<f64> {
    let prec = precision(y_true, y_pred)?;
    let rec = recall(y_true, y_pred)?;

    if prec + rec == 0.0 {
        return Ok(0.0);
    }

    Ok(2.0 * prec * rec / (prec + rec))
}

/// Mean Absolute Error
pub fn mae(y_true: &Array1<f64>, y_pred: &Array1<f64>) -> Result<f64> {
    validate_inputs(y_true, y_pred)?;

    let sum: f64 = y_true
        .iter()
        .zip(y_pred.iter())
        .map(|(t, p)| (t - p).abs())
        .sum();

    Ok(sum / y_true.len() as f64)
}

/// Mean Squared Error
pub fn mse(y_true: &Array1<f64>, y_pred: &Array1<f64>) -> Result<f64> {
    validate_inputs(y_true, y_pred)?;

    let sum: f64 = y_true
        .iter()
        .zip(y_pred.iter())
        .map(|(t, p)| (t - p).powi(2))
        .sum();

    Ok(sum / y_true.len() as f64)
}

/// Root Mean Squared Error
pub fn rmse(y_true: &Array1<f64>, y_pred: &Array1<f64>) -> Result<f64> {
    Ok(mse(y_true, y_pred)?.sqrt())
}

/// R² score (coefficient of determination)
pub fn r2_score(y_true: &Array1<f64>, y_pred: &Array1<f64>) -> Result<f64> {
    validate_inputs(y_true, y_pred)?;

    let mean = y_true.sum() / y_true.len() as f64;

    let ss_tot: f64 = y_true.iter().map(|y| (y - mean).powi(2)).sum();

    let ss_res: f64 = y_true
        .iter()
        .zip(y_pred.iter())
        .map(|(t, p)| (t - p).powi(2))
        .sum();

    if ss_tot == 0.0 {
        return Ok(0.0);
    }

    Ok(1.0 - ss_res / ss_tot)
}

/// Mean Absolute Percentage Error
pub fn mape(y_true: &Array1<f64>, y_pred: &Array1<f64>) -> Result<f64> {
    validate_inputs(y_true, y_pred)?;

    let mut sum = 0.0;
    let mut count = 0;

    for (t, p) in y_true.iter().zip(y_pred.iter()) {
        if t.abs() > 1e-6 {
            sum += ((t - p) / t).abs();
            count += 1;
        }
    }

    if count == 0 {
        return Ok(0.0);
    }

    Ok(sum / count as f64 * 100.0)
}

/// Validate input arrays
fn validate_inputs(y_true: &Array1<f64>, y_pred: &Array1<f64>) -> Result<()> {
    if y_true.len() != y_pred.len() {
        return Err(MlError::Evaluation(format!(
            "Input length mismatch: {} vs {}",
            y_true.len(),
            y_pred.len()
        )));
    }

    if y_true.is_empty() {
        return Err(MlError::Evaluation("Empty input arrays".to_string()));
    }

    Ok(())
}

/// Get unique classes from labels
fn get_unique_classes(y: &Array1<f64>) -> Vec<f64> {
    let mut classes: Vec<f64> = y.iter().copied().collect();
    classes.sort_by(|a, b| a.partial_cmp(b).unwrap());
    classes.dedup();
    classes
}

/// Count true positives for a class
fn true_positives(y_true: &Array1<f64>, y_pred: &Array1<f64>, class: f64) -> usize {
    y_true
        .iter()
        .zip(y_pred.iter())
        .filter(|(t, p)| ((**t - class).abs() < 1e-6) && ((**p - class).abs() < 1e-6))
        .count()
}

/// Count false positives for a class
fn false_positives(y_true: &Array1<f64>, y_pred: &Array1<f64>, class: f64) -> usize {
    y_true
        .iter()
        .zip(y_pred.iter())
        .filter(|(t, p)| ((**t - class).abs() >= 1e-6) && ((**p - class).abs() < 1e-6))
        .count()
}

/// Count false negatives for a class
fn false_negatives(y_true: &Array1<f64>, y_pred: &Array1<f64>, class: f64) -> usize {
    y_true
        .iter()
        .zip(y_pred.iter())
        .filter(|(t, p)| ((**t - class).abs() < 1e-6) && ((**p - class).abs() >= 1e-6))
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accuracy() {
        let y_true = Array1::from_vec(vec![1.0, 0.0, 1.0, 1.0, 0.0]);
        let y_pred = Array1::from_vec(vec![1.0, 0.0, 0.0, 1.0, 0.0]);

        let acc = accuracy(&y_true, &y_pred).unwrap();
        assert!((acc - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_precision_recall_f1() {
        let y_true = Array1::from_vec(vec![1.0, 1.0, 0.0, 1.0, 0.0]);
        let y_pred = Array1::from_vec(vec![1.0, 1.0, 0.0, 0.0, 0.0]);

        let prec = precision(&y_true, &y_pred).unwrap();
        let rec = recall(&y_true, &y_pred).unwrap();
        let f1 = f1_score(&y_true, &y_pred).unwrap();

        assert!(prec > 0.0);
        assert!(rec > 0.0);
        assert!(f1 > 0.0);
    }

    #[test]
    fn test_regression_metrics() {
        let y_true = Array1::from_vec(vec![3.0, -0.5, 2.0, 7.0]);
        let y_pred = Array1::from_vec(vec![2.5, 0.0, 2.0, 8.0]);

        let mae_val = mae(&y_true, &y_pred).unwrap();
        let mse_val = mse(&y_true, &y_pred).unwrap();
        let rmse_val = rmse(&y_true, &y_pred).unwrap();

        assert!(mae_val > 0.0);
        assert!(mse_val > 0.0);
        assert!((rmse_val - mse_val.sqrt()).abs() < 1e-6);
    }

    #[test]
    fn test_r2_score() {
        let y_true = Array1::from_vec(vec![3.0, -0.5, 2.0, 7.0]);
        let y_pred = Array1::from_vec(vec![2.5, 0.0, 2.0, 8.0]);

        let r2 = r2_score(&y_true, &y_pred).unwrap();
        assert!(r2 > 0.8 && r2 <= 1.0);
    }
}
