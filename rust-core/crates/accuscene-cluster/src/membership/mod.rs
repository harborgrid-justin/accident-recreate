//! Cluster membership management.

pub mod gossip;
pub mod view;

pub use gossip::{GossipMessage, GossipProtocol, NodeGossipState};
pub use view::{MemberInfo, MembershipView};

use crate::config::MembershipConfig;
use crate::error::Result;
use crate::node::{Node, NodeState};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Membership service managing cluster membership.
pub struct MembershipService {
    /// Local node ID
    local_id: Uuid,

    /// Membership view
    view: MembershipView,

    /// Gossip protocol
    gossip: Arc<RwLock<GossipProtocol>>,

    /// Configuration
    config: MembershipConfig,

    /// Running flag
    running: Arc<RwLock<bool>>,
}

impl MembershipService {
    /// Create a new membership service.
    pub fn new(local_id: Uuid, config: MembershipConfig) -> Self {
        let gossip = GossipProtocol::new(local_id, config.max_transmissions as u32);

        Self {
            local_id,
            view: MembershipView::new(local_id),
            gossip: Arc::new(RwLock::new(gossip)),
            config,
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start membership service.
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if *running {
            return Ok(());
        }
        *running = true;
        drop(running);

        info!("Starting membership service");

        // Start gossip task
        let service = self.clone_for_task();
        tokio::spawn(async move {
            service.gossip_task().await;
        });

        // Start failure detection task
        let service = self.clone_for_task();
        tokio::spawn(async move {
            service.failure_detection_task().await;
        });

        Ok(())
    }

    /// Stop membership service.
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        info!("Stopped membership service");
    }

    /// Join the cluster.
    pub async fn join(&self, node: Node) -> Result<()> {
        info!("Joining cluster with node: {}", node.id.id);
        self.view.upsert(node);
        Ok(())
    }

    /// Leave the cluster.
    pub async fn leave(&self) -> Result<()> {
        info!("Leaving cluster");
        self.view.update_state(self.local_id, NodeState::Leaving, 0);
        Ok(())
    }

    /// Get membership view.
    pub fn view(&self) -> &MembershipView {
        &self.view
    }

    /// Handle incoming gossip message.
    pub async fn handle_message(&self, message: GossipMessage) -> Option<GossipMessage> {
        match message {
            GossipMessage::Ping { from, sequence } => {
                debug!("Received ping from {} seq {}", from, sequence);
                self.view.touch(&from);
                let gossip = self.gossip.read().await;
                Some(gossip.create_ack(sequence))
            }

            GossipMessage::Ack { from, sequence } => {
                debug!("Received ack from {} seq {}", from, sequence);
                let mut gossip = self.gossip.write().await;
                if let Some(node_id) = gossip.handle_ack(sequence) {
                    self.view.touch(&node_id);
                    // Cancel any suspicion
                    gossip.cancel_suspicion(&node_id);
                }
                None
            }

            GossipMessage::Suspect {
                node_id,
                incarnation,
                ..
            } => {
                debug!("Received suspect for node {}", node_id);
                if node_id == self.local_id {
                    // Refute suspicion
                    let mut gossip = self.gossip.write().await;
                    Some(gossip.create_alive(node_id, incarnation + 1))
                } else {
                    self.view
                        .update_state(node_id, NodeState::Suspected, incarnation);
                    None
                }
            }

            GossipMessage::Alive {
                node_id,
                incarnation,
                ..
            } => {
                debug!("Received alive for node {}", node_id);
                self.view.update_state(node_id, NodeState::Active, incarnation);
                None
            }

            GossipMessage::Dead { node_id, .. } => {
                debug!("Received dead for node {}", node_id);
                self.view.update_state(node_id, NodeState::Failed, 0);
                None
            }

            _ => None,
        }
    }

    /// Gossip task.
    async fn gossip_task(&self) {
        let mut interval = tokio::time::interval(self.config.gossip_interval);

        loop {
            interval.tick().await;

            let running = self.running.read().await;
            if !*running {
                break;
            }
            drop(running);

            // Select random nodes to gossip with
            let members = self.view.random_members(self.config.gossip_fanout);
            for member in members {
                // Send ping
                let mut gossip = self.gossip.write().await;
                let (ping_msg, seq) = gossip.create_ping();
                gossip.record_ping(seq, member.node_id);
                drop(gossip);

                // In a real implementation, send ping_msg to member
                debug!("Would send ping to {}", member.node_id);
            }
        }
    }

    /// Failure detection task.
    async fn failure_detection_task(&self) {
        let mut interval = tokio::time::interval(self.config.gossip_interval);

        loop {
            interval.tick().await;

            let running = self.running.read().await;
            if !*running {
                break;
            }
            drop(running);

            // Check for expired suspicions
            let mut gossip = self.gossip.write().await;
            let expired = gossip.expired_suspicions();

            for node_id in expired {
                warn!("Node {} failed to respond, marking as failed", node_id);
                self.view.update_state(node_id, NodeState::Failed, 0);
                gossip.cancel_suspicion(&node_id);
            }

            // Cleanup old pending pings
            gossip.cleanup_pending_pings(self.config.suspicion_timeout);
        }
    }

    /// Clone for async task.
    fn clone_for_task(&self) -> Self {
        Self {
            local_id: self.local_id,
            view: self.view.clone(),
            gossip: Arc::clone(&self.gossip),
            config: self.config.clone(),
            running: Arc::clone(&self.running),
        }
    }
}
