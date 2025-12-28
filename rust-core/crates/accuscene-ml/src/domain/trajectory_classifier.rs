//! Vehicle trajectory classification

use crate::algorithms::classification::LogisticRegression;
use crate::error::Result;
use crate::model::{Classifier, Model, ModelMetadata, ModelType};
use async_trait::async_trait;
use ndarray::{Array1, Array2};
use serde::{Deserialize, Serialize};

/// Trajectory classifier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectoryClassifier {
    model: LogisticRegression,
    metadata: ModelMetadata,
}

impl TrajectoryClassifier {
    pub fn new() -> Self {
        let metadata = ModelMetadata::new(
            "trajectory_classifier",
            "0.2.0",
            ModelType::Custom("TrajectoryClassifier".to_string()),
        );

        Self {
            model: LogisticRegression::new(),
            metadata,
        }
    }

    /// Classify trajectory type from vehicle motion data
    pub async fn classify_trajectory(&self, trajectory: &Trajectory) -> Result<TrajectoryType> {
        let features = self.extract_features(trajectory);
        let prediction = self.model.predict(&features).await?;

        let trajectory_type = match prediction as usize {
            0 => TrajectoryType::Straight,
            1 => TrajectoryType::LeftTurn,
            2 => TrajectoryType::RightTurn,
            3 => TrajectoryType::UTurn,
            _ => TrajectoryType::Evasive,
        };

        Ok(trajectory_type)
    }

    fn extract_features(&self, trajectory: &Trajectory) -> Array1<f64> {
        ndarray::arr1(&[
            trajectory.avg_velocity,
            trajectory.max_velocity,
            trajectory.avg_acceleration,
            trajectory.max_acceleration,
            trajectory.heading_change,
            trajectory.lateral_displacement,
            trajectory.path_curvature,
        ])
    }
}

impl Default for TrajectoryClassifier {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Model for TrajectoryClassifier {
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

#[async_trait]
impl Classifier for TrajectoryClassifier {
    async fn predict_proba(&self, features: &Array1<f64>) -> Result<Vec<f64>> {
        self.model.predict_proba(features).await
    }

    async fn predict_proba_batch(&self, features: &Array2<f64>) -> Result<Array2<f64>> {
        self.model.predict_proba_batch(features).await
    }

    fn num_classes(&self) -> usize {
        5
    }
}

/// Trajectory data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trajectory {
    pub avg_velocity: f64,
    pub max_velocity: f64,
    pub avg_acceleration: f64,
    pub max_acceleration: f64,
    pub heading_change: f64,
    pub lateral_displacement: f64,
    pub path_curvature: f64,
}

/// Trajectory type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TrajectoryType {
    Straight,
    LeftTurn,
    RightTurn,
    UTurn,
    Evasive,
}
