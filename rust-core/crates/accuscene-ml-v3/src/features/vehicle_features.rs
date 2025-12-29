//! Vehicle property feature extraction

use crate::error::{MlError, Result};
use crate::features::{FeatureExtractor, FeatureVector};
use serde::{Deserialize, Serialize};

/// Vehicle information for feature extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleInfo {
    /// Vehicle make (e.g., "Toyota")
    pub make: String,

    /// Vehicle model (e.g., "Camry")
    pub model: String,

    /// Model year
    pub year: u16,

    /// Vehicle mass in kg
    pub mass: f64,

    /// Wheelbase in meters
    pub wheelbase: f64,

    /// Track width in meters
    pub track_width: f64,

    /// Center of gravity height in meters
    pub cg_height: f64,

    /// Frontal area in square meters
    pub frontal_area: f64,

    /// Structural stiffness coefficient
    pub stiffness: f64,

    /// Safety rating (1-5 stars)
    pub safety_rating: f64,

    /// Vehicle type
    pub vehicle_type: VehicleType,
}

/// Vehicle type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VehicleType {
    Sedan,
    SUV,
    Truck,
    Van,
    Coupe,
    Convertible,
    Motorcycle,
    Bus,
    Unknown,
}

/// Extracted vehicle features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleFeatures {
    /// Normalized vehicle mass (0.0 - 1.0)
    pub mass_normalized: f64,

    /// Vehicle size factor (0.0 - 1.0)
    pub size_factor: f64,

    /// Structural rigidity score (0.0 - 1.0)
    pub rigidity: f64,

    /// Safety score (0.0 - 1.0)
    pub safety_score: f64,

    /// Center of gravity height normalized (0.0 - 1.0)
    pub cg_height_normalized: f64,

    /// Vehicle age factor (0.0 - 1.0, newer = higher)
    pub age_factor: f64,

    /// Vehicle type encoded
    pub vehicle_type_encoded: f64,

    /// Moment of inertia (estimated, normalized)
    pub moment_inertia: f64,
}

/// Vehicle feature extractor
pub struct VehicleFeatureExtractor {
    /// Reference mass for normalization (kg)
    reference_mass: f64,

    /// Reference wheelbase (m)
    reference_wheelbase: f64,

    /// Current year for age calculation
    current_year: u16,
}

impl Default for VehicleFeatureExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl VehicleFeatureExtractor {
    /// Create a new vehicle feature extractor
    pub fn new() -> Self {
        Self {
            reference_mass: 1500.0,      // Average car mass
            reference_wheelbase: 2.8,     // Average wheelbase
            current_year: 2024,
        }
    }

    /// Extract features from vehicle info
    pub fn extract_features(&self, info: &VehicleInfo) -> Result<VehicleFeatures> {
        // Normalize mass (typical range: 500 - 3000 kg)
        let mass_normalized = ((info.mass - 500.0) / 2500.0).clamp(0.0, 1.0);

        // Calculate size factor from wheelbase and track width
        let size_factor = ((info.wheelbase * info.track_width) / (2.8 * 1.5)).clamp(0.0, 1.0);

        // Rigidity score (stiffness normalized)
        let rigidity = info.stiffness.clamp(0.0, 1.0);

        // Safety score (1-5 stars normalized to 0-1)
        let safety_score = (info.safety_rating / 5.0).clamp(0.0, 1.0);

        // CG height normalized (typical range: 0.4 - 1.2 m)
        let cg_height_normalized = ((info.cg_height - 0.4) / 0.8).clamp(0.0, 1.0);

        // Age factor (newer vehicles get higher scores)
        let age = self.current_year.saturating_sub(info.year);
        let age_factor = (1.0 - (age as f64 / 30.0)).clamp(0.0, 1.0);

        // Vehicle type encoding
        let vehicle_type_encoded = self.encode_vehicle_type(info.vehicle_type);

        // Estimate moment of inertia (simplified)
        let moment_inertia = self.calculate_moment_inertia(info);

        Ok(VehicleFeatures {
            mass_normalized,
            size_factor,
            rigidity,
            safety_score,
            cg_height_normalized,
            age_factor,
            vehicle_type_encoded,
            moment_inertia,
        })
    }

