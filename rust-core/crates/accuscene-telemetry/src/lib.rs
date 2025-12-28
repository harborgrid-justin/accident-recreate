//! AccuScene Enterprise Telemetry System
//!
//! Provides comprehensive logging, metrics, distributed tracing, health checks,
//! and performance profiling for the AccuScene platform.
//!
//! # Features
//!
//! - **Structured Logging**: JSON and text formats with rotation
//! - **Metrics**: Prometheus-compatible metrics export
//! - **Distributed Tracing**: Span management and context propagation
//! - **Health Checks**: Liveness and readiness probes
//! - **Performance Profiling**: CPU and memory profiling
//! - **Alerts**: Threshold-based alerting
//!
//! # Example
//!
//! ```rust,no_run
//! use accuscene_telemetry::{TelemetryConfig, TelemetrySystem};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = TelemetryConfig::default();
//! let telemetry = TelemetrySystem::new(config).await?;
//! telemetry.start().await?;
//! # Ok(())
//! # }
//! ```

pub mod config;
pub mod error;
pub mod logging;
pub mod metrics;
pub mod tracing;
pub mod health;
pub mod performance;
pub mod timing;
pub mod events;
pub mod alerts;
pub mod dashboard;

// Re-exports
pub use config::{TelemetryConfig, LoggingConfig, MetricsConfig, TracingConfig, HealthConfig};
pub use error::{TelemetryError, Result};
pub use logging::LoggingSystem;
pub use metrics::MetricsSystem;
pub use tracing::TracingSystem;
pub use health::HealthSystem;
pub use performance::PerformanceProfiler;
pub use timing::{Timer, TimingGuard};
pub use events::{Event, EventLogger};
pub use alerts::{Alert, AlertManager, AlertThreshold};
pub use dashboard::Dashboard;

use parking_lot::RwLock;
use std::sync::Arc;

/// Main telemetry system
pub struct TelemetrySystem {
    config: TelemetryConfig,
    logging: Arc<LoggingSystem>,
    metrics: Arc<MetricsSystem>,
    tracing: Arc<TracingSystem>,
    health: Arc<HealthSystem>,
    performance: Arc<RwLock<PerformanceProfiler>>,
    events: Arc<EventLogger>,
    alerts: Arc<RwLock<AlertManager>>,
    dashboard: Arc<RwLock<Dashboard>>,
}

impl TelemetrySystem {
    /// Create a new telemetry system
    pub async fn new(config: TelemetryConfig) -> Result<Self> {
        // Validate configuration
        config.validate()?;

        // Initialize logging
        let logging = Arc::new(LoggingSystem::new(&config.logging)?);
        logging.init()?;

        // Initialize metrics
        let metrics = Arc::new(MetricsSystem::new(&config.metrics)?);

        // Initialize tracing
        let tracing = Arc::new(TracingSystem::new(&config.tracing)?);

        // Initialize health checks
        let health = Arc::new(HealthSystem::new(&config.health)?);

        // Initialize performance profiler
        let performance = Arc::new(RwLock::new(PerformanceProfiler::new(&config.performance)?));

        // Initialize event logger
        let events = Arc::new(EventLogger::new());

        // Initialize alert manager
        let alerts = Arc::new(RwLock::new(AlertManager::new()));

        // Initialize dashboard
        let dashboard = Arc::new(RwLock::new(Dashboard::new()));

        Ok(Self {
            config,
            logging,
            metrics,
            tracing,
            health,
            performance,
            events,
            alerts,
            dashboard,
        })
    }

    /// Start the telemetry system
    pub async fn start(&self) -> Result<()> {
        tracing::info!("Starting AccuScene telemetry system");

        // Start metrics exporter
        if self.config.metrics.enabled && self.config.metrics.prometheus {
            self.metrics.start_exporter().await?;
        }

        // Start health checks
        if self.config.health.enabled {
            self.health.start().await?;
        }

        // Start performance profiling
        if self.config.performance.enabled {
            self.performance.write().start()?;
        }

        tracing::info!("Telemetry system started successfully");
        Ok(())
    }

    /// Stop the telemetry system
    pub async fn stop(&self) -> Result<()> {
        tracing::info!("Stopping AccuScene telemetry system");

        // Stop health checks
        self.health.stop().await?;

        // Stop performance profiling
        self.performance.write().stop()?;

        // Stop metrics exporter
        self.metrics.stop_exporter().await?;

        tracing::info!("Telemetry system stopped");
        Ok(())
    }

    /// Get the logging system
    pub fn logging(&self) -> &LoggingSystem {
        &self.logging
    }

    /// Get the metrics system
    pub fn metrics(&self) -> &MetricsSystem {
        &self.metrics
    }

    /// Get the tracing system
    pub fn tracing(&self) -> &TracingSystem {
        &self.tracing
    }

    /// Get the health system
    pub fn health(&self) -> &HealthSystem {
        &self.health
    }

    /// Get the event logger
    pub fn events(&self) -> &EventLogger {
        &self.events
    }

    /// Get the alert manager
    pub fn alerts(&self) -> Arc<RwLock<AlertManager>> {
        Arc::clone(&self.alerts)
    }

    /// Get the dashboard
    pub fn dashboard(&self) -> Arc<RwLock<Dashboard>> {
        Arc::clone(&self.dashboard)
    }

    /// Record a custom event
    pub fn record_event(&self, event: Event) {
        self.events.record(event);
    }

    /// Get the configuration
    pub fn config(&self) -> &TelemetryConfig {
        &self.config
    }
}

/// Initialize global telemetry with default configuration
pub async fn init() -> Result<Arc<TelemetrySystem>> {
    let config = TelemetryConfig::default();
    let system = TelemetrySystem::new(config).await?;
    system.start().await?;
    Ok(Arc::new(system))
}

/// Initialize global telemetry with custom configuration
pub async fn init_with_config(config: TelemetryConfig) -> Result<Arc<TelemetrySystem>> {
    let system = TelemetrySystem::new(config).await?;
    system.start().await?;
    Ok(Arc::new(system))
}
