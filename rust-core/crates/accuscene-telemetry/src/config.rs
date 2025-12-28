//! Telemetry configuration

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Complete telemetry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    /// Logging configuration
    pub logging: LoggingConfig,

    /// Metrics configuration
    pub metrics: MetricsConfig,

    /// Tracing configuration
    pub tracing: TracingConfig,

    /// Health check configuration
    pub health: HealthConfig,

    /// Performance profiling configuration
    pub performance: PerformanceConfig,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            logging: LoggingConfig::default(),
            metrics: MetricsConfig::default(),
            tracing: TracingConfig::default(),
            health: HealthConfig::default(),
            performance: PerformanceConfig::default(),
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: String,

    /// Log format (json, text)
    pub format: LogFormat,

    /// Enable file logging
    pub file_logging: bool,

    /// Log file directory
    pub log_dir: PathBuf,

    /// Log file name prefix
    pub file_prefix: String,

    /// Enable log rotation
    pub rotation: bool,

    /// Maximum log file size in MB
    pub max_file_size_mb: u64,

    /// Maximum number of log files to keep
    pub max_files: usize,

    /// Enable console logging
    pub console: bool,

    /// Enable ANSI colors in console
    pub ansi: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: LogFormat::Text,
            file_logging: true,
            log_dir: PathBuf::from("./logs"),
            file_prefix: "accuscene".to_string(),
            rotation: true,
            max_file_size_mb: 100,
            max_files: 10,
            console: true,
            ansi: true,
        }
    }
}

/// Log format
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    /// JSON format
    Json,
    /// Plain text format
    Text,
}

/// Metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Enable metrics collection
    pub enabled: bool,

    /// Enable Prometheus exporter
    pub prometheus: bool,

    /// Prometheus endpoint address
    pub prometheus_addr: String,

    /// Metrics collection interval in seconds
    pub collection_interval_secs: u64,

    /// Enable system metrics (CPU, memory, etc.)
    pub system_metrics: bool,

    /// Enable application metrics
    pub app_metrics: bool,

    /// Custom metric prefix
    pub prefix: String,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            prometheus: true,
            prometheus_addr: "0.0.0.0:9090".to_string(),
            collection_interval_secs: 60,
            system_metrics: true,
            app_metrics: true,
            prefix: "accuscene".to_string(),
        }
    }
}

/// Tracing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    /// Enable distributed tracing
    pub enabled: bool,

    /// Service name for tracing
    pub service_name: String,

    /// Trace sampling rate (0.0 to 1.0)
    pub sampling_rate: f64,

    /// Enable span context propagation
    pub propagation: bool,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            service_name: "accuscene".to_string(),
            sampling_rate: 1.0,
            propagation: true,
        }
    }
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthConfig {
    /// Enable health checks
    pub enabled: bool,

    /// Health check interval in seconds
    pub interval_secs: u64,

    /// Timeout for health checks in seconds
    pub timeout_secs: u64,

    /// Enable liveness probe
    pub liveness: bool,

    /// Enable readiness probe
    pub readiness: bool,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_secs: 30,
            timeout_secs: 5,
            liveness: true,
            readiness: true,
        }
    }
}

/// Performance profiling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Enable performance profiling
    pub enabled: bool,

    /// Enable CPU profiling
    pub cpu_profiling: bool,

    /// Enable memory profiling
    pub memory_profiling: bool,

    /// Profiling sample rate
    pub sample_rate_hz: u64,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cpu_profiling: true,
            memory_profiling: true,
            sample_rate_hz: 100,
        }
    }
}

impl TelemetryConfig {
    /// Create a new configuration with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Load configuration from a file
    pub fn from_file(path: impl AsRef<std::path::Path>) -> crate::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to a file
    pub fn save(&self, path: impl AsRef<std::path::Path>) -> crate::Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Validate the configuration
    pub fn validate(&self) -> crate::Result<()> {
        // Validate log level
        let valid_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_levels.contains(&self.logging.level.as_str()) {
            return Err(crate::error::TelemetryError::config(
                format!("Invalid log level: {}", self.logging.level)
            ));
        }

        // Validate sampling rate
        if self.tracing.sampling_rate < 0.0 || self.tracing.sampling_rate > 1.0 {
            return Err(crate::error::TelemetryError::config(
                "Sampling rate must be between 0.0 and 1.0"
            ));
        }

        Ok(())
    }
}
