//! Collision response and impulse resolution.
//!
//! Implements impulse-based collision resolution with friction and restitution.

use super::Collision;
use nalgebra::{Matrix3, Vector3};
use serde::{Deserialize, Serialize};

/// Impulse response calculated for a collision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpulseResponse {
    /// Impulse to apply to object A
    pub impulse_a: Vector3<f64>,
    /// Impulse to apply to object B
    pub impulse_b: Vector3<f64>,
    /// Angular impulse to apply to object A
    pub angular_impulse_a: Vector3<f64>,
    /// Angular impulse to apply to object B
    pub angular_impulse_b: Vector3<f64>,
}

/// Collision resolver using impulse-based physics.
pub struct CollisionResolver {
    /// Coefficient of restitution (bounciness)
    restitution: f64,
    /// Coefficient of friction
    friction: f64,
    /// Position correction percentage (Baumgarte stabilization)
    position_correction_percent: f64,
    /// Slop allowance for penetration
    slop: f64,
}

impl CollisionResolver {
    /// Creates a new collision resolver.
    pub fn new() -> Self {
        Self {
            restitution: 0.3,
            friction: 0.6,
            position_correction_percent: 0.8,
            slop: 0.01,
        }
    }

    /// Sets the coefficient of restitution.
    pub fn with_restitution(mut self, restitution: f64) -> Self {
        self.restitution = restitution.clamp(0.0, 1.0);
        self
    }

    /// Sets the coefficient of friction.
    pub fn with_friction(mut self, friction: f64) -> Self {
        self.friction = friction.max(0.0);
        self
    }

    /// Resolves a collision and returns the impulse response.
    pub fn resolve_collision(
        &self,
        collision: &Collision,
        mass_a: f64,
        mass_b: f64,
        velocity_a: Vector3<f64>,
        velocity_b: Vector3<f64>,
        angular_velocity_a: Vector3<f64>,
        angular_velocity_b: Vector3<f64>,
        inertia_a: Matrix3<f64>,
        inertia_b: Matrix3<f64>,
    ) -> ImpulseResponse {
        // Calculate relative velocity
        let r_a = collision.point.coords - Vector3::zeros(); // Assuming center of mass at origin
        let r_b = collision.point.coords - Vector3::zeros();

        let velocity_at_a = velocity_a + angular_velocity_a.cross(&r_a);
        let velocity_at_b = velocity_b + angular_velocity_b.cross(&r_b);
        let relative_velocity = velocity_at_a - velocity_at_b;

        // Calculate relative velocity along normal
        let velocity_along_normal = relative_velocity.dot(&collision.normal);

        // Don't resolve if velocities are separating
        if velocity_along_normal > 0.0 {
            return ImpulseResponse {
                impulse_a: Vector3::zeros(),
                impulse_b: Vector3::zeros(),
                angular_impulse_a: Vector3::zeros(),
                angular_impulse_b: Vector3::zeros(),
            };
        }

        // Calculate impulse magnitude
        let inv_mass_a = if mass_a > 0.0 { 1.0 / mass_a } else { 0.0 };
        let inv_mass_b = if mass_b > 0.0 { 1.0 / mass_b } else { 0.0 };

        let inv_inertia_a = if mass_a > 0.0 {
            inertia_a.try_inverse().unwrap_or(Matrix3::zeros())
        } else {
            Matrix3::zeros()
        };

        let inv_inertia_b = if mass_b > 0.0 {
            inertia_b.try_inverse().unwrap_or(Matrix3::zeros())
        } else {
            Matrix3::zeros()
        };

        let r_a_cross_n = r_a.cross(&collision.normal);
        let r_b_cross_n = r_b.cross(&collision.normal);

        let angular_effect_a = (inv_inertia_a * r_a_cross_n).cross(&r_a);
        let angular_effect_b = (inv_inertia_b * r_b_cross_n).cross(&r_b);

        let inv_mass_sum = inv_mass_a
            + inv_mass_b
            + angular_effect_a.dot(&collision.normal)
            + angular_effect_b.dot(&collision.normal);

        // Calculate impulse with restitution
        let j = -(1.0 + self.restitution) * velocity_along_normal / inv_mass_sum;
        let impulse_normal = collision.normal * j;

        // Calculate friction impulse
        let tangent = (relative_velocity - collision.normal * velocity_along_normal).normalize();
        let velocity_along_tangent = relative_velocity.dot(&tangent);

        let friction_mass_sum = inv_mass_a
            + inv_mass_b
            + (inv_inertia_a * r_a.cross(&tangent))
                .cross(&r_a)
                .dot(&tangent)
            + (inv_inertia_b * r_b.cross(&tangent))
                .cross(&r_b)
                .dot(&tangent);

        let mut jt = -velocity_along_tangent / friction_mass_sum;

        // Coulomb's law: friction impulse can't exceed mu * normal impulse
        let max_friction = self.friction * j;
        jt = jt.clamp(-max_friction, max_friction);

        let impulse_friction = tangent * jt;

        // Total impulse
        let total_impulse = impulse_normal + impulse_friction;

        ImpulseResponse {
            impulse_a: total_impulse,
            impulse_b: -total_impulse,
            angular_impulse_a: r_a.cross(&total_impulse),
            angular_impulse_b: r_b.cross(&-total_impulse),
        }
    }

