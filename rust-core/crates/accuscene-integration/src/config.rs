//! Unified configuration management for AccuScene Enterprise
//!
//! This module provides a centralized configuration system that manages
//! settings for all enterprise components.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

/// Configuration error types
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Failed to load configuration file
    #[error("Failed to load configuration: {0}")]
    LoadError(String),

    /// Invalid configuration value
    #[error("Invalid configuration value: {0}")]
    ValidationError(String),

    /// Missing required configuration
    #[error("Missing required configuration: {0}")]
    MissingError(String),

    /// Environment variable error
    #[error("Environment variable error: {0}")]
    EnvError(String),
}

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Application-level settings
    pub app: AppConfig,

    /// Database configuration
    pub database: DatabaseConfig,

    /// Cache configuration
    pub cache: CacheConfig,

    /// Security configuration
    pub security: SecurityConfig,

    /// Analytics configuration
    pub analytics: AnalyticsConfig,

    /// Cluster configuration
    pub cluster: ClusterConfig,

    /// Telemetry configuration
    pub telemetry: TelemetryConfig,

    /// User experience configuration (v0.2.5)
    pub ux: UxConfig,
}

/// Application-level configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Application name
    pub name: String,

    /// Application version
    pub version: String,

    /// Environment (development, staging, production)
    pub environment: Environment,

    /// Debug mode
    pub debug: bool,

    /// Log level
    pub log_level: LogLevel,

    /// Data directory
    pub data_dir: PathBuf,

    /// Temporary directory
    pub temp_dir: PathBuf,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database URL
    pub url: String,

    /// Maximum number of connections
    pub max_connections: u32,

    /// Minimum number of connections
    pub min_connections: u32,

    /// Connection timeout in seconds
    pub connect_timeout: u64,

    /// Enable migrations
    pub auto_migrate: bool,
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Enable caching
    pub enabled: bool,

    /// Memory cache size in MB
    pub memory_size_mb: usize,

    /// Disk cache size in MB
    pub disk_size_mb: usize,

    /// TTL for cached items in seconds
    pub ttl_seconds: u64,

    /// Enable Redis cache
    pub redis_enabled: bool,

    /// Redis connection string
    pub redis_url: Option<String>,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable authentication
    pub auth_enabled: bool,

    /// JWT secret key
    pub jwt_secret: String,

    /// JWT expiration in seconds
    pub jwt_expiration: u64,

    /// Enable SSO
    pub sso_enabled: bool,

    /// SSO provider
    pub sso_provider: Option<String>,

    /// Enable encryption at rest
    pub encryption_enabled: bool,

    /// Enable audit logging
    pub audit_enabled: bool,
}

/// Analytics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsConfig {
    /// Enable analytics
    pub enabled: bool,

    /// Analytics buffer size
    pub buffer_size: usize,

    /// Batch size for analytics events
    pub batch_size: usize,

    /// Flush interval in seconds
    pub flush_interval: u64,

    /// Enable real-time analytics
    pub realtime_enabled: bool,
}

/// Cluster configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterConfig {
    /// Enable clustering
    pub enabled: bool,

    /// Node ID
    pub node_id: String,

    /// Cluster nodes
    pub nodes: Vec<String>,

    /// Cluster port
    pub port: u16,

    /// Enable auto-discovery
    pub auto_discovery: bool,

    /// Heartbeat interval in seconds
    pub heartbeat_interval: u64,
}

/// Telemetry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    /// Enable telemetry
    pub enabled: bool,

    /// Metrics export interval in seconds
    pub export_interval: u64,

    /// Enable Prometheus exporter
    pub prometheus_enabled: bool,

    /// Prometheus port
    pub prometheus_port: u16,

    /// Enable OpenTelemetry
    pub otel_enabled: bool,

    /// OpenTelemetry endpoint
    pub otel_endpoint: Option<String>,
}

