//! Collision-specific analytics and pattern detection

use crate::error::Result;
use crate::statistics::descriptive::{DescriptiveStats, Statistics};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollisionData {
    pub id: String,
    pub severity: CollisionSeverity,
    pub impact_speed: f64,      // km/h
    pub impact_angle: f64,       // degrees
    pub impact_force: f64,       // kN
    pub vehicle_mass_1: f64,     // kg
    pub vehicle_mass_2: f64,     // kg
    pub deformation_depth: f64,  // cm
    pub road_condition: RoadCondition,
    pub weather: WeatherCondition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CollisionSeverity {
    Minor,
    Moderate,
    Severe,
    Fatal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoadCondition {
    Dry,
    Wet,
    Icy,
    Snowy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeatherCondition {
    Clear,
    Rainy,
    Foggy,
    Snowy,
}

/// Analytics for collision data
pub struct CollisionAnalytics;

impl CollisionAnalytics {
    /// Analyze collision severity distribution
    pub fn severity_distribution(collisions: &[CollisionData]) -> SeverityDistribution {
        let mut counts = [0; 4];

        for collision in collisions {
            let idx = match collision.severity {
                CollisionSeverity::Minor => 0,
                CollisionSeverity::Moderate => 1,
                CollisionSeverity::Severe => 2,
                CollisionSeverity::Fatal => 3,
            };
            counts[idx] += 1;
        }

        let total = collisions.len();

        SeverityDistribution {
            minor_count: counts[0],
            moderate_count: counts[1],
            severe_count: counts[2],
            fatal_count: counts[3],
            total,
            minor_pct: counts[0] as f64 / total as f64 * 100.0,
            moderate_pct: counts[1] as f64 / total as f64 * 100.0,
            severe_pct: counts[2] as f64 / total as f64 * 100.0,
            fatal_pct: counts[3] as f64 / total as f64 * 100.0,
        }
    }

    /// Analyze impact speed statistics
    pub fn impact_speed_analysis(collisions: &[CollisionData]) -> Result<SpeedAnalysis> {
        let speeds: Vec<f64> = collisions.iter().map(|c| c.impact_speed).collect();

        let stats = DescriptiveStats::from_data(&speeds)?;

        // Categorize by speed ranges
        let low_speed = speeds.iter().filter(|&&s| s < 30.0).count();
        let medium_speed = speeds.iter().filter(|&&s| s >= 30.0 && s < 60.0).count();
        let high_speed = speeds.iter().filter(|&&s| s >= 60.0).count();

        Ok(SpeedAnalysis {
            stats,
            low_speed_count: low_speed,
            medium_speed_count: medium_speed,
            high_speed_count: high_speed,
        })
    }

    /// Analyze impact force distribution
    pub fn impact_force_analysis(collisions: &[CollisionData]) -> Result<ForceAnalysis> {
        let forces: Vec<f64> = collisions.iter().map(|c| c.impact_force).collect();

        let stats = DescriptiveStats::from_data(&forces)?;

        // Calculate energy absorption
        let avg_deformation = Statistics::mean(
            &collisions
                .iter()
                .map(|c| c.deformation_depth)
                .collect::<Vec<_>>(),
        );

        Ok(ForceAnalysis {
            stats,
            avg_deformation,
            max_force: stats.max,
            min_force: stats.min,
        })
    }

    /// Detect collision patterns
    pub fn detect_patterns(collisions: &[CollisionData]) -> CollisionPatterns {
        // Analyze road condition correlation
        let dry_severe = collisions
            .iter()
            .filter(|c| matches!(c.road_condition, RoadCondition::Dry))
            .filter(|c| matches!(c.severity, CollisionSeverity::Severe | CollisionSeverity::Fatal))
            .count();

        let wet_severe = collisions
            .iter()
            .filter(|c| matches!(c.road_condition, RoadCondition::Wet))
            .filter(|c| matches!(c.severity, CollisionSeverity::Severe | CollisionSeverity::Fatal))
            .count();

        let icy_severe = collisions
            .iter()
            .filter(|c| matches!(c.road_condition, RoadCondition::Icy))
            .filter(|c| matches!(c.severity, CollisionSeverity::Severe | CollisionSeverity::Fatal))
            .count();

        // High-speed collisions
        let high_speed_severe = collisions
            .iter()
            .filter(|c| c.impact_speed > 80.0)
            .filter(|c| matches!(c.severity, CollisionSeverity::Severe | CollisionSeverity::Fatal))
            .count();

        // Angle analysis
        let head_on = collisions
            .iter()
            .filter(|c| (c.impact_angle - 180.0).abs() < 30.0)
            .count();

        let t_bone = collisions
            .iter()
            .filter(|c| (c.impact_angle - 90.0).abs() < 20.0 || (c.impact_angle - 270.0).abs() < 20.0)
            .count();

        CollisionPatterns {
            dry_severe_count: dry_severe,
            wet_severe_count: wet_severe,
            icy_severe_count: icy_severe,
            high_speed_severe_count: high_speed_severe,
            head_on_count: head_on,
            t_bone_count: t_bone,
        }
    }

    /// Calculate risk score for a collision configuration
    pub fn calculate_risk_score(data: &CollisionData) -> f64 {
        let mut score = 0.0;

        // Speed factor (0-40 points)
        score += (data.impact_speed / 3.0).min(40.0);

        // Mass difference factor (0-20 points)
        let mass_ratio = (data.vehicle_mass_1 / data.vehicle_mass_2).max(data.vehicle_mass_2 / data.vehicle_mass_1);
        score += ((mass_ratio - 1.0) * 10.0).min(20.0);

        // Angle factor (0-20 points) - head-on is highest risk
        let angle_risk = ((data.impact_angle - 180.0).abs() / 180.0) * 20.0;
        score += 20.0 - angle_risk;

        // Road condition factor (0-10 points)
        score += match data.road_condition {
            RoadCondition::Dry => 0.0,
            RoadCondition::Wet => 5.0,
            RoadCondition::Icy => 10.0,
            RoadCondition::Snowy => 8.0,
        };

        // Weather factor (0-10 points)
        score += match data.weather {
            WeatherCondition::Clear => 0.0,
            WeatherCondition::Rainy => 4.0,
            WeatherCondition::Foggy => 6.0,
            WeatherCondition::Snowy => 5.0,
        };

        score.min(100.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeverityDistribution {
    pub minor_count: usize,
    pub moderate_count: usize,
    pub severe_count: usize,
    pub fatal_count: usize,
    pub total: usize,
    pub minor_pct: f64,
    pub moderate_pct: f64,
    pub severe_pct: f64,
    pub fatal_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedAnalysis {
    pub stats: DescriptiveStats,
    pub low_speed_count: usize,    // < 30 km/h
    pub medium_speed_count: usize, // 30-60 km/h
    pub high_speed_count: usize,   // > 60 km/h
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForceAnalysis {
    pub stats: DescriptiveStats,
    pub avg_deformation: f64,
    pub max_force: f64,
    pub min_force: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollisionPatterns {
    pub dry_severe_count: usize,
    pub wet_severe_count: usize,
    pub icy_severe_count: usize,
    pub high_speed_severe_count: usize,
    pub head_on_count: usize,
    pub t_bone_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_collision() -> CollisionData {
        CollisionData {
            id: "C001".to_string(),
            severity: CollisionSeverity::Moderate,
            impact_speed: 50.0,
            impact_angle: 90.0,
            impact_force: 150.0,
            vehicle_mass_1: 1500.0,
            vehicle_mass_2: 1800.0,
            deformation_depth: 25.0,
            road_condition: RoadCondition::Dry,
            weather: WeatherCondition::Clear,
        }
    }

    #[test]
    fn test_severity_distribution() {
        let collisions = vec![sample_collision()];
        let dist = CollisionAnalytics::severity_distribution(&collisions);

        assert_eq!(dist.total, 1);
        assert_eq!(dist.moderate_count, 1);
    }

    #[test]
    fn test_risk_score() {
        let collision = sample_collision();
        let risk = CollisionAnalytics::calculate_risk_score(&collision);

        assert!(risk >= 0.0 && risk <= 100.0);
    }
}
