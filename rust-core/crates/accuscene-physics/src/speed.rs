//! Speed estimation from physical evidence.

use crate::energy::EnergyCalculator;
use crate::friction::SurfaceType;
use serde::{Deserialize, Serialize};

/// Speed estimation result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedEstimate {
    /// Estimated speed (m/s)
    pub speed_mps: f64,
    /// Estimated speed (km/h)
    pub speed_kmh: f64,
    /// Estimated speed (mph)
    pub speed_mph: f64,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f64,
    /// Minimum speed (m/s)
    pub min_speed_mps: f64,
    /// Maximum speed (m/s)
    pub max_speed_mps: f64,
    /// Method used for estimation
    pub method: String,
}

impl SpeedEstimate {
    /// Creates a new speed estimate.
    pub fn new(speed_mps: f64, confidence: f64, method: String) -> Self {
        Self {
            speed_mps,
            speed_kmh: speed_mps * 3.6,
            speed_mph: speed_mps * 2.23694,
            confidence,
            min_speed_mps: speed_mps * 0.9,
            max_speed_mps: speed_mps * 1.1,
            method,
        }
    }

    /// Sets the uncertainty range.
    pub fn with_range(mut self, min_mps: f64, max_mps: f64) -> Self {
        self.min_speed_mps = min_mps;
        self.max_speed_mps = max_mps;
        self
    }
}

/// Speed estimator using various physical evidence.
pub struct SpeedEstimator;

impl SpeedEstimator {
    /// Estimates speed from skid mark length.
    ///
    /// Uses energy equation: v = sqrt(2 * μ * g * d)
    pub fn from_skid_marks(
        skid_length: f64,
        surface: SurfaceType,
        grade_percent: f64,
    ) -> SpeedEstimate {
        const GRAVITY: f64 = 9.81;

        let friction = surface.kinetic_friction();

        // Adjust for road grade (slope)
        let grade_factor = if grade_percent.abs() > 0.1 {
            let grade_radians = (grade_percent / 100.0).atan();
            grade_radians.cos() + grade_radians.sin() * friction
        } else {
            1.0
        };

        let speed = (2.0 * friction * GRAVITY * skid_length * grade_factor).sqrt();

        // Confidence depends on surface certainty
        let confidence = match surface {
            SurfaceType::AsphaltDry | SurfaceType::ConcreteDry => 0.85,
            SurfaceType::AsphaltWet | SurfaceType::ConcreteWet => 0.75,
            _ => 0.65,
        };

        SpeedEstimate::new(
            speed,
            confidence,
            format!("Skid marks on {}", surface.name()),
        )
        .with_range(speed * 0.85, speed * 1.15)
    }

    /// Estimates speed from yaw marks (curved skid marks).
    ///
    /// Uses centripetal force equation: v = sqrt(μ * g * r)
    pub fn from_yaw_marks(
        radius: f64,
        surface: SurfaceType,
        superelevation: f64,
    ) -> SpeedEstimate {
        const GRAVITY: f64 = 9.81;

        let friction = surface.kinetic_friction();

        // Adjust for road banking (superelevation)
        let effective_friction = friction + superelevation;

        let speed = (effective_friction * GRAVITY * radius).sqrt();

        let confidence = 0.7; // Yaw marks are less certain than straight skids

        SpeedEstimate::new(
            speed,
            confidence,
            format!("Yaw marks on {}", surface.name()),
        )
        .with_range(speed * 0.8, speed * 1.2)
    }

    /// Estimates speed from crush depth (damage-based analysis).
    ///
    /// Uses energy equation: v = sqrt(2 * E / m)
    pub fn from_crush_depth(
        crush_depth: f64,
        vehicle_mass: f64,
        vehicle_stiffness: f64,
        contact_area: f64,
    ) -> SpeedEstimate {
        let crush_energy = EnergyCalculator::crush_energy(crush_depth, vehicle_stiffness, contact_area);

        let speed = EnergyCalculator::energy_equivalent_speed(crush_energy, vehicle_mass);

        // Crush-based estimates are less certain
        let confidence = 0.6;

        SpeedEstimate::new(speed, confidence, "Crush depth analysis".to_string())
            .with_range(speed * 0.75, speed * 1.25)
    }

    /// Estimates speed from vault distance (distance body travels after ejection).
    ///
    /// Assumes projectile motion: R = v₀²sin(2θ)/g
    pub fn from_vault_distance(
        vault_distance: f64,
        launch_angle_degrees: f64,
        initial_height: f64,
    ) -> SpeedEstimate {
        const GRAVITY: f64 = 9.81;

        let angle_rad = launch_angle_degrees.to_radians();

        // For projectile motion with initial height:
        // More complex calculation, simplified here
        let horizontal_velocity = if initial_height.abs() < 0.1 {
            // Simple case: flat launch
            (vault_distance * GRAVITY / (2.0 * angle_rad).sin()).sqrt()
        } else {
            // With height difference
            let time_of_flight = ((2.0 * initial_height / GRAVITY).sqrt()
                + (2.0 * (initial_height + vault_distance * angle_rad.tan()) / GRAVITY).sqrt())
                / 2.0;
            vault_distance / (time_of_flight * angle_rad.cos())
        };

        let speed = horizontal_velocity / angle_rad.cos();

        // Very uncertain due to many assumptions
        let confidence = 0.5;

        SpeedEstimate::new(speed, confidence, "Vault trajectory analysis".to_string())
            .with_range(speed * 0.6, speed * 1.4)
    }

