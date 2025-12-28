//! Conservation of momentum calculations for accident reconstruction.

use nalgebra::{Matrix3, Vector3};
use serde::{Deserialize, Serialize};

/// Momentum analysis for a collision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MomentumAnalysis {
    /// Total momentum before collision
    pub momentum_before: Vector3<f64>,
    /// Total momentum after collision
    pub momentum_after: Vector3<f64>,
    /// Momentum change (impulse)
    pub momentum_change: Vector3<f64>,
    /// Total angular momentum before
    pub angular_momentum_before: Vector3<f64>,
    /// Total angular momentum after
    pub angular_momentum_after: Vector3<f64>,
    /// Total kinetic energy before
    pub kinetic_energy_before: f64,
    /// Total kinetic energy after
    pub kinetic_energy_after: f64,
    /// Energy lost in collision
    pub energy_lost: f64,
    /// Coefficient of restitution
    pub coefficient_of_restitution: f64,
}

/// Conservation of momentum calculator.
pub struct MomentumConservation;

impl MomentumConservation {
    /// Calculates post-collision velocities using conservation of momentum.
    ///
    /// For a perfectly inelastic collision (objects stick together).
    pub fn inelastic_collision(
        mass_a: f64,
        velocity_a: Vector3<f64>,
        mass_b: f64,
        velocity_b: Vector3<f64>,
    ) -> Vector3<f64> {
        // Conservation of momentum: m₁v₁ + m₂v₂ = (m₁ + m₂)v_final
        let total_momentum = mass_a * velocity_a + mass_b * velocity_b;
        let total_mass = mass_a + mass_b;

        total_momentum / total_mass
    }

    /// Calculates post-collision velocities for elastic collision.
    ///
    /// Returns (velocity_a_final, velocity_b_final).
    pub fn elastic_collision(
        mass_a: f64,
        velocity_a: Vector3<f64>,
        mass_b: f64,
        velocity_b: Vector3<f64>,
    ) -> (Vector3<f64>, Vector3<f64>) {
        // For 1D elastic collision:
        // v₁' = ((m₁-m₂)v₁ + 2m₂v₂) / (m₁+m₂)
        // v₂' = ((m₂-m₁)v₂ + 2m₁v₁) / (m₁+m₂)

        let total_mass = mass_a + mass_b;

        let velocity_a_final = ((mass_a - mass_b) * velocity_a + 2.0 * mass_b * velocity_b)
            / total_mass;

        let velocity_b_final = ((mass_b - mass_a) * velocity_b + 2.0 * mass_a * velocity_a)
            / total_mass;

        (velocity_a_final, velocity_b_final)
    }

    /// Calculates post-collision velocities with coefficient of restitution.
    ///
    /// e = 0: perfectly inelastic
    /// e = 1: perfectly elastic
    /// 0 < e < 1: real-world collisions
    pub fn collision_with_restitution(
        mass_a: f64,
        velocity_a: Vector3<f64>,
        mass_b: f64,
        velocity_b: Vector3<f64>,
        coefficient_of_restitution: f64,
    ) -> (Vector3<f64>, Vector3<f64>) {
        // Conservation of momentum
        let _total_momentum = mass_a * velocity_a + mass_b * velocity_b;

        // Relative velocity
        let relative_velocity = velocity_a - velocity_b;

        // Coefficient of restitution equation: e = -(v₁' - v₂') / (v₁ - v₂)
        // Combined with momentum conservation:
        let total_mass = mass_a + mass_b;

        let velocity_a_final = (mass_a * velocity_a + mass_b * velocity_b
            - mass_b * (1.0 + coefficient_of_restitution) * relative_velocity)
            / total_mass;

        let velocity_b_final = (mass_a * velocity_a + mass_b * velocity_b
            + mass_a * (1.0 + coefficient_of_restitution) * relative_velocity)
            / total_mass;

        (velocity_a_final, velocity_b_final)
    }

