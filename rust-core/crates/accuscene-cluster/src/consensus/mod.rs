//! Consensus protocols for distributed agreement.

pub mod leader;
pub mod raft_lite;

pub use leader::{LeaderElection, LeaderInfo, LeaderState};
pub use raft_lite::{LogEntry, NextIndexTracker, RaftMessage, ReplicatedLog};

use crate::config::ConsensusConfig;
use crate::error::{ClusterError, Result};
use parking_lot::RwLock;
use std::sync::Arc;
use uuid::Uuid;

/// Consensus service.
pub struct ConsensusService {
    /// Local node ID
    local_id: Uuid,

    /// Leader election
    leader_election: Arc<LeaderElection>,

    /// Replicated log
    log: Arc<RwLock<ReplicatedLog>>,

    /// Configuration
    config: ConsensusConfig,
}

impl ConsensusService {
    /// Create a new consensus service.
    pub fn new(local_id: Uuid, config: ConsensusConfig) -> Self {
        let leader_election = Arc::new(LeaderElection::new(
            local_id,
            config.election_timeout.0,
            config.leader_lease,
        ));

        let log = Arc::new(RwLock::new(ReplicatedLog::new(
            config.max_log_entries,
        )));

        Self {
            local_id,
            leader_election,
            log,
            config,
        }
    }

    /// Check if this node is the leader.
    pub fn is_leader(&self) -> bool {
        self.leader_election.is_leader()
    }

    /// Get current leader.
    pub fn current_leader(&self) -> Option<Uuid> {
        self.leader_election.current_leader()
    }

    /// Get leader information.
    pub fn leader_info(&self) -> LeaderInfo {
        self.leader_election.leader_info()
    }

    /// Propose a value for consensus.
    pub async fn propose(&self, data: Vec<u8>) -> Result<u64> {
        if !self.is_leader() {
            return Err(ClusterError::NotLeader(self.current_leader()));
        }

        let mut log = self.log.write();
        let term = self.leader_election.current_term();
        let (last_index, _) = log.last_log_info();
        let new_index = last_index + 1;

        let entry = LogEntry::new(term, new_index, data);
        let index = log.append(entry);

        Ok(index)
    }

    /// Get committed entries.
    pub fn get_committed_entries(&self) -> Vec<LogEntry> {
        let log = self.log.read();
        log.unapplied_entries()
    }

    /// Handle Raft message.
    pub fn handle_message(&self, message: RaftMessage) -> Option<RaftMessage> {
        match message {
            RaftMessage::RequestVote {
                term,
                candidate_id,
                last_log_index,
                last_log_term,
            } => {
                let vote_granted = self.handle_request_vote(
                    term,
                    candidate_id,
                    last_log_index,
                    last_log_term,
                );

                Some(RaftMessage::RequestVoteReply {
                    term: self.leader_election.current_term(),
                    vote_granted,
                })
            }

            RaftMessage::AppendEntries {
                term,
                leader_id,
                prev_log_index,
                prev_log_term,
                entries,
                leader_commit,
            } => {
                let (success, match_index) = self.handle_append_entries(
                    term,
                    leader_id,
                    prev_log_index,
                    prev_log_term,
                    entries,
                    leader_commit,
                );

                Some(RaftMessage::AppendEntriesReply {
                    term: self.leader_election.current_term(),
                    success,
                    match_index,
                })
            }

            RaftMessage::RequestVoteReply { term, vote_granted } => {
                if vote_granted {
                    // Handle vote in election logic
                }
                None
            }

            RaftMessage::AppendEntriesReply {
                term,
                success,
                match_index,
            } => {
                // Handle response in replication logic
                None
            }
        }
    }

    /// Handle request vote RPC.
    fn handle_request_vote(
        &self,
        term: u64,
        candidate_id: Uuid,
        last_log_index: u64,
        last_log_term: u64,
    ) -> bool {
        let current_term = self.leader_election.current_term();

        // Reject if term is old
        if term < current_term {
            return false;
        }

        // Check if log is up-to-date
        let log = self.log.read();
        let (our_last_index, our_last_term) = log.last_log_info();

        let log_ok = last_log_term > our_last_term
            || (last_log_term == our_last_term && last_log_index >= our_last_index);

        if !log_ok {
            return false;
        }

        // Vote for candidate
        self.leader_election.vote(candidate_id, term)
    }

    /// Handle append entries RPC.
    fn handle_append_entries(
        &self,
        term: u64,
        leader_id: Uuid,
        prev_log_index: u64,
        prev_log_term: u64,
        entries: Vec<LogEntry>,
        leader_commit: u64,
    ) -> (bool, u64) {
        let current_term = self.leader_election.current_term();

        // Reject if term is old
        if term < current_term {
            return (false, 0);
        }

        // Update leader
        self.leader_election.handle_heartbeat(leader_id, term);

        let mut log = self.log.write();

        // Check previous log entry
        if prev_log_index > 0 {
            if let Some(entry) = log.get(prev_log_index) {
                if entry.term != prev_log_term {
                    return (false, 0);
                }
            } else {
                return (false, 0);
            }
        }

        // Append new entries
        if !entries.is_empty() {
            // Truncate conflicting entries
            if let Some(first) = entries.first() {
                log.truncate_from(first.index);
            }

            log.append_entries(entries);
        }

        // Update commit index
        if leader_commit > log.commit_index() {
            let (last_index, _) = log.last_log_info();
            let new_commit = leader_commit.min(last_index);
            log.set_commit_index(new_commit);
        }

        let (match_index, _) = log.last_log_info();
        (true, match_index)
    }

    /// Start election.
    pub fn start_election(&self) -> u64 {
        self.leader_election.start_election()
    }

    /// Become leader.
    pub fn become_leader(&self) {
        self.leader_election.become_leader();
    }

    /// Get leader election reference.
    pub fn leader_election(&self) -> &Arc<LeaderElection> {
        &self.leader_election
    }
}
