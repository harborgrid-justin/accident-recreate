//! Brute force attack detection

use crate::error::{Result, SecurityError};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Brute force detector
pub struct BruteForceDetector {
    attempts: Arc<RwLock<HashMap<String, Vec<chrono::DateTime<chrono::Utc>>>>>,
    max_attempts: u32,
    window_secs: u64,
}

impl BruteForceDetector {
    /// Create a new brute force detector
    pub fn new(max_attempts: u32, window_secs: u64) -> Self {
        Self {
            attempts: Arc::new(RwLock::new(HashMap::new())),
            max_attempts,
            window_secs,
        }
    }

    /// Record failed login attempt
    pub async fn record_failed_attempt(&self, identifier: &str) -> Result<()> {
        let mut attempts = self.attempts.write().await;
        let now = chrono::Utc::now();

        let entry = attempts.entry(identifier.to_string()).or_insert_with(Vec::new);

        // Remove old attempts outside the window
        let cutoff = now - chrono::Duration::seconds(self.window_secs as i64);
        entry.retain(|t| *t > cutoff);

        // Add new attempt
        entry.push(now);

        // Check if threshold exceeded
        if entry.len() as u32 >= self.max_attempts {
            return Err(SecurityError::BruteForceDetected {
                source: identifier.to_string(),
            });
        }

        Ok(())
    }

    /// Check if identifier is currently locked out
    pub async fn is_locked_out(&self, identifier: &str) -> bool {
        let attempts = self.attempts.read().await;

        if let Some(entry) = attempts.get(identifier) {
            let now = chrono::Utc::now();
            let cutoff = now - chrono::Duration::seconds(self.window_secs as i64);

            let recent_attempts = entry.iter().filter(|t| **t > cutoff).count();
            recent_attempts as u32 >= self.max_attempts
        } else {
            false
        }
    }

    /// Clear attempts for identifier
    pub async fn clear_attempts(&self, identifier: &str) {
        let mut attempts = self.attempts.write().await;
        attempts.remove(identifier);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_brute_force_detection() {
        let detector = BruteForceDetector::new(3, 300);

        // First 2 attempts should be ok
        assert!(detector.record_failed_attempt("user123").await.is_ok());
        assert!(detector.record_failed_attempt("user123").await.is_ok());

        // 3rd attempt should trigger detection
        assert!(detector.record_failed_attempt("user123").await.is_err());

        // Should be locked out
        assert!(detector.is_locked_out("user123").await);
    }

    #[tokio::test]
    async fn test_clear_attempts() {
        let detector = BruteForceDetector::new(3, 300);

        detector.record_failed_attempt("user123").await.ok();
        detector.clear_attempts("user123").await;

        assert!(!detector.is_locked_out("user123").await);
    }
}
