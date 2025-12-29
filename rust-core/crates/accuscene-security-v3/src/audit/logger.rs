//! Immutable audit logger with tamper detection

use super::events::AuditEvent;
use crate::config::AuditConfig;
use crate::error::{SecurityError, SecurityResult};
use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use parking_lot::RwLock;

/// Audit log entry with immutability guarantees
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    /// The audit event
    pub event: AuditEvent,

    /// Sequence number
    pub sequence: u64,

    /// Written at timestamp
    pub written_at: DateTime<Utc>,
}

/// Immutable audit logger
#[derive(Debug)]
pub struct AuditLogger {
    /// Configuration
    config: AuditConfig,

    /// Log storage (in-memory for this implementation)
    /// In production, this would write to a database or append-only log file
    logs: Arc<DashMap<String, AuditLog>>,

    /// Ordered log entries
    ordered_logs: Arc<RwLock<Vec<String>>>,

    /// Sequence counter
    sequence: Arc<RwLock<u64>>,

    /// Last event hash (for chain integrity)
    last_hash: Arc<RwLock<Option<String>>>,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new(config: AuditConfig) -> Self {
        Self {
            config,
            logs: Arc::new(DashMap::new()),
            ordered_logs: Arc::new(RwLock::new(Vec::new())),
            sequence: Arc::new(RwLock::new(0)),
            last_hash: Arc::new(RwLock::new(None)),
        }
    }

    /// Log an audit event
    pub fn log(&self, mut event: AuditEvent) -> SecurityResult<()> {
        // Check if we should log this event
        if !self.should_log(&event) {
            return Ok(());
        }

        // Set previous hash for chain integrity
        event.previous_hash = self.last_hash.read().clone();

        // Calculate event hash
        let event_hash = event.calculate_hash();
        event.event_hash = Some(event_hash.clone());

        // Get sequence number
        let mut seq = self.sequence.write();
        *seq += 1;
        let sequence = *seq;
        drop(seq);

        // Create log entry
        let log = AuditLog {
            event: event.clone(),
            sequence,
            written_at: Utc::now(),
        };

        // Store log
        self.logs.insert(event.id.clone(), log);
        self.ordered_logs.write().push(event.id.clone());

        // Update last hash
        *self.last_hash.write() = Some(event_hash);

        // Emit to tracing if available
        self.emit_trace(&event);

        Ok(())
    }

    /// Check if we should log this event based on configuration
    fn should_log(&self, event: &AuditEvent) -> bool {
        if !self.config.enable_audit_log {
            return false;
        }

        match &event.event_type {
            super::events::EventType::LoginSuccess => self.config.log_successful_auth,
            super::events::EventType::LoginFailure => self.config.log_failed_auth,
            super::events::EventType::PermissionGranted
            | super::events::EventType::PermissionDenied => self.config.log_authz_decisions,
            super::events::EventType::DataAccessed
            | super::events::EventType::DataCreated
            | super::events::EventType::DataUpdated
            | super::events::EventType::DataDeleted => self.config.log_data_access,
            super::events::EventType::ConfigurationChanged
            | super::events::EventType::SettingsUpdated => self.config.log_config_changes,
            _ => true,
        }
    }

    /// Emit trace log
    fn emit_trace(&self, event: &AuditEvent) {
        tracing::info!(
            event_id = %event.id,
            event_type = ?event.event_type,
            severity = ?event.severity,
            user_id = ?event.user_id,
            action = %event.action,
            outcome = %event.outcome,
            "Audit event logged"
        );
    }

    /// Get a log entry by ID
    pub fn get(&self, id: &str) -> Option<AuditLog> {
        self.logs.get(id).map(|entry| entry.clone())
    }

    /// Query logs with filters
    pub fn query(&self, filter: &AuditFilter) -> Vec<AuditLog> {
        let ordered = self.ordered_logs.read();
        let mut results = Vec::new();

        for id in ordered.iter() {
            if let Some(log) = self.logs.get(id) {
                if filter.matches(&log.event) {
                    results.push(log.clone());
                }
            }
        }

        // Apply limit
        if let Some(limit) = filter.limit {
            results.truncate(limit);
        }

        results
    }

    /// Get logs in a time range
    pub fn get_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<AuditLog> {
        let filter = AuditFilter {
            start_time: Some(start),
            end_time: Some(end),
            ..Default::default()
        };

        self.query(&filter)
    }

    /// Get recent logs
    pub fn get_recent(&self, count: usize) -> Vec<AuditLog> {
        let ordered = self.ordered_logs.read();
        let mut results = Vec::new();

        for id in ordered.iter().rev().take(count) {
            if let Some(log) = self.logs.get(id) {
                results.push(log.clone());
            }
        }

        results
    }

    /// Verify log integrity
    pub fn verify_integrity(&self) -> SecurityResult<bool> {
        let ordered = self.ordered_logs.read();
        let mut prev_hash: Option<String> = None;

        for id in ordered.iter() {
            if let Some(log) = self.logs.get(id) {
                let event = &log.event;

                // Verify event hash
                if !event.verify_hash() {
                    return Ok(false);
                }

                // Verify chain
                if event.previous_hash != prev_hash {
                    return Ok(false);
                }

                prev_hash = event.event_hash.clone();
            }
        }

        Ok(true)
    }

