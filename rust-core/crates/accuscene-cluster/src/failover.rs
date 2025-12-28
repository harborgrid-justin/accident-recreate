//! Automatic failover mechanisms.

use crate::consensus::LeaderElection;
use crate::error::Result;
use chrono::{DateTime, Utc};
use parking_lot::RwLock as ParkingRwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

/// Failover event types.
#[derive(Debug, Clone)]
pub enum FailoverEvent {
    /// Node failed
    NodeFailed {
        node_id: Uuid,
        timestamp: DateTime<Utc>,
    },

    /// Node recovered
    NodeRecovered {
        node_id: Uuid,
        timestamp: DateTime<Utc>,
    },

    /// Leader failed
    LeaderFailed {
        old_leader: Uuid,
        timestamp: DateTime<Utc>,
    },

    /// New leader elected
    LeaderElected {
        new_leader: Uuid,
        term: u64,
        timestamp: DateTime<Utc>,
    },

    /// Partition detected
    PartitionDetected {
        affected_nodes: Vec<Uuid>,
        timestamp: DateTime<Utc>,
    },

    /// Partition healed
    PartitionHealed {
        timestamp: DateTime<Utc>,
    },
}

/// Failover strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailoverStrategy {
    /// Automatic failover with leader election
    Automatic,
    /// Manual failover (requires operator intervention)
    Manual,
    /// Fast failover (minimal checks)
    Fast,
}

/// Failover configuration.
#[derive(Debug, Clone)]
pub struct FailoverConfig {
    /// Failover strategy
    pub strategy: FailoverStrategy,

    /// Failure detection timeout
    pub detection_timeout: Duration,

    /// Minimum nodes for quorum
    pub min_quorum: usize,

    /// Enable automatic recovery
    pub auto_recovery: bool,

    /// Recovery grace period
    pub recovery_grace_period: Duration,
}

impl Default for FailoverConfig {
    fn default() -> Self {
        Self {
            strategy: FailoverStrategy::Automatic,
            detection_timeout: Duration::from_secs(10),
            min_quorum: 2,
            auto_recovery: true,
            recovery_grace_period: Duration::from_secs(30),
        }
    }
}

/// Failover manager.
pub struct FailoverManager {
    /// Local node ID
    local_id: Uuid,

    /// Configuration
    config: FailoverConfig,

    /// Leader election
    leader_election: Arc<LeaderElection>,

    /// Failed nodes
    failed_nodes: Arc<ParkingRwLock<HashMap<Uuid, DateTime<Utc>>>>,

    /// Event channel
    event_tx: mpsc::UnboundedSender<FailoverEvent>,
    event_rx: Arc<RwLock<mpsc::UnboundedReceiver<FailoverEvent>>>,

    /// Running flag
    running: Arc<RwLock<bool>>,
}

impl FailoverManager {
    /// Create a new failover manager.
    pub fn new(
        local_id: Uuid,
        config: FailoverConfig,
        leader_election: Arc<LeaderElection>,
    ) -> Self {
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        Self {
            local_id,
            config,
            leader_election,
            failed_nodes: Arc::new(ParkingRwLock::new(HashMap::new())),
            event_tx,
            event_rx: Arc::new(RwLock::new(event_rx)),
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start failover manager.
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if *running {
            return Ok(());
        }
        *running = true;
        drop(running);

        tracing::info!("Starting failover manager");

        // Start monitoring task
        let manager = self.clone_for_task();
        tokio::spawn(async move {
            manager.monitor_task().await;
        });

        Ok(())
    }

    /// Stop failover manager.
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        tracing::info!("Stopped failover manager");
    }

    /// Handle node failure.
    pub async fn handle_node_failure(&self, node_id: Uuid) -> Result<()> {
        tracing::warn!("Handling failure for node: {}", node_id);

        let now = Utc::now();
        self.failed_nodes.write().insert(node_id, now);

        // Send event
        let _ = self.event_tx.send(FailoverEvent::NodeFailed {
            node_id,
            timestamp: now,
        });

        // Check if failed node was the leader
        if let Some(leader_id) = self.leader_election.current_leader() {
            if leader_id == node_id {
                return self.handle_leader_failure(leader_id).await;
            }
        }

        Ok(())
    }