    /// Calculates position correction using Baumgarte stabilization.
    pub fn calculate_position_correction(&self, collision: &Collision) -> Vector3<f64> {
        let correction_magnitude = (collision.penetration - self.slop).max(0.0)
            * self.position_correction_percent;
        collision.normal * correction_magnitude
    }

    /// Applies impulse response to velocities.
    pub fn apply_impulse(
        &self,
        impulse_response: &ImpulseResponse,
        mass_a: f64,
        mass_b: f64,
        velocity_a: &mut Vector3<f64>,
        velocity_b: &mut Vector3<f64>,
        angular_velocity_a: &mut Vector3<f64>,
        angular_velocity_b: &mut Vector3<f64>,
        inertia_a: Matrix3<f64>,
        inertia_b: Matrix3<f64>,
    ) {
        if mass_a > 0.0 {
            *velocity_a += impulse_response.impulse_a / mass_a;

            let inv_inertia_a = inertia_a.try_inverse().unwrap_or(Matrix3::zeros());
            *angular_velocity_a += inv_inertia_a * impulse_response.angular_impulse_a;
        }

        if mass_b > 0.0 {
            *velocity_b += impulse_response.impulse_b / mass_b;

            let inv_inertia_b = inertia_b.try_inverse().unwrap_or(Matrix3::zeros());
            *angular_velocity_b += inv_inertia_b * impulse_response.angular_impulse_b;
        }
    }
}

impl Default for CollisionResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::Point3;

    #[test]
    fn test_collision_resolution() {
        let resolver = CollisionResolver::new();

        let collision = Collision {
            object_a: 1,
            object_b: 2,
            point: Point3::new(0.0, 0.0, 0.0),
            normal: Vector3::new(1.0, 0.0, 0.0),
            penetration: 0.1,
            time_of_impact: 0.0,
            relative_velocity: Vector3::new(-10.0, 0.0, 0.0),
        };

        let mass_a = 1000.0; // 1000 kg
        let mass_b = 1500.0; // 1500 kg
        let velocity_a = Vector3::new(10.0, 0.0, 0.0); // 10 m/s
        let velocity_b = Vector3::new(-5.0, 0.0, 0.0); // -5 m/s
        let angular_velocity_a = Vector3::zeros();
        let angular_velocity_b = Vector3::zeros();
        let inertia_a = Matrix3::identity() * 1000.0;
        let inertia_b = Matrix3::identity() * 1500.0;

        let response = resolver.resolve_collision(
            &collision,
            mass_a,
            mass_b,
            velocity_a,
            velocity_b,
            angular_velocity_a,
            angular_velocity_b,
            inertia_a,
            inertia_b,
        );

        // Impulse should oppose the relative velocity
        assert!(response.impulse_a.x < 0.0);
        assert!(response.impulse_b.x > 0.0);
    }

    #[test]
    fn test_position_correction() {
        let resolver = CollisionResolver::new();

        let collision = Collision {
            object_a: 1,
            object_b: 2,
            point: Point3::new(0.0, 0.0, 0.0),
            normal: Vector3::new(0.0, 1.0, 0.0),
            penetration: 0.2,
            time_of_impact: 0.0,
            relative_velocity: Vector3::zeros(),
        };

        let correction = resolver.calculate_position_correction(&collision);

        // Correction should be along the normal
        assert!(correction.y > 0.0);
        assert_eq!(correction.x, 0.0);
        assert_eq!(correction.z, 0.0);
    }
}
