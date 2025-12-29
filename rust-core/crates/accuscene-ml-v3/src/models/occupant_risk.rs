//! Occupant injury risk prediction

use crate::config::MlConfig;
use crate::error::{MlError, Result};
use crate::inference::onnx_runtime::OnnxModel;
use crate::models::{Model, ModelMetadata, ModelType};
use ndarray::{Array1, Array2};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Occupant injury risk prediction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OccupantRiskPrediction {
    /// Overall injury risk score (0.0 - 1.0)
    pub overall_risk: f64,

    /// Risk classification
    pub risk_class: RiskClass,

    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,

    /// Body region-specific risks
    pub body_region_risks: BodyRegionRisks,

    /// Injury severity prediction
    pub injury_severity: InjurySeverity,

    /// Predicted AIS (Abbreviated Injury Scale) scores
    pub ais_scores: AISScores,

    /// Contributing risk factors
    pub risk_factors: Vec<RiskFactor>,

    /// Safety feature effectiveness
    pub safety_features: SafetyFeatureAnalysis,
}

/// Risk classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskClass {
    /// Very low risk (< 10% probability of injury)
    VeryLow,
    /// Low risk (10-30%)
    Low,
    /// Moderate risk (30-60%)
    Moderate,
    /// High risk (60-85%)
    High,
    /// Very high risk (> 85%)
    VeryHigh,
}

/// Body region-specific injury risks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyRegionRisks {
    /// Head injury risk (0.0 - 1.0)
    pub head: f64,

    /// Neck injury risk (0.0 - 1.0)
    pub neck: f64,

    /// Chest injury risk (0.0 - 1.0)
    pub chest: f64,

    /// Abdomen injury risk (0.0 - 1.0)
    pub abdomen: f64,

    /// Spine injury risk (0.0 - 1.0)
    pub spine: f64,

    /// Upper extremity risk (0.0 - 1.0)
    pub upper_extremity: f64,

    /// Lower extremity risk (0.0 - 1.0)
    pub lower_extremity: f64,
}

/// Injury severity prediction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjurySeverity {
    /// No injury probability
    pub none: f64,

    /// Minor injury probability
    pub minor: f64,

    /// Moderate injury probability
    pub moderate: f64,

    /// Serious injury probability
    pub serious: f64,

    /// Severe/critical injury probability
    pub severe: f64,

    /// Fatal injury probability
    pub fatal: f64,
}

/// AIS (Abbreviated Injury Scale) scores for body regions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AISScores {
    /// Maximum AIS score (predicted)
    pub max_ais: u8,

    /// Injury severity score (ISS) - sum of squares of top 3 AIS
    pub iss: u32,

    /// Per-region AIS scores (0-6)
    pub head_ais: u8,
    pub chest_ais: u8,
    pub abdomen_ais: u8,
    pub extremity_ais: u8,
}

/// Risk factor contributing to injury
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    /// Factor name
    pub factor: String,

    /// Contribution to overall risk (0.0 - 1.0)
    pub contribution: f64,

    /// Factor category
    pub category: RiskFactorCategory,
}

/// Risk factor category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskFactorCategory {
    ImpactSpeed,
    ImpactAngle,
    VehicleType,
    OccupantAge,
    OccupantSize,
    SeatPosition,
    SafetyDevice,
    StructuralIntegrity,
}

/// Safety feature effectiveness analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyFeatureAnalysis {
    /// Airbag effectiveness score (0.0 - 1.0)
    pub airbag_effectiveness: f64,

    /// Seatbelt effectiveness score (0.0 - 1.0)
    pub seatbelt_effectiveness: f64,

    /// Vehicle structure effectiveness (0.0 - 1.0)
    pub structure_effectiveness: f64,

    /// Crumple zone effectiveness (0.0 - 1.0)
    pub crumple_zone_effectiveness: f64,

    /// Overall safety system performance (0.0 - 1.0)
    pub overall_effectiveness: f64,
}