    /// Handle leader failure.
    async fn handle_leader_failure(&self, old_leader: Uuid) -> Result<()> {
        tracing::warn!("Leader failed: {}", old_leader);

        let now = Utc::now();
        let _ = self.event_tx.send(FailoverEvent::LeaderFailed {
            old_leader,
            timestamp: now,
        });

        match self.config.strategy {
            FailoverStrategy::Automatic | FailoverStrategy::Fast => {
                // Trigger new election
                self.trigger_election().await?;
            }
            FailoverStrategy::Manual => {
                tracing::info!("Manual failover strategy - waiting for operator");
            }
        }

        Ok(())
    }

    /// Handle node recovery.
    pub async fn handle_node_recovery(&self, node_id: Uuid) -> Result<()> {
        tracing::info!("Node recovered: {}", node_id);

        if self.failed_nodes.write().remove(&node_id).is_some() {
            let now = Utc::now();
            let _ = self.event_tx.send(FailoverEvent::NodeRecovered {
                node_id,
                timestamp: now,
            });
        }

        Ok(())
    }

    /// Trigger leader election.
    async fn trigger_election(&self) -> Result<()> {
        tracing::info!("Triggering leader election");

        let term = self.leader_election.start_election();

        // In a real implementation, broadcast vote requests to other nodes
        // For now, just simulate becoming leader if we're the only node

        // Simulate successful election
        self.leader_election.become_leader();

        let _ = self.event_tx.send(FailoverEvent::LeaderElected {
            new_leader: self.local_id,
            term,
            timestamp: Utc::now(),
        });

        Ok(())
    }

    /// Check quorum.
    pub fn check_quorum(&self, active_nodes: usize) -> bool {
        active_nodes >= self.config.min_quorum
    }

    /// Get failed nodes.
    pub fn failed_nodes(&self) -> Vec<Uuid> {
        self.failed_nodes.read().keys().copied().collect()
    }

    /// Get failover events.
    pub fn subscribe_events(&self) -> mpsc::UnboundedReceiver<FailoverEvent> {
        let (tx, rx) = mpsc::unbounded_channel();
        // Note: In a real implementation, use broadcast channel
        rx
    }

    /// Monitor task.
    async fn monitor_task(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(5));

        loop {
            interval.tick().await;

            let running = self.running.read().await;
            if !*running {
                break;
            }
            drop(running);

            // Check leader health
            if let Some(leader_info) = self.leader_election.leader_info().leader_id {
                if self.leader_election.leader_info().is_lease_expired() {
                    tracing::warn!("Leader lease expired");
                    if self.config.strategy == FailoverStrategy::Automatic {
                        let _ = self.handle_leader_failure(leader_info).await;
                    }
                }
            }

            // Check for stale failures (potential recovery)
            if self.config.auto_recovery {
                let cutoff = Utc::now() - chrono::Duration::from_std(self.config.recovery_grace_period).unwrap();
                let stale_failures: Vec<Uuid> = self
                    .failed_nodes
                    .read()
                    .iter()
                    .filter(|(_, &timestamp)| timestamp < cutoff)
                    .map(|(&id, _)| id)
                    .collect();

                for node_id in stale_failures {
                    tracing::debug!("Attempting recovery for node: {}", node_id);
                    // In a real implementation, probe the node
                }
            }
        }
    }

    /// Clone for async task.
    fn clone_for_task(&self) -> Self {
        Self {
            local_id: self.local_id,
            config: self.config.clone(),
            leader_election: Arc::clone(&self.leader_election),
            failed_nodes: Arc::clone(&self.failed_nodes),
            event_tx: self.event_tx.clone(),
            event_rx: Arc::clone(&self.event_rx),
            running: Arc::clone(&self.running),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_failover_config() {
        let config = FailoverConfig::default();
        assert_eq!(config.strategy, FailoverStrategy::Automatic);
        assert!(config.auto_recovery);
    }
}
