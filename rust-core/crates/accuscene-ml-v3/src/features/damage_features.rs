//! Damage pattern feature extraction

use crate::error::{MlError, Result};
use crate::features::{FeatureExtractor, FeatureVector};
use serde::{Deserialize, Serialize};

/// Damage information for feature extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamageInfo {
    /// Deformation depth in meters
    pub deformation_depth: f64,

    /// Deformation width in meters
    pub deformation_width: f64,

    /// Deformation height in meters
    pub deformation_height: f64,

    /// Total deformation area in square meters
    pub deformation_area: f64,

    /// Contact area in square meters
    pub contact_area: f64,

    /// Impact angle in radians (0 = head-on)
    pub impact_angle: f64,

    /// Damage zones (bitflags or individual booleans)
    pub damage_zones: DamageZones,

    /// Paint transfer present
    pub paint_transfer: bool,

    /// Glass damage present
    pub glass_damage: bool,

    /// Airbag deployment
    pub airbag_deployed: bool,

    /// Structural damage present
    pub structural_damage: bool,
}

/// Damage zones on vehicle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamageZones {
    pub front: f64,           // 0.0 - 1.0
    pub front_left: f64,
    pub front_right: f64,
    pub side_left: f64,
    pub side_right: f64,
    pub rear: f64,
    pub rear_left: f64,
    pub rear_right: f64,
    pub roof: f64,
    pub undercarriage: f64,
}

/// Extracted damage features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamageFeatures {
    /// Normalized deformation depth (0.0 - 1.0)
    pub deformation_depth_norm: f64,

    /// Normalized deformation width (0.0 - 1.0)
    pub deformation_width_norm: f64,

    /// Normalized deformation height (0.0 - 1.0)
    pub deformation_height_norm: f64,

    /// Normalized deformation area (0.0 - 1.0)
    pub deformation_area_norm: f64,

    /// Impact angle normalized (0.0 - 1.0)
    pub impact_angle_norm: f64,

    /// Contact area normalized (0.0 - 1.0)
    pub contact_area_norm: f64,

    /// Damage concentration score (0.0 - 1.0)
    pub damage_concentration: f64,

    /// Overall damage severity (0.0 - 1.0)
    pub overall_severity: f64,

    /// Front damage score (0.0 - 1.0)
    pub front_damage: f64,

    /// Side damage score (0.0 - 1.0)
    pub side_damage: f64,

    /// Rear damage score (0.0 - 1.0)
    pub rear_damage: f64,

    /// Binary features
    pub has_paint_transfer: f64,
    pub has_glass_damage: f64,
    pub airbag_deployed: f64,
    pub has_structural_damage: f64,
}

/// Damage feature extractor
pub struct DamageFeatureExtractor {
    /// Maximum expected deformation depth (m)
    max_deformation_depth: f64,

    /// Maximum expected deformation area (m²)
    max_deformation_area: f64,
}

impl Default for DamageFeatureExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl DamageFeatureExtractor {
    /// Create a new damage feature extractor
    pub fn new() -> Self {
        Self {
            max_deformation_depth: 1.5,  // meters
            max_deformation_area: 5.0,   // square meters
        }
    }

    /// Extract features from damage info
    pub fn extract_features(&self, info: &DamageInfo) -> Result<DamageFeatures> {
        // Normalize deformation measurements
        let deformation_depth_norm = (info.deformation_depth / self.max_deformation_depth)
            .clamp(0.0, 1.0);

        let deformation_width_norm = (info.deformation_width / 3.0).clamp(0.0, 1.0);

        let deformation_height_norm = (info.deformation_height / 2.0).clamp(0.0, 1.0);

        let deformation_area_norm = (info.deformation_area / self.max_deformation_area)
            .clamp(0.0, 1.0);

        // Normalize impact angle (0 to π)
        let impact_angle_norm = (info.impact_angle / std::f64::consts::PI).clamp(0.0, 1.0);

        // Normalize contact area
        let contact_area_norm = (info.contact_area / 3.0).clamp(0.0, 1.0);

        // Calculate damage concentration (ratio of contact area to deformation area)
        let damage_concentration = if info.deformation_area > 0.0 {
            (info.contact_area / info.deformation_area).clamp(0.0, 1.0)
        } else {
            0.0
        };

        // Calculate overall severity
        let overall_severity = self.calculate_overall_severity(info, deformation_depth_norm, deformation_area_norm);

        // Aggregate damage zones
        let front_damage = (info.damage_zones.front +
                           info.damage_zones.front_left +
                           info.damage_zones.front_right) / 3.0;

        let side_damage = (info.damage_zones.side_left + info.damage_zones.side_right) / 2.0;

        let rear_damage = (info.damage_zones.rear +
                          info.damage_zones.rear_left +
                          info.damage_zones.rear_right) / 3.0;

        // Binary features
        let has_paint_transfer = if info.paint_transfer { 1.0 } else { 0.0 };
        let has_glass_damage = if info.glass_damage { 1.0 } else { 0.0 };
        let airbag_deployed = if info.airbag_deployed { 1.0 } else { 0.0 };
        let has_structural_damage = if info.structural_damage { 1.0 } else { 0.0 };

        Ok(DamageFeatures {
            deformation_depth_norm,
            deformation_width_norm,
            deformation_height_norm,
            deformation_area_norm,
            impact_angle_norm,
            contact_area_norm,
            damage_concentration,
            overall_severity,
            front_damage,
            side_damage,
            rear_damage,
            has_paint_transfer,
            has_glass_damage,
            airbag_deployed,
            has_structural_damage,
        })
    }

