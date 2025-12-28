//! Vehicle dynamics simulation module.
//!
//! Provides detailed vehicle physics including:
//! - Mass properties and inertia
//! - Tire friction modeling (Pacejka Magic Formula)
//! - Suspension dynamics
//! - Center of gravity calculations

pub mod suspension;
pub mod tire;
pub mod vehicle;

pub use suspension::{SuspensionConfig, SuspensionState};
pub use tire::{TireForces, TireModel, TireState};
pub use vehicle::{VehicleDynamics, VehicleState};
