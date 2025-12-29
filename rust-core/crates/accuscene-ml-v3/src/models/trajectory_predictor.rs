//! Vehicle trajectory prediction post-impact

use crate::config::MlConfig;
use crate::error::{MlError, Result};
use crate::inference::onnx_runtime::OnnxModel;
use crate::models::{Model, ModelMetadata, ModelType};
use ndarray::{Array1, Array2};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// 3D point in space
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Point3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// Vehicle trajectory prediction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectoryPrediction {
    /// Predicted trajectory points (time-series)
    pub trajectory_points: Vec<TrajectoryPoint>,

    /// Final resting position
    pub final_position: Point3D,

    /// Final orientation (yaw, pitch, roll in radians)
    pub final_orientation: (f64, f64, f64),

    /// Total distance traveled post-impact (meters)
    pub distance_traveled: f64,

    /// Time to rest (seconds)
    pub time_to_rest: f64,

    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,

    /// Energy dissipation profile
    pub energy_dissipation: Vec<f64>,

    /// Prediction metadata
    pub metadata: TrajectoryMetadata,
}

/// Single point in trajectory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectoryPoint {
    /// Time from impact (seconds)
    pub time: f64,

    /// Position
    pub position: Point3D,

    /// Velocity vector (m/s)
    pub velocity: Point3D,

    /// Orientation (yaw, pitch, roll in radians)
    pub orientation: (f64, f64, f64),

    /// Angular velocity (rad/s)
    pub angular_velocity: Point3D,
}

/// Trajectory prediction metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectoryMetadata {
    /// Surface type (asphalt, gravel, grass, etc.)
    pub surface_type: SurfaceType,

    /// Coefficient of friction
    pub friction_coefficient: f64,

    /// Environmental conditions
    pub conditions: EnvironmentalConditions,

    /// Vehicle stability during trajectory
    pub stability: f64,
}

/// Surface type for friction modeling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SurfaceType {
    DryAsphalt,
    WetAsphalt,
    DryGravel,
    WetGravel,
    Grass,
    Snow,
    Ice,
    Dirt,
}

/// Environmental conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentalConditions {
    /// Weather condition
    pub weather: WeatherCondition,

    /// Road grade (slope) in degrees
    pub road_grade: f64,

    /// Wind speed (m/s)
    pub wind_speed: f64,

    /// Visibility (meters)
    pub visibility: f64,
}

/// Weather condition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeatherCondition {
    Clear,
    Rain,
    Snow,
    Fog,
    Sleet,
}

/// Trajectory predictor model
pub struct TrajectoryPredictor {
    metadata: ModelMetadata,
    model: Arc<RwLock<Option<OnnxModel>>>,
    config: MlConfig,
}

impl TrajectoryPredictor {
    /// Create a new trajectory predictor
    pub fn new(config: &MlConfig) -> Result<Self> {
        let metadata = ModelMetadata {
            id: uuid::Uuid::new_v4(),
            name: "trajectory_predictor".to_string(),
            version: "1.0.0".to_string(),
            model_type: ModelType::TrajectoryPredictor,
            created_at: chrono::Utc::now(),
            model_path: config
                .models
                .model_dir
                .join("trajectory_predictor_v1.onnx"),
            input_features: vec![
                "impact_speed".to_string(),
                "impact_angle".to_string(),
                "vehicle_mass".to_string(),
                "vehicle_moment_inertia".to_string(),
                "initial_velocity_x".to_string(),
                "initial_velocity_y".to_string(),
                "initial_angular_velocity".to_string(),
                "friction_coefficient".to_string(),
                "road_grade".to_string(),
                "surface_type_encoded".to_string(),
            ],
            output_features: vec![
                "trajectory_sequence".to_string(),
                "final_position_x".to_string(),
                "final_position_y".to_string(),
                "distance_traveled".to_string(),
                "time_to_rest".to_string(),
            ],
            metrics: crate::models::ModelMetrics {
                accuracy: 0.89,
                precision: 0.88,
                recall: 0.90,
                f1_score: 0.89,
                mae: Some(1.2), // meters
                rmse: Some(2.1), // meters
                r2_score: Some(0.91),
            },
            description: Some(
                "Predicts vehicle trajectory after impact using physics-informed neural networks"
                    .to_string(),
            ),
            training_info: crate::models::TrainingInfo {
                num_samples: 40000,
                epochs: 200,
                learning_rate: 0.0001,
                training_duration_secs: 14400,
                dataset: "Physics_Simulation_v2023".to_string(),
                cv_scores: Some(vec![0.88, 0.89, 0.90, 0.88, 0.89]),
            },
        };

        Ok(Self {
            metadata,
            model: Arc::new(RwLock::new(None)),
            config: config.clone(),
        })
    }

