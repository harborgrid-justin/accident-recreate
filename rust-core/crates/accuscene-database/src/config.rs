//! Database configuration for AccuScene Enterprise
//!
//! Provides comprehensive configuration options for database connections,
//! connection pooling, performance tuning, and feature flags.

use crate::error::{DatabaseError, DbResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database connection URL or file path
    pub url: String,

    /// Connection pool configuration
    pub pool: PoolConfig,

    /// Performance tuning options
    pub performance: PerformanceConfig,

    /// Feature flags
    pub features: FeatureConfig,

    /// Backup configuration
    pub backup: BackupConfig,
}

/// Connection pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    /// Maximum number of connections in the pool
    pub max_size: u32,

    /// Minimum number of idle connections
    pub min_idle: Option<u32>,

    /// Connection timeout in seconds
    pub connection_timeout: u64,

    /// Maximum connection lifetime in seconds
    pub max_lifetime: Option<u64>,

    /// Idle connection timeout in seconds
    pub idle_timeout: Option<u64>,

    /// Health check interval in seconds
    pub health_check_interval: u64,
}

/// Performance tuning configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Enable WAL (Write-Ahead Logging) mode for SQLite
    pub wal_mode: bool,

    /// Synchronous mode (OFF, NORMAL, FULL, EXTRA)
    pub synchronous: SynchronousMode,

    /// Cache size in pages (-KB for KB)
    pub cache_size: i32,

    /// Memory-mapped I/O size in bytes (0 to disable)
    pub mmap_size: i64,

    /// Page size in bytes
    pub page_size: u32,

    /// Enable automatic vacuum
    pub auto_vacuum: AutoVacuum,

    /// Journal mode
    pub journal_mode: JournalMode,

    /// Busy timeout in milliseconds
    pub busy_timeout: u64,
}

/// Synchronous mode for SQLite
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SynchronousMode {
    Off,
    Normal,
    Full,
    Extra,
}

impl SynchronousMode {
    pub fn as_str(&self) -> &str {
        match self {
            SynchronousMode::Off => "OFF",
            SynchronousMode::Normal => "NORMAL",
            SynchronousMode::Full => "FULL",
            SynchronousMode::Extra => "EXTRA",
        }
    }
}

/// Auto vacuum mode
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AutoVacuum {
    None,
    Full,
    Incremental,
}

impl AutoVacuum {
    pub fn as_str(&self) -> &str {
        match self {
            AutoVacuum::None => "NONE",
            AutoVacuum::Full => "FULL",
            AutoVacuum::Incremental => "INCREMENTAL",
        }
    }
}

/// Journal mode
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum JournalMode {
    Delete,
    Truncate,
    Persist,
    Memory,
    Wal,
    Off,
}

impl JournalMode {
    pub fn as_str(&self) -> &str {
        match self {
            JournalMode::Delete => "DELETE",
            JournalMode::Truncate => "TRUNCATE",
            JournalMode::Persist => "PERSIST",
            JournalMode::Memory => "MEMORY",
            JournalMode::Wal => "WAL",
            JournalMode::Off => "OFF",
        }
    }
}

/// Feature flags configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureConfig {
    /// Enable full-text search
    pub full_text_search: bool,

    /// Enable audit logging
    pub audit_logging: bool,

    /// Enable automatic backups
    pub auto_backup: bool,

    /// Enable query logging
    pub query_logging: bool,

    /// Enable connection pooling health checks
    pub pool_health_checks: bool,

    /// Enable automatic migrations on startup
    pub auto_migrate: bool,
}

/// Backup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    /// Backup directory path
    pub directory: PathBuf,

    /// Backup interval in seconds (0 to disable)
    pub interval: u64,

    /// Maximum number of backups to retain
    pub retention_count: u32,

    /// Compress backups
    pub compress: bool,

    /// Backup on shutdown
    pub backup_on_shutdown: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "accuscene.db".to_string(),
            pool: PoolConfig::default(),
            performance: PerformanceConfig::default(),
            features: FeatureConfig::default(),
            backup: BackupConfig::default(),
        }
    }
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_size: 32,
            min_idle: Some(4),
            connection_timeout: 30,
            max_lifetime: Some(1800), // 30 minutes
            idle_timeout: Some(600),   // 10 minutes
            health_check_interval: 60,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            wal_mode: true,
            synchronous: SynchronousMode::Normal,
            cache_size: 10000, // ~40MB with 4KB pages
            mmap_size: 268435456, // 256MB
            page_size: 4096,
            auto_vacuum: AutoVacuum::Incremental,
            journal_mode: JournalMode::Wal,
            busy_timeout: 5000,
        }
    }
}

