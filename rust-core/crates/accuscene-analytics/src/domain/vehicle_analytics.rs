//! Vehicle behavior analysis

use crate::error::Result;
use crate::statistics::descriptive::{DescriptiveStats, Statistics};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleData {
    pub id: String,
    pub vehicle_type: VehicleType,
    pub mass: f64,              // kg
    pub speed: f64,             // km/h
    pub acceleration: f64,      // m/s²
    pub braking_distance: f64,  // m
    pub tire_condition: TireCondition,
    pub brake_performance: f64, // 0-100 scale
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VehicleType {
    Sedan,
    SUV,
    Truck,
    Motorcycle,
    Bus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TireCondition {
    Good,
    Fair,
    Poor,
    Worn,
}

/// Vehicle behavior analytics
pub struct VehicleAnalytics;

impl VehicleAnalytics {
    /// Analyze braking performance across vehicles
    pub fn braking_analysis(vehicles: &[VehicleData]) -> Result<BrakingAnalysis> {
        let distances: Vec<f64> = vehicles.iter().map(|v| v.braking_distance).collect();
        let stats = DescriptiveStats::from_data(&distances)?;

        // Calculate average braking efficiency by type
        let sedan_avg = Self::avg_braking_distance(vehicles, VehicleType::Sedan);
        let suv_avg = Self::avg_braking_distance(vehicles, VehicleType::SUV);
        let truck_avg = Self::avg_braking_distance(vehicles, VehicleType::Truck);

        Ok(BrakingAnalysis {
            stats,
            sedan_avg_distance: sedan_avg,
            suv_avg_distance: suv_avg,
            truck_avg_distance: truck_avg,
        })
    }

    /// Analyze speed distribution
    pub fn speed_analysis(vehicles: &[VehicleData]) -> Result<SpeedAnalysis> {
        let speeds: Vec<f64> = vehicles.iter().map(|v| v.speed).collect();
        let stats = DescriptiveStats::from_data(&speeds)?;

        let speeding_count = vehicles.iter().filter(|v| v.speed > 100.0).count();
        let safe_speed_count = vehicles.iter().filter(|v| v.speed <= 60.0).count();

        Ok(SpeedAnalysis {
            stats,
            speeding_count,
            safe_speed_count,
            avg_speed_by_type: Self::avg_speed_by_type(vehicles),
        })
    }

    /// Analyze tire condition impact
    pub fn tire_condition_impact(vehicles: &[VehicleData]) -> TireImpactAnalysis {
        let good_avg = Self::avg_braking_for_tire(vehicles, TireCondition::Good);
        let fair_avg = Self::avg_braking_for_tire(vehicles, TireCondition::Fair);
        let poor_avg = Self::avg_braking_for_tire(vehicles, TireCondition::Poor);
        let worn_avg = Self::avg_braking_for_tire(vehicles, TireCondition::Worn);

        TireImpactAnalysis {
            good_tire_avg_braking: good_avg,
            fair_tire_avg_braking: fair_avg,
            poor_tire_avg_braking: poor_avg,
            worn_tire_avg_braking: worn_avg,
            impact_factor: if good_avg > 0.0 {
                worn_avg / good_avg
            } else {
                1.0
            },
        }
    }

    /// Calculate stopping distance based on physics
    pub fn calculate_stopping_distance(
        speed_kmh: f64,
        brake_performance: f64,
        road_friction: f64,
    ) -> f64 {
        let speed_ms = speed_kmh / 3.6; // Convert to m/s
        let reaction_distance = speed_ms * 1.5; // 1.5s reaction time

        // Braking distance: v² / (2 * μ * g)
        let g = 9.81; // m/s²
        let effective_friction = road_friction * (brake_performance / 100.0);
        let braking_distance = (speed_ms * speed_ms) / (2.0 * effective_friction * g);

        reaction_distance + braking_distance
    }

    /// Estimate impact severity based on vehicle parameters
    pub fn estimate_impact_severity(vehicle1: &VehicleData, vehicle2: &VehicleData) -> ImpactSeverity {
        let momentum1 = vehicle1.mass * vehicle1.speed;
        let momentum2 = vehicle2.mass * vehicle2.speed;
        let total_energy = momentum1 + momentum2;

        let severity_score = total_energy / 10000.0; // Normalize

        if severity_score < 10.0 {
            ImpactSeverity::Low
        } else if severity_score < 30.0 {
            ImpactSeverity::Medium
        } else if severity_score < 60.0 {
            ImpactSeverity::High
        } else {
            ImpactSeverity::Critical
        }
    }

    fn avg_braking_distance(vehicles: &[VehicleData], vtype: VehicleType) -> f64 {
        let distances: Vec<f64> = vehicles
            .iter()
            .filter(|v| v.vehicle_type == vtype)
            .map(|v| v.braking_distance)
            .collect();

        if distances.is_empty() {
            0.0
        } else {
            Statistics::mean(&distances)
        }
    }

    fn avg_speed_by_type(vehicles: &[VehicleData]) -> Vec<(VehicleType, f64)> {
        use VehicleType::*;

        vec![Sedan, SUV, Truck, Motorcycle, Bus]
            .into_iter()
            .map(|vtype| {
                let speeds: Vec<f64> = vehicles
                    .iter()
                    .filter(|v| v.vehicle_type == vtype)
                    .map(|v| v.speed)
                    .collect();

                let avg = if speeds.is_empty() {
                    0.0
                } else {
                    Statistics::mean(&speeds)
                };

                (vtype, avg)
            })
            .collect()
    }

    fn avg_braking_for_tire(vehicles: &[VehicleData], condition: TireCondition) -> f64 {
        let distances: Vec<f64> = vehicles
            .iter()
            .filter(|v| v.tire_condition == condition)
            .map(|v| v.braking_distance)
            .collect();

        if distances.is_empty() {
            0.0
        } else {
            Statistics::mean(&distances)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrakingAnalysis {
    pub stats: DescriptiveStats,
    pub sedan_avg_distance: f64,
    pub suv_avg_distance: f64,
    pub truck_avg_distance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedAnalysis {
    pub stats: DescriptiveStats,
    pub speeding_count: usize,
    pub safe_speed_count: usize,
    pub avg_speed_by_type: Vec<(VehicleType, f64)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TireImpactAnalysis {
    pub good_tire_avg_braking: f64,
    pub fair_tire_avg_braking: f64,
    pub poor_tire_avg_braking: f64,
    pub worn_tire_avg_braking: f64,
    pub impact_factor: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImpactSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stopping_distance() {
        let distance = VehicleAnalytics::calculate_stopping_distance(60.0, 100.0, 0.7);
        assert!(distance > 0.0);
        assert!(distance < 100.0); // Reasonable range
    }

    #[test]
    fn test_impact_severity() {
        let v1 = VehicleData {
            id: "V1".to_string(),
            vehicle_type: VehicleType::Sedan,
            mass: 1500.0,
            speed: 60.0,
            acceleration: 0.0,
            braking_distance: 30.0,
            tire_condition: TireCondition::Good,
            brake_performance: 90.0,
        };

        let v2 = VehicleData {
            id: "V2".to_string(),
            vehicle_type: VehicleType::SUV,
            mass: 2000.0,
            speed: 50.0,
            acceleration: 0.0,
            braking_distance: 35.0,
            tire_condition: TireCondition::Good,
            brake_performance: 85.0,
        };

        let severity = VehicleAnalytics::estimate_impact_severity(&v1, &v2);
        assert!(matches!(severity, ImpactSeverity::Low | ImpactSeverity::Medium));
    }
}
