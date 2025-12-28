//! Configuration for the event sourcing system.

use crate::error::{EventSourcingError, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Configuration for the event sourcing system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSourcingConfig {
    /// Event store configuration.
    pub event_store: EventStoreConfig,

    /// Snapshot configuration.
    pub snapshot: SnapshotConfig,

    /// Projection configuration.
    pub projection: ProjectionConfig,

    /// Bus configuration.
    pub bus: BusConfig,

    /// Saga configuration.
    pub saga: SagaConfig,
}

impl EventSourcingConfig {
    /// Creates a new configuration with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Loads configuration from a file.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = std::fs::read_to_string(path)
            .map_err(|e| EventSourcingError::configuration(e))?;

        Self::from_str(&contents)
    }

    /// Parses configuration from a string.
    pub fn from_str(s: &str) -> Result<Self> {
        serde_json::from_str(s).map_err(|e| EventSourcingError::configuration(e))
    }

    /// Saves configuration to a file.
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let contents =
            serde_json::to_string_pretty(self).map_err(|e| EventSourcingError::configuration(e))?;

        std::fs::write(path, contents).map_err(|e| EventSourcingError::configuration(e))?;

        Ok(())
    }

    /// Validates the configuration.
    pub fn validate(&self) -> Result<()> {
        self.event_store.validate()?;
        self.snapshot.validate()?;
        self.projection.validate()?;
        self.bus.validate()?;
        self.saga.validate()?;
        Ok(())
    }
}

impl Default for EventSourcingConfig {
    fn default() -> Self {
        Self {
            event_store: EventStoreConfig::default(),
            snapshot: SnapshotConfig::default(),
            projection: ProjectionConfig::default(),
            bus: BusConfig::default(),
            saga: SagaConfig::default(),
        }
    }
}

/// Event store configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventStoreConfig {
    /// Store type (memory, postgres).
    pub store_type: StoreType,

    /// Database connection URL (for postgres).
    pub database_url: Option<String>,

    /// Maximum number of database connections.
    pub max_connections: u32,

    /// Enable event batching.
    pub enable_batching: bool,

    /// Batch size for event writes.
    pub batch_size: usize,

    /// Batch timeout in milliseconds.
    pub batch_timeout_ms: u64,
}

impl EventStoreConfig {
    /// Validates the configuration.
    pub fn validate(&self) -> Result<()> {
        if matches!(self.store_type, StoreType::Postgres) && self.database_url.is_none() {
            return Err(EventSourcingError::configuration(
                "database_url required for postgres store",
            ));
        }

        if self.max_connections == 0 {
            return Err(EventSourcingError::configuration(
                "max_connections must be greater than 0",
            ));
        }

        if self.batch_size == 0 {
            return Err(EventSourcingError::configuration(
                "batch_size must be greater than 0",
            ));
        }

        Ok(())
    }
}

impl Default for EventStoreConfig {
    fn default() -> Self {
        Self {
            store_type: StoreType::Memory,
            database_url: None,
            max_connections: 10,
            enable_batching: false,
            batch_size: 100,
            batch_timeout_ms: 1000,
        }
    }
}

/// Store type enumeration.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum StoreType {
    /// In-memory store.
    Memory,

    /// PostgreSQL store.
    Postgres,
}

/// Snapshot configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotConfig {
    /// Enable snapshotting.
    pub enabled: bool,

    /// Snapshot strategy.
    pub strategy: SnapshotStrategyConfig,

    /// Snapshot interval (events between snapshots).
    pub interval: u64,

    /// Enable snapshot compression.
    pub compression: bool,

    /// Compression algorithm.
    pub compression_algorithm: Option<String>,
}

impl SnapshotConfig {
    /// Validates the configuration.
    pub fn validate(&self) -> Result<()> {
        if self.enabled && self.interval == 0 {
            return Err(EventSourcingError::configuration(
                "snapshot interval must be greater than 0",
            ));
        }
        Ok(())
    }
}

impl Default for SnapshotConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            strategy: SnapshotStrategyConfig::EveryNEvents,
            interval: 100,
            compression: false,
            compression_algorithm: None,
        }
    }
}

/// Snapshot strategy configuration.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SnapshotStrategyConfig {
    /// Take snapshot every N events.
    EveryNEvents,

    /// Never take snapshots.
    Never,

    /// Always take snapshots.
    Always,
}

