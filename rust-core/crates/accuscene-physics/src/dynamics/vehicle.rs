//! Vehicle dynamics modeling.

use nalgebra::{Matrix3, Point3, UnitQuaternion, Vector3};
use serde::{Deserialize, Serialize};

/// Complete vehicle dynamic state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleState {
    /// Position in world space
    pub position: Point3<f64>,
    /// Orientation quaternion
    pub orientation: UnitQuaternion<f64>,
    /// Linear velocity (m/s)
    pub velocity: Vector3<f64>,
    /// Angular velocity (rad/s)
    pub angular_velocity: Vector3<f64>,
    /// Linear acceleration (m/s²)
    pub acceleration: Vector3<f64>,
    /// Angular acceleration (rad/s²)
    pub angular_acceleration: Vector3<f64>,
}

impl Default for VehicleState {
    fn default() -> Self {
        Self {
            position: Point3::origin(),
            orientation: UnitQuaternion::identity(),
            velocity: Vector3::zeros(),
            angular_velocity: Vector3::zeros(),
            acceleration: Vector3::zeros(),
            angular_acceleration: Vector3::zeros(),
        }
    }
}

/// Vehicle dynamics properties and simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleDynamics {
    /// Mass in kg
    pub mass: f64,
    /// Moment of inertia tensor (kg·m²)
    pub inertia_tensor: Matrix3<f64>,
    /// Center of gravity position relative to vehicle origin
    pub center_of_gravity: Vector3<f64>,
    /// Wheelbase (distance between front and rear axles) in meters
    pub wheelbase: f64,
    /// Track width (distance between left and right wheels) in meters
    pub track_width: f64,
    /// Vehicle length in meters
    pub length: f64,
    /// Vehicle width in meters
    pub width: f64,
    /// Vehicle height in meters
    pub height: f64,
    /// Frontal area for drag calculation (m²)
    pub frontal_area: f64,
    /// Drag coefficient
    pub drag_coefficient: f64,
}

impl VehicleDynamics {
    /// Creates a new vehicle with specified parameters.
    pub fn new(mass: f64, wheelbase: f64, track_width: f64) -> Self {
        // Estimate inertia tensor for a box-shaped vehicle
        let length = wheelbase * 1.5;
        let width = track_width;
        let height = 1.5;

        let ixx = (1.0 / 12.0) * mass * (width * width + height * height);
        let iyy = (1.0 / 12.0) * mass * (length * length + height * height);
        let izz = (1.0 / 12.0) * mass * (length * length + width * width);

        Self {
            mass,
            inertia_tensor: Matrix3::from_diagonal(&Vector3::new(ixx, iyy, izz)),
            center_of_gravity: Vector3::new(0.0, 0.0, -0.5), // Slightly below origin
            wheelbase,
            track_width,
            length,
            width,
            height,
            frontal_area: width * height * 0.85, // Approximate frontal area
            drag_coefficient: 0.35,               // Typical for cars
        }
    }

    /// Creates a standard sedan vehicle.
    pub fn sedan() -> Self {
        Self::new(1500.0, 2.7, 1.5)
    }

    /// Creates a standard SUV vehicle.
    pub fn suv() -> Self {
        let mut vehicle = Self::new(2000.0, 2.85, 1.65);
        vehicle.height = 1.8;
        vehicle.drag_coefficient = 0.40;
        vehicle.frontal_area = vehicle.width * vehicle.height * 0.85;
        vehicle
    }

    /// Creates a pickup truck vehicle.
    pub fn pickup_truck() -> Self {
        let mut vehicle = Self::new(2200.0, 3.5, 1.7);
        vehicle.height = 1.9;
        vehicle.drag_coefficient = 0.45;
        vehicle.frontal_area = vehicle.width * vehicle.height * 0.85;
        vehicle
    }

    /// Computes forces acting on the vehicle.
    pub fn compute_forces(
        &self,
        state: &VehicleState,
        engine_force: f64,
        brake_force: f64,
        _steering_angle: f64,
    ) -> Vector3<f64> {
        let mut total_force = Vector3::zeros();

        // Forward direction in world space
        let forward = state.orientation * Vector3::new(1.0, 0.0, 0.0);

        // Engine force
        total_force += forward * engine_force;

        // Brake force (opposes velocity)
        if state.velocity.norm() > 0.01 {
            let brake_direction = -state.velocity.normalize();
            total_force += brake_direction * brake_force;
        }

        // Aerodynamic drag: F_drag = 0.5 * ρ * v² * Cd * A
        let air_density = 1.225; // kg/m³ at sea level
        let speed = state.velocity.norm();
        let drag_force = 0.5 * air_density * speed * speed * self.drag_coefficient * self.frontal_area;
        if speed > 0.01 {
            total_force -= state.velocity.normalize() * drag_force;
        }

        // Gravity (if we're not on flat ground)
        // For now, assuming flat ground, so gravity is balanced by normal force

        total_force
    }

