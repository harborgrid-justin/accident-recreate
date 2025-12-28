//! Vehicle damage estimation model

use crate::algorithms::ensemble::RandomForestRegressor;
use crate::error::Result;
use crate::model::{Model, ModelMetadata, ModelType};
use async_trait::async_trait;
use ndarray::{Array1, Array2};
use serde::{Deserialize, Serialize};

/// Damage estimator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamageEstimator {
    model: RandomForestRegressor,
    metadata: ModelMetadata,
}

impl DamageEstimator {
    pub fn new() -> Self {
        let metadata = ModelMetadata::new(
            "damage_estimator",
            "0.2.0",
            ModelType::Custom("DamageEstimator".to_string()),
        );

        Self {
            model: RandomForestRegressor::new(100),
            metadata,
        }
    }

    /// Estimate damage cost from accident parameters
    pub async fn estimate_damage(&self, damage_params: &DamageParameters) -> Result<DamageEstimate> {
        let features = self.extract_features(damage_params);
        let cost_estimate = self.model.predict(&features).await?;

        let severity = if cost_estimate < 1000.0 {
            DamageSeverity::Cosmetic
        } else if cost_estimate < 5000.0 {
            DamageSeverity::Light
        } else if cost_estimate < 15000.0 {
            DamageSeverity::Moderate
        } else if cost_estimate < 30000.0 {
            DamageSeverity::Heavy
        } else {
            DamageSeverity::Total
        };

        Ok(DamageEstimate {
            estimated_cost: cost_estimate,
            severity,
            repair_time_days: (cost_estimate / 500.0).ceil() as u32,
        })
    }

    fn extract_features(&self, params: &DamageParameters) -> Array1<f64> {
        ndarray::arr1(&[
            params.impact_speed,
            params.impact_force,
            params.deformation_depth,
            params.affected_area,
            if params.airbag_deployed { 1.0 } else { 0.0 },
            if params.frame_damage { 1.0 } else { 0.0 },
            params.vehicle_value,
        ])
    }
}

impl Default for DamageEstimator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Model for DamageEstimator {
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

/// Damage parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamageParameters {
    pub impact_speed: f64,
    pub impact_force: f64,
    pub deformation_depth: f64,
    pub affected_area: f64,
    pub airbag_deployed: bool,
    pub frame_damage: bool,
    pub vehicle_value: f64,
}

/// Damage estimate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamageEstimate {
    pub estimated_cost: f64,
    pub severity: DamageSeverity,
    pub repair_time_days: u32,
}

/// Damage severity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DamageSeverity {
    Cosmetic,
    Light,
    Moderate,
    Heavy,
    Total,
}
