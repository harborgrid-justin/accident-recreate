//! Scene context feature extraction

use crate::error::{MlError, Result};
use crate::features::{FeatureExtractor, FeatureVector};
use serde::{Deserialize, Serialize};

/// Scene information for feature extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneInfo {
    /// Road type
    pub road_type: RoadType,

    /// Road surface condition
    pub surface_condition: SurfaceCondition,

    /// Weather conditions
    pub weather: WeatherCondition,

    /// Lighting conditions
    pub lighting: LightingCondition,

    /// Road geometry
    pub road_geometry: RoadGeometry,

    /// Traffic density (0.0 - 1.0)
    pub traffic_density: f64,

    /// Visibility in meters
    pub visibility: f64,

    /// Road speed limit in km/h
    pub speed_limit: f64,

    /// Presence of traffic control
    pub traffic_control: TrafficControl,
}

/// Road type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoadType {
    Highway,
    Arterial,
    Collector,
    LocalStreet,
    Parking,
    PrivateProperty,
}

/// Surface condition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SurfaceCondition {
    DryPavement,
    WetPavement,
    IceCovered,
    SnowCovered,
    Gravel,
    Dirt,
}

/// Weather condition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeatherCondition {
    Clear,
    Cloudy,
    Rain,
    HeavyRain,
    Snow,
    Fog,
    Sleet,
}

/// Lighting condition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LightingCondition {
    Daylight,
    Dawn,
    Dusk,
    Darkness,
    DarknessLighted,
}

/// Road geometry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadGeometry {
    /// Number of lanes
    pub lanes: u8,

    /// Lane width in meters
    pub lane_width: f64,

    /// Road curvature (0 = straight, higher = more curved)
    pub curvature: f64,

    /// Road grade (slope) in degrees
    pub grade: f64,

    /// Is intersection
    pub is_intersection: bool,

    /// Intersection type (if applicable)
    pub intersection_type: Option<IntersectionType>,
}

/// Intersection type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntersectionType {
    FourWay,
    ThreeWay,
    Roundabout,
    OnOffRamp,
}

/// Traffic control
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrafficControl {
    None,
    YieldSign,
    StopSign,
    TrafficSignal,
    Officer,
}

/// Extracted scene features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneFeatures {
    /// Road type encoded (0.0 - 1.0)
    pub road_type_encoded: f64,

    /// Surface friction coefficient (0.0 - 1.0)
    pub friction_coefficient: f64,

    /// Weather severity (0.0 - 1.0)
    pub weather_severity: f64,

    /// Lighting quality (0.0 - 1.0, higher = better)
    pub lighting_quality: f64,

    /// Road complexity (0.0 - 1.0)
    pub road_complexity: f64,

    /// Traffic density normalized (0.0 - 1.0)
    pub traffic_density_norm: f64,

    /// Visibility normalized (0.0 - 1.0)
    pub visibility_norm: f64,

    /// Speed environment factor (0.0 - 1.0)
    pub speed_environment: f64,

    /// Safety infrastructure score (0.0 - 1.0)
    pub safety_infrastructure: f64,

    /// Environmental risk factor (0.0 - 1.0)
    pub environmental_risk: f64,
}

/// Scene feature extractor
pub struct SceneFeatureExtractor;

impl Default for SceneFeatureExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl SceneFeatureExtractor {
    /// Create a new scene feature extractor
    pub fn new() -> Self {
        Self
    }

