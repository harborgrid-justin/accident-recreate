//! Powertrain dynamics simulation.
//!
//! Implements:
//! - Engine torque curves
//! - Transmission (manual/automatic)
//! - Drivetrain (FWD/RWD/AWD)
//! - Braking system

use serde::{Deserialize, Serialize};

/// Complete powertrain system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Powertrain {
    /// Engine model.
    pub engine: Engine,

    /// Transmission model.
    pub transmission: Transmission,

    /// Drivetrain type.
    pub drivetrain: DrivetrainType,

    /// Differential ratio.
    pub differential_ratio: f64,

    /// Braking system.
    pub brakes: BrakingSystem,

    /// Current throttle position (0-1).
    pub throttle: f64,

    /// Current brake pressure (0-1).
    pub brake_pressure: f64,
}

impl Powertrain {
    /// Creates a new powertrain.
    pub fn new(
        engine: Engine,
        transmission: Transmission,
        drivetrain: DrivetrainType,
        differential_ratio: f64,
    ) -> Self {
        Self {
            engine,
            transmission,
            drivetrain,
            differential_ratio,
            brakes: BrakingSystem::default(),
            throttle: 0.0,
            brake_pressure: 0.0,
        }
    }

    /// Creates a typical passenger car powertrain.
    pub fn passenger_car() -> Self {
        Self::new(
            Engine::passenger_car(),
            Transmission::automatic_5speed(),
            DrivetrainType::FrontWheelDrive,
            3.7, // Final drive ratio
        )
    }

    /// Creates a sports car powertrain.
    pub fn sports_car() -> Self {
        Self::new(
            Engine::sports_car(),
            Transmission::manual_6speed(),
            DrivetrainType::RearWheelDrive,
            3.4,
        )
    }

    /// Creates an SUV powertrain.
    pub fn suv() -> Self {
        Self::new(
            Engine::suv(),
            Transmission::automatic_8speed(),
            DrivetrainType::AllWheelDrive,
            3.9,
        )
    }

    /// Updates powertrain for one time step.
    ///
    /// Returns the total drive torque to be distributed to wheels.
    pub fn update(&mut self, dt: f64, throttle: f64, brake: f64) -> f64 {
        self.throttle = throttle.clamp(0.0, 1.0);
        self.brake_pressure = brake.clamp(0.0, 1.0);

        // Update engine
        let engine_torque = self.engine.update(dt, self.throttle);

        // Update transmission
        let transmission_output = self.transmission.update(dt, self.engine.rpm);

        // Total gear reduction
        let total_ratio = transmission_output.gear_ratio * self.differential_ratio;

        // Output torque at wheels
        let wheel_torque = engine_torque * total_ratio * transmission_output.efficiency;

        wheel_torque
    }

    /// Returns the number of driven wheels.
    pub fn num_driven_wheels(&self) -> usize {
        match self.drivetrain {
            DrivetrainType::FrontWheelDrive | DrivetrainType::RearWheelDrive => 2,
            DrivetrainType::AllWheelDrive => 4,
        }
    }

    /// Computes brake torque per wheel.
    pub fn brake_torque_per_wheel(&self) -> f64 {
        self.brakes.max_brake_torque * self.brake_pressure / 4.0 // Distribute to all wheels
    }
}

impl Default for Powertrain {
    fn default() -> Self {
        Self::passenger_car()
    }
}

/// Engine model with torque curve.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Engine {
    /// Maximum power (W).
    pub max_power: f64,

    /// RPM at maximum power.
    pub max_power_rpm: f64,

    /// Maximum torque (N·m).
    pub max_torque: f64,

    /// RPM at maximum torque.
    pub max_torque_rpm: f64,

    /// Idle RPM.
    pub idle_rpm: f64,

    /// Redline RPM.
    pub redline_rpm: f64,

    /// Current RPM.
    pub rpm: f64,

    /// Engine inertia (kg·m²).
    pub inertia: f64,
}

impl Engine {
    /// Creates a new engine.
    pub fn new(
        max_power: f64,
        max_power_rpm: f64,
        max_torque: f64,
        max_torque_rpm: f64,
        idle_rpm: f64,
        redline_rpm: f64,
    ) -> Self {
        Self {
            max_power,
            max_power_rpm,
            max_torque,
            max_torque_rpm,
            idle_rpm,
            redline_rpm,
            rpm: idle_rpm,
            inertia: 0.15, // Typical engine inertia
        }
    }

