//! Hyperparameter tuning utilities

use crate::error::{MlError, Result};
use crate::training::{CrossValidator, Dataset};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Parameter value types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterValue {
    Float(f64),
    Int(i64),
    String(String),
    Bool(bool),
}

/// Parameter grid for grid search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterGrid {
    parameters: HashMap<String, Vec<ParameterValue>>,
}

/// Hyperparameter combination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterSet {
    pub values: HashMap<String, ParameterValue>,
}

/// Hyperparameter tuning result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuningResult {
    /// Best parameter set
    pub best_params: ParameterSet,

    /// Best score achieved
    pub best_score: f64,

    /// All parameter sets tried
    pub all_params: Vec<ParameterSet>,

    /// Scores for each parameter set
    pub all_scores: Vec<f64>,

    /// Number of combinations tried
    pub n_combinations: usize,
}

impl ParameterGrid {
    /// Create a new parameter grid
    pub fn new() -> Self {
        Self {
            parameters: HashMap::new(),
        }
    }

    /// Add a parameter with possible values
    pub fn add_parameter(&mut self, name: String, values: Vec<ParameterValue>) {
        self.parameters.insert(name, values);
    }

    /// Add float parameter range
    pub fn add_float_range(&mut self, name: String, start: f64, end: f64, steps: usize) {
        let step_size = (end - start) / (steps - 1) as f64;
        let values: Vec<ParameterValue> = (0..steps)
            .map(|i| ParameterValue::Float(start + i as f64 * step_size))
            .collect();

        self.add_parameter(name, values);
    }

    /// Add integer parameter range
    pub fn add_int_range(&mut self, name: String, start: i64, end: i64, step: i64) {
        let values: Vec<ParameterValue> = (start..=end)
            .step_by(step as usize)
            .map(ParameterValue::Int)
            .collect();

        self.add_parameter(name, values);
    }

    /// Generate all parameter combinations
    pub fn combinations(&self) -> Vec<ParameterSet> {
        if self.parameters.is_empty() {
            return vec![ParameterSet {
                values: HashMap::new(),
            }];
        }

        let param_names: Vec<String> = self.parameters.keys().cloned().collect();
        let param_values: Vec<&Vec<ParameterValue>> =
            param_names.iter().map(|k| &self.parameters[k]).collect();

        Self::generate_combinations(&param_names, &param_values, 0, &HashMap::new())
    }

    /// Recursively generate combinations
    fn generate_combinations(
        names: &[String],
        values: &[&Vec<ParameterValue>],
        idx: usize,
        current: &HashMap<String, ParameterValue>,
    ) -> Vec<ParameterSet> {
        if idx >= names.len() {
            return vec![ParameterSet {
                values: current.clone(),
            }];
        }

        let mut result = Vec::new();

        for value in values[idx] {
            let mut new_current = current.clone();
            new_current.insert(names[idx].clone(), value.clone());

            let combinations = Self::generate_combinations(names, values, idx + 1, &new_current);
            result.extend(combinations);
        }

        result
    }

    /// Get total number of combinations
    pub fn n_combinations(&self) -> usize {
        if self.parameters.is_empty() {
            return 0;
        }

        self.parameters
            .values()
            .map(|v| v.len())
            .product()
    }
}

impl Default for ParameterGrid {
    fn default() -> Self {
        Self::new()
    }
}

/// Hyperparameter tuner
pub struct HyperparameterTuner {
    cv: CrossValidator,
}

impl HyperparameterTuner {
    /// Create a new hyperparameter tuner
    pub fn new(cv_folds: usize) -> Result<Self> {
        Ok(Self {
            cv: CrossValidator::new(cv_folds)?,
        })
    }

