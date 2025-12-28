//! AccuScene Enterprise Integration Layer v0.2.5
//!
//! This crate provides a unified integration layer that ties together all
//! AccuScene Enterprise v0.2.5 features into a cohesive, enterprise-ready system.
//!
//! ## Features
//!
//! ### Core Systems
//! - Event sourcing & CQRS architecture
//! - Advanced analytics engine
//! - Real-time data streaming
//! - Machine learning integration
//! - High-performance physics simulation
//!
//! ### Security & Compliance
//! - Enterprise security framework
//! - Cryptographic primitives
//! - Single Sign-On (SSO) integration
//! - Audit logging and compliance
//!
//! ### Data Management
//! - Multi-tier caching system
//! - Database abstraction layer
//! - Data compression and serialization
//! - Real-time data transfer
//!
//! ### Infrastructure
//! - Distributed clustering
//! - Job processing and scheduling
//! - Telemetry and monitoring
//! - Health check aggregation
//!
//! ### User Experience (v0.2.5)
//! - Accessibility (a11y) support
//! - Interactive dashboards
//! - Gesture recognition
//! - Push notifications
//! - Offline-first capabilities
//! - User preferences management
//! - Full-text search
//! - Advanced data visualization
//!
//! ## Architecture
//!
//! The integration layer follows a service-oriented architecture with:
//! - Unified configuration management
//! - Runtime initialization and lifecycle management
//! - Facade pattern for simplified API access
//! - Cross-crate event system
//! - Service registry for dependency injection
//! - Aggregated health checks
//!
//! ## Usage
//!
//! ```rust,no_run
//! use accuscene_integration::{Runtime, Config};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Load configuration
//!     let config = Config::load()?;
//!
//!     // Initialize runtime
//!     let runtime = Runtime::new(config).await?;
//!
//!     // Start all services
//!     runtime.start().await?;
//!
//!     // Services are now available through the facade
//!     let facade = runtime.facade();
//!
//!     // Use enterprise features...
//!
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

// ============================================================================
// Public Modules
// ============================================================================

pub mod config;
pub mod events;
pub mod facade;
pub mod health;
pub mod registry;
pub mod runtime;

// ============================================================================
// Re-exports from Core Crates
// ============================================================================

/// Core functionality and types
pub mod core {
    pub use accuscene_core::*;
}

/// Error types and handling
pub mod errors {
    pub use accuscene_errors::*;
}

/// Foreign Function Interface (FFI) for Node.js integration
pub mod ffi {
    pub use accuscene_ffi::*;
}

// ============================================================================
// Data Layer Re-exports
// ============================================================================

/// Database abstraction and ORM
pub mod database {
    pub use accuscene_database::*;
}

/// Multi-tier caching system
pub mod cache {
    pub use accuscene_cache::*;
}

/// Event sourcing and CQRS
pub mod eventsourcing {
    pub use accuscene_eventsourcing::*;
}

// ============================================================================
// Security Re-exports
// ============================================================================

/// Enterprise security framework
pub mod security {
    pub use accuscene_security::*;
}

/// Cryptographic primitives
pub mod crypto {
    pub use accuscene_crypto::*;
}

/// Single Sign-On integration
pub mod sso {
    pub use accuscene_sso::*;
}

// ============================================================================
// Processing & Computation Re-exports
// ============================================================================

/// Physics simulation engine
pub mod physics {
    pub use accuscene_physics::*;
}

/// Machine learning integration
pub mod ml {
    pub use accuscene_ml::*;
}

/// Advanced analytics engine
pub mod analytics {
    pub use accuscene_analytics::*;
}

/// Job processing and scheduling
pub mod jobs {
    pub use accuscene_jobs::*;
}

// ============================================================================
// Data Handling Re-exports
// ============================================================================

/// Data compression and serialization
pub mod compression {
    pub use accuscene_compression::*;
}

/// Real-time data streaming
pub mod streaming {
    pub use accuscene_streaming::*;
}

/// Data transfer and synchronization
pub mod transfer {
    pub use accuscene_transfer::*;
}

// ============================================================================
// Infrastructure Re-exports
// ============================================================================

/// Distributed clustering
pub mod cluster {
    pub use accuscene_cluster::*;
}

/// Telemetry and monitoring
pub mod telemetry {
    pub use accuscene_telemetry::*;
}

// ============================================================================
// User Experience Re-exports (v0.2.5 Features)
// ============================================================================

/// Accessibility support
pub mod a11y {
    pub use accuscene_a11y::*;
}

/// Interactive dashboards
pub mod dashboard {
    pub use accuscene_dashboard::*;
}

/// Gesture recognition
pub mod gestures {
    pub use accuscene_gestures::*;
}

/// Push notifications
pub mod notifications {
    pub use accuscene_notifications::*;
}

/// Offline-first capabilities
pub mod offline {
    pub use accuscene_offline::*;
}

/// User preferences management
pub mod preferences {
    pub use accuscene_preferences::*;
}

/// Full-text search
pub mod search {
    pub use accuscene_search::*;
}

/// Advanced data visualization
pub mod visualization {
    pub use accuscene_visualization::*;
}

// ============================================================================
// Version Information
// ============================================================================

/// The version of the AccuScene Integration layer
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The version of the AccuScene Enterprise platform
pub const ENTERPRISE_VERSION: &str = "0.2.5";

/// Build information
pub const BUILD_INFO: BuildInfo = BuildInfo {
    version: VERSION,
    enterprise_version: ENTERPRISE_VERSION,
    commit_hash: option_env!("GIT_HASH").unwrap_or("unknown"),
    build_date: option_env!("BUILD_DATE").unwrap_or("unknown"),
};

/// Build information structure
#[derive(Debug, Clone, Copy)]
pub struct BuildInfo {
    /// Crate version
    pub version: &'static str,
    /// Enterprise platform version
    pub enterprise_version: &'static str,
    /// Git commit hash
    pub commit_hash: &'static str,
    /// Build date
    pub build_date: &'static str,
}

// ============================================================================
// Prelude
// ============================================================================

/// Commonly used types and traits
pub mod prelude {
    pub use crate::config::{Config, ConfigLoader};
    pub use crate::events::{Event, EventBus, EventHandler};
    pub use crate::facade::Facade;
    pub use crate::health::{HealthCheck, HealthStatus};
    pub use crate::registry::{Registry, ServiceDescriptor};
    pub use crate::runtime::Runtime;
    pub use crate::{BuildInfo, ENTERPRISE_VERSION, VERSION};
}