impl Default for FeatureConfig {
    fn default() -> Self {
        Self {
            full_text_search: true,
            audit_logging: true,
            auto_backup: true,
            query_logging: false,
            pool_health_checks: true,
            auto_migrate: true,
        }
    }
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            directory: PathBuf::from("./backups"),
            interval: 3600, // 1 hour
            retention_count: 24, // Keep 24 backups (1 day if hourly)
            compress: true,
            backup_on_shutdown: true,
        }
    }
}

impl DatabaseConfig {
    /// Create a new configuration from a URL
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            ..Default::default()
        }
    }

    /// Create an in-memory database configuration
    pub fn in_memory() -> Self {
        Self {
            url: ":memory:".to_string(),
            pool: PoolConfig {
                max_size: 1, // In-memory databases should use a single connection
                ..Default::default()
            },
            performance: PerformanceConfig {
                journal_mode: JournalMode::Memory,
                ..Default::default()
            },
            features: FeatureConfig {
                auto_backup: false, // Can't backup in-memory databases
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Create a configuration for testing
    pub fn test() -> Self {
        Self {
            url: ":memory:".to_string(),
            pool: PoolConfig {
                max_size: 4,
                ..Default::default()
            },
            performance: PerformanceConfig {
                journal_mode: JournalMode::Memory,
                synchronous: SynchronousMode::Off,
                ..Default::default()
            },
            features: FeatureConfig {
                auto_backup: false,
                query_logging: true,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Create a high-performance configuration
    pub fn high_performance(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            pool: PoolConfig {
                max_size: 64,
                min_idle: Some(8),
                ..Default::default()
            },
            performance: PerformanceConfig {
                wal_mode: true,
                synchronous: SynchronousMode::Normal,
                cache_size: 50000, // ~200MB
                mmap_size: 1073741824, // 1GB
                journal_mode: JournalMode::Wal,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> DbResult<()> {
        if self.pool.max_size == 0 {
            return Err(DatabaseError::ConfigError(
                "Pool max_size must be greater than 0".to_string(),
            ));
        }

        if let Some(min_idle) = self.pool.min_idle {
            if min_idle > self.pool.max_size {
                return Err(DatabaseError::ConfigError(
                    "Pool min_idle cannot exceed max_size".to_string(),
                ));
            }
        }

        if self.performance.page_size < 512 || self.performance.page_size > 65536 {
            return Err(DatabaseError::ConfigError(
                "Page size must be between 512 and 65536 bytes".to_string(),
            ));
        }

        // Check if page size is a power of 2
        if !self.performance.page_size.is_power_of_two() {
            return Err(DatabaseError::ConfigError(
                "Page size must be a power of 2".to_string(),
            ));
        }

        Ok(())
    }

    /// Get connection timeout as Duration
    pub fn connection_timeout(&self) -> Duration {
        Duration::from_secs(self.pool.connection_timeout)
    }

    /// Get max lifetime as Duration
    pub fn max_lifetime(&self) -> Option<Duration> {
        self.pool.max_lifetime.map(Duration::from_secs)
    }

    /// Get idle timeout as Duration
    pub fn idle_timeout(&self) -> Option<Duration> {
        self.pool.idle_timeout.map(Duration::from_secs)
    }

    /// Get busy timeout as Duration
    pub fn busy_timeout(&self) -> Duration {
        Duration::from_millis(self.performance.busy_timeout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DatabaseConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_in_memory_config() {
        let config = DatabaseConfig::in_memory();
        assert_eq!(config.url, ":memory:");
        assert_eq!(config.pool.max_size, 1);
        assert!(!config.features.auto_backup);
    }

    #[test]
    fn test_test_config() {
        let config = DatabaseConfig::test();
        assert!(config.features.query_logging);
        assert!(!config.features.auto_backup);
    }

    #[test]
    fn test_validation_invalid_pool_size() {
        let mut config = DatabaseConfig::default();
        config.pool.max_size = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_invalid_min_idle() {
        let mut config = DatabaseConfig::default();
        config.pool.min_idle = Some(100);
        config.pool.max_size = 10;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_invalid_page_size() {
        let mut config = DatabaseConfig::default();
        config.performance.page_size = 100;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_high_performance_config() {
        let config = DatabaseConfig::high_performance("test.db");
        assert_eq!(config.pool.max_size, 64);
        assert!(config.performance.wal_mode);
    }
}
