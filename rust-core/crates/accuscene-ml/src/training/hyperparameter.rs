//! Hyperparameter tuning utilities

use crate::error::Result;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Parameter grid for hyperparameter search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamGrid {
    /// Parameters and their possible values
    params: HashMap<String, Vec<serde_json::Value>>,
}

impl ParamGrid {
    /// Create a new parameter grid
    pub fn new() -> Self {
        Self {
            params: HashMap::new(),
        }
    }

    /// Add a parameter with possible values
    pub fn add_param(&mut self, name: impl Into<String>, values: Vec<serde_json::Value>) {
        self.params.insert(name.into(), values);
    }

    /// Get all parameter combinations
    pub fn combinations(&self) -> Vec<HashMap<String, serde_json::Value>> {
        if self.params.is_empty() {
            return vec![HashMap::new()];
        }

        let mut result = vec![HashMap::new()];

        for (param_name, values) in &self.params {
            let mut new_result = Vec::new();

            for combo in &result {
                for value in values {
                    let mut new_combo = combo.clone();
                    new_combo.insert(param_name.clone(), value.clone());
                    new_result.push(new_combo);
                }
            }

            result = new_result;
        }

        result
    }

    /// Get number of combinations
    pub fn num_combinations(&self) -> usize {
        if self.params.is_empty() {
            return 0;
        }

        self.params.values().map(|v| v.len()).product()
    }
}

impl Default for ParamGrid {
    fn default() -> Self {
        Self::new()
    }
}

/// Hyperparameter tuner trait
pub trait HyperparameterTuner {
    /// Search for best hyperparameters
    fn search(&self, param_grid: &ParamGrid) -> Result<TuningResults>;

    /// Get number of iterations
    fn num_iterations(&self) -> usize;
}

/// Grid search hyperparameter tuner
pub struct GridSearch {
    /// Number of cross-validation folds
    cv_folds: usize,

    /// Scoring metric
    scoring: String,

    /// Verbose output
    verbose: bool,
}

impl GridSearch {
    /// Create a new grid search tuner
    pub fn new() -> Self {
        Self {
            cv_folds: 5,
            scoring: String::from("accuracy"),
            verbose: true,
        }
    }

    /// Set number of CV folds
    pub fn with_cv_folds(mut self, folds: usize) -> Self {
        self.cv_folds = folds;
        self
    }

    /// Set scoring metric
    pub fn with_scoring(mut self, scoring: impl Into<String>) -> Self {
        self.scoring = scoring.into();
        self
    }

    /// Set verbose
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
}

impl Default for GridSearch {
    fn default() -> Self {
        Self::new()
    }
}

impl HyperparameterTuner for GridSearch {
    fn search(&self, param_grid: &ParamGrid) -> Result<TuningResults> {
        let combinations = param_grid.combinations();
        let mut results = Vec::new();

        for params in combinations {
            // Simplified: random score for demonstration
            let score = rand::thread_rng().gen_range(0.5..1.0);

            results.push(TuningResult {
                params: params.clone(),
                score,
                cv_scores: vec![score; self.cv_folds],
            });
        }

        Ok(TuningResults::from_results(results))
    }

    fn num_iterations(&self) -> usize {
        0 // Grid search doesn't have iterations
    }
}

/// Random search hyperparameter tuner
pub struct RandomSearch {
    /// Number of iterations
    n_iterations: usize,

    /// Number of cross-validation folds
    cv_folds: usize,

    /// Random seed
    random_seed: u64,

    /// Scoring metric
    scoring: String,

    /// Verbose output
    verbose: bool,
}

impl RandomSearch {
    /// Create a new random search tuner
    pub fn new(n_iterations: usize) -> Self {
        Self {
            n_iterations,
            cv_folds: 5,
            random_seed: 42,
            scoring: String::from("accuracy"),
            verbose: true,
        }
    }

    /// Set number of CV folds
    pub fn with_cv_folds(mut self, folds: usize) -> Self {
        self.cv_folds = folds;
        self
    }

    /// Set random seed
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.random_seed = seed;
        self
    }

    /// Set scoring metric
    pub fn with_scoring(mut self, scoring: impl Into<String>) -> Self {
        self.scoring = scoring.into();
        self
    }

    /// Set verbose
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
}

impl HyperparameterTuner for RandomSearch {
    fn search(&self, param_grid: &ParamGrid) -> Result<TuningResults> {
        let mut rng = StdRng::seed_from_u64(self.random_seed);
        let mut results = Vec::new();

        for _ in 0..self.n_iterations {
            // Random sample from parameter space
            let mut params = HashMap::new();

            for (param_name, values) in &param_grid.params {
                let idx = rng.gen_range(0..values.len());
                params.insert(param_name.clone(), values[idx].clone());
            }

            // Simplified: random score for demonstration
            let score = rng.gen_range(0.5..1.0);

            results.push(TuningResult {
                params,
                score,
                cv_scores: vec![score; self.cv_folds],
            });
        }

        Ok(TuningResults::from_results(results))
    }

    fn num_iterations(&self) -> usize {
        self.n_iterations
    }
}

/// Single tuning result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuningResult {
    /// Parameters used
    pub params: HashMap<String, serde_json::Value>,

    /// Mean score
    pub score: f64,

    /// Cross-validation scores
    pub cv_scores: Vec<f64>,
}

/// Hyperparameter tuning results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuningResults {
    /// All tuning results
    pub results: Vec<TuningResult>,

    /// Best parameters
    pub best_params: HashMap<String, serde_json::Value>,

    /// Best score
    pub best_score: f64,

    /// Index of best result
    pub best_index: usize,
}

impl TuningResults {
    /// Create tuning results from individual results
    pub fn from_results(results: Vec<TuningResult>) -> Self {
        let (best_index, best_result) = results
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.score.partial_cmp(&b.score).unwrap())
            .unwrap_or((0, &results[0]));

        Self {
            results,
            best_params: best_result.params.clone(),
            best_score: best_result.score,
            best_index,
        }
    }

    /// Get top N results
    pub fn top_n(&self, n: usize) -> Vec<&TuningResult> {
        let mut sorted: Vec<&TuningResult> = self.results.iter().collect();
        sorted.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        sorted.into_iter().take(n).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_param_grid() {
        let mut grid = ParamGrid::new();
        grid.add_param("lr", vec![
            serde_json::json!(0.01),
            serde_json::json!(0.1),
        ]);
        grid.add_param("epochs", vec![
            serde_json::json!(10),
            serde_json::json!(20),
        ]);

        assert_eq!(grid.num_combinations(), 4);
        assert_eq!(grid.combinations().len(), 4);
    }

    #[test]
    fn test_grid_search() -> Result<()> {
        let mut grid = ParamGrid::new();
        grid.add_param("param1", vec![serde_json::json!(1), serde_json::json!(2)]);

        let search = GridSearch::new();
        let results = search.search(&grid)?;

        assert_eq!(results.results.len(), 2);
        assert!(results.best_score > 0.0);

        Ok(())
    }
}
