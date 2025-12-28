//! Audit trail with tamper detection
//!
//! Implements cryptographically secured audit trail using hash chains.

use crate::audit::event::AuditEvent;
use crate::error::{Result, SecurityError};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Audit trail with tamper detection
pub struct AuditTrail {
    entries: Vec<TrailEntry>,
    last_hash: Option<String>,
}

impl AuditTrail {
    /// Create a new audit trail
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            last_hash: None,
        }
    }

    /// Add an event to the trail
    pub fn add_event(&mut self, event: AuditEvent) -> Result<String> {
        let entry = TrailEntry::new(event, self.last_hash.clone());
        let entry_hash = entry.hash.clone();

        self.entries.push(entry);
        self.last_hash = Some(entry_hash.clone());

        Ok(entry_hash)
    }

    /// Verify trail integrity
    pub fn verify_integrity(&self) -> Result<()> {
        let mut expected_hash: Option<String> = None;

        for (i, entry) in self.entries.iter().enumerate() {
            // Check if previous hash matches
            if entry.previous_hash != expected_hash {
                return Err(SecurityError::AuditTrailCompromised(format!(
                    "Hash chain broken at entry {}",
                    i
                )));
            }

            // Verify entry hash
            if !entry.verify_hash() {
                return Err(SecurityError::AuditTrailCompromised(format!(
                    "Entry {} hash mismatch",
                    i
                )));
            }

            expected_hash = Some(entry.hash.clone());
        }

        Ok(())
    }

    /// Get all entries
    pub fn entries(&self) -> &[TrailEntry] {
        &self.entries
    }

    /// Get entry count
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if trail is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get the last hash
    pub fn last_hash(&self) -> Option<&str> {
        self.last_hash.as_deref()
    }

    /// Export trail for archival
    pub fn export(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(&self.entries)
            .map_err(|e| SecurityError::Internal(format!("Export failed: {}", e)))
    }

    /// Import trail from archive
    pub fn import(data: &[u8]) -> Result<Self> {
        let entries: Vec<TrailEntry> = serde_json::from_slice(data)
            .map_err(|e| SecurityError::Internal(format!("Import failed: {}", e)))?;

        let last_hash = entries.last().map(|e| e.hash.clone());

        let trail = Self {
            entries,
            last_hash,
        };

        // Verify imported trail
        trail.verify_integrity()?;

        Ok(trail)
    }
}

impl Default for AuditTrail {
    fn default() -> Self {
        Self::new()
    }
}

/// Audit trail entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrailEntry {
    /// The audit event
    pub event: AuditEvent,
    /// Hash of previous entry
    pub previous_hash: Option<String>,
    /// Hash of this entry
    pub hash: String,
    /// Sequence number
    pub sequence: u64,
    /// Entry timestamp (when added to trail)
    pub trail_timestamp: chrono::DateTime<chrono::Utc>,
}

impl TrailEntry {
    /// Create a new trail entry
    pub fn new(event: AuditEvent, previous_hash: Option<String>) -> Self {
        let sequence = previous_hash
            .as_ref()
            .map(|_| 1u64) // In production, would increment from last
            .unwrap_or(0);

        let trail_timestamp = chrono::Utc::now();

        let hash = Self::calculate_hash(&event, &previous_hash, sequence, &trail_timestamp);

        Self {
            event,
            previous_hash,
            hash,
            sequence,
            trail_timestamp,
        }
    }

    /// Calculate hash for this entry
    fn calculate_hash(
        event: &AuditEvent,
        previous_hash: &Option<String>,
        sequence: u64,
        timestamp: &chrono::DateTime<chrono::Utc>,
    ) -> String {
        let mut hasher = Sha256::new();

        // Hash the event
        if let Ok(event_json) = serde_json::to_string(event) {
            hasher.update(event_json.as_bytes());
        }

        // Hash previous hash
        if let Some(prev) = previous_hash {
            hasher.update(prev.as_bytes());
        }

        // Hash sequence number
        hasher.update(sequence.to_le_bytes());

        // Hash timestamp
        hasher.update(timestamp.to_rfc3339().as_bytes());

        hex::encode(hasher.finalize())
    }

