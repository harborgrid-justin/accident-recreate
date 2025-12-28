//! Node state machine and transitions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Node state in the cluster.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeState {
    /// Node is initializing
    Initializing,
    /// Node is joining the cluster
    Joining,
    /// Node is active and healthy
    Active,
    /// Node is suspected to be failed
    Suspected,
    /// Node has failed
    Failed,
    /// Node is leaving gracefully
    Leaving,
    /// Node has left the cluster
    Left,
    /// Node is in maintenance mode
    Maintenance,
}

impl NodeState {
    /// Check if node is operational.
    pub fn is_operational(&self) -> bool {
        matches!(
            self,
            NodeState::Active | NodeState::Joining | NodeState::Maintenance
        )
    }

    /// Check if node can accept requests.
    pub fn can_accept_requests(&self) -> bool {
        matches!(self, NodeState::Active | NodeState::Maintenance)
    }

    /// Check if node is failed or leaving.
    pub fn is_unavailable(&self) -> bool {
        matches!(
            self,
            NodeState::Failed | NodeState::Leaving | NodeState::Left
        )
    }
}

/// State transition event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    /// Previous state
    pub from: NodeState,

    /// New state
    pub to: NodeState,

    /// Transition timestamp
    pub timestamp: DateTime<Utc>,

    /// Reason for transition
    pub reason: String,
}

impl StateTransition {
    /// Create a new state transition.
    pub fn new(from: NodeState, to: NodeState, reason: impl Into<String>) -> Self {
        Self {
            from,
            to,
            timestamp: Utc::now(),
            reason: reason.into(),
        }
    }
}

/// Node state machine with transition history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStateMachine {
    /// Current state
    current: NodeState,

    /// State transition history
    history: Vec<StateTransition>,

    /// Maximum history size
    max_history: usize,
}

impl NodeStateMachine {
    /// Create a new state machine.
    pub fn new() -> Self {
        Self {
            current: NodeState::Initializing,
            history: Vec::new(),
            max_history: 100,
        }
    }

    /// Get current state.
    pub fn current(&self) -> NodeState {
        self.current
    }

    /// Transition to a new state.
    pub fn transition(&mut self, new_state: NodeState, reason: impl Into<String>) -> bool {
        if self.can_transition(new_state) {
            let transition = StateTransition::new(self.current, new_state, reason);
            self.current = new_state;
            self.history.push(transition);

            // Trim history if needed
            if self.history.len() > self.max_history {
                self.history.remove(0);
            }

            true
        } else {
            false
        }
    }

    /// Check if transition is valid.
    fn can_transition(&self, new_state: NodeState) -> bool {
        use NodeState::*;

        match (self.current, new_state) {
            // Same state is always allowed
            (a, b) if a == b => true,

            // Valid transitions
            (Initializing, Joining) => true,
            (Joining, Active) => true,
            (Active, Suspected) => true,
            (Active, Leaving) => true,
            (Active, Maintenance) => true,
            (Suspected, Active) => true,
            (Suspected, Failed) => true,
            (Maintenance, Active) => true,
            (Leaving, Left) => true,
            (Failed, Initializing) => true, // Recovery

            // Invalid transitions
            _ => false,
        }
    }

    /// Get transition history.
    pub fn history(&self) -> &[StateTransition] {
        &self.history
    }

    /// Get last transition.
    pub fn last_transition(&self) -> Option<&StateTransition> {
        self.history.last()
    }
}

impl Default for NodeStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_transitions() {
        let mut sm = NodeStateMachine::new();

        assert_eq!(sm.current(), NodeState::Initializing);
        assert!(sm.transition(NodeState::Joining, "Starting join"));
        assert_eq!(sm.current(), NodeState::Joining);
        assert!(sm.transition(NodeState::Active, "Join complete"));
        assert_eq!(sm.current(), NodeState::Active);

        // Invalid transition
        assert!(!sm.transition(NodeState::Left, "Invalid"));
        assert_eq!(sm.current(), NodeState::Active);

        // Valid suspicion
        assert!(sm.transition(NodeState::Suspected, "Health check failed"));
        assert_eq!(sm.current(), NodeState::Suspected);
    }
}
