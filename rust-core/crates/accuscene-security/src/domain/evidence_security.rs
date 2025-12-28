//! Evidence chain of custody

use crate::error::{Result, SecurityError};
use serde::{Deserialize, Serialize};

/// Chain of custody entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainOfCustodyEntry {
    pub evidence_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub handler: String,
    pub action: String,
    pub hash: String,
}

impl ChainOfCustodyEntry {
    /// Create a new entry
    pub fn new(evidence_id: String, handler: String, action: String) -> Self {
        let timestamp = chrono::Utc::now();
        let hash = Self::calculate_hash(&evidence_id, &handler, &action, &timestamp);

        Self {
            evidence_id,
            timestamp,
            handler,
            action,
            hash,
        }
    }

    /// Calculate hash for tamper detection
    fn calculate_hash(
        evidence_id: &str,
        handler: &str,
        action: &str,
        timestamp: &chrono::DateTime<chrono::Utc>,
    ) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(evidence_id.as_bytes());
        hasher.update(handler.as_bytes());
        hasher.update(action.as_bytes());
        hasher.update(timestamp.to_rfc3339().as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Verify entry integrity
    pub fn verify(&self) -> bool {
        let expected_hash = Self::calculate_hash(
            &self.evidence_id,
            &self.handler,
            &self.action,
            &self.timestamp,
        );
        self.hash == expected_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_of_custody() {
        let entry = ChainOfCustodyEntry::new(
            "evidence-123".to_string(),
            "investigator-1".to_string(),
            "collected".to_string(),
        );

        assert!(entry.verify());
    }
}