    /// Estimates speed from rollover distance and number of quarter-turns.
    pub fn from_rollover(
        rollover_distance: f64,
        num_quarter_turns: u32,
        _vehicle_height: f64,
    ) -> SpeedEstimate {
        // Empirical formula based on rollover dynamics
        // Each quarter turn dissipates energy
        let energy_per_turn = 0.3; // Rough estimate

        let effective_distance = rollover_distance / (1.0 + num_quarter_turns as f64 * energy_per_turn);

        // Use friction-based estimate with reduced coefficient for rolling
        let rolling_friction = 0.4; // Lower than sliding friction
        const GRAVITY: f64 = 9.81;

        let speed = (2.0 * rolling_friction * GRAVITY * effective_distance).sqrt();

        let confidence = 0.55;

        SpeedEstimate::new(
            speed,
            confidence,
            format!("Rollover analysis ({} quarter-turns)", num_quarter_turns),
        )
        .with_range(speed * 0.7, speed * 1.3)
    }

    /// Estimates speed from fall distance (vehicle went airborne).
    ///
    /// Uses free fall equations.
    pub fn from_fall_distance(
        horizontal_distance: f64,
        vertical_drop: f64,
    ) -> SpeedEstimate {
        const GRAVITY: f64 = 9.81;

        // Time of flight from vertical drop: t = sqrt(2h/g)
        let time_of_flight = (2.0 * vertical_drop / GRAVITY).sqrt();

        // Horizontal speed: v = d/t
        let speed = horizontal_distance / time_of_flight;

        let confidence = 0.75;

        SpeedEstimate::new(speed, confidence, "Free fall trajectory".to_string())
            .with_range(speed * 0.85, speed * 1.15)
    }

    /// Estimates speed from post-impact displacement (using momentum).
    pub fn from_post_impact_displacement(
        displacement: f64,
        _combined_mass: f64,
        friction_coefficient: f64,
    ) -> SpeedEstimate {
        const GRAVITY: f64 = 9.81;

        // Speed at start of slide
        let slide_speed = (2.0 * friction_coefficient * GRAVITY * displacement).sqrt();

        let confidence = 0.7;

        SpeedEstimate::new(
            slide_speed,
            confidence,
            "Post-impact displacement".to_string(),
        )
        .with_range(slide_speed * 0.8, slide_speed * 1.2)
    }

    /// Combines multiple speed estimates using weighted average.
    pub fn combine_estimates(estimates: &[SpeedEstimate]) -> Option<SpeedEstimate> {
        if estimates.is_empty() {
            return None;
        }

        let total_confidence: f64 = estimates.iter().map(|e| e.confidence).sum();
        if total_confidence < 0.001 {
            return None;
        }

        let weighted_speed: f64 = estimates
            .iter()
            .map(|e| e.speed_mps * e.confidence)
            .sum::<f64>()
            / total_confidence;

        let avg_confidence = total_confidence / estimates.len() as f64;

        let min_speed = estimates
            .iter()
            .map(|e| e.min_speed_mps)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(weighted_speed * 0.8);

        let max_speed = estimates
            .iter()
            .map(|e| e.max_speed_mps)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(weighted_speed * 1.2);

        Some(
            SpeedEstimate::new(
                weighted_speed,
                avg_confidence,
                "Combined estimate".to_string(),
            )
            .with_range(min_speed, max_speed),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_speed_from_skid_marks() {
        let skid_length = 50.0; // 50 meters
        let surface = SurfaceType::AsphaltDry;

        let estimate = SpeedEstimator::from_skid_marks(skid_length, surface, 0.0);

        // v = sqrt(2 * 0.9 * 9.81 * 50) ≈ 29.7 m/s ≈ 107 km/h
        assert!((estimate.speed_mps - 29.7).abs() < 1.0);
        assert!((estimate.speed_kmh - 107.0).abs() < 5.0);
    }

    #[test]
    fn test_speed_from_yaw_marks() {
        let radius = 30.0; // 30 meter radius
        let surface = SurfaceType::AsphaltDry;

        let estimate = SpeedEstimator::from_yaw_marks(radius, surface, 0.0);

        // v = sqrt(0.9 * 9.81 * 30) ≈ 16.3 m/s ≈ 58.7 km/h
        assert!((estimate.speed_mps - 16.3).abs() < 1.0);
    }

    #[test]
    fn test_speed_from_fall() {
        let horizontal = 20.0; // 20 meters
        let vertical = 5.0; // 5 meters drop

        let estimate = SpeedEstimator::from_fall_distance(horizontal, vertical);

        // t = sqrt(2*5/9.81) ≈ 1.01 s
        // v = 20/1.01 ≈ 19.8 m/s
        assert!((estimate.speed_mps - 19.8).abs() < 1.0);
    }

    #[test]
    fn test_combine_estimates() {
        let estimate1 = SpeedEstimate::new(20.0, 0.8, "Method 1".to_string());
        let estimate2 = SpeedEstimate::new(22.0, 0.6, "Method 2".to_string());

        let combined = SpeedEstimator::combine_estimates(&[estimate1, estimate2]).unwrap();

        // Weighted average: (20*0.8 + 22*0.6)/(0.8+0.6) = 20.86
        assert!((combined.speed_mps - 20.86).abs() < 0.1);
    }

    #[test]
    fn test_unit_conversions() {
        let estimate = SpeedEstimate::new(20.0, 0.9, "Test".to_string());

        // 20 m/s = 72 km/h
        assert!((estimate.speed_kmh - 72.0).abs() < 0.1);

        // 20 m/s ≈ 44.74 mph
        assert!((estimate.speed_mph - 44.74).abs() < 0.1);
    }
}
