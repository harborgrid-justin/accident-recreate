//! SWIM-like gossip protocol for failure detection.

use crate::node::{Node, NodeState};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Gossip message types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GossipMessage {
    /// Ping request
    Ping {
        from: Uuid,
        sequence: u64,
    },

    /// Ping acknowledgment
    Ack {
        from: Uuid,
        sequence: u64,
    },

    /// Indirect ping request
    PingReq {
        target: Uuid,
        from: Uuid,
        sequence: u64,
    },

    /// Suspect a node
    Suspect {
        node_id: Uuid,
        incarnation: u64,
        from: Uuid,
    },

    /// Confirm a node is alive
    Alive {
        node_id: Uuid,
        incarnation: u64,
        from: Uuid,
    },

    /// Confirm a node is dead
    Dead {
        node_id: Uuid,
        from: Uuid,
    },

    /// Full state sync request
    StateSync {
        from: Uuid,
    },

    /// Full state sync response
    StateSyncReply {
        from: Uuid,
        nodes: Vec<NodeGossipState>,
    },
}

/// Node state for gossip.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeGossipState {
    pub node_id: Uuid,
    pub state: NodeState,
    pub incarnation: u64,
    pub last_seen: DateTime<Utc>,
}

impl From<&Node> for NodeGossipState {
    fn from(node: &Node) -> Self {
        Self {
            node_id: node.id.id,
            state: node.state,
            incarnation: node.incarnation,
            last_seen: node.last_seen,
        }
    }
}

/// Gossip protocol state machine.
pub struct GossipProtocol {
    /// Local node ID
    local_id: Uuid,

    /// Sequence number for ping/ack
    sequence: u64,

    /// Pending ping requests
    pending_pings: HashMap<u64, PingRequest>,

    /// Suspicion timers
    suspicions: HashMap<Uuid, SuspicionTimer>,

    /// Transmission counters for dissemination
    transmissions: HashMap<Uuid, u32>,

    /// Maximum transmissions before stopping
    max_transmissions: u32,
}

impl GossipProtocol {
    /// Create a new gossip protocol.
    pub fn new(local_id: Uuid, max_transmissions: u32) -> Self {
        Self {
            local_id,
            sequence: 0,
            pending_pings: HashMap::new(),
            suspicions: HashMap::new(),
            transmissions: HashMap::new(),
            max_transmissions,
        }
    }

    /// Create a ping message.
    pub fn create_ping(&mut self) -> (GossipMessage, u64) {
        self.sequence += 1;
        let seq = self.sequence;
        (
            GossipMessage::Ping {
                from: self.local_id,
                sequence: seq,
            },
            seq,
        )
    }

    /// Create an ack message.
    pub fn create_ack(&self, sequence: u64) -> GossipMessage {
        GossipMessage::Ack {
            from: self.local_id,
            sequence,
        }
    }

    /// Create an indirect ping request.
    pub fn create_ping_req(&mut self, target: Uuid) -> (GossipMessage, u64) {
        self.sequence += 1;
        let seq = self.sequence;
        (
            GossipMessage::PingReq {
                target,
                from: self.local_id,
                sequence: seq,
            },
            seq,
        )
    }

    /// Create a suspect message.
    pub fn create_suspect(&mut self, node_id: Uuid, incarnation: u64) -> GossipMessage {
        self.increment_transmission(node_id);
        GossipMessage::Suspect {
            node_id,
            incarnation,
            from: self.local_id,
        }
    }

    /// Create an alive message.
    pub fn create_alive(&mut self, node_id: Uuid, incarnation: u64) -> GossipMessage {
        self.increment_transmission(node_id);
        GossipMessage::Alive {
            node_id,
            incarnation,
            from: self.local_id,
        }
    }

    /// Create a dead message.
    pub fn create_dead(&mut self, node_id: Uuid) -> GossipMessage {
        self.increment_transmission(node_id);
        GossipMessage::Dead {
            node_id,
            from: self.local_id,
        }
    }

    /// Record a pending ping.
    pub fn record_ping(&mut self, sequence: u64, target: Uuid) {
        self.pending_pings.insert(
            sequence,
            PingRequest {
                target,
                sent_at: Utc::now(),
            },
        );
    }

    /// Handle received ack.
    pub fn handle_ack(&mut self, sequence: u64) -> Option<Uuid> {
        self.pending_pings.remove(&sequence).map(|req| req.target)
    }

    /// Start suspicion timer for a node.
    pub fn start_suspicion(&mut self, node_id: Uuid, timeout: std::time::Duration) {
        self.suspicions.insert(
            node_id,
            SuspicionTimer {
                started_at: Utc::now(),
                timeout,
            },
        );
    }

    /// Cancel suspicion for a node.
    pub fn cancel_suspicion(&mut self, node_id: &Uuid) -> bool {
        self.suspicions.remove(node_id).is_some()
    }

    /// Get expired suspicions.
    pub fn expired_suspicions(&self) -> Vec<Uuid> {
        let now = Utc::now();
        self.suspicions
            .iter()
            .filter(|(_, timer)| timer.is_expired(now))
            .map(|(id, _)| *id)
            .collect()
    }

    /// Increment transmission counter.
    fn increment_transmission(&mut self, node_id: Uuid) {
        *self.transmissions.entry(node_id).or_insert(0) += 1;
    }

    /// Check if should continue transmitting.
    pub fn should_transmit(&self, node_id: &Uuid) -> bool {
        self.transmissions
            .get(node_id)
            .map(|&count| count < self.max_transmissions)
            .unwrap_or(true)
    }

    /// Clean up old pending pings.
    pub fn cleanup_pending_pings(&mut self, timeout: std::time::Duration) {
        let cutoff = Utc::now() - chrono::Duration::from_std(timeout).unwrap();
        self.pending_pings
            .retain(|_, req| req.sent_at > cutoff);
    }
}

/// Pending ping request.
#[derive(Debug)]
struct PingRequest {
    target: Uuid,
    sent_at: DateTime<Utc>,
}

/// Suspicion timer.
#[derive(Debug)]
struct SuspicionTimer {
    started_at: DateTime<Utc>,
    timeout: std::time::Duration,
}

impl SuspicionTimer {
    fn is_expired(&self, now: DateTime<Utc>) -> bool {
        let elapsed = (now - self.started_at)
            .to_std()
            .unwrap_or(std::time::Duration::from_secs(0));
        elapsed >= self.timeout
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gossip_protocol() {
        let local_id = Uuid::new_v4();
        let mut protocol = GossipProtocol::new(local_id, 15);

        // Create ping
        let (msg, seq) = protocol.create_ping();
        match msg {
            GossipMessage::Ping { from, sequence } => {
                assert_eq!(from, local_id);
                assert_eq!(sequence, seq);
            }
            _ => panic!("Expected Ping message"),
        }

        // Record ping
        let target_id = Uuid::new_v4();
        protocol.record_ping(seq, target_id);

        // Handle ack
        let acked = protocol.handle_ack(seq);
        assert_eq!(acked, Some(target_id));
    }
}