/// User experience configuration (v0.2.5)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UxConfig {
    /// Accessibility configuration
    pub accessibility: AccessibilityConfig,

    /// Dashboard configuration
    pub dashboard: DashboardConfig,

    /// Gestures configuration
    pub gestures: GesturesConfig,

    /// Notifications configuration
    pub notifications: NotificationsConfig,

    /// Offline configuration
    pub offline: OfflineConfig,

    /// Preferences configuration
    pub preferences: PreferencesConfig,

    /// Search configuration
    pub search: SearchConfig,

    /// Visualization configuration
    pub visualization: VisualizationConfig,
}

/// Accessibility configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityConfig {
    /// Enable accessibility features
    pub enabled: bool,

    /// Screen reader support
    pub screen_reader: bool,

    /// High contrast mode
    pub high_contrast: bool,

    /// Keyboard navigation
    pub keyboard_nav: bool,
}

/// Dashboard configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    /// Enable dashboards
    pub enabled: bool,

    /// Refresh interval in seconds
    pub refresh_interval: u64,

    /// Maximum widgets per dashboard
    pub max_widgets: usize,
}

/// Gestures configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GesturesConfig {
    /// Enable gesture recognition
    pub enabled: bool,

    /// Touch sensitivity
    pub sensitivity: f32,

    /// Enable multitouch
    pub multitouch: bool,
}

/// Notifications configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationsConfig {
    /// Enable notifications
    pub enabled: bool,

    /// Maximum notifications to keep
    pub max_notifications: usize,

    /// Enable push notifications
    pub push_enabled: bool,

    /// Push service URL
    pub push_url: Option<String>,
}

/// Offline configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineConfig {
    /// Enable offline mode
    pub enabled: bool,

    /// Offline storage size in MB
    pub storage_size_mb: usize,

    /// Auto-sync on reconnect
    pub auto_sync: bool,

    /// Sync interval in seconds
    pub sync_interval: u64,
}

/// Preferences configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferencesConfig {
    /// Enable user preferences
    pub enabled: bool,

    /// Preferences storage backend
    pub backend: String,

    /// Enable cloud sync
    pub cloud_sync: bool,
}

/// Search configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    /// Enable full-text search
    pub enabled: bool,

    /// Search index size in MB
    pub index_size_mb: usize,

    /// Enable fuzzy search
    pub fuzzy_enabled: bool,

    /// Maximum search results
    pub max_results: usize,
}

/// Visualization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationConfig {
    /// Enable advanced visualizations
    pub enabled: bool,

    /// Rendering backend (webgl, webgpu, canvas)
    pub backend: String,

    /// Enable hardware acceleration
    pub hardware_accel: bool,

    /// Maximum chart points
    pub max_chart_points: usize,
}

/// Environment type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    /// Development environment
    Development,
    /// Staging environment
    Staging,
    /// Production environment
    Production,
}

/// Log level
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    /// Trace level logging
    Trace,
    /// Debug level logging
    Debug,
    /// Info level logging
    Info,
    /// Warn level logging
    Warn,
    /// Error level logging
    Error,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            app: AppConfig::default(),
            database: DatabaseConfig::default(),
            cache: CacheConfig::default(),
            security: SecurityConfig::default(),
            analytics: AnalyticsConfig::default(),
            cluster: ClusterConfig::default(),
            telemetry: TelemetryConfig::default(),
            ux: UxConfig::default(),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            name: "AccuScene Enterprise".to_string(),
            version: crate::ENTERPRISE_VERSION.to_string(),
            environment: Environment::Development,
            debug: true,
            log_level: LogLevel::Info,
            data_dir: PathBuf::from("./data"),
            temp_dir: PathBuf::from("./tmp"),
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "sqlite://./data/accuscene.db".to_string(),
            max_connections: 10,
            min_connections: 2,
            connect_timeout: 30,
            auto_migrate: true,
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            memory_size_mb: 512,
            disk_size_mb: 2048,
            ttl_seconds: 3600,
            redis_enabled: false,
            redis_url: None,
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            auth_enabled: true,
            jwt_secret: "change-me-in-production".to_string(),
            jwt_expiration: 86400, // 24 hours
            sso_enabled: false,
            sso_provider: None,
            encryption_enabled: true,
            audit_enabled: true,
        }
    }
}

impl Default for AnalyticsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            buffer_size: 10000,
            batch_size: 100,
            flush_interval: 60,
            realtime_enabled: true,
        }
    }
}

