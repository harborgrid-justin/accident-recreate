//! AccuScene Physics Engine - High-Performance Physics Simulation for Accident Reconstruction
//!
//! This crate provides a comprehensive physics simulation engine designed specifically
//! for accident reconstruction analysis. It includes:
//!
//! - **Collision Detection**: GJK and SAT algorithms for accurate collision detection
//! - **Vehicle Dynamics**: Realistic vehicle physics with tire models (Pacejka Magic Formula)
//! - **Kinematics**: Trajectory prediction and momentum conservation analysis
//! - **Energy Analysis**: Crush energy, speed estimation from skid marks
//! - **Reconstruction Tools**: Complete accident reconstruction with validation
//! - **Parallel Processing**: High-performance simulation using Rayon
//!
//! # Examples
//!
//! ## Basic Physics Simulation
//!
//! ```no_run
//! use accuscene_physics::{PhysicsEngine, EngineConfig, RigidBody};
//! use nalgebra::{Point3, Vector3};
//!
//! // Create physics engine with default config (1000Hz simulation)
//! let mut engine = PhysicsEngine::new(EngineConfig::default());
//!
//! // Add a vehicle
//! let vehicle = RigidBody::new(1, 1500.0) // ID 1, 1500 kg mass
//!     .with_position(Point3::new(0.0, 0.0, 0.0))
//!     .with_velocity(Vector3::new(20.0, 0.0, 0.0)); // 20 m/s forward
//!
//! engine.add_body(vehicle);
//!
//! // Simulate for 1 second
//! engine.step(1.0);
//! ```
//!
//! ## Speed Estimation from Skid Marks
//!
//! ```
//! use accuscene_physics::{SpeedEstimator, SurfaceType};
//!
//! let skid_length = 50.0; // 50 meters
//! let surface = SurfaceType::AsphaltDry;
//! let grade = 0.0; // Flat road
//!
//! let speed = SpeedEstimator::from_skid_marks(skid_length, surface, grade);
//!
//! println!("Estimated speed: {:.1} km/h", speed.speed_kmh);
//! println!("Confidence: {:.0}%", speed.confidence * 100.0);
//! ```
//!
//! ## Accident Reconstruction
//!
//! ```
//! use accuscene_physics::{AccidentReconstruction, SpeedEstimator, SurfaceType};
//!
//! let mut reconstruction = AccidentReconstruction::new();
//!
//! // Add speed estimate from skid marks
//! let speed = SpeedEstimator::from_skid_marks(45.0, SurfaceType::AsphaltDry, 0.0);
//! reconstruction.add_speed_estimate(speed);
//!
//! // Add more evidence...
//! reconstruction.add_note("Vehicle came to rest 30m past impact point".to_string());
//!
//! println!("Reconstruction confidence: {:.0}%", reconstruction.confidence * 100.0);
//! ```

// Re-export core dependencies for convenience
pub use nalgebra;
pub use serde;

// Module declarations
pub mod collision;
pub mod dynamics;
pub mod energy;
pub mod engine;
pub mod friction;
pub mod kinematics;
pub mod parallel;
pub mod reconstruction;
pub mod simulation;
pub mod speed;

// Re-export commonly used types
pub use collision::{
    Aabb, Collision, CollisionDetector, CollisionResolver, GjkAlgorithm, ImpulseResponse,
    SatAlgorithm,
};

pub use dynamics::{
    SuspensionConfig, SuspensionState, TireForces, TireModel, TireState, VehicleDynamics,
    VehicleState, VehicleSuspension,
};

pub use energy::{EnergyAnalysis, EnergyCalculator, VehicleType};

pub use engine::{EngineConfig, PhysicsEngine};

pub use friction::{FrictionModel, SurfaceType};

pub use kinematics::{
    MomentumAnalysis, MomentumConservation, Trajectory, TrajectoryPoint, TrajectoryPredictor,
};

pub use parallel::{ParallelPhysics, ParameterSweep, SimulationResult};

pub use reconstruction::{
    AccidentReconstruction, AnalysisResult, ReconstructionCalculator, ValidationResult,
};

pub use simulation::{RigidBody, SimulationRecording, SimulationSnapshot, SimulationState};

pub use speed::{SpeedEstimate, SpeedEstimator};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Library name
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// Returns version information
pub fn version() -> String {
    format!("{} v{}", NAME, VERSION)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        let v = version();
        assert!(v.contains("accuscene-physics"));
        assert!(v.contains("0.1.5"));
    }
}