/// Projection configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionConfig {
    /// Enable projections.
    pub enabled: bool,

    /// Auto-start projections on system startup.
    pub auto_start: bool,

    /// Projection update interval in milliseconds.
    pub update_interval_ms: u64,

    /// Enable projection rebuilding.
    pub enable_rebuild: bool,

    /// Maximum concurrent projections.
    pub max_concurrent: usize,
}

impl ProjectionConfig {
    /// Validates the configuration.
    pub fn validate(&self) -> Result<()> {
        if self.max_concurrent == 0 {
            return Err(EventSourcingError::configuration(
                "max_concurrent must be greater than 0",
            ));
        }
        Ok(())
    }
}

impl Default for ProjectionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_start: true,
            update_interval_ms: 1000,
            enable_rebuild: true,
            max_concurrent: 10,
        }
    }
}

/// Bus configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusConfig {
    /// Event bus configuration.
    pub event_bus: EventBusConfig,

    /// Command bus configuration.
    pub command_bus: CommandBusConfig,
}

impl BusConfig {
    /// Validates the configuration.
    pub fn validate(&self) -> Result<()> {
        self.event_bus.validate()?;
        self.command_bus.validate()?;
        Ok(())
    }
}

impl Default for BusConfig {
    fn default() -> Self {
        Self {
            event_bus: EventBusConfig::default(),
            command_bus: CommandBusConfig::default(),
        }
    }
}

/// Event bus configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventBusConfig {
    /// Buffer size for event channel.
    pub buffer_size: usize,

    /// Enable async event processing.
    pub async_processing: bool,

    /// Maximum retry attempts for failed event handlers.
    pub max_retries: u32,

    /// Retry delay in milliseconds.
    pub retry_delay_ms: u64,
}

impl EventBusConfig {
    /// Validates the configuration.
    pub fn validate(&self) -> Result<()> {
        if self.buffer_size == 0 {
            return Err(EventSourcingError::configuration(
                "buffer_size must be greater than 0",
            ));
        }
        Ok(())
    }
}

impl Default for EventBusConfig {
    fn default() -> Self {
        Self {
            buffer_size: 1000,
            async_processing: true,
            max_retries: 3,
            retry_delay_ms: 100,
        }
    }
}

/// Command bus configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandBusConfig {
    /// Enable command validation.
    pub enable_validation: bool,

    /// Enable command middleware.
    pub enable_middleware: bool,

    /// Command timeout in milliseconds.
    pub timeout_ms: u64,

    /// Maximum retry attempts for failed commands.
    pub max_retries: u32,
}

impl CommandBusConfig {
    /// Validates the configuration.
    pub fn validate(&self) -> Result<()> {
        if self.timeout_ms == 0 {
            return Err(EventSourcingError::configuration(
                "timeout_ms must be greater than 0",
            ));
        }
        Ok(())
    }
}

impl Default for CommandBusConfig {
    fn default() -> Self {
        Self {
            enable_validation: true,
            enable_middleware: true,
            timeout_ms: 30000,
            max_retries: 3,
        }
    }
}

/// Saga configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaConfig {
    /// Enable saga orchestration.
    pub enabled: bool,

    /// Saga timeout in milliseconds.
    pub timeout_ms: u64,

    /// Enable automatic compensation on failure.
    pub auto_compensate: bool,

    /// Maximum saga instances to keep in memory.
    pub max_instances: usize,

    /// Cleanup interval for completed sagas in minutes.
    pub cleanup_interval_minutes: u64,
}

impl SagaConfig {
    /// Validates the configuration.
    pub fn validate(&self) -> Result<()> {
        if self.max_instances == 0 {
            return Err(EventSourcingError::configuration(
                "max_instances must be greater than 0",
            ));
        }
        Ok(())
    }
}

impl Default for SagaConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            timeout_ms: 300000, // 5 minutes
            auto_compensate: true,
            max_instances: 1000,
            cleanup_interval_minutes: 60,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = EventSourcingConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_event_store_config_validation() {
        let mut config = EventStoreConfig::default();
        config.store_type = StoreType::Postgres;
        assert!(config.validate().is_err());

        config.database_url = Some("postgres://localhost".to_string());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_snapshot_config() {
        let config = SnapshotConfig::default();
        assert!(config.validate().is_ok());

        let mut invalid_config = SnapshotConfig::default();
        invalid_config.interval = 0;
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_config_serialization() {
        let config = EventSourcingConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: EventSourcingConfig = serde_json::from_str(&json).unwrap();
        assert!(deserialized.validate().is_ok());
    }
}
