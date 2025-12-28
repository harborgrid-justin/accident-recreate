//! Kinematics calculations for accident reconstruction.
//!
//! Provides:
//! - Trajectory prediction and analysis
//! - Conservation of momentum calculations
//! - Velocity and acceleration analysis

pub mod momentum;
pub mod trajectory;

pub use momentum::{MomentumAnalysis, MomentumConservation};
pub use trajectory::{Trajectory, TrajectoryPoint, TrajectoryPredictor};
