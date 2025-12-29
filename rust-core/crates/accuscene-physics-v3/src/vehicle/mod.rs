//! Vehicle-specific physics models.
//!
//! This module implements specialized vehicle dynamics:
//! - Tire models (Pacejka Magic Formula)
//! - Suspension systems (spring-damper)
//! - Powertrain dynamics (engine, transmission, drivetrain)

pub mod powertrain;
pub mod suspension;
pub mod tire_model;

pub use powertrain::*;
pub use suspension::*;
pub use tire_model::*;

use nalgebra::Vector3;
use serde::{Deserialize, Serialize};

use crate::rigid_body::RigidBody;

/// Complete vehicle dynamics model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vehicle {
    /// Unique identifier.
    pub id: usize,

    /// Main chassis rigid body.
    pub chassis: usize, // Body ID

    /// Wheels (tire models).
    pub wheels: Vec<Wheel>,

    /// Suspension systems.
    pub suspensions: Vec<SuspensionSystem>,

    /// Powertrain.
    pub powertrain: Powertrain,

    /// Wheelbase (m).
    pub wheelbase: f64,

    /// Track width (m).
    pub track_width: f64,

    /// Center of gravity height (m).
    pub cg_height: f64,
}

impl Vehicle {
    /// Creates a new vehicle.
    pub fn new(
        id: usize,
        chassis_body_id: usize,
        wheelbase: f64,
        track_width: f64,
        cg_height: f64,
    ) -> Self {
        Self {
            id,
            chassis: chassis_body_id,
            wheels: Vec::new(),
            suspensions: Vec::new(),
            powertrain: Powertrain::default(),
            wheelbase,
            track_width,
            cg_height,
        }
    }

    /// Adds a wheel to the vehicle.
    pub fn add_wheel(&mut self, wheel: Wheel, suspension: SuspensionSystem) {
        self.wheels.push(wheel);
        self.suspensions.push(suspension);
    }

    /// Creates a typical 4-wheel passenger vehicle.
    pub fn create_passenger_car(
        id: usize,
        chassis_body_id: usize,
    ) -> Self {
        let mut vehicle = Self::new(id, chassis_body_id, 2.7, 1.5, 0.5);

        // Front wheels
        let tire_params = TireParameters::passenger_car();
        let wheel_fl = Wheel::new(
            Vector3::new(vehicle.wheelbase / 2.0, vehicle.track_width / 2.0, 0.0),
            tire_params.clone(),
        );
        let wheel_fr = Wheel::new(
            Vector3::new(vehicle.wheelbase / 2.0, -vehicle.track_width / 2.0, 0.0),
            tire_params.clone(),
        );

        // Rear wheels
        let wheel_rl = Wheel::new(
            Vector3::new(-vehicle.wheelbase / 2.0, vehicle.track_width / 2.0, 0.0),
            tire_params.clone(),
        );
        let wheel_rr = Wheel::new(
            Vector3::new(-vehicle.wheelbase / 2.0, -vehicle.track_width / 2.0, 0.0),
            tire_params,
        );

        // Suspensions
        let suspension_params = SuspensionParameters::passenger_car();
        vehicle.add_wheel(wheel_fl, SuspensionSystem::new(suspension_params.clone()));
        vehicle.add_wheel(wheel_fr, SuspensionSystem::new(suspension_params.clone()));
        vehicle.add_wheel(wheel_rl, SuspensionSystem::new(suspension_params.clone()));
        vehicle.add_wheel(wheel_rr, SuspensionSystem::new(suspension_params));

        // Powertrain
        vehicle.powertrain = Powertrain::passenger_car();

        vehicle
    }

    /// Updates vehicle dynamics for one time step.
    pub fn update(
        &mut self,
        chassis_body: &mut RigidBody,
        dt: f64,
        throttle: f64,
        brake: f64,
        steering_angle: f64,
    ) {
        // Update powertrain
        let drive_torque = self.powertrain.update(dt, throttle, brake);

        // Update each wheel
        for (i, wheel) in self.wheels.iter_mut().enumerate() {
            // Determine if this is a driven wheel
            let is_driven = match self.powertrain.drivetrain {
                DrivetrainType::FrontWheelDrive => i < 2,
                DrivetrainType::RearWheelDrive => i >= 2,
                DrivetrainType::AllWheelDrive => true,
            };

            let wheel_torque = if is_driven {
                drive_torque / self.powertrain.num_driven_wheels() as f64
            } else {
                0.0
            };

            // Apply steering (front wheels only)
            let steer_angle = if i < 2 { steering_angle } else { 0.0 };

            // Update wheel
            wheel.update(chassis_body, &mut self.suspensions[i], dt, wheel_torque, steer_angle, brake);
        }
    }

