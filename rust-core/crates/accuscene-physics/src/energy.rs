//! Energy transfer and dissipation calculations.

use nalgebra::Vector3;
use serde::{Deserialize, Serialize};

/// Energy analysis for a collision or event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergyAnalysis {
    /// Initial kinetic energy (J)
    pub initial_kinetic: f64,
    /// Final kinetic energy (J)
    pub final_kinetic: f64,
    /// Energy dissipated as heat/deformation (J)
    pub dissipated: f64,
    /// Energy dissipated percentage
    pub dissipated_percent: f64,
    /// Energy transferred to object A (J)
    pub transferred_to_a: f64,
    /// Energy transferred to object B (J)
    pub transferred_to_b: f64,
}

/// Energy calculator for accident reconstruction.
pub struct EnergyCalculator;

impl EnergyCalculator {
    /// Calculates translational kinetic energy.
    ///
    /// KE = ½mv²
    pub fn kinetic_energy(mass: f64, velocity: Vector3<f64>) -> f64 {
        0.5 * mass * velocity.norm_squared()
    }

    /// Calculates rotational kinetic energy.
    ///
    /// KE_rot = ½Iω²
    pub fn rotational_energy(moment_of_inertia: f64, angular_velocity: f64) -> f64 {
        0.5 * moment_of_inertia * angular_velocity * angular_velocity
    }

    /// Calculates total kinetic energy (translational + rotational).
    pub fn total_kinetic_energy(
        mass: f64,
        velocity: Vector3<f64>,
        moment_of_inertia: f64,
        angular_velocity: f64,
    ) -> f64 {
        Self::kinetic_energy(mass, velocity) + Self::rotational_energy(moment_of_inertia, angular_velocity)
    }

    /// Calculates gravitational potential energy.
    ///
    /// PE = mgh
    pub fn potential_energy(mass: f64, height: f64) -> f64 {
        const GRAVITY: f64 = 9.81;
        mass * GRAVITY * height
    }

    /// Analyzes energy transfer in a collision.
    pub fn analyze_collision(
        mass_a: f64,
        velocity_a_before: Vector3<f64>,
        velocity_a_after: Vector3<f64>,
        mass_b: f64,
        velocity_b_before: Vector3<f64>,
        velocity_b_after: Vector3<f64>,
    ) -> EnergyAnalysis {
        let ke_a_before = Self::kinetic_energy(mass_a, velocity_a_before);
        let ke_b_before = Self::kinetic_energy(mass_b, velocity_b_before);
        let initial_kinetic = ke_a_before + ke_b_before;

        let ke_a_after = Self::kinetic_energy(mass_a, velocity_a_after);
        let ke_b_after = Self::kinetic_energy(mass_b, velocity_b_after);
        let final_kinetic = ke_a_after + ke_b_after;

        let dissipated = initial_kinetic - final_kinetic;
        let dissipated_percent = if initial_kinetic > 0.0 {
            (dissipated / initial_kinetic) * 100.0
        } else {
            0.0
        };

        let transferred_to_a = ke_a_after - ke_a_before;
        let transferred_to_b = ke_b_after - ke_b_before;

        EnergyAnalysis {
            initial_kinetic,
            final_kinetic,
            dissipated,
            dissipated_percent,
            transferred_to_a,
            transferred_to_b,
        }
    }

    /// Estimates crush energy from deformation.
    ///
    /// Uses simplified spring model: E = ½kx²
    /// where k is material stiffness and x is crush depth.
    pub fn crush_energy(crush_depth: f64, stiffness_coefficient: f64, contact_area: f64) -> f64 {
        // Stiffness is typically given per unit area
        let effective_stiffness = stiffness_coefficient * contact_area;
        0.5 * effective_stiffness * crush_depth * crush_depth
    }

    /// Estimates vehicle stiffness coefficient from crash tests.
    ///
    /// Returns stiffness in N/m per m² of contact area.
    pub fn vehicle_stiffness_coefficient(vehicle_type: VehicleType) -> f64 {
        match vehicle_type {
            VehicleType::Sedan => 5.0e6,      // 5 MN/m per m²
            VehicleType::Suv => 6.0e6,        // 6 MN/m per m²
            VehicleType::Truck => 7.0e6,      // 7 MN/m per m²
            VehicleType::CompactCar => 4.0e6, // 4 MN/m per m²
            VehicleType::SportsCar => 5.5e6,  // 5.5 MN/m per m²
        }
    }

    /// Calculates energy equivalent speed (EES).
    ///
    /// EES is the speed at which a vehicle would have to impact a fixed barrier
    /// to produce the same amount of crush energy.
    pub fn energy_equivalent_speed(crush_energy: f64, mass: f64) -> f64 {
        // KE = ½mv² => v = sqrt(2*KE/m)
        (2.0 * crush_energy / mass).sqrt()
    }

    /// Calculates delta-V from energy dissipation.
    ///
    /// Delta-V is the change in velocity magnitude during a collision.
    pub fn delta_v_from_energy(energy_dissipated: f64, mass: f64) -> f64 {
        (2.0 * energy_dissipated / mass).sqrt()
    }

