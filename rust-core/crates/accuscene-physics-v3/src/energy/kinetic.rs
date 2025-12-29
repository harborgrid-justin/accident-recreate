//! Kinetic energy calculations.
//!
//! Provides detailed kinetic energy analysis for rigid bodies including:
//! - Linear kinetic energy
//! - Rotational kinetic energy
//! - Momentum calculations
//! - Impact speed estimation

use nalgebra::Vector3;
use serde::{Deserialize, Serialize};

use crate::rigid_body::RigidBody;

/// Kinetic energy components for a rigid body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KineticEnergy {
    /// Linear kinetic energy (J).
    pub linear: f64,

    /// Rotational kinetic energy (J).
    pub rotational: f64,

    /// Total kinetic energy (J).
    pub total: f64,
}

impl KineticEnergy {
    /// Computes kinetic energy for a rigid body.
    pub fn from_body(body: &RigidBody) -> Self {
        let linear = 0.5 * body.mass_props.mass * body.linear_velocity.norm_squared();

        let inertia_world = body.world_inertia_tensor();
        let rotational = 0.5 * body.angular_velocity.dot(&(inertia_world * body.angular_velocity));

        let total = linear + rotational;

        Self {
            linear,
            rotational,
            total,
        }
    }

    /// Returns the percentage of energy that is rotational.
    pub fn rotational_percentage(&self) -> f64 {
        if self.total > 1e-6 {
            (self.rotational / self.total) * 100.0
        } else {
            0.0
        }
    }
}

/// Momentum components for a rigid body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Momentum {
    /// Linear momentum (kg·m/s).
    pub linear: Vector3<f64>,

    /// Angular momentum (kg·m²/s).
    pub angular: Vector3<f64>,

    /// Linear momentum magnitude.
    pub linear_magnitude: f64,

    /// Angular momentum magnitude.
    pub angular_magnitude: f64,
}

impl Momentum {
    /// Computes momentum for a rigid body.
    pub fn from_body(body: &RigidBody) -> Self {
        let linear = body.linear_momentum();
        let angular = body.angular_momentum();

        Self {
            linear_magnitude: linear.norm(),
            angular_magnitude: angular.norm(),
            linear,
            angular,
        }
    }

    /// Computes total momentum magnitude (simplified scalar).
    pub fn total_magnitude(&self) -> f64 {
        // This is a simplification; momentum is vectorial
        self.linear_magnitude
    }
}

/// Impact analysis between two bodies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAnalysis {
    /// Relative velocity before impact (m/s).
    pub approach_velocity: Vector3<f64>,

    /// Approach speed (m/s).
    pub approach_speed: f64,

    /// Relative velocity after impact (m/s).
    pub separation_velocity: Vector3<f64>,

    /// Separation speed (m/s).
    pub separation_speed: f64,

    /// Coefficient of restitution (measured).
    pub restitution_coefficient: f64,

    /// Kinetic energy before impact (J).
    pub energy_before: f64,

    /// Kinetic energy after impact (J).
    pub energy_after: f64,

    /// Energy lost in collision (J).
    pub energy_lost: f64,

    /// Percentage of energy lost.
    pub energy_loss_percentage: f64,
}

impl ImpactAnalysis {
    /// Analyzes impact between two bodies.
    pub fn analyze(
        body_a_before: &RigidBody,
        body_b_before: &RigidBody,
        body_a_after: &RigidBody,
        body_b_after: &RigidBody,
        contact_point: Vector3<f64>,
    ) -> Self {
        // Velocities at contact point
        let va_before = body_a_before.velocity_at_point(contact_point);
        let vb_before = body_b_before.velocity_at_point(contact_point);
        let approach_velocity = vb_before - va_before;
        let approach_speed = approach_velocity.norm();

        let va_after = body_a_after.velocity_at_point(contact_point);
        let vb_after = body_b_after.velocity_at_point(contact_point);
        let separation_velocity = vb_after - va_after;
        let separation_speed = separation_velocity.norm();

        // Coefficient of restitution
        let restitution_coefficient = if approach_speed > 1e-6 {
            separation_speed / approach_speed
        } else {
            0.0
        };

        // Energy analysis
        let energy_before = body_a_before.kinetic_energy() + body_b_before.kinetic_energy();
        let energy_after = body_a_after.kinetic_energy() + body_b_after.kinetic_energy();
        let energy_lost = energy_before - energy_after;

        let energy_loss_percentage = if energy_before > 1e-6 {
            (energy_lost / energy_before) * 100.0
        } else {
            0.0
        };

        Self {
            approach_velocity,
            approach_speed,
            separation_velocity,
            separation_speed,
            restitution_coefficient,
            energy_before,
            energy_after,
            energy_lost,
            energy_loss_percentage,
        }
    }

    /// Estimates impact force (simplified).
    ///
    /// F = Δp / Δt
    /// Assumes impact duration based on approach speed and deformation.
    pub fn estimate_impact_force(&self, combined_mass: f64, impact_duration: f64) -> f64 {
        if impact_duration > 1e-6 {
            let delta_v = self.approach_speed + self.separation_speed;
            combined_mass * delta_v / impact_duration
        } else {
            0.0
        }
    }

    /// Converts energy lost to equivalent vehicle deceleration distance.
    ///
    /// Useful for accident reconstruction: "Energy dissipated equals a vehicle
    /// decelerating from V to 0 over distance D."
    pub fn equivalent_braking_distance(&self, mass: f64, deceleration: f64) -> f64 {
        if mass > 1e-6 && deceleration > 1e-6 {
            self.energy_lost / (mass * deceleration)
        } else {
            0.0
        }
    }
}

