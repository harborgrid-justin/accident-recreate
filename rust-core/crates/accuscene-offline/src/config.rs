use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for offline sync system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineConfig {
    /// Node identifier for this client
    pub node_id: String,

    /// Storage backend configuration
    pub storage: StorageConfig,

    /// Sync configuration
    pub sync: SyncConfig,

    /// Network configuration
    pub network: NetworkConfig,

    /// Retry configuration
    pub retry: RetryConfig,

    /// Performance tuning
    pub performance: PerformanceConfig,
}

impl Default for OfflineConfig {
    fn default() -> Self {
        Self {
            node_id: uuid::Uuid::new_v4().to_string(),
            storage: StorageConfig::default(),
            sync: SyncConfig::default(),
            network: NetworkConfig::default(),
            retry: RetryConfig::default(),
            performance: PerformanceConfig::default(),
        }
    }
}

/// Storage backend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Backend type (sqlite, rocksdb)
    pub backend: StorageBackend,

    /// Database path
    pub db_path: String,

    /// Maximum database size in bytes
    pub max_size: u64,

    /// Enable WAL mode for SQLite
    pub wal_mode: bool,

    /// Cache size in pages
    pub cache_size: usize,

    /// Synchronous mode
    pub synchronous: SynchronousMode,

    /// Auto-vacuum configuration
    pub auto_vacuum: bool,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            backend: StorageBackend::Sqlite,
            db_path: "./accuscene_offline.db".to_string(),
            max_size: 10 * 1024 * 1024 * 1024, // 10 GB
            wal_mode: true,
            cache_size: 10000,
            synchronous: SynchronousMode::Normal,
            auto_vacuum: true,
        }
    }
}

/// Storage backend types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageBackend {
    Sqlite,
    RocksDb,
}

/// SQLite synchronous modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SynchronousMode {
    Off,
    Normal,
    Full,
    Extra,
}

/// Sync configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// Enable automatic sync
    pub auto_sync: bool,

    /// Sync interval in milliseconds
    pub sync_interval_ms: u64,

    /// Batch size for sync operations
    pub batch_size: usize,

    /// Maximum pending operations before blocking
    pub max_pending_operations: usize,

    /// Enable delta encoding
    pub delta_encoding: bool,

    /// Enable compression
    pub compression: bool,

    /// Compression algorithm
    pub compression_algorithm: CompressionAlgorithm,

    /// Conflict resolution strategy
    pub conflict_resolution: ConflictResolution,

    /// Enable optimistic locking
    pub optimistic_locking: bool,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            auto_sync: true,
            sync_interval_ms: 30000, // 30 seconds
            batch_size: 100,
            max_pending_operations: 10000,
            delta_encoding: true,
            compression: true,
            compression_algorithm: CompressionAlgorithm::Lz4,
            conflict_resolution: ConflictResolution::LastWriteWins,
            optimistic_locking: true,
        }
    }
}

/// Compression algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionAlgorithm {
    None,
    Gzip,
    Lz4,
    Zstd,
}

/// Conflict resolution strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictResolution {
    /// Last write wins based on timestamp
    LastWriteWins,

    /// First write wins
    FirstWriteWins,

    /// Server version always wins
    ServerWins,

    /// Client version always wins
    ClientWins,

    /// Manual resolution required
    Manual,

    /// Merge using operational transformation
    OperationalTransform,

    /// Use custom resolution logic
    Custom,
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// API endpoint
    pub api_endpoint: String,

    /// Connection timeout in milliseconds
    pub connect_timeout_ms: u64,

    /// Request timeout in milliseconds
    pub request_timeout_ms: u64,

    /// Enable network state monitoring
    pub monitor_network_state: bool,

    /// Network check interval in milliseconds
    pub network_check_interval_ms: u64,

    /// Minimum network quality threshold (0.0 - 1.0)
    pub min_network_quality: f32,

    /// Enable bandwidth optimization
    pub bandwidth_optimization: bool,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            api_endpoint: "https://api.accuscene.com".to_string(),
            connect_timeout_ms: 10000,
            request_timeout_ms: 30000,
            monitor_network_state: true,
            network_check_interval_ms: 5000,
            min_network_quality: 0.5,
            bandwidth_optimization: true,
        }
    }
}

/// Retry configuration with exponential backoff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: usize,

    /// Initial retry delay in milliseconds
    pub initial_delay_ms: u64,

    /// Maximum retry delay in milliseconds
    pub max_delay_ms: u64,

    /// Backoff multiplier
    pub multiplier: f64,

    /// Jitter factor (0.0 - 1.0)
    pub jitter: f64,

    /// Retry on specific error types
    pub retry_on_conflict: bool,
    pub retry_on_network_error: bool,
    pub retry_on_timeout: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            initial_delay_ms: 1000,
            max_delay_ms: 60000,
            multiplier: 2.0,
            jitter: 0.1,
            retry_on_conflict: true,
            retry_on_network_error: true,
            retry_on_timeout: true,
        }
    }
}

impl RetryConfig {
    /// Calculate retry delay for attempt number
    pub fn calculate_delay(&self, attempt: usize) -> Duration {
        if attempt == 0 {
            return Duration::from_millis(0);
        }

        let base_delay = self.initial_delay_ms as f64 * self.multiplier.powi(attempt as i32 - 1);
        let capped_delay = base_delay.min(self.max_delay_ms as f64);

        // Add jitter
        let jitter = capped_delay * self.jitter * (rand::random::<f64>() * 2.0 - 1.0);
        let final_delay = (capped_delay + jitter).max(0.0);

        Duration::from_millis(final_delay as u64)
    }
}

/// Performance tuning configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Number of worker threads
    pub worker_threads: usize,

    /// Enable parallel sync
    pub parallel_sync: bool,

    /// Maximum concurrent sync operations
    pub max_concurrent_syncs: usize,

    /// Enable write coalescing
    pub write_coalescing: bool,

    /// Write coalescing delay in milliseconds
    pub write_coalescing_delay_ms: u64,

    /// Enable prefetching
    pub prefetch: bool,

    /// Prefetch batch size
    pub prefetch_batch_size: usize,

    /// Memory cache size in bytes
    pub memory_cache_size: usize,

    /// Enable metrics collection
    pub metrics_enabled: bool,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            worker_threads: num_cpus::get(),
            parallel_sync: true,
            max_concurrent_syncs: 10,
            write_coalescing: true,
            write_coalescing_delay_ms: 100,
            prefetch: true,
            prefetch_batch_size: 50,
            memory_cache_size: 100 * 1024 * 1024, // 100 MB
            metrics_enabled: true,
        }
    }
}

// Add rand dependency for jitter calculation
use rand;

// Add num_cpus for worker thread detection
mod num_cpus {
    pub fn get() -> usize {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
    }
}