    /// Estimates energy dissipated through friction (sliding).
    ///
    /// E_friction = μ * m * g * d
    pub fn friction_energy(
        mass: f64,
        friction_coefficient: f64,
        slide_distance: f64,
    ) -> f64 {
        const GRAVITY: f64 = 9.81;
        friction_coefficient * mass * GRAVITY * slide_distance
    }

    /// Calculates work done by a force.
    ///
    /// W = F · d
    pub fn work(force: Vector3<f64>, displacement: Vector3<f64>) -> f64 {
        force.dot(&displacement)
    }

    /// Estimates energy absorbed by tires during skidding.
    pub fn tire_skid_energy(
        mass: f64,
        friction_coefficient: f64,
        skid_distance: f64,
    ) -> f64 {
        Self::friction_energy(mass, friction_coefficient, skid_distance)
    }

    /// Calculates energy dissipated through aerodynamic drag.
    ///
    /// E = ∫ F_drag dx = ∫ (½ρv²CdA) dx
    /// For constant deceleration from drag: E ≈ ½ρCdA(v₀³-v₁³)/(3a)
    pub fn drag_energy(
        initial_speed: f64,
        final_speed: f64,
        drag_coefficient: f64,
        frontal_area: f64,
        distance: f64,
    ) -> f64 {
        const AIR_DENSITY: f64 = 1.225; // kg/m³

        // Simplified: use average speed
        let avg_speed = (initial_speed + final_speed) / 2.0;
        let avg_drag_force =
            0.5 * AIR_DENSITY * avg_speed * avg_speed * drag_coefficient * frontal_area;

        avg_drag_force * distance
    }
}

/// Vehicle type for stiffness calculations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VehicleType {
    Sedan,
    Suv,
    Truck,
    CompactCar,
    SportsCar,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kinetic_energy() {
        let mass = 1500.0; // 1500 kg
        let velocity = Vector3::new(20.0, 0.0, 0.0); // 20 m/s

        let ke = EnergyCalculator::kinetic_energy(mass, velocity);

        // KE = ½ * 1500 * 20² = 300,000 J
        assert!((ke - 300000.0).abs() < 1.0);
    }

    #[test]
    fn test_energy_analysis() {
        let mass_a = 1500.0;
        let velocity_a_before = Vector3::new(20.0, 0.0, 0.0);
        let velocity_a_after = Vector3::new(10.0, 0.0, 0.0);

        let mass_b = 1500.0;
        let velocity_b_before = Vector3::new(0.0, 0.0, 0.0);
        let velocity_b_after = Vector3::new(10.0, 0.0, 0.0);

        let analysis = EnergyCalculator::analyze_collision(
            mass_a,
            velocity_a_before,
            velocity_a_after,
            mass_b,
            velocity_b_before,
            velocity_b_after,
        );

        // Initial: ½*1500*20² = 300,000 J
        // Final: ½*1500*10² + ½*1500*10² = 150,000 J
        // Dissipated: 150,000 J (50%)
        assert!((analysis.initial_kinetic - 300000.0).abs() < 1.0);
        assert!((analysis.final_kinetic - 150000.0).abs() < 1.0);
        assert!((analysis.dissipated - 150000.0).abs() < 1.0);
        assert!((analysis.dissipated_percent - 50.0).abs() < 0.1);
    }

    #[test]
    fn test_crush_energy() {
        let crush_depth = 0.3; // 30 cm
        let stiffness = EnergyCalculator::vehicle_stiffness_coefficient(VehicleType::Sedan);
        let contact_area = 0.5; // 0.5 m²

        let energy = EnergyCalculator::crush_energy(crush_depth, stiffness, contact_area);

        // E = ½ * (5e6 * 0.5) * 0.3² = 112,500 J
        assert!(energy > 100000.0 && energy < 130000.0);
    }

    #[test]
    fn test_ees() {
        let crush_energy = 100000.0; // 100 kJ
        let mass = 1500.0; // 1500 kg

        let ees = EnergyCalculator::energy_equivalent_speed(crush_energy, mass);

        // v = sqrt(2 * 100000 / 1500) = 11.55 m/s ≈ 41.6 km/h
        assert!((ees - 11.55).abs() < 0.1);
    }

    #[test]
    fn test_friction_energy() {
        let mass = 1500.0;
        let friction = 0.7;
        let distance = 50.0; // 50 meters skid

        let energy = EnergyCalculator::friction_energy(mass, friction, distance);

        // E = 0.7 * 1500 * 9.81 * 50 = 515,025 J
        assert!((energy - 515025.0).abs() < 100.0);
    }

    #[test]
    fn test_delta_v() {
        let energy = 100000.0; // 100 kJ
        let mass = 1500.0;

        let delta_v = EnergyCalculator::delta_v_from_energy(energy, mass);

        // Δv = sqrt(2 * 100000 / 1500) = 11.55 m/s
        assert!((delta_v - 11.55).abs() < 0.1);
    }

    #[test]
    fn test_work() {
        let force = Vector3::new(1000.0, 0.0, 0.0);
        let displacement = Vector3::new(10.0, 0.0, 0.0);

        let work = EnergyCalculator::work(force, displacement);

        // W = 1000 * 10 = 10,000 J
        assert_eq!(work, 10000.0);
    }
}
