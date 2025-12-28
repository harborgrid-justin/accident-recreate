//! Accident reconstruction algorithms and analysis.

use crate::energy::EnergyAnalysis;
use crate::kinematics::{MomentumAnalysis, Trajectory};
use crate::speed::SpeedEstimate;
use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};

/// Complete accident reconstruction analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccidentReconstruction {
    /// Estimated pre-collision speeds
    pub pre_collision_speeds: Vec<SpeedEstimate>,
    /// Energy analysis
    pub energy_analysis: Option<EnergyAnalysis>,
    /// Momentum analysis
    pub momentum_analysis: Option<MomentumAnalysis>,
    /// Reconstructed trajectories
    pub trajectories: Vec<Trajectory>,
    /// Point of impact location
    pub impact_location: Option<Point3<f64>>,
    /// Impact angle (degrees)
    pub impact_angle: Option<f64>,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,
    /// Reconstruction notes and findings
    pub notes: Vec<String>,
}

impl AccidentReconstruction {
    /// Creates a new empty reconstruction.
    pub fn new() -> Self {
        Self {
            pre_collision_speeds: Vec::new(),
            energy_analysis: None,
            momentum_analysis: None,
            trajectories: Vec::new(),
            impact_location: None,
            impact_angle: None,
            confidence: 0.0,
            notes: Vec::new(),
        }
    }

    /// Adds a speed estimate to the reconstruction.
    pub fn add_speed_estimate(&mut self, estimate: SpeedEstimate) {
        self.pre_collision_speeds.push(estimate);
        self.update_confidence();
    }

    /// Sets the energy analysis.
    pub fn set_energy_analysis(&mut self, analysis: EnergyAnalysis) {
        self.energy_analysis = Some(analysis);
        self.update_confidence();
    }

    /// Sets the momentum analysis.
    pub fn set_momentum_analysis(&mut self, analysis: MomentumAnalysis) {
        self.momentum_analysis = Some(analysis);
        self.update_confidence();
    }

    /// Adds a trajectory to the reconstruction.
    pub fn add_trajectory(&mut self, trajectory: Trajectory) {
        self.trajectories.push(trajectory);
    }

    /// Sets the impact location.
    pub fn set_impact_location(&mut self, location: Point3<f64>) {
        self.impact_location = Some(location);
    }

    /// Sets the impact angle.
    pub fn set_impact_angle(&mut self, angle: f64) {
        self.impact_angle = Some(angle);
    }

    /// Adds a note to the reconstruction.
    pub fn add_note(&mut self, note: String) {
        self.notes.push(note);
    }

    /// Updates the overall confidence score based on available data.
    fn update_confidence(&mut self) {
        let mut confidence_sum = 0.0;
        let mut confidence_count = 0;

        // Speed estimates contribute to confidence
        if !self.pre_collision_speeds.is_empty() {
            let avg_speed_confidence: f64 = self
                .pre_collision_speeds
                .iter()
                .map(|s| s.confidence)
                .sum::<f64>()
                / self.pre_collision_speeds.len() as f64;
            confidence_sum += avg_speed_confidence;
            confidence_count += 1;
        }

        // Energy analysis contributes if available
        if self.energy_analysis.is_some() {
            confidence_sum += 0.8;
            confidence_count += 1;
        }

        // Momentum analysis contributes if available
        if self.momentum_analysis.is_some() {
            confidence_sum += 0.85;
            confidence_count += 1;
        }

        // Trajectories contribute if available
        if !self.trajectories.is_empty() {
            confidence_sum += 0.7;
            confidence_count += 1;
        }

        self.confidence = if confidence_count > 0 {
            confidence_sum / confidence_count as f64
        } else {
            0.0
        };
    }
}

impl Default for AccidentReconstruction {
    fn default() -> Self {
        Self::new()
    }
}

/// Accident reconstruction calculator.
pub struct ReconstructionCalculator;

