//! Rigid body dynamics system.
//!
//! This module implements rigid body physics including:
//! - Mass and inertia tensor calculations
//! - Linear and angular momentum integration
//! - Force and torque accumulation
//! - Constraint systems

pub mod constraints;
pub mod dynamics;

pub use constraints::*;
pub use dynamics::*;

use nalgebra::{Matrix3, UnitQuaternion, Vector3};
use serde::{Deserialize, Serialize};

/// Complete rigid body state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RigidBody {
    /// Unique identifier.
    pub id: usize,

    /// Position in world space (m).
    pub position: Vector3<f64>,

    /// Orientation as unit quaternion.
    pub orientation: UnitQuaternion<f64>,

    /// Linear velocity (m/s).
    pub linear_velocity: Vector3<f64>,

    /// Angular velocity (rad/s) in world space.
    pub angular_velocity: Vector3<f64>,

    /// Mass properties.
    pub mass_props: MassProperties,

    /// Accumulated forces (N).
    pub force: Vector3<f64>,

    /// Accumulated torques (N·m) in world space.
    pub torque: Vector3<f64>,

    /// Is the body static (infinite mass)?
    pub is_static: bool,

    /// Is the body awake (active in simulation)?
    pub is_awake: bool,

    /// Sleep threshold for energy-based sleep.
    pub sleep_threshold: f64,

    /// Time at rest (for sleep detection).
    pub time_at_rest: f64,
}

impl RigidBody {
    /// Creates a new rigid body with given mass properties.
    pub fn new(id: usize, mass_props: MassProperties) -> Self {
        Self {
            id,
            position: Vector3::zeros(),
            orientation: UnitQuaternion::identity(),
            linear_velocity: Vector3::zeros(),
            angular_velocity: Vector3::zeros(),
            mass_props,
            force: Vector3::zeros(),
            torque: Vector3::zeros(),
            is_static: false,
            is_awake: true,
            sleep_threshold: 0.01,
            time_at_rest: 0.0,
        }
    }

    /// Creates a static rigid body (infinite mass).
    pub fn new_static(id: usize) -> Self {
        let mut body = Self::new(id, MassProperties::infinite());
        body.is_static = true;
        body
    }

    /// Applies a force at the center of mass.
    pub fn apply_force(&mut self, force: Vector3<f64>) {
        if !self.is_static {
            self.force += force;
        }
    }

    /// Applies a force at a world-space point.
    ///
    /// Generates both linear force and torque about center of mass.
    pub fn apply_force_at_point(&mut self, force: Vector3<f64>, point: Vector3<f64>) {
        if !self.is_static {
            self.force += force;
            let r = point - self.position;
            self.torque += r.cross(&force);
        }
    }

    /// Applies a torque about the center of mass.
    pub fn apply_torque(&mut self, torque: Vector3<f64>) {
        if !self.is_static {
            self.torque += torque;
        }
    }

    /// Clears all accumulated forces and torques.
    pub fn clear_forces(&mut self) {
        self.force = Vector3::zeros();
        self.torque = Vector3::zeros();
    }

    /// Computes the inertia tensor in world space.
    ///
    /// I_world = R * I_local * R^T
    pub fn world_inertia_tensor(&self) -> Matrix3<f64> {
        let rotation = self.orientation.to_rotation_matrix();
        rotation.matrix() * self.mass_props.inertia_tensor * rotation.matrix().transpose()
    }

    /// Computes the inverse inertia tensor in world space.
    pub fn world_inverse_inertia_tensor(&self) -> Matrix3<f64> {
        if self.is_static {
            Matrix3::zeros()
        } else {
            let rotation = self.orientation.to_rotation_matrix();
            rotation.matrix()
                * self.mass_props.inverse_inertia_tensor
                * rotation.matrix().transpose()
        }
    }

    /// Computes velocity at a world-space point.
    ///
    /// v = v_cm + ω × r
    pub fn velocity_at_point(&self, point: Vector3<f64>) -> Vector3<f64> {
        let r = point - self.position;
        self.linear_velocity + self.angular_velocity.cross(&r)
    }

    /// Computes kinetic energy (J).
    ///
    /// KE = 0.5 * m * v² + 0.5 * ω^T * I * ω
    pub fn kinetic_energy(&self) -> f64 {
        let translational = 0.5 * self.mass_props.mass * self.linear_velocity.norm_squared();
        let inertia_world = self.world_inertia_tensor();
        let rotational = 0.5 * self.angular_velocity.dot(&(inertia_world * self.angular_velocity));
        translational + rotational
    }

    /// Computes linear momentum (kg·m/s).
    pub fn linear_momentum(&self) -> Vector3<f64> {
        self.mass_props.mass * self.linear_velocity
    }

    /// Computes angular momentum in world space (kg·m²/s).
    ///
    /// L = I * ω
    pub fn angular_momentum(&self) -> Vector3<f64> {
        let inertia_world = self.world_inertia_tensor();
        inertia_world * self.angular_velocity
    }

    /// Checks if the body should go to sleep based on kinetic energy.
    pub fn check_sleep(&mut self, dt: f64) {
        if self.is_static {
            return;
        }

        let ke = self.kinetic_energy();
        if ke < self.sleep_threshold {
            self.time_at_rest += dt;
            if self.time_at_rest > 0.5 {
                // 0.5 seconds at rest
                self.is_awake = false;
                self.linear_velocity = Vector3::zeros();
                self.angular_velocity = Vector3::zeros();
            }
        } else {
            self.time_at_rest = 0.0;
            self.is_awake = true;
        }
    }

    /// Wakes up the body.
    pub fn wake(&mut self) {
        self.is_awake = true;
        self.time_at_rest = 0.0;
    }

    /// Transforms a local point to world space.
    pub fn local_to_world(&self, local_point: Vector3<f64>) -> Vector3<f64> {
        self.position + self.orientation * local_point
    }

    /// Transforms a world point to local space.
    pub fn world_to_local(&self, world_point: Vector3<f64>) -> Vector3<f64> {
        self.orientation.inverse() * (world_point - self.position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_rigid_body_creation() {
        let mass_props = MassProperties::from_box(1000.0, Vector3::new(2.0, 1.0, 4.0));
        let body = RigidBody::new(0, mass_props);
        assert_eq!(body.id, 0);
        assert!(body.is_awake);
        assert!(!body.is_static);
    }

    #[test]
    fn test_apply_force() {
        let mass_props = MassProperties::from_box(1000.0, Vector3::new(1.0, 1.0, 1.0));
        let mut body = RigidBody::new(0, mass_props);

        body.apply_force(Vector3::new(100.0, 0.0, 0.0));
        assert_relative_eq!(body.force.x, 100.0);
    }

    #[test]
    fn test_kinetic_energy() {
        let mass_props = MassProperties::from_box(1000.0, Vector3::new(1.0, 1.0, 1.0));
        let mut body = RigidBody::new(0, mass_props);

        body.linear_velocity = Vector3::new(10.0, 0.0, 0.0);
        let ke = body.kinetic_energy();
        let expected = 0.5 * 1000.0 * 100.0; // 0.5 * m * v²
        assert_relative_eq!(ke, expected);
    }
}