    /// Perform grid search
    pub fn grid_search<F>(
        &self,
        dataset: &Dataset,
        param_grid: &ParameterGrid,
        evaluate_fn: F,
    ) -> Result<TuningResult>
    where
        F: Fn(&Dataset, &ParameterSet) -> Result<f64>,
    {
        let combinations = param_grid.combinations();
        let n_combinations = combinations.len();

        tracing::info!("Starting grid search with {} combinations", n_combinations);

        let mut all_params = Vec::with_capacity(n_combinations);
        let mut all_scores = Vec::with_capacity(n_combinations);
        let mut best_score = f64::NEG_INFINITY;
        let mut best_params = combinations[0].clone();

        for (idx, params) in combinations.iter().enumerate() {
            tracing::debug!(
                "Evaluating combination {}/{}: {:?}",
                idx + 1,
                n_combinations,
                params
            );

            let score = evaluate_fn(dataset, params)?;

            all_params.push(params.clone());
            all_scores.push(score);

            if score > best_score {
                best_score = score;
                best_params = params.clone();
            }

            if (idx + 1) % 10 == 0 {
                tracing::info!(
                    "Progress: {}/{}, Best score so far: {:.4}",
                    idx + 1,
                    n_combinations,
                    best_score
                );
            }
        }

        tracing::info!(
            "Grid search completed. Best score: {:.4}",
            best_score
        );

        Ok(TuningResult {
            best_params,
            best_score,
            all_params,
            all_scores,
            n_combinations,
        })
    }

    /// Perform random search
    pub fn random_search<F>(
        &self,
        dataset: &Dataset,
        param_grid: &ParameterGrid,
        n_iter: usize,
        evaluate_fn: F,
    ) -> Result<TuningResult>
    where
        F: Fn(&Dataset, &ParameterSet) -> Result<f64>,
    {
        use rand::seq::SliceRandom;

        let all_combinations = param_grid.combinations();

        if all_combinations.is_empty() {
            return Err(MlError::Training("Empty parameter grid".to_string()));
        }

        let n_samples = n_iter.min(all_combinations.len());

        tracing::info!(
            "Starting random search with {} iterations",
            n_samples
        );

        // Sample random combinations
        let mut rng = rand::thread_rng();
        let mut sampled: Vec<_> = all_combinations
            .choose_multiple(&mut rng, n_samples)
            .cloned()
            .collect();

        let mut all_params = Vec::with_capacity(n_samples);
        let mut all_scores = Vec::with_capacity(n_samples);
        let mut best_score = f64::NEG_INFINITY;
        let mut best_params = sampled[0].clone();

        for (idx, params) in sampled.iter().enumerate() {
            tracing::debug!(
                "Evaluating iteration {}/{}: {:?}",
                idx + 1,
                n_samples,
                params
            );

            let score = evaluate_fn(dataset, params)?;

            all_params.push(params.clone());
            all_scores.push(score);

            if score > best_score {
                best_score = score;
                best_params = params.clone();
            }
        }

        tracing::info!(
            "Random search completed. Best score: {:.4}",
            best_score
        );

        Ok(TuningResult {
            best_params,
            best_score,
            all_params,
            all_scores,
            n_combinations: n_samples,
        })
    }
}

impl ParameterSet {
    /// Get float parameter
    pub fn get_float(&self, name: &str) -> Option<f64> {
        self.values.get(name).and_then(|v| match v {
            ParameterValue::Float(f) => Some(*f),
            _ => None,
        })
    }

    /// Get int parameter
    pub fn get_int(&self, name: &str) -> Option<i64> {
        self.values.get(name).and_then(|v| match v {
            ParameterValue::Int(i) => Some(*i),
            _ => None,
        })
    }

    /// Get bool parameter
    pub fn get_bool(&self, name: &str) -> Option<bool> {
        self.values.get(name).and_then(|v| match v {
            ParameterValue::Bool(b) => Some(*b),
            _ => None,
        })
    }

    /// Get string parameter
    pub fn get_string(&self, name: &str) -> Option<String> {
        self.values.get(name).and_then(|v| match v {
            ParameterValue::String(s) => Some(s.clone()),
            _ => None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parameter_grid() {
        let mut grid = ParameterGrid::new();
        grid.add_float_range("learning_rate".to_string(), 0.001, 0.1, 3);
        grid.add_int_range("batch_size".to_string(), 16, 64, 16);

        assert_eq!(grid.n_combinations(), 12); // 3 learning rates × 4 batch sizes

        let combinations = grid.combinations();
        assert_eq!(combinations.len(), 12);
    }

    #[test]
    fn test_parameter_set() {
        let mut values = HashMap::new();
        values.insert("lr".to_string(), ParameterValue::Float(0.01));
        values.insert("batch".to_string(), ParameterValue::Int(32));

        let params = ParameterSet { values };

        assert_eq!(params.get_float("lr"), Some(0.01));
        assert_eq!(params.get_int("batch"), Some(32));
        assert_eq!(params.get_float("batch"), None);
    }
}
