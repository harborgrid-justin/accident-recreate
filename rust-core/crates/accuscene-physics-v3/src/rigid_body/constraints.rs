//! Constraint systems for rigid bodies.
//!
//! Implements various constraint types:
//! - Contact constraints (non-penetration, friction)
//! - Joint constraints (ball-socket, hinge, slider)
//! - Limit constraints

use nalgebra::{Matrix3, Vector3};
use serde::{Deserialize, Serialize};

use super::RigidBody;
use crate::error::{PhysicsError, PhysicsResult};

/// Contact constraint between two bodies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactConstraint {
    /// First body ID.
    pub body_a: usize,

    /// Second body ID.
    pub body_b: usize,

    /// Contact point on body A (world space).
    pub point_a: Vector3<f64>,

    /// Contact point on body B (world space).
    pub point_b: Vector3<f64>,

    /// Contact normal (from A to B).
    pub normal: Vector3<f64>,

    /// Penetration depth (negative = separation).
    pub penetration: f64,

    /// Coefficient of restitution.
    pub restitution: f64,

    /// Coefficient of friction.
    pub friction: f64,

    /// Accumulated normal impulse (for warm starting).
    pub accumulated_normal_impulse: f64,

    /// Accumulated tangent impulse (for friction).
    pub accumulated_tangent_impulse: Vector3<f64>,
}

impl ContactConstraint {
    /// Creates a new contact constraint.
    pub fn new(
        body_a: usize,
        body_b: usize,
        point_a: Vector3<f64>,
        point_b: Vector3<f64>,
        normal: Vector3<f64>,
        penetration: f64,
        restitution: f64,
        friction: f64,
    ) -> Self {
        Self {
            body_a,
            body_b,
            point_a,
            point_b,
            normal,
            penetration,
            restitution,
            friction,
            accumulated_normal_impulse: 0.0,
            accumulated_tangent_impulse: Vector3::zeros(),
        }
    }

    /// Solves the contact constraint (non-penetration + friction).
    ///
    /// Uses sequential impulse method with Coulomb friction cone.
    pub fn solve(
        &mut self,
        body_a: &mut RigidBody,
        body_b: &mut RigidBody,
        dt: f64,
        baumgarte_factor: f64,
        contact_slop: f64,
    ) -> PhysicsResult<()> {
        // Contact points relative to centers of mass
        let ra = self.point_a - body_a.position;
        let rb = self.point_b - body_b.position;

        // --- Normal constraint (non-penetration) ---

        // Relative velocity at contact point
        let va = body_a.velocity_at_point(self.point_a);
        let vb = body_b.velocity_at_point(self.point_b);
        let relative_velocity = vb - va;

        // Normal velocity
        let normal_velocity = relative_velocity.dot(&self.normal);

        // Compute effective mass along normal
        let ra_cross_n = ra.cross(&self.normal);
        let rb_cross_n = rb.cross(&self.normal);

        let inv_inertia_a = body_a.world_inverse_inertia_tensor();
        let inv_inertia_b = body_b.world_inverse_inertia_tensor();

        let angular_factor_a = ra_cross_n.dot(&(inv_inertia_a * ra_cross_n));
        let angular_factor_b = rb_cross_n.dot(&(inv_inertia_b * rb_cross_n));

        let inv_mass_sum =
            body_a.mass_props.inverse_mass + body_b.mass_props.inverse_mass;

        let effective_mass = inv_mass_sum + angular_factor_a + angular_factor_b;

        if effective_mass < 1e-10 {
            return Ok(());
        }

        let k = 1.0 / effective_mass;

        // Baumgarte stabilization for position correction
        let position_correction = if self.penetration > contact_slop {
            baumgarte_factor * (self.penetration - contact_slop) / dt
        } else {
            0.0
        };

        // Restitution bias
        let restitution_bias = if normal_velocity < -1.0 {
            -self.restitution * normal_velocity
        } else {
            0.0
        };

        // Compute normal impulse
        let bias = position_correction + restitution_bias;
        let lambda = -k * (normal_velocity - bias);

        // Clamp accumulated impulse to be non-negative (no pull)
        let old_impulse = self.accumulated_normal_impulse;
        self.accumulated_normal_impulse = (old_impulse + lambda).max(0.0);
        let delta_impulse = self.accumulated_normal_impulse - old_impulse;

        // Apply normal impulse
        let impulse_vector = self.normal * delta_impulse;
        self.apply_impulse(body_a, body_b, -impulse_vector, impulse_vector, ra, rb);

        // --- Friction constraint ---

        if self.friction > 0.0 {
            // Recompute relative velocity after normal impulse
            let va = body_a.velocity_at_point(self.point_a);
            let vb = body_b.velocity_at_point(self.point_b);
            let relative_velocity = vb - va;

            // Tangent velocity (perpendicular to normal)
            let tangent_velocity = relative_velocity - self.normal * relative_velocity.dot(&self.normal);
            let tangent_speed = tangent_velocity.norm();

            if tangent_speed > 1e-6 {
                let tangent_dir = tangent_velocity / tangent_speed;

                // Compute effective mass along tangent
                let ra_cross_t = ra.cross(&tangent_dir);
                let rb_cross_t = rb.cross(&tangent_dir);

                let angular_factor_a_t = ra_cross_t.dot(&(inv_inertia_a * ra_cross_t));
                let angular_factor_b_t = rb_cross_t.dot(&(inv_inertia_b * rb_cross_t));

                let effective_mass_tangent = inv_mass_sum + angular_factor_a_t + angular_factor_b_t;

                if effective_mass_tangent > 1e-10 {
                    let k_tangent = 1.0 / effective_mass_tangent;

                    // Compute friction impulse
                    let lambda_tangent = -k_tangent * tangent_speed;

                    // Clamp to friction cone (Coulomb friction)
                    let max_friction = self.friction * self.accumulated_normal_impulse;
                    let old_tangent_impulse = self.accumulated_tangent_impulse.dot(&tangent_dir);
                    let new_tangent_impulse = (old_tangent_impulse + lambda_tangent)
                        .max(-max_friction)
                        .min(max_friction);
                    let delta_tangent_impulse = new_tangent_impulse - old_tangent_impulse;

                    self.accumulated_tangent_impulse += tangent_dir * delta_tangent_impulse;

                    // Apply friction impulse
                    let friction_impulse = tangent_dir * delta_tangent_impulse;
                    self.apply_impulse(body_a, body_b, -friction_impulse, friction_impulse, ra, rb);
                }
            }
        }

        Ok(())
    }

