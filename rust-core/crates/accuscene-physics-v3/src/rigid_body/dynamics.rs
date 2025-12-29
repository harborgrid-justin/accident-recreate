//! Rigid body dynamics calculations.
//!
//! This module implements:
//! - Mass properties (mass, center of mass, inertia tensor)
//! - Time integration for rigid bodies
//! - Force and torque calculations

use nalgebra::{Matrix3, Quaternion, UnitQuaternion, Vector3};
use serde::{Deserialize, Serialize};

use super::RigidBody;
use crate::error::{PhysicsError, PhysicsResult};

/// Mass properties for a rigid body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MassProperties {
    /// Total mass (kg).
    pub mass: f64,

    /// Inverse mass (1/kg) for optimization.
    pub inverse_mass: f64,

    /// Inertia tensor in local space (kg·m²).
    pub inertia_tensor: Matrix3<f64>,

    /// Inverse inertia tensor in local space.
    pub inverse_inertia_tensor: Matrix3<f64>,

    /// Center of mass in local coordinates.
    pub center_of_mass: Vector3<f64>,
}

impl MassProperties {
    /// Creates mass properties from explicit values.
    pub fn new(
        mass: f64,
        inertia_tensor: Matrix3<f64>,
        center_of_mass: Vector3<f64>,
    ) -> PhysicsResult<Self> {
        if mass <= 0.0 {
            return Err(PhysicsError::invalid_state(format!(
                "Mass must be positive, got {}",
                mass
            )));
        }

        let inverse_mass = 1.0 / mass;

        // Attempt to invert inertia tensor
        let inverse_inertia_tensor = inertia_tensor
            .try_inverse()
            .ok_or_else(|| PhysicsError::SingularMatrix {
                operation: "inertia tensor inversion".to_string(),
                determinant: inertia_tensor.determinant(),
            })?;

        Ok(Self {
            mass,
            inverse_mass,
            inertia_tensor,
            inverse_inertia_tensor,
            center_of_mass,
        })
    }

    /// Creates mass properties for a box.
    ///
    /// Inertia tensor for a box with dimensions (w, h, d):
    /// Ixx = (1/12) * m * (h² + d²)
    /// Iyy = (1/12) * m * (w² + d²)
    /// Izz = (1/12) * m * (w² + h²)
    pub fn from_box(mass: f64, dimensions: Vector3<f64>) -> Self {
        let w = dimensions.x;
        let h = dimensions.y;
        let d = dimensions.z;

        let ixx = (mass / 12.0) * (h * h + d * d);
        let iyy = (mass / 12.0) * (w * w + d * d);
        let izz = (mass / 12.0) * (w * w + h * h);

        let inertia = Matrix3::from_diagonal(&Vector3::new(ixx, iyy, izz));

        Self::new(mass, inertia, Vector3::zeros()).unwrap()
    }

    /// Creates mass properties for a sphere.
    ///
    /// Inertia for a solid sphere:
    /// I = (2/5) * m * r²
    pub fn from_sphere(mass: f64, radius: f64) -> Self {
        let inertia_scalar = (2.0 / 5.0) * mass * radius * radius;
        let inertia = Matrix3::from_diagonal(&Vector3::new(
            inertia_scalar,
            inertia_scalar,
            inertia_scalar,
        ));

        Self::new(mass, inertia, Vector3::zeros()).unwrap()
    }

    /// Creates mass properties for a cylinder.
    ///
    /// Inertia for a solid cylinder (axis along z):
    /// Ixx = Iyy = (1/12) * m * (3r² + h²)
    /// Izz = (1/2) * m * r²
    pub fn from_cylinder(mass: f64, radius: f64, height: f64) -> Self {
        let ixx = (mass / 12.0) * (3.0 * radius * radius + height * height);
        let iyy = ixx;
        let izz = 0.5 * mass * radius * radius;

        let inertia = Matrix3::from_diagonal(&Vector3::new(ixx, iyy, izz));

        Self::new(mass, inertia, Vector3::zeros()).unwrap()
    }

