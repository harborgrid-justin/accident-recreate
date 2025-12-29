//! Runtime initialization and lifecycle management
//!
//! This module provides the main runtime that initializes and manages
//! all AccuScene Enterprise services.

use crate::config::Config;
use crate::events::EventBus;
use crate::facade::Facade;
use crate::health::HealthChecker;
use crate::registry::Registry;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Runtime state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeState {
    /// Runtime is initializing
    Initializing,
    /// Runtime is starting services
    Starting,
    /// Runtime is running
    Running,
    /// Runtime is shutting down
    ShuttingDown,
    /// Runtime has stopped
    Stopped,
    /// Runtime encountered an error
    Error,
}

/// The main runtime for AccuScene Enterprise
pub struct Runtime {
    /// Configuration
    config: Config,

    /// Runtime state
    state: Arc<RwLock<RuntimeState>>,

    /// Service registry
    registry: Arc<Registry>,

    /// Event bus
    event_bus: Arc<EventBus>,

    /// Health checker
    health_checker: Arc<HealthChecker>,

    /// Facade for simplified API access
    facade: Arc<Facade>,
}

impl Runtime {
    /// Create a new runtime with the given configuration
    pub async fn new(config: Config) -> Result<Self> {
        info!("Initializing AccuScene Enterprise v{}", crate::ENTERPRISE_VERSION);

        // Validate configuration
        config.validate()?;

        // Initialize tracing
        Self::init_tracing(&config)?;

        let state = Arc::new(RwLock::new(RuntimeState::Initializing));
        let registry = Arc::new(Registry::new());
        let event_bus = Arc::new(EventBus::new());
        let health_checker = Arc::new(HealthChecker::new());

        // Create facade
        let facade = Arc::new(Facade::new(
            config.clone(),
            Arc::clone(&registry),
            Arc::clone(&event_bus),
        ));

        Ok(Self {
            config,
            state,
            registry,
            event_bus,
            health_checker,
            facade,
        })
    }

    /// Initialize tracing/logging
    fn init_tracing(config: &Config) -> Result<()> {
        use tracing_subscriber::prelude::*;

        let level = match config.app.log_level {
            crate::config::LogLevel::Trace => tracing::Level::TRACE,
            crate::config::LogLevel::Debug => tracing::Level::DEBUG,
            crate::config::LogLevel::Info => tracing::Level::INFO,
            crate::config::LogLevel::Warn => tracing::Level::WARN,
            crate::config::LogLevel::Error => tracing::Level::ERROR,
        };

        let subscriber = tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .with_target(true)
                    .with_level(true)
                    .with_thread_ids(true)
            )
            .with(
                tracing_subscriber::EnvFilter::from_default_env()
                    .add_directive(level.into())
            );

        if config.app.debug {
            tracing::subscriber::set_global_default(subscriber)?;
        } else {
            tracing::subscriber::set_global_default(
                subscriber.with(tracing_subscriber::fmt::layer().json())
            )?;
        }