    /// Creates a passenger car engine (2.0L, 150 hp).
    pub fn passenger_car() -> Self {
        Self::new(
            112000.0,  // 150 hp = 112 kW
            5500.0,    // Peak power RPM
            250.0,     // N·m
            3500.0,    // Peak torque RPM
            800.0,     // Idle
            6500.0,    // Redline
        )
    }

    /// Creates a sports car engine (3.5L, 300 hp).
    pub fn sports_car() -> Self {
        Self::new(
            224000.0,  // 300 hp
            6500.0,
            420.0,     // N·m
            4500.0,
            900.0,
            7500.0,
        )
    }

    /// Creates an SUV engine (3.0L turbo, 250 hp).
    pub fn suv() -> Self {
        Self::new(
            186000.0,  // 250 hp
            5000.0,
            500.0,     // High torque for towing
            2500.0,    // Low-end torque
            700.0,
            6000.0,
        )
    }

    /// Updates engine for one time step.
    pub fn update(&mut self, _dt: f64, throttle: f64) -> f64 {
        // Simplified: assume RPM tracks transmission demands
        // In full simulation, would integrate angular acceleration

        let torque = self.torque_at_rpm(self.rpm) * throttle;

        torque
    }

    /// Computes engine torque at a given RPM using a simplified curve.
    ///
    /// Uses a polynomial fit to create realistic torque curve.
    pub fn torque_at_rpm(&self, rpm: f64) -> f64 {
        if rpm < self.idle_rpm || rpm > self.redline_rpm {
            return 0.0;
        }

        // Normalize RPM to 0-1 range
        let rpm_normalized = (rpm - self.idle_rpm) / (self.redline_rpm - self.idle_rpm);

        // Create torque curve with peak at max_torque_rpm
        let peak_pos = (self.max_torque_rpm - self.idle_rpm) / (self.redline_rpm - self.idle_rpm);

        let torque_factor = if rpm_normalized < peak_pos {
            // Rising to peak
            rpm_normalized / peak_pos
        } else {
            // Falling after peak
            1.0 - (rpm_normalized - peak_pos) / (1.0 - peak_pos) * 0.3
        };

        self.max_torque * torque_factor.clamp(0.0, 1.0)
    }

    /// Computes power at current RPM.
    ///
    /// P = τ * ω = τ * (RPM * 2π / 60)
    pub fn power_at_rpm(&self, rpm: f64) -> f64 {
        let torque = self.torque_at_rpm(rpm);
        let omega = rpm * 2.0 * std::f64::consts::PI / 60.0;
        torque * omega
    }
}

/// Transmission model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transmission {
    /// Transmission type.
    pub transmission_type: TransmissionType,

    /// Gear ratios (1st, 2nd, 3rd, ...).
    pub gear_ratios: Vec<f64>,

    /// Reverse gear ratio.
    pub reverse_ratio: f64,

    /// Current gear (0 = neutral, -1 = reverse, 1+ = forward gears).
    pub current_gear: i32,

    /// Transmission efficiency (0-1).
    pub efficiency: f64,

    /// Shift time (s).
    pub shift_time: f64,

    /// Time since last shift.
    pub time_since_shift: f64,
}

impl Transmission {
    /// Creates a new transmission.
    pub fn new(
        transmission_type: TransmissionType,
        gear_ratios: Vec<f64>,
        reverse_ratio: f64,
    ) -> Self {
        Self {
            transmission_type,
            gear_ratios,
            reverse_ratio,
            current_gear: 1, // Start in 1st gear
            efficiency: 0.95,
            shift_time: 0.3,
            time_since_shift: 0.0,
        }
    }

    /// Creates a 5-speed automatic transmission.
    pub fn automatic_5speed() -> Self {
        Self::new(
            TransmissionType::Automatic,
            vec![3.5, 2.1, 1.4, 1.0, 0.75],
            -3.8,
        )
    }

    /// Creates a 6-speed manual transmission.
    pub fn manual_6speed() -> Self {
        Self::new(
            TransmissionType::Manual,
            vec![3.8, 2.2, 1.5, 1.1, 0.85, 0.65],
            -4.0,
        )
    }

    /// Creates an 8-speed automatic transmission.
    pub fn automatic_8speed() -> Self {
        Self::new(
            TransmissionType::Automatic,
            vec![4.7, 3.1, 2.1, 1.7, 1.3, 1.0, 0.82, 0.64],
            -4.2,
        )
    }