    /// Clean up old logs based on retention policy
    pub fn cleanup_old_logs(&self) -> SecurityResult<usize> {
        let cutoff = Utc::now() - Duration::days(self.config.retention_days as i64);
        let ordered = self.ordered_logs.read().clone();
        drop(ordered);

        let mut removed = 0;

        let ordered_clone = self.ordered_logs.read().clone();
        for id in ordered_clone.iter() {
            if let Some(log) = self.logs.get(id) {
                if log.written_at < cutoff {
                    self.logs.remove(id);
                    removed += 1;
                }
            }
        }

        // Update ordered list
        let mut ordered_write = self.ordered_logs.write();
        ordered_write.retain(|id| self.logs.contains_key(id));

        Ok(removed)
    }

    /// Get total log count
    pub fn count(&self) -> usize {
        self.logs.len()
    }

    /// Export logs to JSON
    pub fn export_json(&self, filter: &AuditFilter) -> SecurityResult<String> {
        let logs = self.query(filter);
        serde_json::to_string_pretty(&logs)
            .map_err(|e| SecurityError::SerializationError(e))
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new(AuditConfig::default())
    }
}

/// Filter for querying audit logs
#[derive(Debug, Clone, Default)]
pub struct AuditFilter {
    /// Filter by user ID
    pub user_id: Option<String>,

    /// Filter by event type
    pub event_type: Option<super::events::EventType>,

    /// Filter by severity (minimum)
    pub min_severity: Option<super::events::EventSeverity>,

    /// Filter by resource type
    pub resource_type: Option<String>,

    /// Filter by resource ID
    pub resource_id: Option<String>,

    /// Filter by outcome
    pub outcome: Option<String>,

    /// Start time
    pub start_time: Option<DateTime<Utc>>,

    /// End time
    pub end_time: Option<DateTime<Utc>>,

    /// Limit results
    pub limit: Option<usize>,
}

impl AuditFilter {
    /// Create a new filter
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by user
    pub fn with_user(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Filter by event type
    pub fn with_event_type(mut self, event_type: super::events::EventType) -> Self {
        self.event_type = Some(event_type);
        self
    }

    /// Filter by minimum severity
    pub fn with_min_severity(mut self, severity: super::events::EventSeverity) -> Self {
        self.min_severity = Some(severity);
        self
    }

    /// Limit results
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Check if event matches filter
    pub fn matches(&self, event: &AuditEvent) -> bool {
        if let Some(user_id) = &self.user_id {
            if event.user_id.as_ref() != Some(user_id) {
                return false;
            }
        }

        if let Some(event_type) = &self.event_type {
            if &event.event_type != event_type {
                return false;
            }
        }

        if let Some(min_severity) = &self.min_severity {
            if event.severity < *min_severity {
                return false;
            }
        }

        if let Some(resource_type) = &self.resource_type {
            if event.resource_type.as_ref() != Some(resource_type) {
                return false;
            }
        }

        if let Some(outcome) = &self.outcome {
            if &event.outcome != outcome {
                return false;
            }
        }

        if let Some(start) = self.start_time {
            if event.timestamp < start {
                return false;
            }
        }

        if let Some(end) = self.end_time {
            if event.timestamp > end {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::events::EventType;

    #[test]
    fn test_log_event() {
        let logger = AuditLogger::default();
        let event = AuditEvent::login_success("user123", "192.168.1.1");

        logger.log(event.clone()).unwrap();

        let retrieved = logger.get(&event.id).unwrap();
        assert_eq!(retrieved.event.id, event.id);
        assert_eq!(retrieved.sequence, 1);
    }

    #[test]
    fn test_log_chain_integrity() {
        let logger = AuditLogger::default();

        for i in 0..5 {
            let event = AuditEvent::new(EventType::LoginSuccess, "test")
                .with_user(format!("user{}", i));
            logger.log(event).unwrap();
        }

        assert!(logger.verify_integrity().unwrap());
    }

    #[test]
    fn test_query_filter() {
        let logger = AuditLogger::default();

        logger.log(AuditEvent::login_success("user1", "192.168.1.1")).unwrap();
        logger.log(AuditEvent::login_success("user2", "192.168.1.2")).unwrap();
        logger.log(AuditEvent::login_failure("user1", "192.168.1.1", "bad password")).unwrap();

        let filter = AuditFilter::new()
            .with_user("user1");

        let results = logger.query(&filter);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_get_recent() {
        let logger = AuditLogger::default();

        for i in 0..10 {
            let event = AuditEvent::new(EventType::DataAccessed, "test")
                .with_user(format!("user{}", i));
            logger.log(event).unwrap();
        }

        let recent = logger.get_recent(5);
        assert_eq!(recent.len(), 5);
    }

    #[test]
    fn test_tamper_detection() {
        let logger = AuditLogger::default();
        let event = AuditEvent::login_success("user123", "192.168.1.1");

        logger.log(event.clone()).unwrap();

        // Tamper with the log
        if let Some(mut log_entry) = logger.logs.get_mut(&event.id) {
            log_entry.event.action = "tampered".to_string();
        }

        // Integrity check should fail
        assert!(!logger.verify_integrity().unwrap());
    }

    #[test]
    fn test_export_json() {
        let logger = AuditLogger::default();
        logger.log(AuditEvent::login_success("user123", "192.168.1.1")).unwrap();

        let json = logger.export_json(&AuditFilter::new()).unwrap();
        assert!(json.contains("user123"));
    }
}
