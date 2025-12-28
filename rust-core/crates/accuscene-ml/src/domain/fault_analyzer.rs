//! Accident fault analysis model

use crate::algorithms::classification::DecisionTreeClassifier;
use crate::error::Result;
use crate::model::{Classifier, Model, ModelMetadata, ModelType};
use async_trait::async_trait;
use ndarray::{Array1, Array2};
use serde::{Deserialize, Serialize};

/// Fault analyzer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaultAnalyzer {
    model: DecisionTreeClassifier,
    metadata: ModelMetadata,
}

impl FaultAnalyzer {
    pub fn new() -> Self {
        let metadata = ModelMetadata::new(
            "fault_analyzer",
            "0.2.0",
            ModelType::Custom("FaultAnalyzer".to_string()),
        );

        Self {
            model: DecisionTreeClassifier::new(),
            metadata,
        }
    }

    /// Analyze fault distribution from accident evidence
    pub async fn analyze_fault(&self, evidence: &AccidentEvidence) -> Result<FaultAnalysis> {
        let features = self.extract_features(evidence);
        let prediction = self.model.predict(&features).await?;

        let primary_fault = match prediction as usize {
            0 => FaultParty::Driver1,
            1 => FaultParty::Driver2,
            2 => FaultParty::Both,
            3 => FaultParty::Environmental,
            _ => FaultParty::Unknown,
        };

        let fault_distribution = self.calculate_distribution(&evidence);

        Ok(FaultAnalysis {
            primary_fault,
            driver1_percentage: fault_distribution.0,
            driver2_percentage: fault_distribution.1,
            contributing_factors: evidence.contributing_factors.clone(),
        })
    }

    fn extract_features(&self, evidence: &AccidentEvidence) -> Array1<f64> {
        ndarray::arr1(&[
            if evidence.driver1_speeding { 1.0 } else { 0.0 },
            if evidence.driver2_speeding { 1.0 } else { 0.0 },
            if evidence.driver1_right_of_way { 1.0 } else { 0.0 },
            if evidence.driver2_right_of_way { 1.0 } else { 0.0 },
            if evidence.poor_weather { 1.0 } else { 0.0 },
            if evidence.road_defect { 1.0 } else { 0.0 },
            evidence.driver1_violations as f64,
            evidence.driver2_violations as f64,
        ])
    }

    fn calculate_distribution(&self, evidence: &AccidentEvidence) -> (f64, f64) {
        let mut driver1_fault = 0.0;
        let mut driver2_fault = 0.0;

        if evidence.driver1_speeding {
            driver1_fault += 20.0;
        }
        if evidence.driver2_speeding {
            driver2_fault += 20.0;
        }
        if !evidence.driver1_right_of_way {
            driver1_fault += 30.0;
        }
        if !evidence.driver2_right_of_way {
            driver2_fault += 30.0;
        }

        driver1_fault += evidence.driver1_violations as f64 * 10.0;
        driver2_fault += evidence.driver2_violations as f64 * 10.0;

        let total = driver1_fault + driver2_fault;
        if total > 0.0 {
            (
                (driver1_fault / total * 100.0).min(100.0),
                (driver2_fault / total * 100.0).min(100.0),
            )
        } else {
            (50.0, 50.0)
        }
    }
}

impl Default for FaultAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Model for FaultAnalyzer {
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
impl Classifier for FaultAnalyzer {
    async fn predict_proba(&self, _features: &Array1<f64>) -> Result<Vec<f64>> {
        Ok(vec![0.0; 5])
    }

    async fn predict_proba_batch(&self, features: &Array2<f64>) -> Result<Array2<f64>> {
        Ok(Array2::zeros((features.nrows(), 5)))
    }

    fn num_classes(&self) -> usize {
        5
    }
}

/// Accident evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccidentEvidence {
    pub driver1_speeding: bool,
    pub driver2_speeding: bool,
    pub driver1_right_of_way: bool,
    pub driver2_right_of_way: bool,
    pub poor_weather: bool,
    pub road_defect: bool,
    pub driver1_violations: u32,
    pub driver2_violations: u32,
    pub contributing_factors: Vec<String>,
}

/// Fault analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaultAnalysis {
    pub primary_fault: FaultParty,
    pub driver1_percentage: f64,
    pub driver2_percentage: f64,
    pub contributing_factors: Vec<String>,
}

/// Fault party
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FaultParty {
    Driver1,
    Driver2,
    Both,
    Environmental,
    Unknown,
}