        Ok(())
    }

    /// Start the runtime and all services
    pub async fn start(&self) -> Result<()> {
        info!("Starting AccuScene Enterprise runtime");

        *self.state.write().await = RuntimeState::Starting;

        // Initialize core services
        self.init_database().await?;
        self.init_cache().await?;
        self.init_security().await?;

        // Initialize processing services
        self.init_analytics().await?;
        self.init_physics().await?;
        self.init_ml().await?;
        self.init_jobs().await?;

        // Initialize infrastructure services
        self.init_cluster().await?;
        self.init_telemetry().await?;
        self.init_streaming().await?;

        // Initialize UX services (v0.2.5)
        self.init_ux_services().await?;

        // Register health checks
        self.register_health_checks().await?;

        *self.state.write().await = RuntimeState::Running;
        info!("AccuScene Enterprise runtime started successfully");

        Ok(())
    }

    /// Initialize database service
    async fn init_database(&self) -> Result<()> {
        info!("Initializing database service");

        // Database initialization would happen here
        // For now, we just register it
        self.registry.register(
            "database".to_string(),
            "Database service".to_string(),
            "v0.2.0".to_string(),
        );

        Ok(())
    }

    /// Initialize cache service
    async fn init_cache(&self) -> Result<()> {
        if !self.config.cache.enabled {
            info!("Cache service disabled");
            return Ok(());
        }

        info!("Initializing cache service");

        self.registry.register(
            "cache".to_string(),
            "Multi-tier cache service".to_string(),
            "v0.2.0".to_string(),
        );

        Ok(())
    }

    /// Initialize security service
    async fn init_security(&self) -> Result<()> {
        info!("Initializing security service");

        self.registry.register(
            "security".to_string(),
            "Security and authentication service".to_string(),
            "v0.2.0".to_string(),
        );

        // Initialize SSO if enabled
        if self.config.security.sso_enabled {
            info!("Initializing SSO integration");
            self.registry.register(
                "sso".to_string(),
                "Single Sign-On service".to_string(),
                "v0.2.5".to_string(),
            );
        }

        Ok(())
    }

    /// Initialize analytics service
    async fn init_analytics(&self) -> Result<()> {
        if !self.config.analytics.enabled {
            info!("Analytics service disabled");
            return Ok(());
        }

        info!("Initializing analytics service");

        self.registry.register(
            "analytics".to_string(),
            "Advanced analytics engine".to_string(),
            "v0.2.0".to_string(),
        );

        Ok(())
    }

    /// Initialize physics service
    async fn init_physics(&self) -> Result<()> {
        info!("Initializing physics simulation service");

        self.registry.register(
            "physics".to_string(),
            "Physics simulation engine".to_string(),
            "v0.2.0".to_string(),
        );

        Ok(())
    }

    /// Initialize ML service
    async fn init_ml(&self) -> Result<()> {
        info!("Initializing machine learning service");

        self.registry.register(
            "ml".to_string(),
            "Machine learning service".to_string(),
            "v0.2.0".to_string(),
        );

        Ok(())
    }

    /// Initialize jobs service
    async fn init_jobs(&self) -> Result<()> {
        info!("Initializing job processing service");

        self.registry.register(
            "jobs".to_string(),
            "Job processing and scheduling".to_string(),
            "v0.2.0".to_string(),
        );

        Ok(())
    }

    /// Initialize cluster service
    async fn init_cluster(&self) -> Result<()> {
        if !self.config.cluster.enabled {
            info!("Cluster service disabled");
            return Ok(());
        }

        info!("Initializing cluster service");

        self.registry.register(
            "cluster".to_string(),
            "Distributed clustering service".to_string(),
            "v0.2.0".to_string(),
        );

        Ok(())
    }

    /// Initialize telemetry service
    async fn init_telemetry(&self) -> Result<()> {
        if !self.config.telemetry.enabled {
            info!("Telemetry service disabled");
            return Ok(());
        }

        info!("Initializing telemetry service");

        self.registry.register(
            "telemetry".to_string(),
            "Telemetry and monitoring service".to_string(),
            "v0.2.0".to_string(),
        );

        Ok(())
    }

    /// Initialize streaming service
    async fn init_streaming(&self) -> Result<()> {
        info!("Initializing streaming service");

        self.registry.register(
            "streaming".to_string(),
            "Real-time data streaming service".to_string(),
            "v0.2.0".to_string(),
        );

        Ok(())
    }

    /// Initialize UX services (v0.2.5)
    async fn init_ux_services(&self) -> Result<()> {
        info!("Initializing user experience services");

        // Accessibility
        if self.config.ux.accessibility.enabled {
            self.registry.register(
                "accessibility".to_string(),
                "Accessibility support service".to_string(),
                "v0.2.5".to_string(),
            );
        }

        // Dashboard
        if self.config.ux.dashboard.enabled {
            self.registry.register(
                "dashboard".to_string(),
                "Interactive dashboard service".to_string(),
                "v0.2.5".to_string(),
            );
        }

        // Gestures
        if self.config.ux.gestures.enabled {
            self.registry.register(
                "gestures".to_string(),
                "Gesture recognition service".to_string(),
                "v0.2.5".to_string(),
            );
        }

        // Notifications
        if self.config.ux.notifications.enabled {
            self.registry.register(
                "notifications".to_string(),
                "Push notifications service".to_string(),
                "v0.2.5".to_string(),
            );
        }

        // Offline
        if self.config.ux.offline.enabled {
            self.registry.register(
                "offline".to_string(),
                "Offline-first capabilities service".to_string(),
                "v0.2.5".to_string(),
            );
        }

        // Preferences
        if self.config.ux.preferences.enabled {
            self.registry.register(
                "preferences".to_string(),
                "User preferences service".to_string(),
                "v0.2.5".to_string(),
            );
        }

        // Search
        if self.config.ux.search.enabled {
            self.registry.register(
                "search".to_string(),
                "Full-text search service".to_string(),
                "v0.2.5".to_string(),
            );
        }

        // Visualization
        if self.config.ux.visualization.enabled {
            self.registry.register(
                "visualization".to_string(),
                "Advanced data visualization service".to_string(),
                "v0.2.5".to_string(),
            );
        }

        Ok(())
    }

    /// Register health checks for all services
    async fn register_health_checks(&self) -> Result<()> {
        info!("Registering health checks");

        // Register health check for each service in the registry
        for service in self.registry.list_services() {
            self.health_checker.register_check(service).await;
        }

        Ok(())
    }

    /// Stop the runtime and all services
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping AccuScene Enterprise runtime");

        *self.state.write().await = RuntimeState::ShuttingDown;

        // Stop services in reverse order
        warn!("Gracefully shutting down services...");

        *self.state.write().await = RuntimeState::Stopped;
        info!("AccuScene Enterprise runtime stopped");

        Ok(())
    }

    /// Get the current runtime state
    pub async fn state(&self) -> RuntimeState {
        *self.state.read().await
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

    /// Get the health checker
    pub fn health_checker(&self) -> Arc<HealthChecker> {
        Arc::clone(&self.health_checker)
    }

    /// Get the facade
    pub fn facade(&self) -> Arc<Facade> {
        Arc::clone(&self.facade)
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        info!("Runtime dropped, ensuring cleanup");
    }
}
