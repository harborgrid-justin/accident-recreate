//! ML pipeline builder

use crate::error::Result;
use crate::feature::{FeatureTransformer, MinMaxScaler, StandardScaler};
use crate::model::Model;
use ndarray::{Array1, Array2};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// ML Pipeline
pub struct Pipeline {
    steps: Vec<PipelineStep>,
}

impl Pipeline {
    /// Create a new pipeline
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }

    /// Add a preprocessing step
    pub fn add_preprocessor(mut self, name: impl Into<String>, preprocessor: Box<dyn FeatureTransformer>) -> Self {
        self.steps.push(PipelineStep::Preprocessor {
            name: name.into(),
            transformer: preprocessor,
        });
        self
    }

    /// Add a model step
    pub fn add_model(mut self, model: Arc<dyn Model>) -> Self {
        self.steps.push(PipelineStep::Model { model });
        self
    }

    /// Transform features through the pipeline
    pub async fn transform(&self, features: &Array2<f64>) -> Result<Array2<f64>> {
        let mut result = features.clone();

        for step in &self.steps {
            match step {
                PipelineStep::Preprocessor { transformer, .. } => {
                    result = transformer.transform(&result)?;
                }
                PipelineStep::Model { .. } => {
                    break; // Stop at model step
                }
            }
        }

        Ok(result)
    }

    /// Predict using the pipeline
    pub async fn predict(&self, features: &Array2<f64>) -> Result<Array1<f64>> {
        let transformed = self.transform(features).await?;

        // Find the model step
        for step in &self.steps {
            if let PipelineStep::Model { model } = step {
                return model.predict_batch(&transformed).await;
            }
        }

        Err(crate::error::MLError::Pipeline("No model in pipeline".to_string()))
    }
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Pipeline step
enum PipelineStep {
    Preprocessor {
        name: String,
        transformer: Box<dyn FeatureTransformer>,
    },
    Model {
        model: Arc<dyn Model>,
    },
}

/// Pipeline builder
pub struct PipelineBuilder {
    pipeline: Pipeline,
}

impl PipelineBuilder {
    /// Create a new pipeline builder
    pub fn new() -> Self {
        Self {
            pipeline: Pipeline::new(),
        }
    }

    /// Add standard scaling
    pub fn add_standard_scaler(self) -> Self {
        let scaler = StandardScaler::new();
        let pipeline = self.pipeline.add_preprocessor("standard_scaler", Box::new(scaler));
        Self { pipeline }
    }

    /// Add min-max scaling
    pub fn add_minmax_scaler(self) -> Self {
        let scaler = MinMaxScaler::new();
        let pipeline = self.pipeline.add_preprocessor("minmax_scaler", Box::new(scaler));
        Self { pipeline }
    }

    /// Add a model
    pub fn add_model(self, model: Arc<dyn Model>) -> Self {
        let pipeline = self.pipeline.add_model(model);
        Self { pipeline }
    }

    /// Build the pipeline
    pub fn build(self) -> Pipeline {
        self.pipeline
    }
}

impl Default for PipelineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Serializable pipeline configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    pub steps: Vec<StepConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepConfig {
    pub name: String,
    pub step_type: String,
    pub params: std::collections::HashMap<String, serde_json::Value>,
}