    /// Computes torques acting on the vehicle.
    pub fn compute_torques(
        &self,
        state: &VehicleState,
        steering_angle: f64,
    ) -> Vector3<f64> {
        let mut total_torque = Vector3::zeros();

        // Steering torque around Z-axis
        let speed = state.velocity.norm();
        if speed > 0.1 {
            // Simplified steering torque model
            let steering_torque_magnitude = steering_angle * speed * 1000.0;
            total_torque.z += steering_torque_magnitude;
        }

        total_torque
    }

    /// Updates vehicle state using Euler integration.
    pub fn integrate(
        &self,
        state: &mut VehicleState,
        forces: Vector3<f64>,
        torques: Vector3<f64>,
        dt: f64,
    ) {
        // Linear dynamics: F = ma
        state.acceleration = forces / self.mass;
        state.velocity += state.acceleration * dt;
        state.position += state.velocity * dt;

        // Angular dynamics: τ = I·α
        let inv_inertia = self
            .inertia_tensor
            .try_inverse()
            .unwrap_or(Matrix3::zeros());
        state.angular_acceleration = inv_inertia * torques;
        state.angular_velocity += state.angular_acceleration * dt;

        // Update orientation
        if state.angular_velocity.norm() > 1e-6 {
            let rotation_axis = state.angular_velocity.normalize();
            let rotation_angle = state.angular_velocity.norm() * dt;
            let delta_rotation = UnitQuaternion::from_axis_angle(
                &nalgebra::Unit::new_normalize(rotation_axis),
                rotation_angle,
            );
            state.orientation = delta_rotation * state.orientation;
            state.orientation = state.orientation.normalize();
        }
    }

    /// Calculates kinetic energy of the vehicle.
    pub fn kinetic_energy(&self, state: &VehicleState) -> f64 {
        // Translational kinetic energy
        let translational = 0.5 * self.mass * state.velocity.norm_squared();

        // Rotational kinetic energy
        let rotational = 0.5
            * (state.angular_velocity.transpose() * self.inertia_tensor * state.angular_velocity)[0];

        translational + rotational
    }

    /// Calculates momentum of the vehicle.
    pub fn momentum(&self, state: &VehicleState) -> Vector3<f64> {
        self.mass * state.velocity
    }

    /// Calculates angular momentum of the vehicle.
    pub fn angular_momentum(&self, state: &VehicleState) -> Vector3<f64> {
        self.inertia_tensor * state.angular_velocity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vehicle_creation() {
        let sedan = VehicleDynamics::sedan();
        assert_eq!(sedan.mass, 1500.0);
        assert_eq!(sedan.wheelbase, 2.7);

        let suv = VehicleDynamics::suv();
        assert_eq!(suv.mass, 2000.0);
        assert!(suv.height > sedan.height);
    }

    #[test]
    fn test_force_computation() {
        let vehicle = VehicleDynamics::sedan();
        let state = VehicleState::default();

        let forces = vehicle.compute_forces(&state, 5000.0, 0.0, 0.0);
        assert!(forces.x > 0.0); // Engine force in forward direction
    }

    #[test]
    fn test_integration() {
        let vehicle = VehicleDynamics::sedan();
        let mut state = VehicleState::default();

        let forces = Vector3::new(5000.0, 0.0, 0.0);
        let torques = Vector3::zeros();

        vehicle.integrate(&mut state, forces, torques, 0.01);

        // Vehicle should have gained velocity
        assert!(state.velocity.norm() > 0.0);
        assert!(state.position.coords.norm() > 0.0);
    }

    #[test]
    fn test_energy_calculations() {
        let vehicle = VehicleDynamics::sedan();
        let mut state = VehicleState::default();
        state.velocity = Vector3::new(20.0, 0.0, 0.0); // 20 m/s ≈ 72 km/h

        let ke = vehicle.kinetic_energy(&state);
        let expected_ke = 0.5 * vehicle.mass * 20.0 * 20.0;
        assert!((ke - expected_ke).abs() < 1.0);

        let momentum = vehicle.momentum(&state);
        assert_eq!(momentum, vehicle.mass * state.velocity);
    }
}