    /// Helper to apply impulse to both bodies.
    fn apply_impulse(
        &self,
        body_a: &mut RigidBody,
        body_b: &mut RigidBody,
        impulse_a: Vector3<f64>,
        impulse_b: Vector3<f64>,
        ra: Vector3<f64>,
        rb: Vector3<f64>,
    ) {
        if !body_a.is_static {
            body_a.linear_velocity += impulse_a * body_a.mass_props.inverse_mass;
            let angular_impulse_a = ra.cross(&impulse_a);
            let inv_inertia_a = body_a.world_inverse_inertia_tensor();
            body_a.angular_velocity += inv_inertia_a * angular_impulse_a;
        }

        if !body_b.is_static {
            body_b.linear_velocity += impulse_b * body_b.mass_props.inverse_mass;
            let angular_impulse_b = rb.cross(&impulse_b);
            let inv_inertia_b = body_b.world_inverse_inertia_tensor();
            body_b.angular_velocity += inv_inertia_b * angular_impulse_b;
        }
    }
}

/// Ball-and-socket joint (spherical joint).
///
/// Constrains two points on different bodies to remain coincident.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BallSocketJoint {
    /// First body ID.
    pub body_a: usize,

    /// Second body ID.
    pub body_b: usize,

    /// Anchor point in body A's local space.
    pub local_anchor_a: Vector3<f64>,

    /// Anchor point in body B's local space.
    pub local_anchor_b: Vector3<f64>,

    /// Accumulated impulse for warm starting.
    pub accumulated_impulse: Vector3<f64>,
}

impl BallSocketJoint {
    /// Creates a new ball-socket joint.
    pub fn new(
        body_a: usize,
        body_b: usize,
        local_anchor_a: Vector3<f64>,
        local_anchor_b: Vector3<f64>,
    ) -> Self {
        Self {
            body_a,
            body_b,
            local_anchor_a,
            local_anchor_b,
            accumulated_impulse: Vector3::zeros(),
        }
    }