    /// Updates transmission for one time step.
    pub fn update(&mut self, dt: f64, engine_rpm: f64) -> TransmissionOutput {
        self.time_since_shift += dt;

        // Auto shift logic (simplified)
        if self.transmission_type == TransmissionType::Automatic {
            self.auto_shift(engine_rpm);
        }

        let gear_ratio = self.current_gear_ratio();
        let efficiency = if self.time_since_shift < self.shift_time {
            // Reduced efficiency during shift
            self.efficiency * 0.5
        } else {
            self.efficiency
        };

        TransmissionOutput {
            gear_ratio,
            efficiency,
            current_gear: self.current_gear,
        }
    }

    /// Returns the current gear ratio.
    pub fn current_gear_ratio(&self) -> f64 {
        if self.current_gear == 0 {
            0.0 // Neutral
        } else if self.current_gear == -1 {
            self.reverse_ratio
        } else {
            let idx = (self.current_gear - 1) as usize;
            self.gear_ratios.get(idx).copied().unwrap_or(1.0)
        }
    }

    /// Automatic shift logic.
    fn auto_shift(&mut self, engine_rpm: f64) {
        // Upshift at 80% of redline
        if engine_rpm > 5500.0 && self.current_gear < self.gear_ratios.len() as i32 {
            self.shift_up();
        }

        // Downshift at 30% of redline
        if engine_rpm < 2000.0 && self.current_gear > 1 {
            self.shift_down();
        }
    }

    /// Shifts to next higher gear.
    pub fn shift_up(&mut self) {
        if self.current_gear < self.gear_ratios.len() as i32 {
            self.current_gear += 1;
            self.time_since_shift = 0.0;
        }
    }

    /// Shifts to next lower gear.
    pub fn shift_down(&mut self) {
        if self.current_gear > 1 {
            self.current_gear -= 1;
            self.time_since_shift = 0.0;
        }
    }
}

/// Transmission output for this time step.
#[derive(Debug, Clone, Copy)]
pub struct TransmissionOutput {
    pub gear_ratio: f64,
    pub efficiency: f64,
    pub current_gear: i32,
}

/// Transmission type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransmissionType {
    Manual,
    Automatic,
    CVT,
    DualClutch,
}

/// Drivetrain configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DrivetrainType {
    FrontWheelDrive,
    RearWheelDrive,
    AllWheelDrive,
}

/// Braking system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrakingSystem {
    /// Maximum brake torque per wheel (N·m).
    pub max_brake_torque: f64,

    /// Front/rear brake bias (0-1, 0.6 = 60% front).
    pub brake_bias: f64,

    /// ABS enabled.
    pub abs_enabled: bool,
}

impl BrakingSystem {
    /// Creates a new braking system.
    pub fn new(max_brake_torque: f64, brake_bias: f64, abs_enabled: bool) -> Self {
        Self {
            max_brake_torque,
            brake_bias,
            abs_enabled,
        }
    }

    /// Computes brake torque for front wheel.
    pub fn front_brake_torque(&self, brake_pressure: f64) -> f64 {
        self.max_brake_torque * self.brake_bias * brake_pressure
    }

    /// Computes brake torque for rear wheel.
    pub fn rear_brake_torque(&self, brake_pressure: f64) -> f64 {
        self.max_brake_torque * (1.0 - self.brake_bias) * brake_pressure
    }
}

impl Default for BrakingSystem {
    fn default() -> Self {
        Self::new(2000.0, 0.6, true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_engine_creation() {
        let engine = Engine::passenger_car();
        assert_eq!(engine.idle_rpm, 800.0);
        assert!(engine.max_torque > 0.0);
    }

    #[test]
    fn test_engine_torque_curve() {
        let engine = Engine::passenger_car();

        let torque_idle = engine.torque_at_rpm(800.0);
        let torque_peak = engine.torque_at_rpm(3500.0);
        let torque_redline = engine.torque_at_rpm(6500.0);

        assert!(torque_peak > torque_idle);
        assert!(torque_peak > torque_redline);
        assert_relative_eq!(torque_peak, engine.max_torque, epsilon = 10.0);
    }

    #[test]
    fn test_transmission_creation() {
        let trans = Transmission::automatic_5speed();
        assert_eq!(trans.gear_ratios.len(), 5);
        assert_eq!(trans.current_gear, 1);
    }

    #[test]
    fn test_transmission_shift() {
        let mut trans = Transmission::manual_6speed();
        assert_eq!(trans.current_gear, 1);

        trans.shift_up();
        assert_eq!(trans.current_gear, 2);

        trans.shift_down();
        assert_eq!(trans.current_gear, 1);
    }

    #[test]
    fn test_powertrain_creation() {
        let powertrain = Powertrain::passenger_car();
        assert_eq!(powertrain.drivetrain, DrivetrainType::FrontWheelDrive);
        assert_eq!(powertrain.num_driven_wheels(), 2);
    }
}
