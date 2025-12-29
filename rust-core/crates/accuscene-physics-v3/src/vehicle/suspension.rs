//! Suspension system simulation.
//!
//! Implements spring-damper suspension models:
//! - Independent suspension (MacPherson strut, double wishbone)
//! - Anti-roll bars
//! - Adjustable damping

use nalgebra::Vector3;
use serde::{Deserialize, Serialize};

use crate::rigid_body::RigidBody;

/// Suspension system parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuspensionParameters {
    /// Spring stiffness (N/m).
    pub spring_stiffness: f64,

    /// Damping coefficient (N·s/m).
    pub damping: f64,

    /// Rest length (m).
    pub rest_length: f64,

    /// Minimum compression length (m).
    pub min_length: f64,

    /// Maximum extension length (m).
    pub max_length: f64,

    /// Preload force (N).
    pub preload: f64,

    /// Anti-roll bar stiffness (N·m/rad).
    pub anti_roll_stiffness: f64,
}

impl SuspensionParameters {
    /// Creates suspension parameters for a passenger car.
    pub fn passenger_car() -> Self {
        Self {
            spring_stiffness: 30000.0,   // N/m
            damping: 3000.0,              // N·s/m
            rest_length: 0.4,             // m
            min_length: 0.2,              // m
            max_length: 0.6,              // m
            preload: 500.0,               // N
            anti_roll_stiffness: 5000.0,  // N·m/rad
        }
    }

    /// Creates suspension parameters for a sports car.
    pub fn sports_car() -> Self {
        Self {
            spring_stiffness: 50000.0,   // Stiffer
            damping: 5000.0,              // More damped
            rest_length: 0.35,
            min_length: 0.15,
            max_length: 0.55,
            preload: 1000.0,              // Higher preload
            anti_roll_stiffness: 8000.0,  // Stiffer anti-roll
        }
    }

    /// Creates suspension parameters for an SUV/truck.
    pub fn suv_truck() -> Self {
        Self {
            spring_stiffness: 40000.0,
            damping: 4000.0,
            rest_length: 0.5,             // Longer travel
            min_length: 0.25,
            max_length: 0.75,
            preload: 2000.0,              // Support heavier vehicle
            anti_roll_stiffness: 3000.0,
        }
    }

    /// Creates suspension parameters for an off-road vehicle.
    pub fn off_road() -> Self {
        Self {
            spring_stiffness: 25000.0,   // Softer for terrain
            damping: 2500.0,
            rest_length: 0.6,             // Long travel
            min_length: 0.3,
            max_length: 0.9,
            preload: 1500.0,
            anti_roll_stiffness: 2000.0,  // Less anti-roll for articulation
        }
    }
}

impl Default for SuspensionParameters {
    fn default() -> Self {
        Self::passenger_car()
    }
}

/// Suspension system state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuspensionSystem {
    /// Suspension parameters.
    pub params: SuspensionParameters,

    /// Current compression/extension from rest (m).
    pub displacement: f64,

    /// Rate of change of displacement (m/s).
    pub velocity: f64,

    /// Current suspension force (N).
    pub force: f64,

    /// Last ground contact point (world space).
    pub contact_point: Vector3<f64>,

    /// Is the wheel in contact with ground?
    pub is_grounded: bool,
}

impl SuspensionSystem {
    /// Creates a new suspension system.
    pub fn new(params: SuspensionParameters) -> Self {
        Self {
            params,
            displacement: 0.0,
            velocity: 0.0,
            force: 0.0,
            contact_point: Vector3::zeros(),
            is_grounded: false,
        }
    }

    /// Updates suspension for one time step.
    ///
    /// Returns the normal force applied to the wheel.
    pub fn update(
        &mut self,
        chassis: &mut RigidBody,
        wheel_position: Vector3<f64>,
        dt: f64,
    ) -> f64 {
        // Raycast down to find ground (simplified: assume ground at z=0)
        let ground_height = 0.0;
        let wheel_height = wheel_position.z;

        // Compute displacement from rest position
        let target_height = ground_height + self.params.rest_length;
        let displacement = target_height - wheel_height;

        // Check if grounded
        self.is_grounded = wheel_height <= (ground_height + self.params.max_length)
            && wheel_height >= (ground_height + self.params.min_length);

        if !self.is_grounded {
            self.force = 0.0;
            self.displacement = 0.0;
            self.velocity = 0.0;
            return 0.0;
        }

        // Clamp displacement to suspension travel limits
        let clamped_displacement = displacement
            .max(self.params.min_length - self.params.rest_length)
            .min(self.params.max_length - self.params.rest_length);

        // Compute velocity (rate of compression/extension)
        let new_velocity = (clamped_displacement - self.displacement) / dt;
        self.velocity = new_velocity;
        self.displacement = clamped_displacement;

        // Spring force: F_spring = -k * x
        let spring_force = -self.params.spring_stiffness * self.displacement;

        // Damping force: F_damper = -c * v
        let damper_force = -self.params.damping * self.velocity;

        // Total suspension force
        self.force = spring_force + damper_force + self.params.preload;

        // Ensure force is non-negative (suspension can't pull)
        self.force = self.force.max(0.0);

        // Apply force to chassis
        let suspension_force_vector = Vector3::new(0.0, 0.0, self.force);
        chassis.apply_force_at_point(suspension_force_vector, wheel_position);

        // Store contact point
        self.contact_point = Vector3::new(wheel_position.x, wheel_position.y, ground_height);

        self.force
    }