    /// Computes total downforce on the vehicle.
    pub fn downforce(&self) -> f64 {
        self.wheels.iter().map(|w| w.normal_force).sum()
    }
}

/// Individual wheel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wheel {
    /// Position relative to chassis center.
    pub position: Vector3<f64>,

    /// Tire parameters.
    pub tire: TireParameters,

    /// Current rotation angle (rad).
    pub rotation: f64,

    /// Angular velocity (rad/s).
    pub angular_velocity: f64,

    /// Normal force (N).
    pub normal_force: f64,

    /// Slip ratio.
    pub slip_ratio: f64,

    /// Slip angle (rad).
    pub slip_angle: f64,

    /// Tire forces in local coordinates.
    pub force_longitudinal: f64,
    pub force_lateral: f64,
}

impl Wheel {
    /// Creates a new wheel.
    pub fn new(position: Vector3<f64>, tire: TireParameters) -> Self {
        Self {
            position,
            tire,
            rotation: 0.0,
            angular_velocity: 0.0,
            normal_force: 0.0,
            slip_ratio: 0.0,
            slip_angle: 0.0,
            force_longitudinal: 0.0,
            force_lateral: 0.0,
        }
    }

    /// Updates wheel dynamics.
    pub fn update(
        &mut self,
        chassis: &mut RigidBody,
        suspension: &mut SuspensionSystem,
        dt: f64,
        drive_torque: f64,
        steer_angle: f64,
        brake_torque: f64,
    ) {
        // Get wheel position in world space
        let wheel_pos_world = chassis.local_to_world(self.position);

        // Update suspension (computes normal force)
        self.normal_force = suspension.update(chassis, wheel_pos_world, dt);

        // Get velocity at wheel contact point
        let wheel_velocity = chassis.velocity_at_point(wheel_pos_world);

        // Compute slip ratio and slip angle
        let longitudinal_velocity = wheel_velocity.x; // Simplified
        let lateral_velocity = wheel_velocity.y;

        let wheel_linear_velocity = self.angular_velocity * self.tire.radius;

        if longitudinal_velocity.abs() > 0.1 {
            self.slip_ratio = (wheel_linear_velocity - longitudinal_velocity) / longitudinal_velocity.abs();
        } else {
            self.slip_ratio = 0.0;
        }

        if longitudinal_velocity.abs() > 0.1 {
            self.slip_angle = (-lateral_velocity / longitudinal_velocity.abs()).atan();
        } else {
            self.slip_angle = 0.0;
        }

        // Compute tire forces using Pacejka model
        self.force_longitudinal = self.tire.longitudinal_force(self.slip_ratio, self.normal_force);
        self.force_lateral = self.tire.lateral_force(self.slip_angle, self.normal_force);

        // Update wheel rotation
        let net_torque = drive_torque - brake_torque - self.force_longitudinal * self.tire.radius;
        let wheel_inertia = 0.5 * self.tire.mass * self.tire.radius * self.tire.radius;

        self.angular_velocity += (net_torque / wheel_inertia) * dt;
        self.rotation += self.angular_velocity * dt;

        // Apply forces to chassis
        // Transform forces to world space (considering steering)
        let cos_steer = steer_angle.cos();
        let sin_steer = steer_angle.sin();

        let force_x = self.force_longitudinal * cos_steer - self.force_lateral * sin_steer;
        let force_y = self.force_longitudinal * sin_steer + self.force_lateral * cos_steer;
        let force_z = self.normal_force;

        let tire_force = Vector3::new(force_x, force_y, force_z);

        chassis.apply_force_at_point(tire_force, wheel_pos_world);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vehicle_creation() {
        let vehicle = Vehicle::create_passenger_car(0, 100);

        assert_eq!(vehicle.wheels.len(), 4);
        assert_eq!(vehicle.suspensions.len(), 4);
        assert!(vehicle.wheelbase > 0.0);
    }

    #[test]
    fn test_wheel_creation() {
        let tire = TireParameters::passenger_car();
        let wheel = Wheel::new(Vector3::new(1.0, 0.5, 0.0), tire);

        assert_eq!(wheel.rotation, 0.0);
        assert_eq!(wheel.angular_velocity, 0.0);
    }
}