    /// Creates infinite mass properties (static body).
    pub fn infinite() -> Self {
        Self {
            mass: f64::INFINITY,
            inverse_mass: 0.0,
            inertia_tensor: Matrix3::zeros(),
            inverse_inertia_tensor: Matrix3::zeros(),
            center_of_mass: Vector3::zeros(),
        }
    }
}

/// Time integration for rigid bodies.
pub struct RigidBodyIntegrator;

impl RigidBodyIntegrator {
    /// Semi-implicit Euler integration.
    ///
    /// Update velocities first, then positions (symplectic).
    /// v(t+dt) = v(t) + a(t) * dt
    /// x(t+dt) = x(t) + v(t+dt) * dt
    pub fn semi_implicit_euler(body: &mut RigidBody, dt: f64, gravity: Vector3<f64>) {
        if body.is_static || !body.is_awake {
            return;
        }

        // Add gravity force
        let gravity_force = gravity * body.mass_props.mass;
        body.force += gravity_force;

        // Linear motion
        let acceleration = body.force * body.mass_props.inverse_mass;
        body.linear_velocity += acceleration * dt;
        body.position += body.linear_velocity * dt;

        // Angular motion
        // α = I^(-1) * τ
        let inverse_inertia_world = body.world_inverse_inertia_tensor();
        let angular_acceleration = inverse_inertia_world * body.torque;
        body.angular_velocity += angular_acceleration * dt;

        // Update orientation using quaternion derivative
        // dq/dt = 0.5 * q * ω
        let omega_quat = Quaternion::new(
            0.0,
            body.angular_velocity.x,
            body.angular_velocity.y,
            body.angular_velocity.z,
        );
        let orientation_quat = body.orientation.quaternion();
        let dq = (orientation_quat * omega_quat) * 0.5;

        let new_quat = Quaternion::new(
            orientation_quat.w + dq.w * dt,
            orientation_quat.i + dq.i * dt,
            orientation_quat.j + dq.j * dt,
            orientation_quat.k + dq.k * dt,
        );

        body.orientation = UnitQuaternion::from_quaternion(new_quat);

        // Clear forces for next step
        body.clear_forces();
    }

    /// Velocity Verlet integration (2nd order accurate).
    ///
    /// x(t+dt) = x(t) + v(t)*dt + 0.5*a(t)*dt²
    /// v(t+dt) = v(t) + 0.5*(a(t) + a(t+dt))*dt
    pub fn velocity_verlet(
        body: &mut RigidBody,
        dt: f64,
        gravity: Vector3<f64>,
        prev_acceleration: &mut Vector3<f64>,
        prev_angular_accel: &mut Vector3<f64>,
    ) {
        if body.is_static || !body.is_awake {
            return;
        }

        // Add gravity
        let gravity_force = gravity * body.mass_props.mass;
        body.force += gravity_force;

        // Current acceleration
        let accel = body.force * body.mass_props.inverse_mass;
        let inverse_inertia_world = body.world_inverse_inertia_tensor();
        let angular_accel = inverse_inertia_world * body.torque;

        // Update position and orientation
        body.position += body.linear_velocity * dt + *prev_acceleration * (0.5 * dt * dt);

        // Update orientation (simplified)
        let omega_quat = Quaternion::new(
            0.0,
            body.angular_velocity.x,
            body.angular_velocity.y,
            body.angular_velocity.z,
        );
        let orientation_quat = body.orientation.quaternion();
        let dq = (orientation_quat * omega_quat) * 0.5;
        let new_quat = Quaternion::new(
            orientation_quat.w + dq.w * dt,
            orientation_quat.i + dq.i * dt,
            orientation_quat.j + dq.j * dt,
            orientation_quat.k + dq.k * dt,
        );
        body.orientation = UnitQuaternion::from_quaternion(new_quat);

        // Update velocities with average acceleration
        body.linear_velocity += (*prev_acceleration + accel) * (0.5 * dt);
        body.angular_velocity += (*prev_angular_accel + angular_accel) * (0.5 * dt);

        // Store current acceleration for next step
        *prev_acceleration = accel;
        *prev_angular_accel = angular_accel;

        body.clear_forces();
    }