    /// Calculates momentum analysis for a collision.
    pub fn analyze_collision(
        mass_a: f64,
        velocity_a_before: Vector3<f64>,
        velocity_a_after: Vector3<f64>,
        mass_b: f64,
        velocity_b_before: Vector3<f64>,
        velocity_b_after: Vector3<f64>,
        inertia_a: Matrix3<f64>,
        angular_velocity_a_before: Vector3<f64>,
        angular_velocity_a_after: Vector3<f64>,
        inertia_b: Matrix3<f64>,
        angular_velocity_b_before: Vector3<f64>,
        angular_velocity_b_after: Vector3<f64>,
    ) -> MomentumAnalysis {
        // Linear momentum
        let momentum_a_before = mass_a * velocity_a_before;
        let momentum_b_before = mass_b * velocity_b_before;
        let momentum_before = momentum_a_before + momentum_b_before;

        let momentum_a_after = mass_a * velocity_a_after;
        let momentum_b_after = mass_b * velocity_b_after;
        let momentum_after = momentum_a_after + momentum_b_after;

        let momentum_change = momentum_after - momentum_before;

        // Angular momentum
        let angular_momentum_a_before = inertia_a * angular_velocity_a_before;
        let angular_momentum_b_before = inertia_b * angular_velocity_b_before;
        let angular_momentum_before = angular_momentum_a_before + angular_momentum_b_before;

        let angular_momentum_a_after = inertia_a * angular_velocity_a_after;
        let angular_momentum_b_after = inertia_b * angular_velocity_b_after;
        let angular_momentum_after = angular_momentum_a_after + angular_momentum_b_after;

        // Kinetic energy
        let ke_a_before = 0.5 * mass_a * velocity_a_before.norm_squared()
            + 0.5
                * (angular_velocity_a_before.transpose()
                    * inertia_a
                    * angular_velocity_a_before)[0];
        let ke_b_before = 0.5 * mass_b * velocity_b_before.norm_squared()
            + 0.5
                * (angular_velocity_b_before.transpose()
                    * inertia_b
                    * angular_velocity_b_before)[0];
        let kinetic_energy_before = ke_a_before + ke_b_before;

        let ke_a_after = 0.5 * mass_a * velocity_a_after.norm_squared()
            + 0.5
                * (angular_velocity_a_after.transpose() * inertia_a * angular_velocity_a_after)
                    [0];
        let ke_b_after = 0.5 * mass_b * velocity_b_after.norm_squared()
            + 0.5
                * (angular_velocity_b_after.transpose() * inertia_b * angular_velocity_b_after)
                    [0];
        let kinetic_energy_after = ke_a_after + ke_b_after;

        let energy_lost = kinetic_energy_before - kinetic_energy_after;

        // Coefficient of restitution (1D approximation)
        let relative_velocity_before = (velocity_a_before - velocity_b_before).norm();
        let relative_velocity_after = (velocity_a_after - velocity_b_after).norm();

        let coefficient_of_restitution = if relative_velocity_before > 0.01 {
            relative_velocity_after / relative_velocity_before
        } else {
            0.0
        };

        MomentumAnalysis {
            momentum_before,
            momentum_after,
            momentum_change,
            angular_momentum_before,
            angular_momentum_after,
            kinetic_energy_before,
            kinetic_energy_after,
            energy_lost,
            coefficient_of_restitution,
        }
    }

    /// Calculates the impulse from momentum change.
    pub fn calculate_impulse(momentum_change: Vector3<f64>) -> Vector3<f64> {
        // Impulse = Δp
        momentum_change
    }

    /// Calculates average force from impulse and time.
    pub fn average_force(impulse: Vector3<f64>, time: f64) -> Vector3<f64> {
        // F_avg = J / Δt
        if time > 0.0 {
            impulse / time
        } else {
            Vector3::zeros()
        }
    }