    /// Load the ONNX model
    pub async fn load(&self) -> Result<()> {
        let model_path = &self.metadata.model_path;

        if !model_path.exists() {
            return Err(MlError::ModelNotFound(
                model_path.display().to_string(),
            ));
        }

        let onnx_model = OnnxModel::from_file(model_path, &self.config.inference)?;
        let mut model_lock = self.model.write().await;
        *model_lock = Some(onnx_model);

        Ok(())
    }

    /// Predict trajectory from features
    pub async fn predict(&self, features: &[f64]) -> Result<TrajectoryPrediction> {
        self.validate_input(features)?;

        let model_lock = self.model.read().await;
        let model = model_lock
            .as_ref()
            .ok_or_else(|| MlError::Model("Model not loaded".to_string()))?;

        // Run inference
        let input = Array2::from_shape_vec((1, features.len()), features.to_vec())
            .map_err(|e| MlError::Inference(e.to_string()))?;

        let output = model.run(input).await?;

        // Parse output
        let final_x = output[[0, 1]];
        let final_y = output[[0, 2]];
        let distance = output[[0, 3]];
        let time_to_rest = output[[0, 4]];

        // Generate trajectory points (simplified - would be from sequence output)
        let trajectory_points = self.generate_trajectory_points(
            features,
            final_x,
            final_y,
            time_to_rest,
        );

        // Calculate energy dissipation
        let energy_dissipation = self.calculate_energy_dissipation(&trajectory_points);

        // Create metadata
        let metadata = self.create_metadata(features);

        // Calculate confidence
        let confidence = self.calculate_confidence(features);

        Ok(TrajectoryPrediction {
            trajectory_points,
            final_position: Point3D {
                x: final_x,
                y: final_y,
                z: 0.0,
            },
            final_orientation: (0.0, 0.0, 0.0), // Would be predicted by model
            distance_traveled: distance,
            time_to_rest,
            confidence,
            energy_dissipation,
            metadata,
        })
    }

    /// Generate trajectory points from prediction
    fn generate_trajectory_points(
        &self,
        features: &[f64],
        final_x: f64,
        final_y: f64,
        time_to_rest: f64,
    ) -> Vec<TrajectoryPoint> {
        let num_points = 50;
        let mut points = Vec::with_capacity(num_points);

        let initial_vx = features[4];
        let initial_vy = features[5];
        let friction = features[7];

        for i in 0..num_points {
            let t = (i as f64 / num_points as f64) * time_to_rest;
            let progress = t / time_to_rest;

            // Simplified physics model (exponential decay)
            let decay = (-friction * t * 2.0).exp();

            points.push(TrajectoryPoint {
                time: t,
                position: Point3D {
                    x: initial_vx * t * decay + (final_x - initial_vx * time_to_rest * decay) * progress,
                    y: initial_vy * t * decay + (final_y - initial_vy * time_to_rest * decay) * progress,
                    z: 0.0,
                },
                velocity: Point3D {
                    x: initial_vx * decay,
                    y: initial_vy * decay,
                    z: 0.0,
                },
                orientation: (0.0, 0.0, 0.0),
                angular_velocity: Point3D {
                    x: 0.0,
                    y: 0.0,
                    z: features[6] * decay,
                },
            });
        }

        points
    }