impl Default for ClusterConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            node_id: uuid::Uuid::new_v4().to_string(),
            nodes: vec![],
            port: 7946,
            auto_discovery: true,
            heartbeat_interval: 30,
        }
    }
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            export_interval: 60,
            prometheus_enabled: true,
            prometheus_port: 9090,
            otel_enabled: false,
            otel_endpoint: None,
        }
    }
}

impl Default for UxConfig {
    fn default() -> Self {
        Self {
            accessibility: AccessibilityConfig::default(),
            dashboard: DashboardConfig::default(),
            gestures: GesturesConfig::default(),
            notifications: NotificationsConfig::default(),
            offline: OfflineConfig::default(),
            preferences: PreferencesConfig::default(),
            search: SearchConfig::default(),
            visualization: VisualizationConfig::default(),
        }
    }
}

impl Default for AccessibilityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            screen_reader: true,
            high_contrast: false,
            keyboard_nav: true,
        }
    }
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            refresh_interval: 30,
            max_widgets: 20,
        }
    }
}

impl Default for GesturesConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sensitivity: 0.8,
            multitouch: true,
        }
    }
}

impl Default for NotificationsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_notifications: 100,
            push_enabled: false,
            push_url: None,
        }
    }
}

impl Default for OfflineConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            storage_size_mb: 1024,
            auto_sync: true,
            sync_interval: 300,
        }
    }
}

impl Default for PreferencesConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            backend: "sqlite".to_string(),
            cloud_sync: false,
        }
    }
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            index_size_mb: 256,
            fuzzy_enabled: true,
            max_results: 100,
        }
    }
}

impl Default for VisualizationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            backend: "webgl".to_string(),
            hardware_accel: true,
            max_chart_points: 10000,
        }
    }
}

/// Configuration loader trait
pub trait ConfigLoader {
    /// Load configuration from a source
    fn load(&self) -> Result<Config, ConfigError>;
}

/// Default configuration loader
pub struct DefaultConfigLoader {
    config_path: Option<PathBuf>,
}

impl DefaultConfigLoader {
    /// Create a new configuration loader
    pub fn new(config_path: Option<PathBuf>) -> Self {
        Self { config_path }
    }
}

impl ConfigLoader for DefaultConfigLoader {
    fn load(&self) -> Result<Config, ConfigError> {
        let mut config = Config::default();

        // Load from file if path is provided
        if let Some(path) = &self.config_path {
            let contents = std::fs::read_to_string(path)
                .map_err(|e| ConfigError::LoadError(e.to_string()))?;

            config = toml::from_str(&contents)
                .map_err(|e| ConfigError::LoadError(e.to_string()))?;
        }

        // Override with environment variables
        if let Ok(env) = std::env::var("ACCUSCENE_ENV") {
            config.app.environment = match env.as_str() {
                "development" => Environment::Development,
                "staging" => Environment::Staging,
                "production" => Environment::Production,
                _ => Environment::Development,
            };
        }

        if let Ok(db_url) = std::env::var("DATABASE_URL") {
            config.database.url = db_url;
        }

        if let Ok(jwt_secret) = std::env::var("JWT_SECRET") {
            config.security.jwt_secret = jwt_secret;
        }

        Ok(config)
    }
}

impl Config {
    /// Load configuration with default loader
    pub fn load() -> Result<Self, ConfigError> {
        let loader = DefaultConfigLoader::new(Some(PathBuf::from("./config.toml")));
        loader.load()
    }

    /// Load configuration from a specific path
    pub fn load_from(path: PathBuf) -> Result<Self, ConfigError> {
        let loader = DefaultConfigLoader::new(Some(path));
        loader.load()
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate security settings
        if self.security.auth_enabled && self.security.jwt_secret == "change-me-in-production" {
            if self.app.environment == Environment::Production {
                return Err(ConfigError::ValidationError(
                    "JWT secret must be changed in production".to_string(),
                ));
            }
        }

        // Validate database settings
        if self.database.max_connections < self.database.min_connections {
            return Err(ConfigError::ValidationError(
                "Max connections must be >= min connections".to_string(),
            ));
        }

        Ok(())
    }
}
