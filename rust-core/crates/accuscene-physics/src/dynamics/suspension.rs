//! Suspension dynamics modeling.

use serde::{Deserialize, Serialize};

/// Suspension configuration parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuspensionConfig {
    /// Spring stiffness (N/m)
    pub spring_stiffness: f64,
    /// Damping coefficient (N·s/m)
    pub damping_coefficient: f64,
    /// Maximum compression (m)
    pub max_compression: f64,
    /// Maximum extension (m)
    pub max_extension: f64,
    /// Rest length (m)
    pub rest_length: f64,
    /// Anti-roll bar stiffness (N·m/rad)
    pub anti_roll_stiffness: f64,
}

impl SuspensionConfig {
    /// Creates a standard passenger car suspension.
    pub fn standard() -> Self {
        Self {
            spring_stiffness: 25000.0,
            damping_coefficient: 3000.0,
            max_compression: 0.1,
            max_extension: 0.1,
            rest_length: 0.3,
            anti_roll_stiffness: 5000.0,
        }
    }

    /// Creates a sports car suspension (stiffer).
    pub fn sport() -> Self {
        Self {
            spring_stiffness: 35000.0,
            damping_coefficient: 4000.0,
            max_compression: 0.08,
            max_extension: 0.08,
            rest_length: 0.28,
            anti_roll_stiffness: 8000.0,
        }
    }

    /// Creates an SUV suspension (softer, more travel).
    pub fn suv() -> Self {
        Self {
            spring_stiffness: 20000.0,
            damping_coefficient: 2500.0,
            max_compression: 0.15,
            max_extension: 0.15,
            rest_length: 0.35,
            anti_roll_stiffness: 4000.0,
        }
    }
}

impl Default for SuspensionConfig {
    fn default() -> Self {
        Self::standard()
    }
}

/// Current state of a suspension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuspensionState {
    /// Current compression/extension (m) - positive is compression
    pub displacement: f64,
    /// Velocity of suspension movement (m/s)
    pub velocity: f64,
    /// Force being applied by the suspension (N)
    pub force: f64,
    /// Is suspension at maximum compression?
    pub bottomed_out: bool,
    /// Is suspension at maximum extension?
    pub topped_out: bool,
}

impl Default for SuspensionState {
    fn default() -> Self {
        Self {
            displacement: 0.0,
            velocity: 0.0,
            force: 0.0,
            bottomed_out: false,
            topped_out: false,
        }
    }
}

impl SuspensionState {
    /// Calculates the suspension force based on configuration.
    pub fn calculate_force(&mut self, config: &SuspensionConfig) -> f64 {
        // Clamp displacement to limits
        self.displacement = self.displacement.clamp(-config.max_extension, config.max_compression);

        // Check if bottomed out or topped out
        self.bottomed_out = self.displacement >= config.max_compression;
        self.topped_out = self.displacement <= -config.max_extension;

        // Spring force: F = -k * x
        let spring_force = -config.spring_stiffness * self.displacement;

        // Damping force: F = -c * v
        let damping_force = -config.damping_coefficient * self.velocity;

        // Bump stops - very stiff force when at limits
        let bump_stop_force = if self.bottomed_out {
            -config.spring_stiffness * 10.0 * (self.displacement - config.max_compression)
        } else if self.topped_out {
            -config.spring_stiffness * 10.0 * (self.displacement + config.max_extension)
        } else {
            0.0
        };

        self.force = spring_force + damping_force + bump_stop_force;
        self.force
    }

    /// Updates suspension state using simple integration.
    pub fn integrate(&mut self, config: &SuspensionConfig, road_height: f64, dt: f64) {
        // Calculate force
        self.calculate_force(config);

        // Update velocity (simplified - in reality this would be coupled with vehicle mass)
        // For now, we just track the displacement based on road input
        let target_displacement = road_height;
        let displacement_error = target_displacement - self.displacement;

        // Simple proportional velocity control
        self.velocity = displacement_error / dt.max(0.001);

        // Update displacement
        self.displacement += self.velocity * dt;
    }
}

/// Four-corner suspension system for a vehicle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleSuspension {
    /// Front-left suspension
    pub front_left: SuspensionState,
    /// Front-right suspension
    pub front_right: SuspensionState,
    /// Rear-left suspension
    pub rear_left: SuspensionState,
    /// Rear-right suspension
    pub rear_right: SuspensionState,
    /// Suspension configuration
    pub config: SuspensionConfig,
}