    /// Encode vehicle type to numerical value
    fn encode_vehicle_type(&self, vehicle_type: VehicleType) -> f64 {
        match vehicle_type {
            VehicleType::Sedan => 0.0,
            VehicleType::SUV => 0.2,
            VehicleType::Truck => 0.4,
            VehicleType::Van => 0.6,
            VehicleType::Coupe => 0.1,
            VehicleType::Convertible => 0.15,
            VehicleType::Motorcycle => 0.8,
            VehicleType::Bus => 1.0,
            VehicleType::Unknown => 0.5,
        }
    }

    /// Calculate normalized moment of inertia
    fn calculate_moment_inertia(&self, info: &VehicleInfo) -> f64 {
        // Simplified calculation: I ≈ m * (L² + W²) / 12
        let length_sq = info.wheelbase.powi(2);
        let width_sq = info.track_width.powi(2);
        let inertia = info.mass * (length_sq + width_sq) / 12.0;

        // Normalize to typical range (500 - 5000 kg⋅m²)
        ((inertia - 500.0) / 4500.0).clamp(0.0, 1.0)
    }
}

impl VehicleFeatures {
    /// Convert to feature vector
    pub fn to_feature_vector(&self) -> Vec<f64> {
        vec![
            self.mass_normalized,
            self.size_factor,
            self.rigidity,
            self.safety_score,
            self.cg_height_normalized,
            self.age_factor,
            self.vehicle_type_encoded,
            self.moment_inertia,
        ]
    }

    /// Get feature names
    pub fn feature_names() -> Vec<String> {
        vec![
            "mass_normalized".to_string(),
            "size_factor".to_string(),
            "rigidity".to_string(),
            "safety_score".to_string(),
            "cg_height_normalized".to_string(),
            "age_factor".to_string(),
            "vehicle_type_encoded".to_string(),
            "moment_inertia".to_string(),
        ]
    }
}

impl FeatureExtractor for VehicleFeatureExtractor {
    fn extract(&self, input: &dyn std::any::Any) -> Result<FeatureVector> {
        let info = input
            .downcast_ref::<VehicleInfo>()
            .ok_or_else(|| MlError::FeatureExtraction("Invalid input type".to_string()))?;

        let features = self.extract_features(info)?;
        Ok(FeatureVector::new(
            features.to_feature_vector(),
            VehicleFeatures::feature_names(),
        ))
    }

    fn feature_names(&self) -> Vec<String> {
        VehicleFeatures::feature_names()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_vehicle() -> VehicleInfo {
        VehicleInfo {
            make: "Toyota".to_string(),
            model: "Camry".to_string(),
            year: 2020,
            mass: 1500.0,
            wheelbase: 2.8,
            track_width: 1.5,
            cg_height: 0.6,
            frontal_area: 2.5,
            stiffness: 0.7,
            safety_rating: 5.0,
            vehicle_type: VehicleType::Sedan,
        }
    }

    #[test]
    fn test_vehicle_feature_extraction() {
        let extractor = VehicleFeatureExtractor::new();
        let vehicle = create_test_vehicle();
        let features = extractor.extract_features(&vehicle).unwrap();

        assert!(features.mass_normalized >= 0.0 && features.mass_normalized <= 1.0);
        assert!(features.safety_score >= 0.0 && features.safety_score <= 1.0);
        assert_eq!(features.safety_score, 1.0); // 5 stars = 1.0
    }

    #[test]
    fn test_vehicle_type_encoding() {
        let extractor = VehicleFeatureExtractor::new();

        assert_eq!(extractor.encode_vehicle_type(VehicleType::Sedan), 0.0);
        assert_eq!(extractor.encode_vehicle_type(VehicleType::SUV), 0.2);
        assert_eq!(extractor.encode_vehicle_type(VehicleType::Bus), 1.0);
    }
}