    /// Solves the ball-socket constraint.
    pub fn solve(
        &mut self,
        body_a: &mut RigidBody,
        body_b: &mut RigidBody,
        dt: f64,
        baumgarte_factor: f64,
    ) -> PhysicsResult<()> {
        // World space anchor points
        let anchor_a = body_a.local_to_world(self.local_anchor_a);
        let anchor_b = body_b.local_to_world(self.local_anchor_b);

        // Position error
        let position_error = anchor_b - anchor_a;

        // Relative to centers of mass
        let ra = anchor_a - body_a.position;
        let rb = anchor_b - body_b.position;

        // Relative velocity
        let va = body_a.velocity_at_point(anchor_a);
        let vb = body_b.velocity_at_point(anchor_b);
        let velocity_error = vb - va;

        // Compute effective mass matrix (3x3)
        let inv_mass_a = body_a.mass_props.inverse_mass;
        let inv_mass_b = body_b.mass_props.inverse_mass;
        let inv_inertia_a = body_a.world_inverse_inertia_tensor();
        let inv_inertia_b = body_b.world_inverse_inertia_tensor();

        let ra_skew = skew_symmetric(&ra);
        let rb_skew = skew_symmetric(&rb);

        let k_matrix = Matrix3::identity() * (inv_mass_a + inv_mass_b)
            + ra_skew.transpose() * inv_inertia_a * ra_skew
            + rb_skew.transpose() * inv_inertia_b * rb_skew;

        // Invert effective mass matrix
        let k_inv = k_matrix.try_inverse().ok_or_else(|| {
            PhysicsError::SingularMatrix {
                operation: "ball-socket joint effective mass".to_string(),
                determinant: k_matrix.determinant(),
            }
        })?;

        // Compute impulse
        let bias = (baumgarte_factor / dt) * position_error;
        let lambda = k_inv * -(velocity_error + bias);

        // Apply impulse
        if !body_a.is_static {
            body_a.linear_velocity -= lambda * inv_mass_a;
            body_a.angular_velocity -= inv_inertia_a * ra.cross(&lambda);
        }

        if !body_b.is_static {
            body_b.linear_velocity += lambda * inv_mass_b;
            body_b.angular_velocity += inv_inertia_b * rb.cross(&lambda);
        }

        self.accumulated_impulse += lambda;

        Ok(())
    }
}

/// Hinge joint (revolute joint).
///
/// Constrains rotation to a single axis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HingeJoint {
    /// First body ID.
    pub body_a: usize,

    /// Second body ID.
    pub body_b: usize,

    /// Anchor point in body A's local space.
    pub local_anchor_a: Vector3<f64>,

    /// Anchor point in body B's local space.
    pub local_anchor_b: Vector3<f64>,

    /// Hinge axis in body A's local space (unit vector).
    pub local_axis_a: Vector3<f64>,

    /// Hinge axis in body B's local space (unit vector).
    pub local_axis_b: Vector3<f64>,

    /// Angular limits (min, max) in radians.
    pub angle_limits: Option<(f64, f64)>,

    /// Current angle.
    pub current_angle: f64,
}

impl HingeJoint {
    /// Creates a new hinge joint.
    pub fn new(
        body_a: usize,
        body_b: usize,
        local_anchor_a: Vector3<f64>,
        local_anchor_b: Vector3<f64>,
        local_axis_a: Vector3<f64>,
        local_axis_b: Vector3<f64>,
    ) -> Self {
        Self {
            body_a,
            body_b,
            local_anchor_a,
            local_anchor_b,
            local_axis_a: local_axis_a.normalize(),
            local_axis_b: local_axis_b.normalize(),
            angle_limits: None,
            current_angle: 0.0,
        }
    }

    /// Sets angular limits.
    pub fn with_limits(mut self, min_angle: f64, max_angle: f64) -> Self {
        self.angle_limits = Some((min_angle, max_angle));
        self
    }
}

/// Helper function to create skew-symmetric matrix from vector.
///
/// Used for cross product: a × b = [a]_× b
fn skew_symmetric(v: &Vector3<f64>) -> Matrix3<f64> {
    Matrix3::new(
        0.0, -v.z, v.y,
        v.z, 0.0, -v.x,
        -v.y, v.x, 0.0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rigid_body::dynamics::MassProperties;
    use approx::assert_relative_eq;

    #[test]
    fn test_contact_constraint_creation() {
        let constraint = ContactConstraint::new(
            0,
            1,
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            0.01,
            0.5,
            0.7,
        );

        assert_eq!(constraint.body_a, 0);
        assert_eq!(constraint.body_b, 1);
        assert_eq!(constraint.restitution, 0.5);
    }

    #[test]
    fn test_ball_socket_joint_creation() {
        let joint = BallSocketJoint::new(
            0,
            1,
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(-1.0, 0.0, 0.0),
        );

        assert_eq!(joint.body_a, 0);
        assert_eq!(joint.body_b, 1);
    }

    #[test]
    fn test_skew_symmetric() {
        let v = Vector3::new(1.0, 2.0, 3.0);
        let skew = skew_symmetric(&v);

        assert_eq!(skew[(0, 1)], -3.0);
        assert_eq!(skew[(1, 0)], 3.0);
        assert_eq!(skew[(0, 2)], 2.0);
    }
}