impl ReconstructionCalculator {
    /// Calculates the impact angle between two vehicles.
    ///
    /// Returns angle in degrees (0-180).
    pub fn calculate_impact_angle(
        velocity_a: Vector3<f64>,
        velocity_b: Vector3<f64>,
    ) -> f64 {
        if velocity_a.norm() < 0.01 || velocity_b.norm() < 0.01 {
            return 0.0;
        }

        let dot_product = velocity_a.normalize().dot(&velocity_b.normalize());
        let angle_rad = dot_product.clamp(-1.0, 1.0).acos();

        angle_rad.to_degrees()
    }

    /// Estimates the point of impact from debris field.
    pub fn estimate_impact_point(debris_locations: &[Point3<f64>]) -> Option<Point3<f64>> {
        if debris_locations.is_empty() {
            return None;
        }

        // Use centroid of debris field
        let sum: Vector3<f64> = debris_locations
            .iter()
            .map(|p| p.coords)
            .sum();

        Some(Point3::from(sum / debris_locations.len() as f64))
    }

    /// Calculates Principal Direction of Force (PDOF) angle.
    ///
    /// PDOF is the direction of the force vector at impact.
    pub fn calculate_pdof(
        velocity_before: Vector3<f64>,
        velocity_after: Vector3<f64>,
    ) -> f64 {
        let delta_v = velocity_after - velocity_before;

        if delta_v.norm() < 0.01 {
            return 0.0;
        }

        // PDOF angle relative to vehicle's forward direction (assuming +X is forward)
        let angle = delta_v.y.atan2(delta_v.x);
        angle.to_degrees()
    }

    /// Estimates delta-V (change in velocity) from damage.
    ///
    /// Returns delta-V in m/s.
    pub fn estimate_delta_v_from_damage(
        crush_depth: f64,
        stiffness: f64,
        mass: f64,
        contact_area: f64,
    ) -> f64 {
        // Using simplified crash pulse model
        let crush_energy = 0.5 * stiffness * contact_area * crush_depth * crush_depth;
        (2.0 * crush_energy / mass).sqrt()
    }

    /// Analyzes vehicle rest positions to determine collision dynamics.
    pub fn analyze_rest_positions(
        initial_positions: &[Point3<f64>],
        final_positions: &[Point3<f64>],
        masses: &[f64],
    ) -> AnalysisResult {
        if initial_positions.len() != final_positions.len()
            || initial_positions.len() != masses.len()
        {
            return AnalysisResult::invalid();
        }

        let mut total_displacement = 0.0;
        let mut weighted_direction = Vector3::zeros();

        for i in 0..initial_positions.len() {
            let displacement = final_positions[i] - initial_positions[i];
            let distance = displacement.norm();
            total_displacement += distance;

            if distance > 0.01 {
                weighted_direction += displacement.normalize() * masses[i];
            }
        }

        AnalysisResult {
            valid: true,
            total_displacement,
            primary_direction: if weighted_direction.norm() > 0.01 {
                Some(weighted_direction.normalize())
            } else {
                None
            },
            confidence: 0.7,
        }
    }

    /// Validates reconstruction using conservation laws.
    pub fn validate_reconstruction(
        momentum_before: Vector3<f64>,
        momentum_after: Vector3<f64>,
        energy_before: f64,
        energy_after: f64,
    ) -> ValidationResult {
        // Check momentum conservation (should be nearly equal)
        let momentum_error = (momentum_after - momentum_before).norm() / momentum_before.norm();

        // Check energy dissipation (energy_after <= energy_before)
        let energy_ratio = energy_after / energy_before;

        ValidationResult {
            momentum_conserved: momentum_error < 0.1, // 10% tolerance
            energy_valid: energy_ratio <= 1.05,       // Allow 5% error
            momentum_error_percent: momentum_error * 100.0,
            energy_dissipation_percent: ((energy_before - energy_after) / energy_before) * 100.0,
            overall_valid: momentum_error < 0.1 && energy_ratio <= 1.05,
        }
    }

