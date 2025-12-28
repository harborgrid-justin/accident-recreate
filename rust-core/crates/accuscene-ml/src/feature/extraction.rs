//! Feature extraction implementations

use crate::error::Result;
use ndarray::{Array1, Array2};
use serde::{Deserialize, Serialize};

/// Trait for feature extractors
pub trait FeatureExtractor: Send + Sync {
    /// Extract features from input data
    fn extract(&self, input: &Array2<f64>) -> Result<Array2<f64>>;

    /// Get number of output features
    fn num_features(&self) -> usize;

    /// Get feature names
    fn feature_names(&self) -> Vec<String>;
}

/// Polynomial features generator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolynomialFeatures {
    /// Polynomial degree
    degree: usize,

    /// Include bias term
    include_bias: bool,

    /// Interaction only (no powers)
    interaction_only: bool,

    /// Number of input features
    n_input_features: Option<usize>,
}

impl PolynomialFeatures {
    /// Create a new polynomial features generator
    pub fn new(degree: usize) -> Self {
        Self {
            degree,
            include_bias: true,
            interaction_only: false,
            n_input_features: None,
        }
    }

    /// Set whether to include bias
    pub fn with_bias(mut self, include_bias: bool) -> Self {
        self.include_bias = include_bias;
        self
    }

    /// Set whether to use interaction only
    pub fn with_interaction_only(mut self, interaction_only: bool) -> Self {
        self.interaction_only = interaction_only;
        self
    }

    /// Fit to input features
    pub fn fit(&mut self, input: &Array2<f64>) -> Result<()> {
        self.n_input_features = Some(input.ncols());
        Ok(())
    }

    /// Transform input to polynomial features
    pub fn transform(&self, input: &Array2<f64>) -> Result<Array2<f64>> {
        let n_samples = input.nrows();
        let n_features = input.ncols();

        let mut features = Vec::new();

        // Add bias if requested
        if self.include_bias {
            features.push(Array1::ones(n_samples));
        }

        // Add original features
        for i in 0..n_features {
            features.push(input.column(i).to_owned());
        }

        // Add polynomial features
        if self.degree > 1 {
            for d in 2..=self.degree {
                if self.interaction_only {
                    // Only interactions, no powers
                    self.add_interactions(&mut features, input, d);
                } else {
                    // Both powers and interactions
                    self.add_polynomial_terms(&mut features, input, d);
                }
            }
        }

        // Stack all features into a matrix
        let n_output_features = features.len();
        let mut output = Array2::zeros((n_samples, n_output_features));
        for (i, feature) in features.iter().enumerate() {
            output.column_mut(i).assign(feature);
        }

        Ok(output)
    }

    fn add_interactions(&self, features: &mut Vec<Array1<f64>>, input: &Array2<f64>, degree: usize) {
        // Simplified: add pairwise interactions for degree=2
        if degree == 2 {
            for i in 0..input.ncols() {
                for j in i + 1..input.ncols() {
                    let interaction = &input.column(i) * &input.column(j);
                    features.push(interaction.to_owned());
                }
            }
        }
    }

    fn add_polynomial_terms(&self, features: &mut Vec<Array1<f64>>, input: &Array2<f64>, degree: usize) {
        // Add squared terms for degree=2
        if degree == 2 {
            for i in 0..input.ncols() {
                let col = input.column(i);
                let squared = &col * &col;
                features.push(squared.to_owned());
            }

            // Add interactions
            for i in 0..input.ncols() {
                for j in i + 1..input.ncols() {
                    let interaction = &input.column(i) * &input.column(j);
                    features.push(interaction.to_owned());
                }
            }
        }
    }
}

impl FeatureExtractor for PolynomialFeatures {
    fn extract(&self, input: &Array2<f64>) -> Result<Array2<f64>> {
        self.transform(input)
    }

    fn num_features(&self) -> usize {
        if let Some(n) = self.n_input_features {
            let mut count = if self.include_bias { 1 } else { 0 };
            count += n; // Original features

            // Add polynomial terms count
            if self.degree >= 2 {
                if self.interaction_only {
                    // Only interactions
                    count += n * (n - 1) / 2; // C(n, 2)
                } else {
                    // Powers and interactions
                    count += n; // Squared terms
                    count += n * (n - 1) / 2; // Interactions
                }
            }

            count
        } else {
            0
        }
    }

