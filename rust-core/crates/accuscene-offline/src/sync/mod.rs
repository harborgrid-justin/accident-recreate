pub mod queue;
pub mod resolver;
pub mod diff;
pub mod delta;

use crate::config::{OfflineConfig, SyncConfig};
use crate::error::{OfflineError, Result};
use crate::network::detector::NetworkDetector;
use crate::network::retry::RetryPolicy;
use crate::storage::Storage;
use crate::versioning::Version;

pub use queue::{OperationQueue, Priority, SyncOperation, OperationType};
pub use resolver::{Conflict, ConflictResolver, ResolutionResult};
pub use diff::{Diff, DiffEngine, DiffOp};
pub use delta::{Delta, DeltaEncoder, DeltaSyncManager};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use parking_lot::RwLock;

/// Sync status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncStatus {
    /// Idle, not syncing
    Idle,

    /// Currently syncing
    Syncing,

    /// Sync completed successfully
    Completed,

    /// Sync failed
    Failed,

    /// Paused
    Paused,

    /// Waiting for network
    WaitingForNetwork,
}

/// Sync statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SyncStats {
    /// Total operations synced
    pub total_synced: usize,

    /// Operations pending
    pub pending: usize,

    /// Operations failed
    pub failed: usize,

    /// Conflicts encountered
    pub conflicts: usize,

    /// Conflicts resolved
    pub conflicts_resolved: usize,

    /// Bytes uploaded
    pub bytes_uploaded: u64,

    /// Bytes downloaded
    pub bytes_downloaded: u64,

    /// Last sync timestamp
    pub last_sync: Option<chrono::DateTime<chrono::Utc>>,

    /// Sync duration in milliseconds
    pub last_sync_duration_ms: Option<u64>,
}

/// Sync engine - orchestrates all sync operations
pub struct SyncEngine<S: Storage> {
    /// Configuration
    config: Arc<OfflineConfig>,

    /// Operation queue
    queue: Arc<OperationQueue>,

    /// Conflict resolver
    resolver: Arc<RwLock<ConflictResolver>>,

    /// Delta sync manager
    delta_manager: Arc<DeltaSyncManager>,

    /// Storage backend
    storage: Arc<S>,

    /// Network detector
    network: Arc<NetworkDetector>,

    /// Retry policy
    retry_policy: Arc<RetryPolicy>,

    /// Current sync status
    status: Arc<RwLock<SyncStatus>>,

    /// Sync statistics
    stats: Arc<RwLock<SyncStats>>,
}

impl<S: Storage> SyncEngine<S> {
    /// Create a new sync engine
    pub fn new(
        config: OfflineConfig,
        storage: S,
    ) -> Self {
        let queue = Arc::new(OperationQueue::new(config.sync.max_pending_operations));

        let resolver = Arc::new(RwLock::new(
            ConflictResolver::new(config.sync.conflict_resolution)
        ));

        let delta_manager = Arc::new(DeltaSyncManager::new(
            config.sync.compression_algorithm,
            100, // Cache last 100 versions
        ));

        let network = Arc::new(NetworkDetector::new(config.network.clone()));
        let retry_policy = Arc::new(RetryPolicy::new(config.retry.clone()));

        Self {
            config: Arc::new(config),
            queue,
            resolver,
            delta_manager,
            storage: Arc::new(storage),
            network,
            retry_policy,
            status: Arc::new(RwLock::new(SyncStatus::Idle)),
            stats: Arc::new(RwLock::new(SyncStats::default())),
        }
    }

    /// Enqueue a sync operation
    pub fn enqueue_operation(&self, operation: SyncOperation) -> Result<()> {
        self.queue.enqueue(operation)?;
        let mut stats = self.stats.write();
        stats.pending = self.queue.len();
        Ok(())
    }

    /// Start syncing
    pub async fn start_sync(&self) -> Result<()> {
        // Check network status
        if !self.network.is_online().await {
            *self.status.write() = SyncStatus::WaitingForNetwork;
            return Err(OfflineError::NetworkUnavailable);
        }

        *self.status.write() = SyncStatus::Syncing;

        let start_time = std::time::Instant::now();
        let result = self.sync_loop().await;
        let duration = start_time.elapsed();

        match result {
            Ok(_) => {
                *self.status.write() = SyncStatus::Completed;
                let mut stats = self.stats.write();
                stats.last_sync = Some(chrono::Utc::now());
                stats.last_sync_duration_ms = Some(duration.as_millis() as u64);
            }
            Err(e) => {
                *self.status.write() = SyncStatus::Failed;
                return Err(e);
            }
        }

        Ok(())
    }