    /// Estimates time of collision from multiple witness observations.
    pub fn estimate_collision_time(observations: &[TimeObservation]) -> Option<f64> {
        if observations.is_empty() {
            return None;
        }

        // Weighted average based on confidence
        let total_confidence: f64 = observations.iter().map(|o| o.confidence).sum();
        if total_confidence < 0.01 {
            return None;
        }

        let weighted_time: f64 = observations
            .iter()
            .map(|o| o.time * o.confidence)
            .sum();

        Some(weighted_time / total_confidence)
    }
}

/// Result of position analysis.
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    pub valid: bool,
    pub total_displacement: f64,
    pub primary_direction: Option<Vector3<f64>>,
    pub confidence: f64,
}

impl AnalysisResult {
    fn invalid() -> Self {
        Self {
            valid: false,
            total_displacement: 0.0,
            primary_direction: None,
            confidence: 0.0,
        }
    }
}

/// Validation result for reconstruction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub momentum_conserved: bool,
    pub energy_valid: bool,
    pub momentum_error_percent: f64,
    pub energy_dissipation_percent: f64,
    pub overall_valid: bool,
}

/// Time observation from witness or evidence.
#[derive(Debug, Clone)]
pub struct TimeObservation {
    pub time: f64,
    pub confidence: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_impact_angle_calculation() {
        // Head-on collision (180 degrees)
        let v1 = Vector3::new(10.0, 0.0, 0.0);
        let v2 = Vector3::new(-10.0, 0.0, 0.0);
        let angle = ReconstructionCalculator::calculate_impact_angle(v1, v2);
        assert!((angle - 180.0).abs() < 1.0);

        // Right angle collision (90 degrees)
        let v1 = Vector3::new(10.0, 0.0, 0.0);
        let v2 = Vector3::new(0.0, 10.0, 0.0);
        let angle = ReconstructionCalculator::calculate_impact_angle(v1, v2);
        assert!((angle - 90.0).abs() < 1.0);
    }

    #[test]
    fn test_impact_point_estimation() {
        let debris = vec![
            Point3::new(10.0, 10.0, 0.0),
            Point3::new(12.0, 10.0, 0.0),
            Point3::new(11.0, 11.0, 0.0),
            Point3::new(11.0, 9.0, 0.0),
        ];

        let impact_point = ReconstructionCalculator::estimate_impact_point(&debris).unwrap();

        // Centroid should be around (11, 10, 0)
        assert!((impact_point.x - 11.0).abs() < 0.1);
        assert!((impact_point.y - 10.0).abs() < 0.1);
    }

    #[test]
    fn test_pdof_calculation() {
        let v_before = Vector3::new(20.0, 0.0, 0.0);
        let v_after = Vector3::new(10.0, 5.0, 0.0);

        let pdof = ReconstructionCalculator::calculate_pdof(v_before, v_after);

        // Delta-V is (-10, 5, 0), angle should be around 153 degrees
        assert!(pdof > 140.0 && pdof < 160.0);
    }

    #[test]
    fn test_validation() {
        let momentum_before = Vector3::new(30000.0, 0.0, 0.0);
        let momentum_after = Vector3::new(29500.0, 0.0, 0.0);
        let energy_before = 300000.0;
        let energy_after = 250000.0;

        let validation = ReconstructionCalculator::validate_reconstruction(
            momentum_before,
            momentum_after,
            energy_before,
            energy_after,
        );

        assert!(validation.momentum_conserved);
        assert!(validation.energy_valid);
        assert!(validation.overall_valid);
    }

    #[test]
    fn test_reconstruction_confidence() {
        let mut reconstruction = AccidentReconstruction::new();

        // Initially zero confidence
        assert_eq!(reconstruction.confidence, 0.0);

        // Add some data
        reconstruction.add_speed_estimate(SpeedEstimate::new(
            20.0,
            0.8,
            "Test".to_string(),
        ));

        // Confidence should increase
        assert!(reconstruction.confidence > 0.0);
    }
}
