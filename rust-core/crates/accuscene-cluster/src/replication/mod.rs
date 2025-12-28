//! Data replication across cluster nodes.

pub mod conflict;
pub mod sync;

pub use conflict::{
    ConflictResolution, ConflictResolver, VectorClock, VectorClockOrdering, VersionedValue,
};
pub use sync::{StateDelta, StateSnapshot, StateSynchronizer, SyncRequest, SyncResponse};

use crate::config::ReplicationConfig;
use crate::error::{ClusterError, Result};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Replication service.
pub struct ReplicationService {
    /// Local node ID
    local_id: Uuid,

    /// State synchronizer
    synchronizer: Arc<StateSynchronizer>,

    /// Configuration
    config: ReplicationConfig,

    /// Vector clock for this node
    vector_clock: Arc<RwLock<VectorClock>>,
}

impl ReplicationService {
    /// Create a new replication service.
    pub fn new(local_id: Uuid, config: ReplicationConfig) -> Self {
        let synchronizer = Arc::new(StateSynchronizer::new(local_id));
        let vector_clock = Arc::new(RwLock::new(VectorClock::new()));

        Self {
            local_id,
            synchronizer,
            config,
            vector_clock,
        }
    }

    /// Write a value with replication.
    pub async fn write(&self, key: String, value: Vec<u8>) -> Result<()> {
        // Increment vector clock
        let mut clock = self.vector_clock.write().await;
        clock.increment(self.local_id);
        let version = clock.clone();
        drop(clock);

        // Create versioned value
        let versioned = VersionedValue::new(value.clone(), self.local_id, version);

        // Apply to local state
        let mut delta = StateDelta::new(
            self.synchronizer.current_version(),
            self.synchronizer.current_version() + 1,
        );
        delta.add_update(key.clone(), bincode::serialize(&versioned)?);

        self.synchronizer.apply_delta(delta)?;

        Ok(())
    }

    /// Read a value.
    pub async fn read(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let snapshot = self.synchronizer.get_snapshot();

        if let Some(data) = snapshot.data.get(key) {
            let versioned: VersionedValue = bincode::deserialize(data)?;

            if !versioned.verify() {
                return Err(ClusterError::ChecksumMismatch);
            }

            Ok(Some(versioned.value))
        } else {
            Ok(None)
        }
    }

    /// Replicate to peer nodes.
    pub async fn replicate_to_peers(&self, peer_ids: &[Uuid]) -> Result<usize> {
        let snapshot = self.synchronizer.get_snapshot();
        let mut successful = 0;

        for peer_id in peer_ids {
            // In a real implementation, send snapshot to peer
            // For now, just count as successful
            successful += 1;
        }

        // Check if we met consistency requirements
        let total_nodes = peer_ids.len() + 1; // +1 for local node
        let required = self.config.write_consistency.required_nodes(total_nodes);

        if successful + 1 < required {
            return Err(ClusterError::QuorumNotReached {
                current: successful + 1,
                required,
            });
        }

        Ok(successful)
    }

    /// Handle sync request from peer.
    pub fn handle_sync_request(&self, request: SyncRequest) -> SyncResponse {
        self.synchronizer.handle_request(request)
    }

    /// Apply sync response.
    pub async fn apply_sync_response(&self, response: SyncResponse) -> Result<()> {
        match response {
            SyncResponse::Snapshot(snapshot) => {
                if !snapshot.verify() {
                    return Err(ClusterError::ChecksumMismatch);
                }

                // Apply snapshot
                for (key, value) in snapshot.data {
                    let mut delta = StateDelta::new(
                        self.synchronizer.current_version(),
                        snapshot.version,
                    );
                    delta.add_update(key, value);
                    self.synchronizer.apply_delta(delta)?;
                }

                Ok(())
            }

            SyncResponse::Delta(delta) => {
                self.synchronizer.apply_delta(delta)?;
                Ok(())
            }

            SyncResponse::AntiEntropyData { data } => {
                // Apply anti-entropy data
                for (key, value) in data {
                    let mut delta = StateDelta::new(
                        self.synchronizer.current_version(),
                        self.synchronizer.current_version() + 1,
                    );
                    delta.add_update(key, value);
                    self.synchronizer.apply_delta(delta)?;
                }

                Ok(())
            }

            SyncResponse::Error(msg) => Err(ClusterError::ReplicationFailed(msg)),
        }
    }

    /// Resolve conflicts for a key.
    pub async fn resolve_conflicts(&self, key: &str, values: Vec<VersionedValue>) -> Result<()> {
        let resolution = match self.config.conflict_resolution {
            crate::config::ConflictResolutionStrategy::LastWriteWins => {
                if let Some(winner) = ConflictResolver::last_write_wins(&values) {
                    ConflictResolution::Resolved(winner)
                } else {
                    ConflictResolution::NoValue
                }
            }

            crate::config::ConflictResolutionStrategy::VectorClock => {
                ConflictResolver::vector_clock_resolve(&values)
            }

            crate::config::ConflictResolutionStrategy::Custom => {
                // Custom resolution - merge values
                let merged = ConflictResolver::merge_values(values);
                ConflictResolution::Resolved(merged)
            }
        };

        // Apply resolution
        match resolution {
            ConflictResolution::Resolved(value) => {
                let mut delta = StateDelta::new(
                    self.synchronizer.current_version(),
                    self.synchronizer.current_version() + 1,
                );
                delta.add_update(key.to_string(), bincode::serialize(&value)?);
                self.synchronizer.apply_delta(delta)?;
                Ok(())
            }

            ConflictResolution::Conflict(values) => {
                // Keep all siblings for application to resolve
                Err(ClusterError::ConflictResolution(format!(
                    "{} conflicting values",
                    values.len()
                )))
            }

            ConflictResolution::NoValue => Ok(()),
        }
    }

    /// Get current snapshot.
    pub fn get_snapshot(&self) -> StateSnapshot {
        self.synchronizer.get_snapshot()
    }

    /// Get current version.
    pub fn current_version(&self) -> u64 {
        self.synchronizer.current_version()
    }
}