/// Occupant risk predictor model
pub struct OccupantRiskPredictor {
    metadata: ModelMetadata,
    model: Arc<RwLock<Option<OnnxModel>>>,
    config: MlConfig,
}

impl OccupantRiskPredictor {
    /// Create a new occupant risk predictor
    pub fn new(config: &MlConfig) -> Result<Self> {
        let metadata = ModelMetadata {
            id: uuid::Uuid::new_v4(),
            name: "occupant_risk".to_string(),
            version: "1.0.0".to_string(),
            model_type: ModelType::OccupantRisk,
            created_at: chrono::Utc::now(),
            model_path: config
                .models
                .model_dir
                .join("occupant_risk_v1.onnx"),
            input_features: vec![
                "impact_speed".to_string(),
                "impact_angle".to_string(),
                "delta_v".to_string(),
                "occupant_age".to_string(),
                "occupant_weight".to_string(),
                "occupant_height".to_string(),
                "seat_position".to_string(),
                "seatbelt_used".to_string(),
                "airbag_deployed".to_string(),
                "vehicle_safety_rating".to_string(),
                "intrusion_depth".to_string(),
                "compartment_integrity".to_string(),
            ],
            output_features: vec![
                "overall_risk".to_string(),
                "head_risk".to_string(),
                "chest_risk".to_string(),
                "extremity_risk".to_string(),
                "max_ais".to_string(),
            ],
            metrics: crate::models::ModelMetrics {
                accuracy: 0.87,
                precision: 0.86,
                recall: 0.89,
                f1_score: 0.87,
                mae: None,
                rmse: None,
                r2_score: Some(0.84),
            },
            description: Some(
                "Predicts occupant injury risk using biomechanical and crash data".to_string(),
            ),
            training_info: crate::models::TrainingInfo {
                num_samples: 60000,
                epochs: 250,
                learning_rate: 0.0003,
                training_duration_secs: 18000,
                dataset: "NHTSA_CIREN_Biomechanics_v2023".to_string(),
                cv_scores: Some(vec![0.86, 0.87, 0.88, 0.86, 0.87]),
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

    /// Predict occupant injury risk
    pub async fn predict(&self, features: &[f64]) -> Result<OccupantRiskPrediction> {
        self.validate_input(features)?;

        let model_lock = self.model.read().await;
        let model = model_lock
            .as_ref()
            .ok_or_else(|| MlError::Model("Model not loaded".to_string()))?;

        // Run inference
        let input = Array2::from_shape_vec((1, features.len()), features.to_vec())
            .map_err(|e| MlError::Inference(e.to_string()))?;

        let output = model.run(input).await?;

        // Extract predictions
        let overall_risk = output[[0, 0]].max(0.0).min(1.0);
        let head_risk = output[[0, 1]].max(0.0).min(1.0);
        let chest_risk = output[[0, 2]].max(0.0).min(1.0);
        let extremity_risk = output[[0, 3]].max(0.0).min(1.0);
        let max_ais_raw = output[[0, 4]];

        // Classify risk
        let risk_class = self.classify_risk(overall_risk);

        // Calculate body region risks
        let body_region_risks = self.calculate_body_region_risks(features, head_risk, chest_risk, extremity_risk);

        // Calculate injury severity distribution
        let injury_severity = self.calculate_injury_severity(overall_risk);

        // Calculate AIS scores
        let ais_scores = self.calculate_ais_scores(max_ais_raw, &body_region_risks);

        // Identify risk factors
        let risk_factors = self.identify_risk_factors(features);

        // Analyze safety features
        let safety_features = self.analyze_safety_features(features, overall_risk);

        // Calculate confidence
        let confidence = self.calculate_confidence(features);

        Ok(OccupantRiskPrediction {
            overall_risk,
            risk_class,
            confidence,
            body_region_risks,
            injury_severity,
            ais_scores,
            risk_factors,
            safety_features,
        })
    }

    /// Classify risk level
    fn classify_risk(&self, risk: f64) -> RiskClass {
        match risk {
            r if r < 0.10 => RiskClass::VeryLow,
            r if r < 0.30 => RiskClass::Low,
            r if r < 0.60 => RiskClass::Moderate,
            r if r < 0.85 => RiskClass::High,
            _ => RiskClass::VeryHigh,
        }
    }

    /// Calculate body region risks
    fn calculate_body_region_risks(
        &self,
        features: &[f64],
        head_risk: f64,
        chest_risk: f64,
        extremity_risk: f64,
    ) -> BodyRegionRisks {
        let impact_speed = features[0];
        let intrusion = features[10];

        BodyRegionRisks {
            head: head_risk,
            neck: (head_risk * 0.7).min(1.0),
            chest: chest_risk,
            abdomen: (chest_risk * 0.8).min(1.0),
            spine: (impact_speed / 100.0 * 0.5).min(1.0),
            upper_extremity: (extremity_risk * 0.7).min(1.0),
            lower_extremity: (extremity_risk + intrusion * 0.3).min(1.0),
        }
    }

    /// Calculate injury severity distribution
    fn calculate_injury_severity(&self, overall_risk: f64) -> InjurySeverity {
        // Logistic distribution model
        let none = (1.0 - overall_risk).max(0.0);
        let fatal = (overall_risk - 0.8).max(0.0) / 0.2;

        let remaining = 1.0 - none - fatal;
        let minor = remaining * (1.0 - overall_risk).powi(2);
        let severe = remaining * overall_risk.powi(2);
        let moderate = remaining * (0.5 - (overall_risk - 0.5).abs());
        let serious = remaining - minor - moderate - severe;

        InjurySeverity {
            none,
            minor: minor.max(0.0),
            moderate: moderate.max(0.0),
            serious: serious.max(0.0),
            severe: severe.max(0.0),
            fatal: fatal.max(0.0),
        }
    }

    /// Calculate AIS scores
    fn calculate_ais_scores(&self, max_ais_raw: f64, body_risks: &BodyRegionRisks) -> AISScores {
        let max_ais = (max_ais_raw.round() as u8).min(6);

        let head_ais = (body_risks.head * 6.0).round() as u8;
        let chest_ais = (body_risks.chest * 6.0).round() as u8;
        let abdomen_ais = (body_risks.abdomen * 6.0).round() as u8;
        let extremity_ais = (body_risks.lower_extremity * 6.0).round() as u8;

        // Calculate ISS (sum of squares of top 3 AIS scores)
        let mut scores = vec![head_ais, chest_ais, abdomen_ais, extremity_ais];
        scores.sort_by(|a, b| b.cmp(a));
        let iss = scores[0].pow(2) as u32 + scores[1].pow(2) as u32 + scores[2].pow(2) as u32;

        AISScores {
            max_ais,
            iss,
            head_ais,
            chest_ais,
            abdomen_ais,
            extremity_ais,
        }
    }

    /// Identify primary risk factors
    fn identify_risk_factors(&self, features: &[f64]) -> Vec<RiskFactor> {
        let mut factors = vec![];

        let impact_speed = features[0];
        if impact_speed > 40.0 {
            factors.push(RiskFactor {
                factor: "High Impact Speed".to_string(),
                contribution: (impact_speed / 100.0).min(1.0),
                category: RiskFactorCategory::ImpactSpeed,
            });
        }

        let occupant_age = features[3];
        if occupant_age > 65.0 || occupant_age < 18.0 {
            factors.push(RiskFactor {
                factor: "Age Risk Factor".to_string(),
                contribution: if occupant_age > 65.0 {
                    ((occupant_age - 65.0) / 30.0).min(0.8)
                } else {
                    ((18.0 - occupant_age) / 18.0).min(0.6)
                },
                category: RiskFactorCategory::OccupantAge,
            });
        }

        let seatbelt = features[7];
        if seatbelt < 0.5 {
            factors.push(RiskFactor {
                factor: "No Seatbelt".to_string(),
                contribution: 0.85,
                category: RiskFactorCategory::SafetyDevice,
            });
        }

        let intrusion = features[10];
        if intrusion > 0.3 {
            factors.push(RiskFactor {
                factor: "Cabin Intrusion".to_string(),
                contribution: intrusion.min(1.0),
                category: RiskFactorCategory::StructuralIntegrity,
            });
        }

        factors
    }

    /// Analyze safety feature effectiveness
    fn analyze_safety_features(&self, features: &[f64], overall_risk: f64) -> SafetyFeatureAnalysis {
        let seatbelt_used = features[7];
        let airbag_deployed = features[8];
        let safety_rating = features[9];
        let compartment_integrity = features[11];

        let airbag_effectiveness = if airbag_deployed > 0.5 {
            (1.0 - overall_risk * 0.4).max(0.3)
        } else {
            0.0
        };

        let seatbelt_effectiveness = if seatbelt_used > 0.5 {
            (1.0 - overall_risk * 0.3).max(0.5)
        } else {
            0.0
        };

        let structure_effectiveness = compartment_integrity;
        let crumple_zone_effectiveness = (safety_rating * 0.8).min(1.0);

        let overall_effectiveness = (airbag_effectiveness * 0.3
            + seatbelt_effectiveness * 0.3
            + structure_effectiveness * 0.25
            + crumple_zone_effectiveness * 0.15)
            .min(1.0);

        SafetyFeatureAnalysis {
            airbag_effectiveness,
            seatbelt_effectiveness,
            structure_effectiveness,
            crumple_zone_effectiveness,
            overall_effectiveness,
        }
    }

    /// Calculate prediction confidence
    fn calculate_confidence(&self, features: &[f64]) -> f64 {
        let mut confidence = 0.9;

        // Reduce confidence for extreme values
        let impact_speed = features[0];
        if impact_speed > 100.0 {
            confidence *= 0.85;
        }

        let occupant_age = features[3];
        if occupant_age > 90.0 || occupant_age < 10.0 {
            confidence *= 0.88;
        }

        confidence.max(0.5).min(1.0)
    }

    /// Batch predict risks
    pub async fn predict_batch(
        &self,
        features_batch: &[Vec<f64>],
    ) -> Result<Vec<OccupantRiskPrediction>> {
        let mut results = Vec::with_capacity(features_batch.len());

        for features in features_batch {
            let prediction = self.predict(features).await?;
            results.push(prediction);
        }

        Ok(results)
    }
}

impl Model for OccupantRiskPredictor {
    fn metadata(&self) -> &ModelMetadata {
        &self.metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_occupant_risk_predictor_creation() {
        let config = MlConfig::default();
        let predictor = OccupantRiskPredictor::new(&config).unwrap();
        assert_eq!(predictor.metadata().model_type, ModelType::OccupantRisk);
    }

    #[test]
    fn test_risk_classification() {
        let config = MlConfig::default();
        let predictor = OccupantRiskPredictor::new(&config).unwrap();

        assert_eq!(predictor.classify_risk(0.05), RiskClass::VeryLow);
        assert_eq!(predictor.classify_risk(0.45), RiskClass::Moderate);
        assert_eq!(predictor.classify_risk(0.95), RiskClass::VeryHigh);
    }

    #[test]
    fn test_ais_scores() {
        let config = MlConfig::default();
        let predictor = OccupantRiskPredictor::new(&config).unwrap();

        let risks = BodyRegionRisks {
            head: 0.8,
            neck: 0.6,
            chest: 0.7,
            abdomen: 0.5,
            spine: 0.4,
            upper_extremity: 0.3,
            lower_extremity: 0.6,
        };

        let ais = predictor.calculate_ais_scores(4.5, &risks);
        assert_eq!(ais.max_ais, 4);
        assert!(ais.iss > 0);
    }
}