    /// Calculate overall damage severity
    fn calculate_overall_severity(
        &self,
        info: &DamageInfo,
        depth_norm: f64,
        area_norm: f64,
    ) -> f64 {
        let mut severity = 0.0;

        // Deformation contributes 40%
        severity += (depth_norm * 0.7 + area_norm * 0.3) * 0.4;

        // Structural damage contributes 30%
        if info.structural_damage {
            severity += 0.3;
        }

        // Airbag deployment contributes 15%
        if info.airbag_deployed {
            severity += 0.15;
        }

        // Glass damage contributes 10%
        if info.glass_damage {
            severity += 0.1;
        }

        // Multiple damage zones contribute 5%
        let zone_count = [
            info.damage_zones.front,
            info.damage_zones.side_left,
            info.damage_zones.side_right,
            info.damage_zones.rear,
        ]
        .iter()
        .filter(|&&z| z > 0.3)
        .count();

        if zone_count > 1 {
            severity += 0.05;
        }

        severity.clamp(0.0, 1.0)
    }
}

impl DamageFeatures {
    /// Convert to feature vector
    pub fn to_feature_vector(&self) -> Vec<f64> {
        vec![
            self.deformation_depth_norm,
            self.deformation_width_norm,
            self.deformation_height_norm,
            self.deformation_area_norm,
            self.impact_angle_norm,
            self.contact_area_norm,
            self.damage_concentration,
            self.overall_severity,
            self.front_damage,
            self.side_damage,
            self.rear_damage,
            self.has_paint_transfer,
            self.has_glass_damage,
            self.airbag_deployed,
            self.has_structural_damage,
        ]
    }

    /// Get feature names
    pub fn feature_names() -> Vec<String> {
        vec![
            "deformation_depth_norm".to_string(),
            "deformation_width_norm".to_string(),
            "deformation_height_norm".to_string(),
            "deformation_area_norm".to_string(),
            "impact_angle_norm".to_string(),
            "contact_area_norm".to_string(),
            "damage_concentration".to_string(),
            "overall_severity".to_string(),
            "front_damage".to_string(),
            "side_damage".to_string(),
            "rear_damage".to_string(),
            "has_paint_transfer".to_string(),
            "has_glass_damage".to_string(),
            "airbag_deployed".to_string(),
            "has_structural_damage".to_string(),
        ]
    }
}

impl FeatureExtractor for DamageFeatureExtractor {
    fn extract(&self, input: &dyn std::any::Any) -> Result<FeatureVector> {
        let info = input
            .downcast_ref::<DamageInfo>()
            .ok_or_else(|| MlError::FeatureExtraction("Invalid input type".to_string()))?;

        let features = self.extract_features(info)?;
        Ok(FeatureVector::new(
            features.to_feature_vector(),
            DamageFeatures::feature_names(),
        ))
    }

    fn feature_names(&self) -> Vec<String> {
        DamageFeatures::feature_names()
    }
}

impl Default for DamageZones {
    fn default() -> Self {
        Self {
            front: 0.0,
            front_left: 0.0,
            front_right: 0.0,
            side_left: 0.0,
            side_right: 0.0,
            rear: 0.0,
            rear_left: 0.0,
            rear_right: 0.0,
            roof: 0.0,
            undercarriage: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_damage() -> DamageInfo {
        DamageInfo {
            deformation_depth: 0.5,
            deformation_width: 1.0,
            deformation_height: 0.8,
            deformation_area: 1.5,
            contact_area: 0.8,
            impact_angle: std::f64::consts::PI / 4.0,
            damage_zones: DamageZones {
                front: 0.8,
                front_left: 0.6,
                ..Default::default()
            },
            paint_transfer: true,
            glass_damage: false,
            airbag_deployed: true,
            structural_damage: false,
        }
    }

    #[test]
    fn test_damage_feature_extraction() {
        let extractor = DamageFeatureExtractor::new();
        let damage = create_test_damage();
        let features = extractor.extract_features(&damage).unwrap();

        assert!(features.deformation_depth_norm >= 0.0 && features.deformation_depth_norm <= 1.0);
        assert!(features.overall_severity > 0.0);
        assert_eq!(features.has_paint_transfer, 1.0);
        assert_eq!(features.has_glass_damage, 0.0);
    }

    #[test]
    fn test_damage_concentration() {
        let extractor = DamageFeatureExtractor::new();
        let damage = create_test_damage();
        let features = extractor.extract_features(&damage).unwrap();

        // Contact area / deformation area should be between 0 and 1
        assert!(features.damage_concentration >= 0.0 && features.damage_concentration <= 1.0);
    }
}
