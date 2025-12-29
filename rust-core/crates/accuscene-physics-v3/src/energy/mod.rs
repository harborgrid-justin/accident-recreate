//! Energy calculations for accident reconstruction.
//!
//! This module computes various energy quantities essential for
//! forensic analysis:
//! - Kinetic energy (translational and rotational)
//! - Deformation energy (crush analysis)
//! - Energy dissipation (friction, air resistance)
//! - Conservation of energy checks

pub mod deformation;
pub mod kinetic;

pub use deformation::*;
pub use kinetic::*;

use nalgebra::Vector3;
use serde::{Deserialize, Serialize};

use crate::deformable::DeformableBody;
use crate::rigid_body::RigidBody;

/// Complete energy analysis for a simulation state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergyAnalysis {
    /// Total kinetic energy (J).
    pub total_kinetic: f64,

    /// Translational kinetic energy (J).
    pub translational_kinetic: f64,

    /// Rotational kinetic energy (J).
    pub rotational_kinetic: f64,

    /// Deformation energy (J).
    pub deformation_energy: f64,

    /// Energy dissipated by friction (J).
    pub friction_dissipation: f64,

    /// Energy dissipated by air resistance (J).
    pub air_resistance_dissipation: f64,

    /// Total initial energy (for conservation check).
    pub initial_total_energy: f64,

    /// Current total energy.
    pub current_total_energy: f64,

    /// Energy conservation error (%).
    pub conservation_error: f64,
}

impl EnergyAnalysis {
    /// Creates a new energy analysis.
    pub fn new() -> Self {
        Self {
            total_kinetic: 0.0,
            translational_kinetic: 0.0,
            rotational_kinetic: 0.0,
            deformation_energy: 0.0,
            friction_dissipation: 0.0,
            air_resistance_dissipation: 0.0,
            initial_total_energy: 0.0,
            current_total_energy: 0.0,
            conservation_error: 0.0,
        }
    }

    /// Computes complete energy analysis for rigid bodies.
    pub fn analyze_rigid_bodies(&mut self, bodies: &[RigidBody]) {
        self.total_kinetic = 0.0;
        self.translational_kinetic = 0.0;
        self.rotational_kinetic = 0.0;

        for body in bodies {
            if body.is_static {
                continue;
            }

            let ke = body.kinetic_energy();
            self.total_kinetic += ke;

            // Split into translational and rotational
            let trans_ke = 0.5 * body.mass_props.mass * body.linear_velocity.norm_squared();
            self.translational_kinetic += trans_ke;
            self.rotational_kinetic += ke - trans_ke;
        }

        self.current_total_energy = self.total_kinetic + self.deformation_energy;
    }

    /// Computes deformation energy from deformable bodies.
    pub fn analyze_deformation(&mut self, deformable_bodies: &[DeformableBody]) {
        self.deformation_energy = 0.0;

        for body in deformable_bodies {
            self.deformation_energy += compute_deformation_energy(body);
        }

        self.current_total_energy = self.total_kinetic + self.deformation_energy;
    }

    /// Sets initial total energy (for conservation tracking).
    pub fn set_initial_energy(&mut self, energy: f64) {
        self.initial_total_energy = energy;
    }

    /// Computes energy conservation error.
    pub fn compute_conservation_error(&mut self) {
        if self.initial_total_energy > 1e-6 {
            let energy_diff = (self.current_total_energy
                + self.friction_dissipation
                + self.air_resistance_dissipation)
                - self.initial_total_energy;

            self.conservation_error = (energy_diff / self.initial_total_energy).abs() * 100.0;
        } else {
            self.conservation_error = 0.0;
        }
    }

    /// Adds friction dissipation energy.
    pub fn add_friction_dissipation(&mut self, energy: f64) {
        self.friction_dissipation += energy;
    }

    /// Adds air resistance dissipation energy.
    pub fn add_air_resistance_dissipation(&mut self, energy: f64) {
        self.air_resistance_dissipation += energy;
    }

    /// Computes equivalent vehicle speed from kinetic energy.
    ///
    /// Useful for accident reconstruction: "This vehicle had kinetic energy
    /// equivalent to traveling at X mph."
    pub fn equivalent_speed(&self, mass: f64) -> f64 {
        if mass > 1e-6 {
            (2.0 * self.total_kinetic / mass).sqrt()
        } else {
            0.0
        }
    }