    /// Applies an impulse at a point (for collision response).
    ///
    /// Δv = J / m
    /// Δω = I^(-1) * (r × J)
    pub fn apply_impulse(body: &mut RigidBody, impulse: Vector3<f64>, point: Vector3<f64>) {
        if body.is_static {
            return;
        }

        // Linear impulse
        body.linear_velocity += impulse * body.mass_props.inverse_mass;

        // Angular impulse
        let r = point - body.position;
        let angular_impulse = r.cross(&impulse);
        let inverse_inertia_world = body.world_inverse_inertia_tensor();
        body.angular_velocity += inverse_inertia_world * angular_impulse;

        body.wake();
    }

    /// Applies angular impulse directly.
    pub fn apply_angular_impulse(body: &mut RigidBody, angular_impulse: Vector3<f64>) {
        if body.is_static {
            return;
        }

        let inverse_inertia_world = body.world_inverse_inertia_tensor();
        body.angular_velocity += inverse_inertia_world * angular_impulse;

        body.wake();
    }
}

/// Computes the effective mass for a constraint between two bodies.
///
/// This is used in constraint solvers for computing constraint forces.
/// K = 1 / (1/m1 + 1/m2 + (r1 × n)^T I1^(-1) (r1 × n) + (r2 × n)^T I2^(-1) (r2 × n))
pub fn compute_effective_mass(
    body1: &RigidBody,
    body2: &RigidBody,
    r1: Vector3<f64>,
    r2: Vector3<f64>,
    normal: Vector3<f64>,
) -> f64 {
    let inv_mass_sum = body1.mass_props.inverse_mass + body2.mass_props.inverse_mass;

    let r1_cross_n = r1.cross(&normal);
    let r2_cross_n = r2.cross(&normal);

    let inv_inertia1 = body1.world_inverse_inertia_tensor();
    let inv_inertia2 = body2.world_inverse_inertia_tensor();

    let angular_factor1 = r1_cross_n.dot(&(inv_inertia1 * r1_cross_n));
    let angular_factor2 = r2_cross_n.dot(&(inv_inertia2 * r2_cross_n));

    let denominator = inv_mass_sum + angular_factor1 + angular_factor2;

    if denominator > 1e-10 {
        1.0 / denominator
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_mass_properties_box() {
        let props = MassProperties::from_box(1000.0, Vector3::new(2.0, 1.0, 4.0));
        assert_eq!(props.mass, 1000.0);
        assert_relative_eq!(props.inverse_mass, 0.001);
        assert!(props.inertia_tensor[(0, 0)] > 0.0);
    }

    #[test]
    fn test_mass_properties_sphere() {
        let props = MassProperties::from_sphere(500.0, 1.5);
        let expected_inertia = 0.4 * 500.0 * 1.5 * 1.5;
        assert_relative_eq!(props.inertia_tensor[(0, 0)], expected_inertia);
        assert_relative_eq!(props.inertia_tensor[(1, 1)], expected_inertia);
        assert_relative_eq!(props.inertia_tensor[(2, 2)], expected_inertia);
    }

    #[test]
    fn test_semi_implicit_euler() {
        let mass_props = MassProperties::from_box(1000.0, Vector3::new(1.0, 1.0, 1.0));
        let mut body = RigidBody::new(0, mass_props);

        let gravity = Vector3::new(0.0, 0.0, -9.81);
        let dt = 0.01;

        let initial_z = body.position.z;
        RigidBodyIntegrator::semi_implicit_euler(&mut body, dt, gravity);

        // After one step, velocity should be v = g*dt
        assert_relative_eq!(body.linear_velocity.z, gravity.z * dt, epsilon = 1e-6);

        // Position should change
        assert!(body.position.z < initial_z);
    }

    #[test]
    fn test_apply_impulse() {
        let mass_props = MassProperties::from_box(1000.0, Vector3::new(1.0, 1.0, 1.0));
        let mut body = RigidBody::new(0, mass_props);

        let impulse = Vector3::new(1000.0, 0.0, 0.0);
        let point = body.position;

        RigidBodyIntegrator::apply_impulse(&mut body, impulse, point);

        // Δv = J / m = 1000 / 1000 = 1.0 m/s
        assert_relative_eq!(body.linear_velocity.x, 1.0);
    }
}
