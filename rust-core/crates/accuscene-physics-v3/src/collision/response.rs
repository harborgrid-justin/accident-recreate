//! Collision response using impulse-based methods.
//!
//! Implements:
//! - Linear and angular impulse calculation
//! - Coefficient of restitution
//! - Friction (Coulomb model)

use serde::{Deserialize, Serialize};

use super::ContactPoint;
use crate::rigid_body::RigidBody;

/// Collision response handler.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollisionResponse {
    /// Global restitution coefficient (0 = inelastic, 1 = elastic).
    pub restitution: f64,

    /// Global friction coefficient.
    pub friction: f64,
}

impl CollisionResponse {
    /// Creates a new collision response handler.
    pub fn new(restitution: f64, friction: f64) -> Self {
        Self {
            restitution,
            friction,
        }
    }

    /// Applies impulse-based collision response.
    ///
    /// Uses the formula:
    /// J = -(1 + e) * v_rel · n / (1/m_a + 1/m_b + (r_a × n)^T I_a^(-1) (r_a × n) + (r_b × n)^T I_b^(-1) (r_b × n))
    pub fn resolve_collision(
        &self,
        body_a: &mut RigidBody,
        body_b: &mut RigidBody,
        contact: &ContactPoint,
    ) {
        if body_a.is_static && body_b.is_static {
            return;
        }

        // Contact points relative to centers of mass
        let ra = contact.point - body_a.position;
        let rb = contact.point - body_b.position;

        // Relative velocity at contact point
        let va = body_a.velocity_at_point(contact.point);
        let vb = body_b.velocity_at_point(contact.point);
        let v_rel = vb - va;

        // Relative velocity along normal
        let v_rel_normal = v_rel.dot(&contact.normal);

        // Bodies separating, no impulse needed
        if v_rel_normal > 0.0 {
            return;
        }

        // Compute effective mass
        let ra_cross_n = ra.cross(&contact.normal);
        let rb_cross_n = rb.cross(&contact.normal);

        let inv_mass_a = if body_a.is_static { 0.0 } else { body_a.mass_props.inverse_mass };
        let inv_mass_b = if body_b.is_static { 0.0 } else { body_b.mass_props.inverse_mass };

        let inv_inertia_a = if body_a.is_static {
            nalgebra::Matrix3::zeros()
        } else {
            body_a.world_inverse_inertia_tensor()
        };

        let inv_inertia_b = if body_b.is_static {
            nalgebra::Matrix3::zeros()
        } else {
            body_b.world_inverse_inertia_tensor()
        };

        let angular_factor_a = ra_cross_n.dot(&(inv_inertia_a * ra_cross_n));
        let angular_factor_b = rb_cross_n.dot(&(inv_inertia_b * rb_cross_n));

        let effective_mass_inv = inv_mass_a + inv_mass_b + angular_factor_a + angular_factor_b;

        if effective_mass_inv < 1e-10 {
            return;
        }

        // Combined restitution (geometric mean)
        let e = self.restitution;

        // Normal impulse magnitude
        let j = -(1.0 + e) * v_rel_normal / effective_mass_inv;

        // Apply normal impulse
        let impulse_normal = contact.normal * j;

        if !body_a.is_static {
            body_a.linear_velocity -= impulse_normal * inv_mass_a;
            body_a.angular_velocity -= inv_inertia_a * ra.cross(&impulse_normal);
        }

        if !body_b.is_static {
            body_b.linear_velocity += impulse_normal * inv_mass_b;
            body_b.angular_velocity += inv_inertia_b * rb.cross(&impulse_normal);
        }

        // Wake up bodies
        body_a.wake();
        body_b.wake();

        // --- Friction ---

        if self.friction > 0.0 {
            // Recompute relative velocity after normal impulse
            let va = body_a.velocity_at_point(contact.point);
            let vb = body_b.velocity_at_point(contact.point);
            let v_rel = vb - va;

            // Tangent velocity (perpendicular to normal)
            let v_tangent = v_rel - contact.normal * v_rel.dot(&contact.normal);
            let v_tangent_mag = v_tangent.norm();

            if v_tangent_mag > 1e-6 {
                let tangent_dir = v_tangent / v_tangent_mag;

                // Effective mass along tangent
                let ra_cross_t = ra.cross(&tangent_dir);
                let rb_cross_t = rb.cross(&tangent_dir);

                let angular_factor_a_t = ra_cross_t.dot(&(inv_inertia_a * ra_cross_t));
                let angular_factor_b_t = rb_cross_t.dot(&(inv_inertia_b * rb_cross_t));

                let effective_mass_tangent_inv = inv_mass_a + inv_mass_b + angular_factor_a_t + angular_factor_b_t;

                if effective_mass_tangent_inv > 1e-10 {
                    // Friction impulse magnitude (clamped to Coulomb friction cone)
                    let mut j_tangent = -v_tangent_mag / effective_mass_tangent_inv;

                    // Coulomb friction limit
                    let max_friction = self.friction * j.abs();
                    j_tangent = j_tangent.max(-max_friction).min(max_friction);

                    // Apply friction impulse
                    let impulse_friction = tangent_dir * j_tangent;

                    if !body_a.is_static {
                        body_a.linear_velocity -= impulse_friction * inv_mass_a;
                        body_a.angular_velocity -= inv_inertia_a * ra.cross(&impulse_friction);
                    }

                    if !body_b.is_static {
                        body_b.linear_velocity += impulse_friction * inv_mass_b;
                        body_b.angular_velocity += inv_inertia_b * rb.cross(&impulse_friction);
                    }
                }
            }
        }
    }