    /// Generates a human-readable summary.
    pub fn summary(&self) -> String {
        format!(
            "Energy Analysis:\n\
             Total Kinetic: {:.2} kJ\n\
             - Translational: {:.2} kJ\n\
             - Rotational: {:.2} kJ\n\
             Deformation: {:.2} kJ\n\
             Friction Loss: {:.2} kJ\n\
             Air Resistance Loss: {:.2} kJ\n\
             Conservation Error: {:.3}%",
            self.total_kinetic / 1000.0,
            self.translational_kinetic / 1000.0,
            self.rotational_kinetic / 1000.0,
            self.deformation_energy / 1000.0,
            self.friction_dissipation / 1000.0,
            self.air_resistance_dissipation / 1000.0,
            self.conservation_error
        )
    }
}

impl Default for EnergyAnalysis {
    fn default() -> Self {
        Self::new()
    }
}

/// Computes energy dissipated by friction over a time step.
///
/// E_friction = μ * N * d
/// Where d is the sliding distance.
pub fn compute_friction_energy(
    friction_coefficient: f64,
    normal_force: f64,
    sliding_velocity: Vector3<f64>,
    dt: f64,
) -> f64 {
    let sliding_distance = sliding_velocity.norm() * dt;
    friction_coefficient * normal_force * sliding_distance
}

/// Computes energy dissipated by air resistance.
///
/// E_drag = 0.5 * ρ * Cd * A * v³ * dt
/// Where:
/// - ρ = air density (1.225 kg/m³ at sea level)
/// - Cd = drag coefficient
/// - A = frontal area (m²)
/// - v = velocity (m/s)
pub fn compute_air_resistance_energy(
    drag_coefficient: f64,
    frontal_area: f64,
    velocity: Vector3<f64>,
    dt: f64,
) -> f64 {
    const AIR_DENSITY: f64 = 1.225; // kg/m³

    let speed = velocity.norm();
    let drag_force = 0.5 * AIR_DENSITY * drag_coefficient * frontal_area * speed * speed;

    // Energy = Force * distance
    drag_force * speed * dt
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rigid_body::dynamics::MassProperties;
    use approx::assert_relative_eq;

    #[test]
    fn test_energy_analysis_creation() {
        let analysis = EnergyAnalysis::new();
        assert_eq!(analysis.total_kinetic, 0.0);
        assert_eq!(analysis.conservation_error, 0.0);
    }

    #[test]
    fn test_analyze_rigid_bodies() {
        let mass_props = MassProperties::from_sphere(1000.0, 1.0);
        let mut body = RigidBody::new(0, mass_props);
        body.linear_velocity = Vector3::new(10.0, 0.0, 0.0);

        let bodies = vec![body];
        let mut analysis = EnergyAnalysis::new();

        analysis.analyze_rigid_bodies(&bodies);

        let expected_ke = 0.5 * 1000.0 * 100.0; // 0.5 * m * v²
        assert_relative_eq!(analysis.total_kinetic, expected_ke);
    }

    #[test]
    fn test_equivalent_speed() {
        let mut analysis = EnergyAnalysis::new();
        analysis.total_kinetic = 50000.0; // J

        let speed = analysis.equivalent_speed(1000.0); // kg
        let expected = (2.0 * 50000.0 / 1000.0).sqrt(); // 10 m/s

        assert_relative_eq!(speed, expected);
    }

    #[test]
    fn test_friction_energy() {
        let energy = compute_friction_energy(
            0.7,                             // friction coefficient
            5000.0,                          // normal force (N)
            Vector3::new(10.0, 0.0, 0.0),   // sliding velocity
            0.1,                             // dt
        );

        // E = μ * N * d = 0.7 * 5000 * (10 * 0.1) = 3500 J
        assert_relative_eq!(energy, 3500.0);
    }

    #[test]
    fn test_air_resistance_energy() {
        let energy = compute_air_resistance_energy(
            0.3,                             // drag coefficient
            2.0,                             // frontal area (m²)
            Vector3::new(20.0, 0.0, 0.0),   // velocity (m/s)
            1.0,                             // dt
        );

        // Should be positive
        assert!(energy > 0.0);
    }

    #[test]
    fn test_conservation_error() {
        let mut analysis = EnergyAnalysis::new();
        analysis.set_initial_energy(100000.0);
        analysis.current_total_energy = 95000.0;
        analysis.friction_dissipation = 5000.0;

        analysis.compute_conservation_error();

        // Should be near zero (perfect conservation)
        assert!(analysis.conservation_error < 0.1);
    }
}