impl VehicleSuspension {
    /// Creates a new vehicle suspension system.
    pub fn new(config: SuspensionConfig) -> Self {
        Self {
            front_left: SuspensionState::default(),
            front_right: SuspensionState::default(),
            rear_left: SuspensionState::default(),
            rear_right: SuspensionState::default(),
            config,
        }
    }

    /// Updates all suspensions with road heights.
    pub fn update(
        &mut self,
        road_heights: [f64; 4], // [FL, FR, RL, RR]
        dt: f64,
    ) {
        self.front_left.integrate(&self.config, road_heights[0], dt);
        self.front_right.integrate(&self.config, road_heights[1], dt);
        self.rear_left.integrate(&self.config, road_heights[2], dt);
        self.rear_right.integrate(&self.config, road_heights[3], dt);

        // Apply anti-roll bar effect
        self.apply_anti_roll();
    }

    /// Applies anti-roll bar forces to reduce body roll.
    fn apply_anti_roll(&mut self) {
        // Front anti-roll
        let front_roll = self.front_left.displacement - self.front_right.displacement;
        let front_anti_roll_force = self.config.anti_roll_stiffness * front_roll;

        self.front_left.force -= front_anti_roll_force;
        self.front_right.force += front_anti_roll_force;

        // Rear anti-roll
        let rear_roll = self.rear_left.displacement - self.rear_right.displacement;
        let rear_anti_roll_force = self.config.anti_roll_stiffness * rear_roll;

        self.rear_left.force -= rear_anti_roll_force;
        self.rear_right.force += rear_anti_roll_force;
    }

    /// Returns the total vertical force from all suspensions.
    pub fn total_force(&self) -> f64 {
        self.front_left.force
            + self.front_right.force
            + self.rear_left.force
            + self.rear_right.force
    }

    /// Returns the pitch moment (front-rear difference).
    pub fn pitch_moment(&self, wheelbase: f64) -> f64 {
        let front_force = self.front_left.force + self.front_right.force;
        let rear_force = self.rear_left.force + self.rear_right.force;

        (front_force - rear_force) * wheelbase / 2.0
    }

    /// Returns the roll moment (left-right difference).
    pub fn roll_moment(&self, track_width: f64) -> f64 {
        let left_force = self.front_left.force + self.rear_left.force;
        let right_force = self.front_right.force + self.rear_right.force;

        (left_force - right_force) * track_width / 2.0
    }
}

impl Default for VehicleSuspension {
    fn default() -> Self {
        Self::new(SuspensionConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suspension_config() {
        let standard = SuspensionConfig::standard();
        let sport = SuspensionConfig::sport();
        let suv = SuspensionConfig::suv();

        assert!(sport.spring_stiffness > standard.spring_stiffness);
        assert!(suv.max_compression > sport.max_compression);
    }

    #[test]
    fn test_suspension_force() {
        let config = SuspensionConfig::standard();
        let mut state = SuspensionState::default();

        // Compress the suspension
        state.displacement = 0.05; // 5 cm compression
        state.velocity = 0.0;

        let force = state.calculate_force(&config);

        // Force should oppose compression (negative)
        assert!(force < 0.0);
    }

    #[test]
    fn test_suspension_damping() {
        let config = SuspensionConfig::standard();
        let mut state = SuspensionState::default();

        state.displacement = 0.0;
        state.velocity = 1.0; // Moving at 1 m/s

        let force = state.calculate_force(&config);

        // Damping force should oppose velocity (negative)
        assert!(force < 0.0);
    }

    #[test]
    fn test_vehicle_suspension() {
        let mut suspension = VehicleSuspension::new(SuspensionConfig::standard());

        // Simulate driving over a bump on the left side
        suspension.update([0.05, 0.0, 0.05, 0.0], 0.01);

        // Left side should be compressed more than right
        assert!(suspension.front_left.displacement > suspension.front_right.displacement);
        assert!(suspension.rear_left.displacement > suspension.rear_right.displacement);

        // Anti-roll should create a roll moment
        let roll_moment = suspension.roll_moment(1.5);
        assert!(roll_moment.abs() > 0.0);
    }
}
