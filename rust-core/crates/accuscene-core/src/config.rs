//! Configuration management for AccuScene Core
//!
//! This module provides type-safe configuration management with
//! validation and default values.

use crate::error::{AccuSceneError, Result};
use crate::traits::{Serializable, Validatable};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Physics engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsConfig {
    /// Gravity acceleration (m/s²)
    pub gravity: f64,

    /// Default coefficient of friction
    pub friction_coefficient: f64,

    /// Air resistance coefficient
    pub air_resistance: f64,

    /// Time step for simulation (seconds)
    pub time_step: f64,

    /// Maximum iterations for physics solver
    pub max_iterations: u32,

    /// Convergence threshold for iterative solvers
    pub convergence_threshold: f64,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            gravity: 9.81,
            friction_coefficient: 0.7,
            air_resistance: 0.0012,
            time_step: 0.016, // ~60 FPS
            max_iterations: 100,
            convergence_threshold: 1e-6,
        }
    }
}

impl Validatable for PhysicsConfig {
    fn validate(&self) -> Result<()> {
        if self.gravity <= 0.0 {
            return Err(AccuSceneError::validation_field(
                "Gravity must be positive",
                "gravity",
            ));
        }

        if self.friction_coefficient < 0.0 || self.friction_coefficient > 1.0 {
            return Err(AccuSceneError::validation_field(
                "Friction coefficient must be between 0 and 1",
                "friction_coefficient",
            ));
        }

        if self.time_step <= 0.0 || self.time_step > 1.0 {
            return Err(AccuSceneError::validation_field(
                "Time step must be between 0 and 1 second",
                "time_step",
            ));
        }

        if self.max_iterations == 0 {
            return Err(AccuSceneError::validation_field(
                "Max iterations must be greater than 0",
                "max_iterations",
            ));
        }

        Ok(())
    }
}

impl Serializable for PhysicsConfig {}

/// Rendering configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderConfig {
    /// Render quality level (0-100)
    pub quality: u8,

    /// Enable shadows
    pub enable_shadows: bool,

    /// Enable reflections
    pub enable_reflections: bool,

    /// Anti-aliasing samples
    pub aa_samples: u8,

    /// Maximum frame rate
    pub max_fps: u32,

    /// Field of view in degrees
    pub fov: f32,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            quality: 80,
            enable_shadows: true,
            enable_reflections: true,
            aa_samples: 4,
            max_fps: 60,
            fov: 75.0,
        }
    }
}

impl Validatable for RenderConfig {
    fn validate(&self) -> Result<()> {
        if self.quality > 100 {
            return Err(AccuSceneError::validation_field(
                "Quality must be between 0 and 100",
                "quality",
            ));
        }

        if ![0, 2, 4, 8, 16].contains(&self.aa_samples) {
            return Err(AccuSceneError::validation_field(
                "AA samples must be 0, 2, 4, 8, or 16",
                "aa_samples",
            ));
        }

        if self.fov < 30.0 || self.fov > 120.0 {
            return Err(AccuSceneError::validation_field(
                "FOV must be between 30 and 120 degrees",
                "fov",
            ));
        }

        Ok(())
    }
}

impl Serializable for RenderConfig {}

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Application name
    pub app_name: String,

    /// Application version
    pub app_version: String,

    /// Enable debug mode
    pub debug_mode: bool,

    /// Enable telemetry
    pub enable_telemetry: bool,

    /// Log level
    pub log_level: LogLevel,

    /// Maximum cache size in MB
    pub max_cache_size_mb: u32,

    /// Auto-save interval in seconds (0 = disabled)
    pub auto_save_interval: u32,
}

/// Log level configuration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum LogLevel {
    /// Trace level logging
    Trace,
    /// Debug level logging
    Debug,
    /// Info level logging
    Info,
    /// Warning level logging
    Warn,
    /// Error level logging
    Error,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            app_name: "AccuScene Enterprise".to_string(),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            debug_mode: cfg!(debug_assertions),
            enable_telemetry: true,
            log_level: if cfg!(debug_assertions) {
                LogLevel::Debug
            } else {
                LogLevel::Info
            },
            max_cache_size_mb: 512,
            auto_save_interval: 300, // 5 minutes
        }
    }
}

impl Validatable for AppConfig {
    fn validate(&self) -> Result<()> {
        if self.app_name.is_empty() {
            return Err(AccuSceneError::validation_field(
                "App name cannot be empty",
                "app_name",
            ));
        }

        if self.max_cache_size_mb == 0 {
            return Err(AccuSceneError::validation_field(
                "Max cache size must be greater than 0",
                "max_cache_size_mb",
            ));
        }

        Ok(())
    }
}

impl Serializable for AppConfig {}

/// Master configuration container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Physics configuration
    pub physics: PhysicsConfig,

    /// Rendering configuration
    pub render: RenderConfig,

    /// Application configuration
    pub app: AppConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            physics: PhysicsConfig::default(),
            render: RenderConfig::default(),
            app: AppConfig::default(),
        }
    }
}

impl Validatable for Config {
    fn validate(&self) -> Result<()> {
        self.physics.validate()?;
        self.render.validate()?;
        self.app.validate()?;
        Ok(())
    }
}

impl Serializable for Config {}

impl Config {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Load configuration from JSON string
    pub fn from_json_str(json: &str) -> Result<Self> {
        let config: Config = serde_json::from_str(json)?;
        config.validate()?;
        Ok(config)
    }

    /// Save configuration to JSON string
    pub fn to_json_str(&self) -> Result<String> {
        self.validate()?;
        Ok(serde_json::to_string_pretty(self)?)
    }
}

/// Thread-safe configuration manager
#[derive(Debug, Clone)]
pub struct ConfigManager {
    config: Arc<RwLock<Config>>,
}

impl ConfigManager {
    /// Create a new configuration manager with default config
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(Config::default())),
        }
    }

    /// Create a configuration manager with custom config
    pub fn with_config(config: Config) -> Result<Self> {
        config.validate()?;
        Ok(Self {
            config: Arc::new(RwLock::new(config)),
        })
    }

    /// Get a copy of the current configuration
    pub fn get(&self) -> Config {
        self.config.read().clone()
    }

    /// Update the configuration
    pub fn set(&self, config: Config) -> Result<()> {
        config.validate()?;
        *self.config.write() = config;
        Ok(())
    }

    /// Update configuration using a closure
    pub fn update<F>(&self, f: F) -> Result<()>
    where
        F: FnOnce(&mut Config),
    {
        let mut config = self.config.write();
        f(&mut config);
        config.validate()?;
        Ok(())
    }

    /// Get physics configuration
    pub fn physics(&self) -> PhysicsConfig {
        self.config.read().physics.clone()
    }

    /// Get render configuration
    pub fn render(&self) -> RenderConfig {
        self.config.read().render.clone()
    }

    /// Get app configuration
    pub fn app(&self) -> AppConfig {
        self.config.read().app.clone()
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_physics_validation() {
        let mut config = PhysicsConfig::default();
        assert!(config.validate().is_ok());

        config.gravity = -1.0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let json = config.to_json_str().unwrap();
        let deserialized = Config::from_json_str(&json).unwrap();
        assert_eq!(config.physics.gravity, deserialized.physics.gravity);
    }

    #[test]
    fn test_config_manager() {
        let manager = ConfigManager::new();
        let config = manager.get();
        assert_eq!(config.physics.gravity, 9.81);

        manager.update(|c| c.physics.gravity = 10.0).unwrap();
        assert_eq!(manager.physics().gravity, 10.0);
    }
}
