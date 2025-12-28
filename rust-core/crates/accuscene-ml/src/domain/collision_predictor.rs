//! Collision severity prediction model

use crate::algorithms::regression::LinearRegression;
use crate::error::Result;
use crate::model::{Model, ModelMetadata, ModelType};
use async_trait::async_trait;
use ndarray::{Array1, Array2, arr1};
use serde::{Deserialize, Serialize};

/// Collision severity predictor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollisionPredictor {
    /// Underlying regression model
    model: LinearRegression,

    /// Model metadata
    metadata: ModelMetadata,
}

impl CollisionPredictor {
    /// Create a new collision predictor
    pub fn new() -> Self {
        let metadata = ModelMetadata::new(
            "collision_predictor",
            "0.2.0",
            ModelType::Custom("CollisionPredictor".to_string()),
        );

        Self {
            model: LinearRegression::new(),
            metadata,
        }
    }

    /// Extract features from collision scenario
    pub fn extract_features(&self, scenario: &CollisionScenario) -> Array1<f64> {
        arr1(&[
            scenario.vehicle1_speed,
            scenario.vehicle2_speed,
            scenario.impact_angle,
            scenario.vehicle1_mass,
            scenario.vehicle2_mass,
            if scenario.road_surface == RoadSurface::Wet { 1.0 } else { 0.0 },
            if scenario.weather == Weather::Rain { 1.0 } else { 0.0 },
        ])
    }

    /// Predict collision severity (0.0 = minor, 1.0 = fatal)
    pub async fn predict_severity(&self, scenario: &CollisionScenario) -> Result<CollisionSeverity> {
        let features = self.extract_features(scenario);
        let severity_score = self.model.predict(&features).await?;

        let severity = if severity_score < 0.3 {
            CollisionSeverity::Minor
        } else if severity_score < 0.6 {
            CollisionSeverity::Moderate
        } else if severity_score < 0.8 {
            CollisionSeverity::Severe
        } else {
            CollisionSeverity::Fatal
        };

        Ok(severity)
    }
}

impl Default for CollisionPredictor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Model for CollisionPredictor {
    fn metadata(&self) -> &ModelMetadata {
        &self.metadata
    }

    async fn train(&mut self, features: &Array2<f64>, targets: &Array1<f64>) -> Result<()> {
        self.model.train(features, targets).await
    }

    async fn predict(&self, features: &Array1<f64>) -> Result<f64> {
        self.model.predict(features).await
    }

    async fn predict_batch(&self, features: &Array2<f64>) -> Result<Array1<f64>> {
        self.model.predict_batch(features).await
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(bincode::serialize(self)?)
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(bincode::deserialize(bytes)?)
    }
}

/// Collision scenario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollisionScenario {
    pub vehicle1_speed: f64,
    pub vehicle2_speed: f64,
    pub impact_angle: f64,
    pub vehicle1_mass: f64,
    pub vehicle2_mass: f64,
    pub road_surface: RoadSurface,
    pub weather: Weather,
}

/// Road surface type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RoadSurface {
    Dry,
    Wet,
    Icy,
    Gravel,
}

/// Weather conditions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Weather {
    Clear,
    Rain,
    Snow,
    Fog,
}

/// Collision severity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CollisionSeverity {
    Minor,
    Moderate,
    Severe,
    Fatal,
}