    /// Resolves collision with custom material properties.
    pub fn resolve_collision_with_materials(
        body_a: &mut RigidBody,
        body_b: &mut RigidBody,
        contact: &ContactPoint,
        restitution_a: f64,
        restitution_b: f64,
        friction_a: f64,
        friction_b: f64,
    ) {
        // Combined properties (geometric mean for restitution, harmonic mean for friction)
        let combined_restitution = (restitution_a * restitution_b).sqrt();
        let combined_friction = if friction_a > 0.0 && friction_b > 0.0 {
            2.0 / (1.0 / friction_a + 1.0 / friction_b)
        } else {
            0.0
        };

        let response = CollisionResponse::new(combined_restitution, combined_friction);
        response.resolve_collision(body_a, body_b, contact);
    }

    /// Computes impulse magnitude without applying it (for constraint solvers).
    pub fn compute_impulse_magnitude(
        body_a: &RigidBody,
        body_b: &RigidBody,
        contact: &ContactPoint,
        restitution: f64,
    ) -> f64 {
        let ra = contact.point - body_a.position;
        let rb = contact.point - body_b.position;

        let va = body_a.velocity_at_point(contact.point);
        let vb = body_b.velocity_at_point(contact.point);
        let v_rel = vb - va;
        let v_rel_normal = v_rel.dot(&contact.normal);

        if v_rel_normal > 0.0 {
            return 0.0;
        }

        let ra_cross_n = ra.cross(&contact.normal);
        let rb_cross_n = rb.cross(&contact.normal);

        let inv_mass_a = if body_a.is_static { 0.0 } else { body_a.mass_props.inverse_mass };
        let inv_mass_b = if body_b.is_static { 0.0 } else { body_b.mass_props.inverse_mass };

        let inv_inertia_a = if body_a.is_static {
            nalgebra::Matrix3::zeros()
        } else {
            body_a.world_inverse_inertia_tensor()
        };

        let inv_inertia_b = if body_b.is_static {
            nalgebra::Matrix3::zeros()
        } else {
            body_b.world_inverse_inertia_tensor()
        };

        let angular_factor_a = ra_cross_n.dot(&(inv_inertia_a * ra_cross_n));
        let angular_factor_b = rb_cross_n.dot(&(inv_inertia_b * rb_cross_n));

        let effective_mass_inv = inv_mass_a + inv_mass_b + angular_factor_a + angular_factor_b;

        if effective_mass_inv < 1e-10 {
            return 0.0;
        }

        -(1.0 + restitution) * v_rel_normal / effective_mass_inv
    }
}

impl Default for CollisionResponse {
    fn default() -> Self {
        Self::new(0.3, 0.7)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rigid_body::dynamics::MassProperties;
    use approx::assert_relative_eq;

    #[test]
    fn test_collision_response_creation() {
        let response = CollisionResponse::new(0.5, 0.8);
        assert_eq!(response.restitution, 0.5);
        assert_eq!(response.friction, 0.8);
    }

    #[test]
    fn test_impulse_calculation() {
        let mass_props_a = MassProperties::from_sphere(1000.0, 1.0);
        let mass_props_b = MassProperties::from_sphere(1000.0, 1.0);

        let mut body_a = RigidBody::new(0, mass_props_a);
        let mut body_b = RigidBody::new(1, mass_props_b);

        body_a.position = Vector3::new(-1.0, 0.0, 0.0);
        body_b.position = Vector3::new(1.0, 0.0, 0.0);

        body_a.linear_velocity = Vector3::new(1.0, 0.0, 0.0);
        body_b.linear_velocity = Vector3::new(-1.0, 0.0, 0.0);

        let contact = ContactPoint {
            point: Vector3::new(0.0, 0.0, 0.0),
            normal: Vector3::new(1.0, 0.0, 0.0),
            penetration: 0.1,
            feature_id: 0,
        };

        let impulse = CollisionResponse::compute_impulse_magnitude(&body_a, &body_b, &contact, 1.0);

        // Head-on collision with e=1.0, should have significant impulse
        assert!(impulse > 0.0);
    }
}
