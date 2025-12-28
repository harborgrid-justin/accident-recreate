//! Lightweight Raft-like consensus protocol.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use uuid::Uuid;

/// Log entry for consensus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Term when entry was received
    pub term: u64,

    /// Index in the log
    pub index: u64,

    /// Command/data
    pub data: Vec<u8>,

    /// Entry timestamp
    pub timestamp: i64,
}

impl LogEntry {
    /// Create a new log entry.
    pub fn new(term: u64, index: u64, data: Vec<u8>) -> Self {
        Self {
            term,
            index,
            data,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
}

/// Raft RPC messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RaftMessage {
    /// Request vote
    RequestVote {
        term: u64,
        candidate_id: Uuid,
        last_log_index: u64,
        last_log_term: u64,
    },

    /// Vote response
    RequestVoteReply {
        term: u64,
        vote_granted: bool,
    },

    /// Append entries (heartbeat when empty)
    AppendEntries {
        term: u64,
        leader_id: Uuid,
        prev_log_index: u64,
        prev_log_term: u64,
        entries: Vec<LogEntry>,
        leader_commit: u64,
    },

    /// Append entries response
    AppendEntriesReply {
        term: u64,
        success: bool,
        match_index: u64,
    },
}

/// Replicated log.
pub struct ReplicatedLog {
    /// Log entries
    entries: VecDeque<LogEntry>,

    /// Commit index
    commit_index: u64,

    /// Last applied index
    last_applied: u64,

    /// Maximum log size
    max_size: usize,
}

impl ReplicatedLog {
    /// Create a new replicated log.
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            commit_index: 0,
            last_applied: 0,
            max_size,
        }
    }

    /// Append a new entry.
    pub fn append(&mut self, entry: LogEntry) -> u64 {
        let index = entry.index;
        self.entries.push_back(entry);

        // Trim old entries if needed
        while self.entries.len() > self.max_size {
            if let Some(first) = self.entries.front() {
                if first.index < self.last_applied {
                    self.entries.pop_front();
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        index
    }

    /// Append multiple entries.
    pub fn append_entries(&mut self, entries: Vec<LogEntry>) {
        for entry in entries {
            self.append(entry);
        }
    }

    /// Get entry at index.
    pub fn get(&self, index: u64) -> Option<&LogEntry> {
        self.entries
            .iter()
            .find(|e| e.index == index)
    }

    /// Get entries from index.
    pub fn get_from(&self, start_index: u64, limit: usize) -> Vec<LogEntry> {
        self.entries
            .iter()
            .filter(|e| e.index >= start_index)
            .take(limit)
            .cloned()
            .collect()
    }

    /// Get last log index and term.
    pub fn last_log_info(&self) -> (u64, u64) {
        if let Some(last) = self.entries.back() {
            (last.index, last.term)
        } else {
            (0, 0)
        }
    }

    /// Set commit index.
    pub fn set_commit_index(&mut self, index: u64) {
        if index > self.commit_index {
            self.commit_index = index;
        }
    }

    /// Get commit index.
    pub fn commit_index(&self) -> u64 {
        self.commit_index
    }

    /// Mark as applied.
    pub fn set_last_applied(&mut self, index: u64) {
        self.last_applied = index;
    }

    /// Get unapplied entries.
    pub fn unapplied_entries(&self) -> Vec<LogEntry> {
        self.entries
            .iter()
            .filter(|e| e.index > self.last_applied && e.index <= self.commit_index)
            .cloned()
            .collect()
    }

    /// Truncate log from index.
    pub fn truncate_from(&mut self, index: u64) {
        self.entries.retain(|e| e.index < index);
    }

    /// Get log length.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if log is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Next index tracker for followers.
pub struct NextIndexTracker {
    /// Next index for each follower
    next_index: std::collections::HashMap<Uuid, u64>,

    /// Match index for each follower
    match_index: std::collections::HashMap<Uuid, u64>,
}

impl NextIndexTracker {
    /// Create a new tracker.
    pub fn new() -> Self {
        Self {
            next_index: std::collections::HashMap::new(),
            match_index: std::collections::HashMap::new(),
        }
    }

    /// Initialize for a follower.
    pub fn init_follower(&mut self, follower_id: Uuid, last_log_index: u64) {
        self.next_index.insert(follower_id, last_log_index + 1);
        self.match_index.insert(follower_id, 0);
    }

    /// Get next index for follower.
    pub fn get_next_index(&self, follower_id: &Uuid) -> u64 {
        self.next_index.get(follower_id).copied().unwrap_or(1)
    }

    /// Get match index for follower.
    pub fn get_match_index(&self, follower_id: &Uuid) -> u64 {
        self.match_index.get(follower_id).copied().unwrap_or(0)
    }

    /// Update on successful append.
    pub fn update_success(&mut self, follower_id: Uuid, match_index: u64) {
        self.match_index.insert(follower_id, match_index);
        self.next_index.insert(follower_id, match_index + 1);
    }

    /// Update on failed append.
    pub fn update_failure(&mut self, follower_id: Uuid) {
        if let Some(next) = self.next_index.get_mut(&follower_id) {
            if *next > 1 {
                *next -= 1;
            }
        }
    }

    /// Calculate commit index based on majority.
    pub fn calculate_commit_index(&self, current_commit: u64, total_nodes: usize) -> u64 {
        let mut indices: Vec<u64> = self.match_index.values().copied().collect();
        indices.sort_unstable();

        // Need majority
        let majority = (total_nodes / 2) + 1;
        if indices.len() + 1 >= majority {
            // +1 for leader itself
            let idx = indices.len().saturating_sub(majority - 1);
            indices.get(idx).copied().unwrap_or(current_commit).max(current_commit)
        } else {
            current_commit
        }
    }

    /// Remove follower.
    pub fn remove_follower(&mut self, follower_id: &Uuid) {
        self.next_index.remove(follower_id);
        self.match_index.remove(follower_id);
    }
}

impl Default for NextIndexTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replicated_log() {
        let mut log = ReplicatedLog::new(100);

        // Append entries
        log.append(LogEntry::new(1, 1, vec![1, 2, 3]));
        log.append(LogEntry::new(1, 2, vec![4, 5, 6]));
        log.append(LogEntry::new(2, 3, vec![7, 8, 9]));

        assert_eq!(log.len(), 3);

        let (last_index, last_term) = log.last_log_info();
        assert_eq!(last_index, 3);
        assert_eq!(last_term, 2);

        // Commit
        log.set_commit_index(2);
        assert_eq!(log.commit_index(), 2);
    }

    #[test]
    fn test_next_index_tracker() {
        let mut tracker = NextIndexTracker::new();
        let follower1 = Uuid::new_v4();
        let follower2 = Uuid::new_v4();

        tracker.init_follower(follower1, 10);
        tracker.init_follower(follower2, 10);

        assert_eq!(tracker.get_next_index(&follower1), 11);

        // Success
        tracker.update_success(follower1, 15);
        assert_eq!(tracker.get_next_index(&follower1), 16);
        assert_eq!(tracker.get_match_index(&follower1), 15);

        // Failure
        tracker.update_failure(follower2);
        assert_eq!(tracker.get_next_index(&follower2), 10);
    }
}
