//! Core type definitions for AccuScene
//!
//! This module contains all the core types used throughout the
//! AccuScene platform, including physics types, vehicle models,
//! accident scenes, cases, and evidence tracking.

pub mod accident;
pub mod case;
pub mod evidence;
pub mod vector;
pub mod vehicle;

// Re-export common types
pub use accident::{Accident, AccidentScene, RoadCondition, WeatherCondition};
pub use case::{Case, CaseMetadata, CaseStatus};
pub use evidence::{Evidence, EvidenceMetadata, EvidenceType};
pub use vector::{Vector2D, Vector3D};
pub use vehicle::{Vehicle, VehicleCategory, VehicleMetadata};
