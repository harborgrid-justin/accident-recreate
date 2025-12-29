//! Facade pattern for simplified API access
//!
//! This module provides a unified facade that simplifies access to all
//! AccuScene Enterprise services through a single, convenient interface.

use crate::config::Config;
use crate::events::EventBus;
use crate::registry::Registry;
use std::sync::Arc;

/// The main facade for AccuScene Enterprise services
///
/// This provides a simplified, unified API for accessing all enterprise features
/// without needing to know the details of individual service implementations.
pub struct Facade {
    config: Config,
    registry: Arc<Registry>,
    event_bus: Arc<EventBus>,
}

impl Facade {
    /// Create a new facade
    pub fn new(config: Config, registry: Arc<Registry>, event_bus: Arc<EventBus>) -> Self {
        Self {
            config,
            registry,
            event_bus,
        }
    }

    /// Get the configuration
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Get the service registry
    pub fn registry(&self) -> Arc<Registry> {
        Arc::clone(&self.registry)
    }

    /// Get the event bus
    pub fn event_bus(&self) -> Arc<EventBus> {
        Arc::clone(&self.event_bus)
    }

    // ========================================================================
    // Core Services
    // ========================================================================

    /// Get database service status
    pub fn database_available(&self) -> bool {
        self.registry.is_registered("database")
    }

    /// Get cache service status
    pub fn cache_available(&self) -> bool {
        self.registry.is_registered("cache")
    }

    /// Get event sourcing service status
    pub fn eventsourcing_available(&self) -> bool {
        self.registry.is_registered("eventsourcing")
    }

    // ========================================================================
    // Security Services
    // ========================================================================

    /// Get security service status
    pub fn security_available(&self) -> bool {
        self.registry.is_registered("security")
    }

    /// Get SSO service status
    pub fn sso_available(&self) -> bool {
        self.registry.is_registered("sso")
    }

    /// Check if authentication is enabled
    pub fn auth_enabled(&self) -> bool {
        self.config.security.auth_enabled
    }

    // ========================================================================
    // Processing Services
    // ========================================================================

    /// Get physics service status
    pub fn physics_available(&self) -> bool {
        self.registry.is_registered("physics")
    }

    /// Get ML service status
    pub fn ml_available(&self) -> bool {
        self.registry.is_registered("ml")
    }

    /// Get analytics service status
    pub fn analytics_available(&self) -> bool {
        self.registry.is_registered("analytics")
    }

    /// Get jobs service status
    pub fn jobs_available(&self) -> bool {
        self.registry.is_registered("jobs")
    }

    // ========================================================================
    // Infrastructure Services
    // ========================================================================

    /// Get cluster service status
    pub fn cluster_available(&self) -> bool {
        self.registry.is_registered("cluster")
    }

    /// Get telemetry service status
    pub fn telemetry_available(&self) -> bool {
        self.registry.is_registered("telemetry")
    }

    /// Get streaming service status
    pub fn streaming_available(&self) -> bool {
        self.registry.is_registered("streaming")
    }

    // ========================================================================
    // UX Services (v0.2.5)
    // ========================================================================

    /// Get accessibility service status
    pub fn accessibility_available(&self) -> bool {
        self.registry.is_registered("accessibility")
    }

    /// Check if accessibility features are enabled
    pub fn accessibility_enabled(&self) -> bool {
        self.config.ux.accessibility.enabled
    }

    /// Get dashboard service status
    pub fn dashboard_available(&self) -> bool {
        self.registry.is_registered("dashboard")
    }

    /// Get gestures service status
    pub fn gestures_available(&self) -> bool {
        self.registry.is_registered("gestures")
    }

    /// Check if gesture recognition is enabled
    pub fn gestures_enabled(&self) -> bool {
        self.config.ux.gestures.enabled
    }

    /// Get notifications service status
    pub fn notifications_available(&self) -> bool {
        self.registry.is_registered("notifications")
    }

    /// Check if push notifications are enabled
    pub fn push_notifications_enabled(&self) -> bool {
        self.config.ux.notifications.push_enabled
    }

    /// Get offline service status
    pub fn offline_available(&self) -> bool {
        self.registry.is_registered("offline")
    }

    /// Check if offline mode is enabled
    pub fn offline_enabled(&self) -> bool {
        self.config.ux.offline.enabled
    }

    /// Get preferences service status
    pub fn preferences_available(&self) -> bool {
        self.registry.is_registered("preferences")
    }

    /// Get search service status
    pub fn search_available(&self) -> bool {
        self.registry.is_registered("search")
    }

    /// Check if fuzzy search is enabled
    pub fn fuzzy_search_enabled(&self) -> bool {
        self.config.ux.search.fuzzy_enabled
    }

    /// Get visualization service status
    pub fn visualization_available(&self) -> bool {
        self.registry.is_registered("visualization")
    }

    /// Get visualization backend
    pub fn visualization_backend(&self) -> &str {
        &self.config.ux.visualization.backend
    }

    // ========================================================================
    // Utility Methods
    // ========================================================================

    /// Get all available services
    pub fn available_services(&self) -> Vec<String> {
        self.registry.list_services()
    }

    /// Get service count
    pub fn service_count(&self) -> usize {
        self.registry.service_count()
    }

    /// Check if all core services are available
    pub fn core_services_ready(&self) -> bool {
        self.database_available()
            && self.security_available()
            && self.physics_available()
    }

    /// Check if all UX services are available
    pub fn ux_services_ready(&self) -> bool {
        (!self.config.ux.accessibility.enabled || self.accessibility_available())
            && (!self.config.ux.dashboard.enabled || self.dashboard_available())
            && (!self.config.ux.gestures.enabled || self.gestures_available())
            && (!self.config.ux.notifications.enabled || self.notifications_available())
            && (!self.config.ux.offline.enabled || self.offline_available())
            && (!self.config.ux.preferences.enabled || self.preferences_available())
            && (!self.config.ux.search.enabled || self.search_available())
            && (!self.config.ux.visualization.enabled || self.visualization_available())
    }

    /// Check if the system is ready for production
    pub fn production_ready(&self) -> bool {
        self.core_services_ready()
            && self.config.app.environment == crate::config::Environment::Production
            && self.config.security.jwt_secret != "change-me-in-production"
            && self.config.security.auth_enabled
            && self.config.security.encryption_enabled
    }

    /// Get system information
    pub fn system_info(&self) -> SystemInfo {
        SystemInfo {
            version: crate::ENTERPRISE_VERSION.to_string(),
            environment: format!("{:?}", self.config.app.environment),
            services_count: self.service_count(),
            core_ready: self.core_services_ready(),
            ux_ready: self.ux_services_ready(),
            production_ready: self.production_ready(),
        }
    }
}

/// System information
#[derive(Debug, Clone)]
pub struct SystemInfo {
    /// Enterprise version
    pub version: String,
    /// Current environment
    pub environment: String,
    /// Number of registered services
    pub services_count: usize,
    /// Core services ready status
    pub core_ready: bool,
    /// UX services ready status
    pub ux_ready: bool,
    /// Production ready status
    pub production_ready: bool,
}