/// Computes kinetic energy for all bodies in a system.
pub fn compute_system_kinetic_energy(bodies: &[RigidBody]) -> f64 {
    bodies
        .iter()
        .filter(|b| !b.is_static)
        .map(|b| b.kinetic_energy())
        .sum()
}

/// Computes total linear momentum for all bodies in a system.
pub fn compute_system_linear_momentum(bodies: &[RigidBody]) -> Vector3<f64> {
    bodies
        .iter()
        .filter(|b| !b.is_static)
        .map(|b| b.linear_momentum())
        .sum()
}

/// Computes total angular momentum for all bodies in a system.
pub fn compute_system_angular_momentum(bodies: &[RigidBody]) -> Vector3<f64> {
    bodies
        .iter()
        .filter(|b| !b.is_static)
        .map(|b| b.angular_momentum())
        .sum()
}

/// Computes center of mass for multiple bodies.
pub fn compute_system_center_of_mass(bodies: &[RigidBody]) -> Vector3<f64> {
    let mut total_mass = 0.0;
    let mut weighted_position = Vector3::zeros();

    for body in bodies {
        if !body.is_static {
            total_mass += body.mass_props.mass;
            weighted_position += body.position * body.mass_props.mass;
        }
    }

    if total_mass > 1e-6 {
        weighted_position / total_mass
    } else {
        Vector3::zeros()
    }
}

/// Computes velocity of system center of mass.
pub fn compute_system_com_velocity(bodies: &[RigidBody]) -> Vector3<f64> {
    let mut total_mass = 0.0;
    let mut momentum = Vector3::zeros();

    for body in bodies {
        if !body.is_static {
            total_mass += body.mass_props.mass;
            momentum += body.linear_momentum();
        }
    }

    if total_mass > 1e-6 {
        momentum / total_mass
    } else {
        Vector3::zeros()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rigid_body::dynamics::MassProperties;
    use approx::assert_relative_eq;

    #[test]
    fn test_kinetic_energy_from_body() {
        let mass_props = MassProperties::from_sphere(1000.0, 1.0);
        let mut body = RigidBody::new(0, mass_props);
        body.linear_velocity = Vector3::new(10.0, 0.0, 0.0);

        let ke = KineticEnergy::from_body(&body);

        let expected = 0.5 * 1000.0 * 100.0;
        assert_relative_eq!(ke.linear, expected);
        assert_relative_eq!(ke.total, ke.linear + ke.rotational);
    }

    #[test]
    fn test_momentum_from_body() {
        let mass_props = MassProperties::from_sphere(1000.0, 1.0);
        let mut body = RigidBody::new(0, mass_props);
        body.linear_velocity = Vector3::new(5.0, 0.0, 0.0);

        let momentum = Momentum::from_body(&body);

        assert_relative_eq!(momentum.linear_magnitude, 5000.0); // m * v
    }

    #[test]
    fn test_impact_analysis() {
        let mass_props_a = MassProperties::from_sphere(1000.0, 1.0);
        let mass_props_b = MassProperties::from_sphere(1000.0, 1.0);

        let mut body_a_before = RigidBody::new(0, mass_props_a.clone());
        let mut body_b_before = RigidBody::new(1, mass_props_b.clone());

        body_a_before.position = Vector3::new(-1.0, 0.0, 0.0);
        body_b_before.position = Vector3::new(1.0, 0.0, 0.0);

        body_a_before.linear_velocity = Vector3::new(10.0, 0.0, 0.0);
        body_b_before.linear_velocity = Vector3::new(-10.0, 0.0, 0.0);

        // After elastic collision (equal masses)
        let mut body_a_after = body_a_before.clone();
        let mut body_b_after = body_b_before.clone();

        body_a_after.linear_velocity = Vector3::new(-10.0, 0.0, 0.0);
        body_b_after.linear_velocity = Vector3::new(10.0, 0.0, 0.0);

        let contact_point = Vector3::zeros();

        let analysis = ImpactAnalysis::analyze(
            &body_a_before,
            &body_b_before,
            &body_a_after,
            &body_b_after,
            contact_point,
        );

        // Elastic collision should have e ≈ 1.0
        assert_relative_eq!(analysis.restitution_coefficient, 1.0, epsilon = 0.01);

        // Energy should be conserved
        assert_relative_eq!(
            analysis.energy_before,
            analysis.energy_after,
            epsilon = 1.0
        );
    }

    #[test]
    fn test_system_kinetic_energy() {
        let mass_props = MassProperties::from_sphere(1000.0, 1.0);

        let mut body1 = RigidBody::new(0, mass_props.clone());
        body1.linear_velocity = Vector3::new(10.0, 0.0, 0.0);

        let mut body2 = RigidBody::new(1, mass_props);
        body2.linear_velocity = Vector3::new(5.0, 0.0, 0.0);

        let bodies = vec![body1, body2];
        let total_ke = compute_system_kinetic_energy(&bodies);

        let expected = 0.5 * 1000.0 * 100.0 + 0.5 * 1000.0 * 25.0;
        assert_relative_eq!(total_ke, expected, epsilon = 1.0);
    }

    #[test]
    fn test_system_momentum() {
        let mass_props = MassProperties::from_sphere(1000.0, 1.0);

        let mut body1 = RigidBody::new(0, mass_props.clone());
        body1.linear_velocity = Vector3::new(10.0, 0.0, 0.0);

        let mut body2 = RigidBody::new(1, mass_props);
        body2.linear_velocity = Vector3::new(-10.0, 0.0, 0.0);

        let bodies = vec![body1, body2];
        let total_momentum = compute_system_linear_momentum(&bodies);

        // Equal and opposite, should cancel
        assert_relative_eq!(total_momentum.norm(), 0.0, epsilon = 1.0);
    }
}
