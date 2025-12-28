//! Categorical feature encoding

use crate::error::{MLError, Result};
use ndarray::{Array1, Array2};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Trait for categorical encoders
pub trait CategoricalEncoder: Send + Sync {
    /// Fit the encoder to training data
    fn fit(&mut self, data: &[String]) -> Result<()>;

    /// Transform categorical data to numerical
    fn transform(&self, data: &[String]) -> Result<Array1<f64>>;

    /// Fit and transform in one step
    fn fit_transform(&mut self, data: &[String]) -> Result<Array1<f64>> {
        self.fit(data)?;
        self.transform(data)
    }

    /// Inverse transform numerical data to categorical
    fn inverse_transform(&self, data: &Array1<f64>) -> Result<Vec<String>>;
}

/// Label encoder (maps categories to integers)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelEncoder {
    /// Category to index mapping
    category_to_index: HashMap<String, usize>,

    /// Index to category mapping
    index_to_category: Vec<String>,

    /// Handle unknown categories
    handle_unknown: UnknownStrategy,
}

impl LabelEncoder {
    /// Create a new label encoder
    pub fn new() -> Self {
        Self {
            category_to_index: HashMap::new(),
            index_to_category: Vec::new(),
            handle_unknown: UnknownStrategy::Error,
        }
    }

    /// Set strategy for handling unknown categories
    pub fn with_unknown_strategy(mut self, strategy: UnknownStrategy) -> Self {
        self.handle_unknown = strategy;
        self
    }

    /// Get number of categories
    pub fn num_categories(&self) -> usize {
        self.index_to_category.len()
    }

    /// Get categories
    pub fn categories(&self) -> &[String] {
        &self.index_to_category
    }
}

impl Default for LabelEncoder {
    fn default() -> Self {
        Self::new()
    }
}

impl CategoricalEncoder for LabelEncoder {
    fn fit(&mut self, data: &[String]) -> Result<()> {
        self.category_to_index.clear();
        self.index_to_category.clear();

        let mut unique_categories: Vec<String> = data.iter()
            .map(|s| s.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        unique_categories.sort();

        for (idx, category) in unique_categories.iter().enumerate() {
            self.category_to_index.insert(category.clone(), idx);
            self.index_to_category.push(category.clone());
        }

        Ok(())
    }

    fn transform(&self, data: &[String]) -> Result<Array1<f64>> {
        let mut result = Array1::zeros(data.len());

        for (i, category) in data.iter().enumerate() {
            let idx = self.category_to_index.get(category);

            match idx {
                Some(&idx) => result[i] = idx as f64,
                None => match self.handle_unknown {
                    UnknownStrategy::Error => {
                        return Err(MLError::invalid_input(format!(
                            "Unknown category: {}",
                            category
                        )));
                    }
                    UnknownStrategy::UseEncodedValue(val) => result[i] = val,
                },
            }
        }

        Ok(result)
    }

    fn inverse_transform(&self, data: &Array1<f64>) -> Result<Vec<String>> {
        let mut result = Vec::new();

        for &value in data.iter() {
            let idx = value.round() as usize;
            if idx < self.index_to_category.len() {
                result.push(self.index_to_category[idx].clone());
            } else {
                return Err(MLError::invalid_input(format!(
                    "Invalid encoded value: {}",
                    value
                )));
            }
        }

        Ok(result)
    }
}

/// One-hot encoder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneHotEncoder {
    /// Category to index mapping
    category_to_index: HashMap<String, usize>,

    /// Index to category mapping
    index_to_category: Vec<String>,

    /// Drop first category (for avoiding multicollinearity)
    drop_first: bool,

    /// Handle unknown categories
    handle_unknown: UnknownStrategy,
}

impl OneHotEncoder {
    /// Create a new one-hot encoder
    pub fn new() -> Self {
        Self {
            category_to_index: HashMap::new(),
            index_to_category: Vec::new(),
            drop_first: false,
            handle_unknown: UnknownStrategy::Error,
        }
    }

    /// Set whether to drop the first category
    pub fn with_drop_first(mut self, drop_first: bool) -> Self {
        self.drop_first = drop_first;
        self
    }

    /// Set strategy for handling unknown categories
    pub fn with_unknown_strategy(mut self, strategy: UnknownStrategy) -> Self {
        self.handle_unknown = strategy;
        self
    }

