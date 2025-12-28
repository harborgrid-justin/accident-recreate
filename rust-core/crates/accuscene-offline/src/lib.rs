//! # AccuScene Offline Sync
//!
//! Comprehensive offline synchronization system with CRDT-based conflict resolution,
//! vector clock versioning, and efficient delta encoding for AccuScene Enterprise.
//!
//! ## Features
//!
//! - **Distributed Versioning**: Vector clocks for causality tracking
//! - **Conflict Resolution**: Multiple strategies including operational transformation
//! - **Delta Encoding**: Bandwidth-optimized differential sync
//! - **Priority Queue**: Smart operation queuing with dependency management
//! - **Multiple Storage Backends**: SQLite and RocksDB support
//! - **Network Resilience**: Automatic retry with exponential backoff
//! - **Optimistic Updates**: Seamless offline/online transitions
//!
//! ## Example
//!
//! ```no_run
//! use accuscene_offline::{OfflineConfig, SyncEngine, SqliteStorage};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create configuration
//!     let config = OfflineConfig::default();
//!
//!     // Initialize storage
//!     let storage = SqliteStorage::new("./offline.db")?;
//!     storage.init().await?;
//!
//!     // Create sync engine
//!     let engine = SyncEngine::new(config, storage);
//!
//!     // Start syncing
//!     engine.start_sync().await?;
//!
//!     Ok(())
//! }
//! ```

pub mod config;
pub mod error;
pub mod network;
pub mod storage;
pub mod sync;
pub mod versioning;

// Re-export commonly used types
pub use config::{
    CompressionAlgorithm, ConflictResolution, OfflineConfig, PerformanceConfig, RetryConfig,
    StorageBackend, StorageConfig, SyncConfig, SynchronousMode,
};

pub use error::{OfflineError, Result};

pub use network::{CircuitBreaker, NetworkDetector, NetworkQuality, NetworkState, RetryPolicy};

pub use storage::{Storage, StorageRecord};
pub use storage::sqlite::SqliteStorage;

#[cfg(feature = "rocksdb-backend")]
pub use storage::rocksdb::RocksDbStorage;

pub use sync::{
    Conflict, ConflictResolver, Delta, DeltaEncoder, DeltaSyncManager, Diff, DiffEngine, DiffOp,
    OperationQueue, OperationType, Priority, ResolutionResult, SyncEngine, SyncOperation,
    SyncStats, SyncStatus,
};

pub use versioning::{NodeId, Ordering, VectorClock, Version};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize logging for the library
pub fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[tokio::test]
    async fn test_basic_workflow() {
        let config = OfflineConfig::default();
        let storage = SqliteStorage::in_memory().unwrap();
        storage.init().await.unwrap();

        let engine = SyncEngine::new(config, storage);

        assert_eq!(engine.status(), SyncStatus::Idle);
    }
}