    /// Extract features from scene info
    pub fn extract_features(&self, info: &SceneInfo) -> Result<SceneFeatures> {
        // Encode road type
        let road_type_encoded = self.encode_road_type(info.road_type);

        // Calculate friction coefficient based on surface
        let friction_coefficient = self.calculate_friction(info.surface_condition);

        // Weather severity
        let weather_severity = self.calculate_weather_severity(info.weather);

        // Lighting quality
        let lighting_quality = self.calculate_lighting_quality(info.lighting);

        // Road complexity from geometry
        let road_complexity = self.calculate_road_complexity(&info.road_geometry);

        // Normalize traffic density (already 0-1)
        let traffic_density_norm = info.traffic_density.clamp(0.0, 1.0);

        // Normalize visibility (assume max 1000m)
        let visibility_norm = (info.visibility / 1000.0).clamp(0.0, 1.0);

        // Speed environment (normalized speed limit)
        let speed_environment = (info.speed_limit / 120.0).clamp(0.0, 1.0);

        // Safety infrastructure score
        let safety_infrastructure = self.calculate_safety_infrastructure(
            &info.road_geometry,
            info.traffic_control,
        );

        // Environmental risk combines weather, lighting, and surface
        let environmental_risk = self.calculate_environmental_risk(
            weather_severity,
            lighting_quality,
            friction_coefficient,
        );

        Ok(SceneFeatures {
            road_type_encoded,
            friction_coefficient,
            weather_severity,
            lighting_quality,
            road_complexity,
            traffic_density_norm,
            visibility_norm,
            speed_environment,
            safety_infrastructure,
            environmental_risk,
        })
    }

    /// Encode road type to numerical value
    fn encode_road_type(&self, road_type: RoadType) -> f64 {
        match road_type {
            RoadType::Parking => 0.0,
            RoadType::PrivateProperty => 0.1,
            RoadType::LocalStreet => 0.3,
            RoadType::Collector => 0.5,
            RoadType::Arterial => 0.7,
            RoadType::Highway => 1.0,
        }
    }

    /// Calculate friction coefficient
    fn calculate_friction(&self, condition: SurfaceCondition) -> f64 {
        match condition {
            SurfaceCondition::DryPavement => 0.9,
            SurfaceCondition::WetPavement => 0.7,
            SurfaceCondition::Gravel => 0.6,
            SurfaceCondition::Dirt => 0.5,
            SurfaceCondition::SnowCovered => 0.3,
            SurfaceCondition::IceCovered => 0.15,
        }
    }

    /// Calculate weather severity
    fn calculate_weather_severity(&self, weather: WeatherCondition) -> f64 {
        match weather {
            WeatherCondition::Clear => 0.0,
            WeatherCondition::Cloudy => 0.1,
            WeatherCondition::Rain => 0.4,
            WeatherCondition::HeavyRain => 0.7,
            WeatherCondition::Fog => 0.5,
            WeatherCondition::Sleet => 0.8,
            WeatherCondition::Snow => 0.9,
        }
    }

    /// Calculate lighting quality
    fn calculate_lighting_quality(&self, lighting: LightingCondition) -> f64 {
        match lighting {
            LightingCondition::Daylight => 1.0,
            LightingCondition::Dawn => 0.7,
            LightingCondition::Dusk => 0.7,
            LightingCondition::DarknessLighted => 0.5,
            LightingCondition::Darkness => 0.2,
        }
    }

    /// Calculate road complexity
    fn calculate_road_complexity(&self, geometry: &RoadGeometry) -> f64 {
        let mut complexity = 0.0;

        // More lanes = more complex
        complexity += (geometry.lanes as f64 / 8.0).clamp(0.0, 0.3);

        // Curvature adds complexity
        complexity += (geometry.curvature / 5.0).clamp(0.0, 0.3);

        // Grade adds complexity
        complexity += (geometry.grade.abs() / 15.0).clamp(0.0, 0.2);

        // Intersection adds complexity
        if geometry.is_intersection {
            complexity += 0.2;
        }

        complexity.clamp(0.0, 1.0)
    }