    /// Estimates impact duration from crush depth and relative velocity.
    ///
    /// Uses simplified spring-mass model.
    pub fn estimate_impact_duration(
        _crush_depth: f64,
        _relative_velocity: f64,
        stiffness_coefficient: f64,
        mass_effective: f64,
    ) -> f64 {
        // Period of oscillation for spring-mass system: T = 2π√(m/k)
        // Impact duration is approximately 1/4 of period
        let period = 2.0 * std::f64::consts::PI * (mass_effective / stiffness_coefficient).sqrt();
        period / 4.0
    }

    /// Calculates effective mass for two-body collision.
    pub fn effective_mass(mass_a: f64, mass_b: f64) -> f64 {
        // m_eff = (m₁ * m₂) / (m₁ + m₂)
        (mass_a * mass_b) / (mass_a + mass_b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inelastic_collision() {
        let mass_a = 1000.0;
        let velocity_a = Vector3::new(20.0, 0.0, 0.0);
        let mass_b = 1500.0;
        let velocity_b = Vector3::new(-10.0, 0.0, 0.0);

        let final_velocity =
            MomentumConservation::inelastic_collision(mass_a, velocity_a, mass_b, velocity_b);

        // Check momentum conservation
        let momentum_before = mass_a * velocity_a + mass_b * velocity_b;
        let momentum_after = (mass_a + mass_b) * final_velocity;

        assert!((momentum_before - momentum_after).norm() < 0.01);
    }

    #[test]
    fn test_elastic_collision() {
        let mass_a = 1000.0;
        let velocity_a = Vector3::new(10.0, 0.0, 0.0);
        let mass_b = 1000.0; // Equal masses
        let velocity_b = Vector3::new(0.0, 0.0, 0.0);

        let (velocity_a_final, velocity_b_final) =
            MomentumConservation::elastic_collision(mass_a, velocity_a, mass_b, velocity_b);

        // For equal masses in elastic collision, velocities should exchange
        assert!((velocity_a_final.x - 0.0).abs() < 0.01);
        assert!((velocity_b_final.x - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_collision_with_restitution() {
        let mass_a = 1000.0;
        let velocity_a = Vector3::new(20.0, 0.0, 0.0);
        let mass_b = 1500.0;
        let velocity_b = Vector3::new(0.0, 0.0, 0.0);
        let e = 0.5; // Coefficient of restitution

        let (velocity_a_final, velocity_b_final) = MomentumConservation::collision_with_restitution(
            mass_a, velocity_a, mass_b, velocity_b, e,
        );

        // Check momentum conservation
        let momentum_before = mass_a * velocity_a + mass_b * velocity_b;
        let momentum_after = mass_a * velocity_a_final + mass_b * velocity_b_final;

        assert!((momentum_before - momentum_after).norm() < 0.01);

        // Check coefficient of restitution
        let relative_velocity_before = (velocity_a - velocity_b).norm();
        let relative_velocity_after = (velocity_b_final - velocity_a_final).norm();
        let calculated_e = relative_velocity_after / relative_velocity_before;

        assert!((calculated_e - e).abs() < 0.01);
    }

    #[test]
    fn test_effective_mass() {
        let mass_a = 1000.0;
        let mass_b = 1500.0;

        let m_eff = MomentumConservation::effective_mass(mass_a, mass_b);

        // m_eff = (1000 * 1500) / (1000 + 1500) = 600
        assert!((m_eff - 600.0).abs() < 0.01);
    }

    #[test]
    fn test_impulse_and_force() {
        let momentum_change = Vector3::new(10000.0, 0.0, 0.0); // 10000 N·s
        let time = 0.1; // 0.1 seconds

        let impulse = MomentumConservation::calculate_impulse(momentum_change);
        let force = MomentumConservation::average_force(impulse, time);

        // F = J/t = 10000/0.1 = 100000 N
        assert!((force.x - 100000.0).abs() < 0.01);
    }
}
