//! Leader election mechanism.

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use std::sync::Arc;
use uuid::Uuid;

/// Leader election state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeaderState {
    /// Node is a follower
    Follower,
    /// Node is a candidate
    Candidate,
    /// Node is the leader
    Leader,
}

/// Leader election information.
#[derive(Debug, Clone)]
pub struct LeaderInfo {
    /// Current leader ID
    pub leader_id: Option<Uuid>,

    /// Current term
    pub term: u64,

    /// Leader lease expiration
    pub lease_expires_at: Option<DateTime<Utc>>,
}

impl LeaderInfo {
    /// Create new leader info.
    pub fn new() -> Self {
        Self {
            leader_id: None,
            term: 0,
            lease_expires_at: None,
        }
    }

    /// Check if there is a valid leader.
    pub fn has_leader(&self) -> bool {
        if let Some(expires_at) = self.lease_expires_at {
            Utc::now() < expires_at
        } else {
            false
        }
    }

    /// Check if lease is expired.
    pub fn is_lease_expired(&self) -> bool {
        if let Some(expires_at) = self.lease_expires_at {
            Utc::now() >= expires_at
        } else {
            true
        }
    }
}

impl Default for LeaderInfo {
    fn default() -> Self {
        Self::new()
    }
}

/// Leader election manager.
pub struct LeaderElection {
    /// Local node ID
    local_id: Uuid,

    /// Current state
    state: Arc<RwLock<LeaderState>>,

    /// Leader information
    leader_info: Arc<RwLock<LeaderInfo>>,

    /// Voted for in current term
    voted_for: Arc<RwLock<Option<Uuid>>>,

    /// Election timeout
    election_timeout: std::time::Duration,

    /// Leader lease duration
    lease_duration: std::time::Duration,
}

impl LeaderElection {
    /// Create a new leader election.
    pub fn new(
        local_id: Uuid,
        election_timeout: std::time::Duration,
        lease_duration: std::time::Duration,
    ) -> Self {
        Self {
            local_id,
            state: Arc::new(RwLock::new(LeaderState::Follower)),
            leader_info: Arc::new(RwLock::new(LeaderInfo::new())),
            voted_for: Arc::new(RwLock::new(None)),
            election_timeout,
            lease_duration,
        }
    }

    /// Get current state.
    pub fn state(&self) -> LeaderState {
        *self.state.read()
    }

    /// Get leader info.
    pub fn leader_info(&self) -> LeaderInfo {
        self.leader_info.read().clone()
    }

    /// Check if this node is the leader.
    pub fn is_leader(&self) -> bool {
        *self.state.read() == LeaderState::Leader
    }

    /// Get current leader ID.
    pub fn current_leader(&self) -> Option<Uuid> {
        self.leader_info.read().leader_id
    }

    /// Get current term.
    pub fn current_term(&self) -> u64 {
        self.leader_info.read().term
    }

    /// Start election as candidate.
    pub fn start_election(&self) -> u64 {
        let mut state = self.state.write();
        let mut leader_info = self.leader_info.write();
        let mut voted_for = self.voted_for.write();

        // Increment term
        leader_info.term += 1;
        let term = leader_info.term;

        // Transition to candidate
        *state = LeaderState::Candidate;

        // Vote for self
        *voted_for = Some(self.local_id);

        // Clear leader
        leader_info.leader_id = None;
        leader_info.lease_expires_at = None;

        term
    }

    /// Vote for a candidate.
    pub fn vote(&self, candidate_id: Uuid, term: u64) -> bool {
        let mut leader_info = self.leader_info.write();
        let mut voted_for = self.voted_for.write();

        // Check if term is valid
        if term < leader_info.term {
            return false;
        }

        // Update term if newer
        if term > leader_info.term {
            leader_info.term = term;
            *voted_for = None;
        }

        // Vote if haven't voted yet
        if voted_for.is_none() {
            *voted_for = Some(candidate_id);
            true
        } else {
            *voted_for == Some(candidate_id)
        }
    }

    /// Become leader.
    pub fn become_leader(&self) {
        let mut state = self.state.write();
        let mut leader_info = self.leader_info.write();

        *state = LeaderState::Leader;
        leader_info.leader_id = Some(self.local_id);
        leader_info.lease_expires_at = Some(Utc::now() + chrono::Duration::from_std(self.lease_duration).unwrap());
    }

    /// Step down from leadership.
    pub fn step_down(&self, new_leader: Option<Uuid>, term: u64) {
        let mut state = self.state.write();
        let mut leader_info = self.leader_info.write();
        let mut voted_for = self.voted_for.write();

        if term > leader_info.term {
            leader_info.term = term;
            *voted_for = None;
        }

        *state = LeaderState::Follower;
        leader_info.leader_id = new_leader;

        if new_leader.is_some() {
            leader_info.lease_expires_at = Some(Utc::now() + chrono::Duration::from_std(self.lease_duration).unwrap());
        } else {
            leader_info.lease_expires_at = None;
        }
    }

    /// Renew leader lease.
    pub fn renew_lease(&self) {
        if self.is_leader() {
            let mut leader_info = self.leader_info.write();
            leader_info.lease_expires_at = Some(Utc::now() + chrono::Duration::from_std(self.lease_duration).unwrap());
        }
    }

    /// Handle heartbeat from leader.
    pub fn handle_heartbeat(&self, leader_id: Uuid, term: u64) {
        let mut state = self.state.write();
        let mut leader_info = self.leader_info.write();

        if term >= leader_info.term {
            leader_info.term = term;
            leader_info.leader_id = Some(leader_id);
            leader_info.lease_expires_at = Some(Utc::now() + chrono::Duration::from_std(self.lease_duration).unwrap());

            // Revert to follower if not already
            if *state != LeaderState::Follower {
                *state = LeaderState::Follower;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_leader_election() {
        let node_id = Uuid::new_v4();
        let election = LeaderElection::new(
            node_id,
            Duration::from_millis(150),
            Duration::from_millis(500),
        );

        // Initially follower
        assert_eq!(election.state(), LeaderState::Follower);
        assert!(!election.is_leader());

        // Start election
        let term = election.start_election();
        assert_eq!(election.state(), LeaderState::Candidate);
        assert_eq!(term, 1);

        // Become leader
        election.become_leader();
        assert_eq!(election.state(), LeaderState::Leader);
        assert!(election.is_leader());
        assert_eq!(election.current_leader(), Some(node_id));
    }
}
