//! Model explainability and interpretability

pub mod feature_importance;
pub mod shap;

use crate::error::{MlError, Result};
use serde::{Deserialize, Serialize};

/// Explanation result for a prediction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Explanation {
    /// Feature contributions to the prediction
    pub feature_contributions: Vec<FeatureContribution>,

    /// Base value (baseline prediction)
    pub base_value: f64,

    /// Final prediction value
    pub prediction: f64,

    /// Explanation method used
    pub method: ExplanationMethod,
}

/// Feature contribution to a prediction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureContribution {
    /// Feature name
    pub feature_name: String,

    /// Feature value
    pub feature_value: f64,

    /// Contribution to prediction (SHAP value or importance)
    pub contribution: f64,

    /// Contribution percentage
    pub contribution_percent: f64,
}

/// Explanation method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExplanationMethod {
    /// SHAP (SHapley Additive exPlanations)
    SHAP,
    /// Permutation feature importance
    Permutation,
    /// Model-specific feature importance
    ModelSpecific,
    /// LIME (Local Interpretable Model-agnostic Explanations)
    LIME,
}

impl Explanation {
    /// Create a new explanation
    pub fn new(
        contributions: Vec<FeatureContribution>,
        base_value: f64,
        prediction: f64,
        method: ExplanationMethod,
    ) -> Self {
        Self {
            feature_contributions: contributions,
            base_value,
            prediction,
            method,
        }
    }

    /// Get top N most important features
    pub fn top_features(&self, n: usize) -> Vec<FeatureContribution> {
        let mut sorted = self.feature_contributions.clone();
        sorted.sort_by(|a, b| {
            b.contribution
                .abs()
                .partial_cmp(&a.contribution.abs())
                .unwrap()
        });

        sorted.into_iter().take(n).collect()
    }

    /// Get positive contributors (features increasing prediction)
    pub fn positive_contributors(&self) -> Vec<FeatureContribution> {
        self.feature_contributions
            .iter()
            .filter(|c| c.contribution > 0.0)
            .cloned()
            .collect()
    }

    /// Get negative contributors (features decreasing prediction)
    pub fn negative_contributors(&self) -> Vec<FeatureContribution> {
        self.feature_contributions
            .iter()
            .filter(|c| c.contribution < 0.0)
            .cloned()
            .collect()
    }

    /// Print explanation in human-readable format
    pub fn print(&self) {
        println!("\n{:?} Explanation", self.method);
        println!("====================");
        println!("Base value:   {:.4}", self.base_value);
        println!("Prediction:   {:.4}", self.prediction);
        println!("\nFeature Contributions:");
        println!("----------------------");

        for contrib in &self.feature_contributions {
            let direction = if contrib.contribution > 0.0 {
                "+"
            } else {
                ""
            };
            println!(
                "{:20} = {:8.4}  -->  {}{:.4} ({:.1}%)",
                contrib.feature_name,
                contrib.feature_value,
                direction,
                contrib.contribution,
                contrib.contribution_percent
            );
        }

        println!("\nTop 5 Most Important Features:");
        println!("------------------------------");
        for (i, contrib) in self.top_features(5).iter().enumerate() {
            println!(
                "{}. {} (contribution: {:.4})",
                i + 1,
                contrib.feature_name,
                contrib.contribution
            );
        }
    }
}

impl FeatureContribution {
    /// Create a new feature contribution
    pub fn new(name: String, value: f64, contribution: f64, total_contribution: f64) -> Self {
        let contribution_percent = if total_contribution.abs() > 1e-10 {
            (contribution.abs() / total_contribution.abs()) * 100.0
        } else {
            0.0
        };

        Self {
            feature_name: name,
            feature_value: value,
            contribution,
            contribution_percent,
        }
    }
}

// Re-export submodules
pub use feature_importance::{FeatureImportance, ImportanceCalculator};
pub use shap::{ShapExplainer, ShapValues};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explanation() {
        let contributions = vec![
            FeatureContribution::new("speed".to_string(), 50.0, 0.8, 2.0),
            FeatureContribution::new("damage".to_string(), 0.7, 0.6, 2.0),
            FeatureContribution::new("angle".to_string(), 30.0, -0.4, 2.0),
            FeatureContribution::new("mass".to_string(), 1500.0, 0.2, 2.0),
        ];

        let explanation = Explanation::new(
            contributions,
            0.5,
            1.7,
            ExplanationMethod::SHAP,
        );

        assert_eq!(explanation.feature_contributions.len(), 4);
        assert_eq!(explanation.base_value, 0.5);

        let top_3 = explanation.top_features(3);
        assert_eq!(top_3.len(), 3);
        assert_eq!(top_3[0].feature_name, "speed");
    }

    #[test]
    fn test_positive_negative_split() {
        let contributions = vec![
            FeatureContribution::new("a".to_string(), 1.0, 0.5, 1.0),
            FeatureContribution::new("b".to_string(), 2.0, -0.3, 1.0),
            FeatureContribution::new("c".to_string(), 3.0, 0.2, 1.0),
        ];

        let explanation = Explanation::new(
            contributions,
            0.0,
            0.4,
            ExplanationMethod::SHAP,
        );

        let positive = explanation.positive_contributors();
        let negative = explanation.negative_contributors();

        assert_eq!(positive.len(), 2);
        assert_eq!(negative.len(), 1);
    }
}
