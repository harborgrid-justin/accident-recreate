//! Cluster membership view.

use crate::node::{Node, NodeState};
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Cluster membership view.
#[derive(Debug, Clone)]
pub struct MembershipView {
    /// All known nodes
    nodes: Arc<RwLock<HashMap<Uuid, MemberInfo>>>,

    /// Local node ID
    local_id: Uuid,

    /// View version (incremented on changes)
    version: Arc<RwLock<u64>>,
}

impl MembershipView {
    /// Create a new membership view.
    pub fn new(local_id: Uuid) -> Self {
        Self {
            nodes: Arc::new(RwLock::new(HashMap::new())),
            local_id,
            version: Arc::new(RwLock::new(0)),
        }
    }

    /// Add or update a node.
    pub fn upsert(&self, node: Node) -> bool {
        let mut nodes = self.nodes.write();
        let mut version = self.version.write();

        let member_info = MemberInfo::from(node);
        let updated = nodes
            .get(&member_info.node_id)
            .map(|existing| existing.should_update(&member_info))
            .unwrap_or(true);

        if updated {
            nodes.insert(member_info.node_id, member_info);
            *version += 1;
        }

        updated
    }

    /// Update node state.
    pub fn update_state(&self, node_id: Uuid, state: NodeState, incarnation: u64) -> bool {
        let mut nodes = self.nodes.write();
        let mut version = self.version.write();

        if let Some(member) = nodes.get_mut(&node_id) {
            if incarnation >= member.incarnation {
                member.state = state;
                member.incarnation = incarnation;
                member.last_updated = Utc::now();
                *version += 1;
                return true;
            }
        }

        false
    }

    /// Get a node.
    pub fn get(&self, node_id: &Uuid) -> Option<MemberInfo> {
        self.nodes.read().get(node_id).cloned()
    }

    /// Remove a node.
    pub fn remove(&self, node_id: &Uuid) -> bool {
        let mut nodes = self.nodes.write();
        let mut version = self.version.write();

        if nodes.remove(node_id).is_some() {
            *version += 1;
            true
        } else {
            false
        }
    }

    /// List all members.
    pub fn members(&self) -> Vec<MemberInfo> {
        self.nodes.read().values().cloned().collect()
    }

    /// List members by state.
    pub fn members_by_state(&self, state: NodeState) -> Vec<MemberInfo> {
        self.nodes
            .read()
            .values()
            .filter(|m| m.state == state)
            .cloned()
            .collect()
    }

    /// Get active members.
    pub fn active_members(&self) -> Vec<MemberInfo> {
        self.members_by_state(NodeState::Active)
    }

    /// Count members.
    pub fn count(&self) -> usize {
        self.nodes.read().len()
    }

    /// Count active members.
    pub fn count_active(&self) -> usize {
        self.nodes
            .read()
            .values()
            .filter(|m| m.state == NodeState::Active)
            .count()
    }

    /// Get current view version.
    pub fn version(&self) -> u64 {
        *self.version.read()
    }

    /// Get random members for gossip.
    pub fn random_members(&self, count: usize) -> Vec<MemberInfo> {
        use rand::seq::SliceRandom;
        let mut members: Vec<_> = self
            .nodes
            .read()
            .values()
            .filter(|m| m.node_id != self.local_id && m.state.is_operational())
            .cloned()
            .collect();

        members.shuffle(&mut rand::thread_rng());
        members.truncate(count);
        members
    }

    /// Touch a node (update last seen).
    pub fn touch(&self, node_id: &Uuid) -> bool {
        let mut nodes = self.nodes.write();
        if let Some(member) = nodes.get_mut(node_id) {
            member.last_updated = Utc::now();
            true
        } else {
            false
        }
    }

    /// Get stale members.
    pub fn stale_members(&self, timeout: std::time::Duration) -> Vec<Uuid> {
        let cutoff = Utc::now() - chrono::Duration::from_std(timeout).unwrap();
        self.nodes
            .read()
            .values()
            .filter(|m| m.last_updated < cutoff && m.node_id != self.local_id)
            .map(|m| m.node_id)
            .collect()
    }
}

/// Member information.
#[derive(Debug, Clone)]
pub struct MemberInfo {
    pub node_id: Uuid,
    pub state: NodeState,
    pub incarnation: u64,
    pub last_updated: DateTime<Utc>,
}

impl MemberInfo {
    /// Check if this member info should update existing.
    fn should_update(&self, other: &MemberInfo) -> bool {
        // Higher incarnation always wins
        if other.incarnation > self.incarnation {
            return true;
        }

        // Same incarnation, but newer state
        if other.incarnation == self.incarnation {
            // Dead > Suspected > Alive
            let self_priority = state_priority(self.state);
            let other_priority = state_priority(other.state);
            if other_priority > self_priority {
                return true;
            }
        }

        false
    }
}

impl From<Node> for MemberInfo {
    fn from(node: Node) -> Self {
        Self {
            node_id: node.id.id,
            state: node.state,
            incarnation: node.incarnation,
            last_updated: node.last_seen,
        }
    }
}

/// Get state priority for conflict resolution.
fn state_priority(state: NodeState) -> u8 {
    match state {
        NodeState::Failed | NodeState::Left => 3,
        NodeState::Suspected => 2,
        _ => 1,
    }
}

// Conditional rand dependency
#[cfg(not(test))]
mod rand {
    pub mod seq {
        pub trait SliceRandom {
            fn shuffle<R>(&mut self, _rng: &mut R) {}
        }
        impl<T> SliceRandom for Vec<T> {}
    }
    pub fn thread_rng() -> () {
        ()
    }
}

#[cfg(test)]
mod rand {
    pub use ::rand::*;
}