    /// Calculate safety infrastructure score
    fn calculate_safety_infrastructure(
        &self,
        geometry: &RoadGeometry,
        traffic_control: TrafficControl,
    ) -> f64 {
        let mut score = 0.0;

        // Good lane width contributes
        if geometry.lane_width >= 3.5 {
            score += 0.3;
        }

        // Traffic control contributes
        score += match traffic_control {
            TrafficControl::None => 0.0,
            TrafficControl::YieldSign => 0.2,
            TrafficControl::StopSign => 0.3,
            TrafficControl::TrafficSignal => 0.4,
            TrafficControl::Officer => 0.5,
        };

        // Well-designed intersections contribute
        if geometry.is_intersection {
            match geometry.intersection_type {
                Some(IntersectionType::Roundabout) => score += 0.3,
                Some(IntersectionType::FourWay) => score += 0.2,
                _ => score += 0.1,
            }
        }

        score.clamp(0.0, 1.0)
    }

    /// Calculate environmental risk
    fn calculate_environmental_risk(
        &self,
        weather: f64,
        lighting: f64,
        friction: f64,
    ) -> f64 {
        // Higher weather severity, lower lighting quality, and lower friction = higher risk
        let risk = (weather * 0.4 + (1.0 - lighting) * 0.3 + (1.0 - friction) * 0.3)
            .clamp(0.0, 1.0);

        risk
    }
}

impl SceneFeatures {
    /// Convert to feature vector
    pub fn to_feature_vector(&self) -> Vec<f64> {
        vec![
            self.road_type_encoded,
            self.friction_coefficient,
            self.weather_severity,
            self.lighting_quality,
            self.road_complexity,
            self.traffic_density_norm,
            self.visibility_norm,
            self.speed_environment,
            self.safety_infrastructure,
            self.environmental_risk,
        ]
    }

    /// Get feature names
    pub fn feature_names() -> Vec<String> {
        vec![
            "road_type_encoded".to_string(),
            "friction_coefficient".to_string(),
            "weather_severity".to_string(),
            "lighting_quality".to_string(),
            "road_complexity".to_string(),
            "traffic_density_norm".to_string(),
            "visibility_norm".to_string(),
            "speed_environment".to_string(),
            "safety_infrastructure".to_string(),
            "environmental_risk".to_string(),
        ]
    }
}

impl FeatureExtractor for SceneFeatureExtractor {
    fn extract(&self, input: &dyn std::any::Any) -> Result<FeatureVector> {
        let info = input
            .downcast_ref::<SceneInfo>()
            .ok_or_else(|| MlError::FeatureExtraction("Invalid input type".to_string()))?;

        let features = self.extract_features(info)?;
        Ok(FeatureVector::new(
            features.to_feature_vector(),
            SceneFeatures::feature_names(),
        ))
    }

    fn feature_names(&self) -> Vec<String> {
        SceneFeatures::feature_names()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_scene() -> SceneInfo {
        SceneInfo {
            road_type: RoadType::Arterial,
            surface_condition: SurfaceCondition::WetPavement,
            weather: WeatherCondition::Rain,
            lighting: LightingCondition::Dusk,
            road_geometry: RoadGeometry {
                lanes: 4,
                lane_width: 3.5,
                curvature: 1.0,
                grade: 2.0,
                is_intersection: true,
                intersection_type: Some(IntersectionType::FourWay),
            },
            traffic_density: 0.6,
            visibility: 500.0,
            speed_limit: 60.0,
            traffic_control: TrafficControl::TrafficSignal,
        }
    }

    #[test]
    fn test_scene_feature_extraction() {
        let extractor = SceneFeatureExtractor::new();
        let scene = create_test_scene();
        let features = extractor.extract_features(&scene).unwrap();

        assert!(features.road_type_encoded >= 0.0 && features.road_type_encoded <= 1.0);
        assert!(features.friction_coefficient >= 0.0 && features.friction_coefficient <= 1.0);
        assert!(features.environmental_risk > 0.0); // Rain + dusk = some risk
    }

    #[test]
    fn test_friction_calculation() {
        let extractor = SceneFeatureExtractor::new();

        assert!(extractor.calculate_friction(SurfaceCondition::DryPavement) > 0.8);
        assert!(extractor.calculate_friction(SurfaceCondition::IceCovered) < 0.2);
    }
}