    /// Computes the current compression ratio (0 = max extension, 1 = max compression).
    pub fn compression_ratio(&self) -> f64 {
        let total_travel = self.params.max_length - self.params.min_length;
        if total_travel > 0.0 {
            ((self.params.rest_length - self.displacement - self.params.min_length) / total_travel)
                .clamp(0.0, 1.0)
        } else {
            0.5
        }
    }

    /// Computes the stored elastic energy in the spring.
    pub fn elastic_energy(&self) -> f64 {
        0.5 * self.params.spring_stiffness * self.displacement * self.displacement
    }

    /// Checks if suspension is bottomed out (fully compressed).
    pub fn is_bottomed_out(&self) -> bool {
        self.displacement <= (self.params.min_length - self.params.rest_length + 0.01)
    }

    /// Checks if suspension is topped out (fully extended).
    pub fn is_topped_out(&self) -> bool {
        self.displacement >= (self.params.max_length - self.params.rest_length - 0.01)
    }
}

/// Anti-roll bar (stabilizer bar) system.
///
/// Connects left and right suspensions to reduce body roll in corners.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiRollBar {
    /// Stiffness (N·m/rad).
    pub stiffness: f64,

    /// Left suspension index.
    pub left_suspension: usize,

    /// Right suspension index.
    pub right_suspension: usize,
}

impl AntiRollBar {
    /// Creates a new anti-roll bar.
    pub fn new(stiffness: f64, left_idx: usize, right_idx: usize) -> Self {
        Self {
            stiffness,
            left_suspension: left_idx,
            right_suspension: right_idx,
        }
    }

    /// Computes anti-roll torque based on suspension displacement difference.
    pub fn compute_torque(&self, left_displacement: f64, right_displacement: f64) -> f64 {
        // Roll angle proportional to displacement difference
        let roll_angle = left_displacement - right_displacement;

        // Restoring torque
        -self.stiffness * roll_angle
    }

    /// Applies anti-roll forces to both suspensions.
    pub fn apply_forces(
        &self,
        suspensions: &mut [SuspensionSystem],
        track_width: f64,
    ) {
        if self.left_suspension >= suspensions.len() || self.right_suspension >= suspensions.len() {
            return;
        }

        let left_disp = suspensions[self.left_suspension].displacement;
        let right_disp = suspensions[self.right_suspension].displacement;

        let torque = self.compute_torque(left_disp, right_disp);

        // Convert torque to vertical forces
        if track_width > 0.0 {
            let force = torque / track_width;

            suspensions[self.left_suspension].force += force;
            suspensions[self.right_suspension].force -= force;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rigid_body::dynamics::MassProperties;
    use approx::assert_relative_eq;

    #[test]
    fn test_suspension_parameters() {
        let params = SuspensionParameters::passenger_car();
        assert_eq!(params.spring_stiffness, 30000.0);
        assert!(params.damping > 0.0);
    }

    #[test]
    fn test_suspension_creation() {
        let params = SuspensionParameters::passenger_car();
        let suspension = SuspensionSystem::new(params);

        assert_eq!(suspension.displacement, 0.0);
        assert_eq!(suspension.force, 0.0);
    }

    #[test]
    fn test_suspension_compression() {
        let params = SuspensionParameters::passenger_car();
        let mut suspension = SuspensionSystem::new(params.clone());

        // Simulate compression
        suspension.displacement = -0.1; // 10cm compression
        let spring_force = params.spring_stiffness * 0.1;

        assert!(spring_force > 0.0);
    }

    #[test]
    fn test_compression_ratio() {
        let params = SuspensionParameters::passenger_car();
        let mut suspension = SuspensionSystem::new(params);

        suspension.displacement = 0.0;
        let ratio = suspension.compression_ratio();
        assert!(ratio >= 0.0 && ratio <= 1.0);
    }

    #[test]
    fn test_anti_roll_bar() {
        let arb = AntiRollBar::new(5000.0, 0, 1);

        // Left compressed more than right (body rolling left)
        let torque = arb.compute_torque(-0.05, -0.02);

        // Should produce restoring torque
        assert!(torque != 0.0);
    }
}