    /// Main sync loop
    async fn sync_loop(&self) -> Result<()> {
        let batch_size = self.config.sync.batch_size;
        let mut batch = Vec::new();

        // Process all pending operations in batches
        while let Some(operation) = self.queue.dequeue() {
            batch.push(operation);

            if batch.len() >= batch_size {
                self.sync_batch(&batch).await?;
                batch.clear();
            }
        }

        // Process remaining operations
        if !batch.is_empty() {
            self.sync_batch(&batch).await?;
        }

        Ok(())
    }

    /// Sync a batch of operations
    async fn sync_batch(&self, operations: &[SyncOperation]) -> Result<()> {
        for operation in operations {
            match self.sync_operation(operation).await {
                Ok(_) => {
                    self.queue.mark_completed(&operation.id);
                    let mut stats = self.stats.write();
                    stats.total_synced += 1;
                    stats.pending = self.queue.len();
                }
                Err(e) if e.is_retryable() => {
                    // Re-enqueue with incremented retry count
                    let mut retry_op = operation.clone();
                    retry_op.increment_retry();
                    retry_op.set_error(e.to_string());

                    if retry_op.retry_count < self.config.retry.max_attempts as u32 {
                        self.queue.enqueue(retry_op)?;
                    } else {
                        let mut stats = self.stats.write();
                        stats.failed += 1;
                    }
                }
                Err(e) if e.is_conflict() => {
                    let mut stats = self.stats.write();
                    stats.conflicts += 1;
                    // Handle conflict resolution
                    // This would involve calling the conflict resolver
                }
                Err(_) => {
                    let mut stats = self.stats.write();
                    stats.failed += 1;
                }
            }
        }

        Ok(())
    }

    /// Sync a single operation
    async fn sync_operation(&self, operation: &SyncOperation) -> Result<()> {
        // This would make actual API call to sync server
        // For now, just a placeholder
        tracing::info!(
            "Syncing operation {} for entity {} ({})",
            operation.id,
            operation.entity_id,
            operation.operation_type
        );

        // Simulate API call
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        Ok(())
    }

    /// Get current sync status
    pub fn status(&self) -> SyncStatus {
        *self.status.read()
    }

    /// Get sync statistics
    pub fn stats(&self) -> SyncStats {
        self.stats.read().clone()
    }

    /// Pause syncing
    pub fn pause(&self) {
        *self.status.write() = SyncStatus::Paused;
    }

    /// Resume syncing
    pub async fn resume(&self) -> Result<()> {
        if *self.status.read() == SyncStatus::Paused {
            self.start_sync().await
        } else {
            Ok(())
        }
    }

    /// Clear all pending operations
    pub fn clear_queue(&self) {
        self.queue.clear();
        let mut stats = self.stats.write();
        stats.pending = 0;
    }

    /// Get operation queue reference
    pub fn queue(&self) -> &Arc<OperationQueue> {
        &self.queue
    }

    /// Get conflict resolver reference
    pub fn resolver(&self) -> Arc<RwLock<ConflictResolver>> {
        Arc::clone(&self.resolver)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::memory::MemoryStorage;
    use crate::versioning::VectorClock;

    fn create_test_config() -> OfflineConfig {
        OfflineConfig::default()
    }

    fn create_test_operation() -> SyncOperation {
        let version = Version {
            clock: VectorClock::new(),
            node_id: "test-node".to_string(),
            timestamp: chrono::Utc::now(),
            content_hash: "test-hash".to_string(),
        };

        SyncOperation::new(
            "entity-1".to_string(),
            "accident".to_string(),
            OperationType::Create,
            serde_json::json!({"test": "data"}),
            version,
        )
    }

    #[tokio::test]
    async fn test_sync_engine_creation() {
        let config = create_test_config();
        let storage = MemoryStorage::new();
        let engine = SyncEngine::new(config, storage);

        assert_eq!(engine.status(), SyncStatus::Idle);
    }

    #[tokio::test]
    async fn test_enqueue_operation() {
        let config = create_test_config();
        let storage = MemoryStorage::new();
        let engine = SyncEngine::new(config, storage);

        let operation = create_test_operation();
        engine.enqueue_operation(operation).unwrap();

        let stats = engine.stats();
        assert_eq!(stats.pending, 1);
    }
}