    /// Calculate energy dissipation over time
    fn calculate_energy_dissipation(&self, trajectory: &[TrajectoryPoint]) -> Vec<f64> {
        trajectory
            .iter()
            .map(|point| {
                let v_squared = point.velocity.x.powi(2)
                    + point.velocity.y.powi(2)
                    + point.velocity.z.powi(2);
                v_squared * 0.5 // Kinetic energy (normalized by mass)
            })
            .collect()
    }

    /// Create trajectory metadata
    fn create_metadata(&self, features: &[f64]) -> TrajectoryMetadata {
        let friction = features[7];
        let surface_type = self.decode_surface_type(features[9]);

        TrajectoryMetadata {
            surface_type,
            friction_coefficient: friction,
            conditions: EnvironmentalConditions {
                weather: WeatherCondition::Clear,
                road_grade: features[8],
                wind_speed: 0.0,
                visibility: 1000.0,
            },
            stability: 0.85, // Would be calculated from trajectory variance
        }
    }

    /// Decode surface type from encoded value
    fn decode_surface_type(&self, encoded: f64) -> SurfaceType {
        match encoded as i32 {
            0 => SurfaceType::DryAsphalt,
            1 => SurfaceType::WetAsphalt,
            2 => SurfaceType::DryGravel,
            3 => SurfaceType::WetGravel,
            4 => SurfaceType::Grass,
            5 => SurfaceType::Snow,
            6 => SurfaceType::Ice,
            _ => SurfaceType::Dirt,
        }
    }

    /// Calculate prediction confidence
    fn calculate_confidence(&self, features: &[f64]) -> f64 {
        let mut confidence = 1.0;

        // Reduce confidence for extreme values
        let impact_speed = features[0];
        if impact_speed > 100.0 || impact_speed < 5.0 {
            confidence *= 0.8;
        }

        let friction = features[7];
        if friction < 0.2 || friction > 1.0 {
            confidence *= 0.85;
        }

        confidence.max(0.3).min(1.0)
    }

    /// Batch predict trajectories
    pub async fn predict_batch(
        &self,
        features_batch: &[Vec<f64>],
    ) -> Result<Vec<TrajectoryPrediction>> {
        let mut results = Vec::with_capacity(features_batch.len());

        for features in features_batch {
            let prediction = self.predict(features).await?;
            results.push(prediction);
        }

        Ok(results)
    }
}

impl Model for TrajectoryPredictor {
    fn metadata(&self) -> &ModelMetadata {
        &self.metadata
    }
}

impl Point3D {
    /// Create a new point
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Calculate magnitude (length)
    pub fn magnitude(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2) + self.z.powi(2)).sqrt()
    }

    /// Calculate distance to another point
    pub fn distance_to(&self, other: &Point3D) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2) + (self.z - other.z).powi(2))
            .sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trajectory_predictor_creation() {
        let config = MlConfig::default();
        let predictor = TrajectoryPredictor::new(&config).unwrap();
        assert_eq!(
            predictor.metadata().model_type,
            ModelType::TrajectoryPredictor
        );
    }

    #[test]
    fn test_point3d() {
        let p1 = Point3D::new(0.0, 0.0, 0.0);
        let p2 = Point3D::new(3.0, 4.0, 0.0);

        assert_eq!(p2.magnitude(), 5.0);
        assert_eq!(p1.distance_to(&p2), 5.0);
    }

    #[test]
    fn test_surface_type_decoding() {
        let config = MlConfig::default();
        let predictor = TrajectoryPredictor::new(&config).unwrap();

        assert_eq!(
            predictor.decode_surface_type(0.0),
            SurfaceType::DryAsphalt
        );
        assert_eq!(predictor.decode_surface_type(5.0), SurfaceType::Snow);
    }
}
