//! Node management and state.

pub mod health;
pub mod identity;
pub mod state;

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use std::sync::Arc;

pub use health::{ComponentHealth, HealthCheck, HealthMonitor, HealthStatus};
pub use identity::{NodeCapabilities, NodeId, NodeVersion};
pub use state::{NodeState, NodeStateMachine, StateTransition};

/// Complete node information.
#[derive(Debug, Clone)]
pub struct Node {
    /// Node identity
    pub id: NodeId,

    /// Node state
    pub state: NodeState,

    /// Node capabilities
    pub capabilities: NodeCapabilities,

    /// Node version
    pub version: NodeVersion,

    /// Last seen timestamp
    pub last_seen: DateTime<Utc>,

    /// Last health check
    pub last_health: Option<HealthCheck>,

    /// Incarnation number (for conflict resolution)
    pub incarnation: u64,
}

impl Node {
    /// Create a new node.
    pub fn new(id: NodeId) -> Self {
        Self {
            id,
            state: NodeState::Initializing,
            capabilities: NodeCapabilities::default(),
            version: NodeVersion::default(),
            last_seen: Utc::now(),
            last_health: None,
            incarnation: 0,
        }
    }

    /// Update last seen timestamp.
    pub fn touch(&mut self) {
        self.last_seen = Utc::now();
    }

    /// Increment incarnation number.
    pub fn increment_incarnation(&mut self) {
        self.incarnation += 1;
    }

    /// Check if node is stale.
    pub fn is_stale(&self, timeout: std::time::Duration) -> bool {
        let now = Utc::now();
        let elapsed = (now - self.last_seen)
            .to_std()
            .unwrap_or(std::time::Duration::from_secs(0));
        elapsed > timeout
    }
}

/// Thread-safe node registry.
#[derive(Debug, Clone)]
pub struct NodeRegistry {
    nodes: Arc<RwLock<std::collections::HashMap<uuid::Uuid, Node>>>,
}

impl NodeRegistry {
    /// Create a new node registry.
    pub fn new() -> Self {
        Self {
            nodes: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Add or update a node.
    pub fn upsert(&self, node: Node) -> Option<Node> {
        self.nodes.write().insert(node.id.id, node)
    }

    /// Get a node by ID.
    pub fn get(&self, id: &uuid::Uuid) -> Option<Node> {
        self.nodes.read().get(id).cloned()
    }

    /// Remove a node.
    pub fn remove(&self, id: &uuid::Uuid) -> Option<Node> {
        self.nodes.write().remove(id)
    }

    /// List all nodes.
    pub fn list(&self) -> Vec<Node> {
        self.nodes.read().values().cloned().collect()
    }

    /// List nodes by state.
    pub fn list_by_state(&self, state: NodeState) -> Vec<Node> {
        self.nodes
            .read()
            .values()
            .filter(|n| n.state == state)
            .cloned()
            .collect()
    }

    /// Count nodes.
    pub fn count(&self) -> usize {
        self.nodes.read().len()
    }

    /// Count operational nodes.
    pub fn count_operational(&self) -> usize {
        self.nodes
            .read()
            .values()
            .filter(|n| n.state.is_operational())
            .count()
    }

    /// Clear all nodes.
    pub fn clear(&self) {
        self.nodes.write().clear();
    }
}

impl Default for NodeRegistry {
    fn default() -> Self {
        Self::new()
    }
}
