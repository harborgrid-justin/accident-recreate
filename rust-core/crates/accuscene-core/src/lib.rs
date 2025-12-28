//! AccuScene Core Library
//!
//! This is the core library for the AccuScene Enterprise accident recreation platform.
//! It provides fundamental types, physics calculations, and utilities for accident
//! reconstruction and analysis.
//!
//! # Features
//!
//! - **Physics Engine**: Complete 2D/3D vector mathematics and vehicle physics
//! - **Type System**: Comprehensive types for vehicles, accidents, cases, and evidence
//! - **Error Handling**: Robust error types with detailed categorization
//! - **Configuration**: Type-safe configuration management
//! - **Traits**: Common traits for serialization, validation, and identification
//!
//! # Example
//!
//! ```rust
//! use accuscene_core::prelude::*;
//!
//! // Create a new accident scene
//! let mut scene = AccidentScene::new("Highway Collision".to_string());
//!
//! // Create vehicles
//! let mut vehicle1 = Vehicle::new(VehicleCategory::Car);
//! vehicle1.position = Vector2D::new(0.0, 0.0);
//! vehicle1.velocity = Vector2D::new(20.0, 0.0); // 20 m/s
//!
//! let mut vehicle2 = Vehicle::new(VehicleCategory::SUV);
//! vehicle2.position = Vector2D::new(50.0, 0.0);
//! vehicle2.velocity = Vector2D::new(-15.0, 0.0); // -15 m/s
//!
//! // Add vehicles to scene
//! scene.add_vehicle(vehicle1).unwrap();
//! scene.add_vehicle(vehicle2).unwrap();
//!
//! // Calculate total kinetic energy
//! let total_ke = scene.total_kinetic_energy();
//! println!("Total kinetic energy: {} J", total_ke);
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![allow(clippy::module_inception)]

pub mod config;
pub mod error;
pub mod traits;
pub mod types;
pub mod utils;

// Re-export commonly used items
pub use error::{AccuSceneError, Result};
pub use config::{Config, ConfigManager};
use traits::Validatable;

/// Prelude module for convenient imports
///
/// This module re-exports the most commonly used types and traits.
/// You can use `use accuscene_core::prelude::*;` to import everything
/// you need for typical usage.
pub mod prelude {
    pub use crate::config::{AppConfig, Config, ConfigManager, LogLevel, PhysicsConfig, RenderConfig};
    pub use crate::error::{AccuSceneError, Result};
    pub use crate::traits::{
        Identifiable, MemoryFootprint, Serializable, Timestamped, Validatable, Versioned,
        WithMetadata,
    };
    pub use crate::types::{
        Accident, AccidentScene, Case, CaseMetadata, CaseStatus, Evidence, EvidenceMetadata,
        EvidenceType, RoadCondition, Vector2D, Vector3D, Vehicle, VehicleCategory,
        VehicleMetadata, WeatherCondition,
    };
    pub use crate::utils::*;
}

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Package name
pub const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");

/// Get version string
pub fn version() -> String {
    format!("{} v{}", PACKAGE_NAME, VERSION)
}

/// Initialize the library with default configuration
///
/// This should be called once at application startup to set up
/// logging and configuration.
pub fn init() -> Result<ConfigManager> {
    let config = Config::default();
    let manager = ConfigManager::with_config(config)?;
    Ok(manager)
}

/// Initialize with custom configuration
pub fn init_with_config(config: Config) -> Result<ConfigManager> {
    config.validate()?;
    let manager = ConfigManager::with_config(config)?;
    Ok(manager)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        let ver = version();
        assert!(ver.contains("accuscene-core"));
    }

    #[test]
    fn test_init() {
        let manager = init();
        assert!(manager.is_ok());
    }

    #[test]
    fn test_prelude_imports() {
        use crate::prelude::*;

        let scene = AccidentScene::new("Test".to_string());
        assert!(scene.validate().is_ok());

        let vehicle = Vehicle::new(VehicleCategory::Car);
        assert!(vehicle.validate().is_ok());

        let case = Case::new("Test Case".to_string());
        assert!(case.validate().is_ok());
    }
}