    /// Verify this entry's hash
    pub fn verify_hash(&self) -> bool {
        let expected_hash = Self::calculate_hash(
            &self.event,
            &self.previous_hash,
            self.sequence,
            &self.trail_timestamp,
        );
        self.hash == expected_hash
    }
}

/// Merkle tree for batch verification
pub struct MerkleTree {
    root: Option<String>,
    leaves: Vec<String>,
}

impl MerkleTree {
    /// Build Merkle tree from trail entries
    pub fn from_trail(trail: &AuditTrail) -> Self {
        let leaves: Vec<String> = trail.entries().iter().map(|e| e.hash.clone()).collect();

        let root = if !leaves.is_empty() {
            Some(Self::build_root(&leaves))
        } else {
            None
        };

        Self { root, leaves }
    }

    /// Build Merkle root from leaves
    fn build_root(leaves: &[String]) -> String {
        if leaves.is_empty() {
            return String::new();
        }

        if leaves.len() == 1 {
            return leaves[0].clone();
        }

        let mut level = leaves.to_vec();

        while level.len() > 1 {
            let mut next_level = Vec::new();

            for chunk in level.chunks(2) {
                let hash = if chunk.len() == 2 {
                    Self::hash_pair(&chunk[0], &chunk[1])
                } else {
                    chunk[0].clone()
                };
                next_level.push(hash);
            }

            level = next_level;
        }

        level[0].clone()
    }

    /// Hash a pair of nodes
    fn hash_pair(left: &str, right: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(left.as_bytes());
        hasher.update(right.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Get Merkle root
    pub fn root(&self) -> Option<&str> {
        self.root.as_deref()
    }

    /// Verify a leaf is in the tree
    pub fn verify_leaf(&self, leaf_hash: &str) -> bool {
        self.leaves.contains(&leaf_hash.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::event::EventType;

    #[test]
    fn test_audit_trail_creation() {
        let trail = AuditTrail::new();
        assert_eq!(trail.len(), 0);
        assert!(trail.is_empty());
    }

    #[test]
    fn test_add_event_to_trail() {
        let mut trail = AuditTrail::new();
        let event = AuditEvent::new(EventType::AuthLogin, "login".to_string());

        let hash = trail.add_event(event).unwrap();
        assert!(!hash.is_empty());
        assert_eq!(trail.len(), 1);
    }

    #[test]
    fn test_trail_integrity_verification() {
        let mut trail = AuditTrail::new();

        // Add multiple events
        for i in 0..5 {
            let event = AuditEvent::new(
                EventType::AuthLogin,
                format!("login-{}", i),
            );
            trail.add_event(event).unwrap();
        }

        // Trail should be valid
        assert!(trail.verify_integrity().is_ok());
    }

    #[test]
    fn test_tamper_detection() {
        let mut trail = AuditTrail::new();

        // Add events
        for i in 0..3 {
            let event = AuditEvent::new(
                EventType::AuthLogin,
                format!("login-{}", i),
            );
            trail.add_event(event).unwrap();
        }

        // Tamper with an entry
        if let Some(entry) = trail.entries.get_mut(1) {
            entry.hash = "tampered_hash".to_string();
        }

        // Verification should fail
        assert!(trail.verify_integrity().is_err());
    }

    #[test]
    fn test_trail_export_import() {
        let mut trail = AuditTrail::new();

        // Add events
        for i in 0..3 {
            let event = AuditEvent::new(
                EventType::AuthLogin,
                format!("login-{}", i),
            );
            trail.add_event(event).unwrap();
        }

        // Export
        let exported = trail.export().unwrap();

        // Import
        let imported = AuditTrail::import(&exported).unwrap();

        assert_eq!(imported.len(), trail.len());
        assert!(imported.verify_integrity().is_ok());
    }

    #[test]
    fn test_merkle_tree() {
        let mut trail = AuditTrail::new();

        // Add events
        for i in 0..4 {
            let event = AuditEvent::new(
                EventType::AuthLogin,
                format!("login-{}", i),
            );
            trail.add_event(event).unwrap();
        }

        let tree = MerkleTree::from_trail(&trail);

        assert!(tree.root().is_some());
        assert_eq!(tree.leaves.len(), 4);

        // Verify leaves
        for entry in trail.entries() {
            assert!(tree.verify_leaf(&entry.hash));
        }
    }
}