    /// Fit the encoder to training data
    pub fn fit(&mut self, data: &[String]) -> Result<()> {
        self.category_to_index.clear();
        self.index_to_category.clear();

        let mut unique_categories: Vec<String> = data.iter()
            .map(|s| s.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        unique_categories.sort();

        for (idx, category) in unique_categories.iter().enumerate() {
            self.category_to_index.insert(category.clone(), idx);
            self.index_to_category.push(category.clone());
        }

        Ok(())
    }

    /// Transform categorical data to one-hot encoded matrix
    pub fn transform_matrix(&self, data: &[String]) -> Result<Array2<f64>> {
        let n_samples = data.len();
        let n_categories = if self.drop_first {
            self.index_to_category.len().saturating_sub(1)
        } else {
            self.index_to_category.len()
        };

        let mut result = Array2::zeros((n_samples, n_categories));

        for (i, category) in data.iter().enumerate() {
            let idx = self.category_to_index.get(category);

            match idx {
                Some(&mut idx) => {
                    let col_idx = if self.drop_first && idx > 0 {
                        idx - 1
                    } else if self.drop_first {
                        continue; // Skip first category
                    } else {
                        idx
                    };

                    if col_idx < n_categories {
                        result[[i, col_idx]] = 1.0;
                    }
                }
                None => match self.handle_unknown {
                    UnknownStrategy::Error => {
                        return Err(MLError::invalid_input(format!(
                            "Unknown category: {}",
                            category
                        )));
                    }
                    UnknownStrategy::UseEncodedValue(_) => {
                        // Leave as zeros for unknown categories
                    }
                },
            }
        }

        Ok(result)
    }

    /// Inverse transform one-hot encoded matrix to categories
    pub fn inverse_transform_matrix(&self, data: &Array2<f64>) -> Result<Vec<String>> {
        let mut result = Vec::new();

        for i in 0..data.nrows() {
            let row = data.row(i);

            // Find the index with value 1.0
            let mut category_idx = None;
            for (j, &val) in row.iter().enumerate() {
                if val > 0.5 {
                    category_idx = Some(if self.drop_first { j + 1 } else { j });
                    break;
                }
            }

            match category_idx {
                Some(idx) if idx < self.index_to_category.len() => {
                    result.push(self.index_to_category[idx].clone());
                }
                _ => {
                    // If no category found, use first category as default
                    if self.drop_first && !self.index_to_category.is_empty() {
                        result.push(self.index_to_category[0].clone());
                    } else {
                        return Err(MLError::invalid_input(
                            "Invalid one-hot encoded row".to_string(),
                        ));
                    }
                }
            }
        }

        Ok(result)
    }

    /// Get number of categories
    pub fn num_categories(&self) -> usize {
        self.index_to_category.len()
    }

    /// Get categories
    pub fn categories(&self) -> &[String] {
        &self.index_to_category
    }
}

impl Default for OneHotEncoder {
    fn default() -> Self {
        Self::new()
    }
}

/// Strategy for handling unknown categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnknownStrategy {
    /// Raise an error
    Error,
    /// Use a specific encoded value
    UseEncodedValue(f64),
}

/// Encoding strategy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EncodingStrategy {
    Label,
    OneHot,
    Target,
    Frequency,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_label_encoder() -> Result<()> {
        let data = vec![
            "cat".to_string(),
            "dog".to_string(),
            "cat".to_string(),
            "bird".to_string(),
        ];

        let mut encoder = LabelEncoder::new();
        let encoded = encoder.fit_transform(&data)?;

        assert_eq!(encoded.len(), 4);
        assert_eq!(encoder.num_categories(), 3);

        let decoded = encoder.inverse_transform(&encoded)?;
        assert_eq!(decoded, data);

        Ok(())
    }

    #[test]
    fn test_onehot_encoder() -> Result<()> {
        let data = vec![
            "red".to_string(),
            "blue".to_string(),
            "red".to_string(),
            "green".to_string(),
        ];

        let mut encoder = OneHotEncoder::new();
        encoder.fit(&data)?;

        let encoded = encoder.transform_matrix(&data)?;

        assert_eq!(encoded.nrows(), 4);
        assert_eq!(encoded.ncols(), 3); // blue, green, red

        // Check first row (red)
        let row0 = encoded.row(0);
        assert_eq!(row0.iter().filter(|&&x| x == 1.0).count(), 1);

        let decoded = encoder.inverse_transform_matrix(&encoded)?;
        assert_eq!(decoded, data);

        Ok(())
    }

    #[test]
    fn test_onehot_drop_first() -> Result<()> {
        let data = vec!["a".to_string(), "b".to_string(), "c".to_string()];

        let mut encoder = OneHotEncoder::new().with_drop_first(true);
        encoder.fit(&data)?;

        let encoded = encoder.transform_matrix(&data)?;

        // Should have 2 columns (dropped first)
        assert_eq!(encoded.ncols(), 2);

        Ok(())
    }
}