    fn feature_names(&self) -> Vec<String> {
        let mut names = Vec::new();

        if self.include_bias {
            names.push("bias".to_string());
        }

        if let Some(n) = self.n_input_features {
            // Original features
            for i in 0..n {
                names.push(format!("x{}", i));
            }

            // Polynomial features
            if self.degree >= 2 {
                if !self.interaction_only {
                    // Squared terms
                    for i in 0..n {
                        names.push(format!("x{}^2", i));
                    }
                }

                // Interactions
                for i in 0..n {
                    for j in i + 1..n {
                        names.push(format!("x{} * x{}", i, j));
                    }
                }
            }
        }

        names
    }
}

/// Statistical features extractor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalFeatures {
    /// Features to compute
    features: Vec<StatFeature>,
}

impl StatisticalFeatures {
    /// Create a new statistical features extractor
    pub fn new() -> Self {
        Self {
            features: vec![
                StatFeature::Mean,
                StatFeature::Std,
                StatFeature::Min,
                StatFeature::Max,
            ],
        }
    }

    /// Set which statistical features to compute
    pub fn with_features(mut self, features: Vec<StatFeature>) -> Self {
        self.features = features;
        self
    }

    /// Extract statistical features from each row
    pub fn extract_row_stats(&self, input: &Array2<f64>) -> Result<Array2<f64>> {
        let n_samples = input.nrows();
        let n_features = self.features.len();

        let mut output = Array2::zeros((n_samples, n_features));

        for i in 0..n_samples {
            let row = input.row(i);
            for (j, feature) in self.features.iter().enumerate() {
                output[[i, j]] = feature.compute(&row);
            }
        }

        Ok(output)
    }
}

impl Default for StatisticalFeatures {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistical feature types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatFeature {
    Mean,
    Std,
    Min,
    Max,
    Median,
    Quantile(f64),
    Skewness,
    Kurtosis,
}

impl StatFeature {
    /// Compute the statistical feature
    pub fn compute(&self, values: &ndarray::ArrayView1<f64>) -> f64 {
        match self {
            StatFeature::Mean => values.mean().unwrap_or(0.0),
            StatFeature::Std => values.std(0.0),
            StatFeature::Min => values.iter().copied().fold(f64::INFINITY, f64::min),
            StatFeature::Max => values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
            StatFeature::Median => {
                let mut sorted: Vec<f64> = values.to_vec();
                sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                let mid = sorted.len() / 2;
                if sorted.len() % 2 == 0 && mid > 0 {
                    (sorted[mid - 1] + sorted[mid]) / 2.0
                } else {
                    sorted.get(mid).copied().unwrap_or(0.0)
                }
            }
            StatFeature::Quantile(q) => {
                let mut sorted: Vec<f64> = values.to_vec();
                sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                let idx = (q * (sorted.len() - 1) as f64).round() as usize;
                sorted.get(idx).copied().unwrap_or(0.0)
            }
            StatFeature::Skewness | StatFeature::Kurtosis => {
                // Simplified implementation
                0.0
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::arr2;

    #[test]
    fn test_polynomial_features() -> Result<()> {
        let input = arr2(&[[1.0, 2.0], [3.0, 4.0]]);

        let mut poly = PolynomialFeatures::new(2);
        poly.fit(&input)?;

        let output = poly.transform(&input)?;

        // Should have: bias, x0, x1, x0^2, x1^2, x0*x1
        assert_eq!(output.ncols(), 6);

        Ok(())
    }

    #[test]
    fn test_statistical_features() -> Result<()> {
        let input = arr2(&[[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]);

        let stats = StatisticalFeatures::new();
        let output = stats.extract_row_stats(&input)?;

        assert_eq!(output.nrows(), 2);
        assert_eq!(output.ncols(), 4); // mean, std, min, max

        Ok(())
    }
}
