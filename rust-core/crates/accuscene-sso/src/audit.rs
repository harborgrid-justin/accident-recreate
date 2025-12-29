//! Authentication Audit Trail Module

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Audit event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type", rename_all = "snake_case")]
pub enum AuditEvent {
    /// User login initiated
    LoginInitiated {
        provider: String,
        timestamp: DateTime<Utc>,
    },

    /// User login succeeded
    LoginSucceeded {
        user_id: String,
        provider: String,
        timestamp: DateTime<Utc>,
    },

    /// User login failed
    LoginFailed {
        reason: String,
        provider: String,
        timestamp: DateTime<Utc>,
    },

    /// Logout completed
    LogoutCompleted {
        session_id: Uuid,
        timestamp: DateTime<Utc>,
    },

    /// Token refreshed
    TokenRefreshed {
        user_id: String,
        session_id: Uuid,
        timestamp: DateTime<Utc>,
    },

    /// Token validation failed
    TokenValidationFailed {
        reason: String,
        timestamp: DateTime<Utc>,
    },

    /// MFA enrollment started
    MFAEnrollmentStarted {
        user_id: String,
        method: String,
        timestamp: DateTime<Utc>,
    },

    /// MFA enrollment completed
    MFAEnrollmentCompleted {
        user_id: String,
        method: String,
        timestamp: DateTime<Utc>,
    },

    /// MFA verification succeeded
    MFAVerificationSucceeded {
        user_id: String,
        method: String,
        timestamp: DateTime<Utc>,
    },

    /// MFA verification failed
    MFAVerificationFailed {
        user_id: String,
        method: String,
        reason: String,
        timestamp: DateTime<Utc>,
    },

    /// Session expired
    SessionExpired {
        session_id: Uuid,
        timestamp: DateTime<Utc>,
    },

    /// Account locked
    AccountLocked {
        user_id: String,
        reason: String,
        timestamp: DateTime<Utc>,
    },

    /// Account unlocked
    AccountUnlocked {
        user_id: String,
        timestamp: DateTime<Utc>,
    },

    /// Password changed
    PasswordChanged {
        user_id: String,
        timestamp: DateTime<Utc>,
    },

    /// Security alert
    SecurityAlert {
        alert_type: String,
        description: String,
        severity: String,
        timestamp: DateTime<Utc>,
    },
}

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    /// Entry ID
    pub id: Uuid,

    /// Event
    pub event: AuditEvent,

    /// IP address
    pub ip_address: Option<String>,

    /// User agent
    pub user_agent: Option<String>,

    /// Additional metadata
    pub metadata: serde_json::Value,

    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

impl AuditLogEntry {
    /// Create new audit log entry
    pub fn new(event: AuditEvent) -> Self {
        Self {
            id: Uuid::new_v4(),
            event,
            ip_address: None,
            user_agent: None,
            metadata: serde_json::json!({}),
            timestamp: Utc::now(),
        }
    }

    /// Set IP address
    pub fn with_ip(mut self, ip: String) -> Self {
        self.ip_address = Some(ip);
        self
    }

    /// Set user agent
    pub fn with_user_agent(mut self, ua: String) -> Self {
        self.user_agent = Some(ua);
        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Audit trail storage
pub struct AuditTrail {
    /// In-memory log storage (use database in production)
    logs: Arc<RwLock<Vec<AuditLogEntry>>>,
}

impl AuditTrail {
    /// Create new audit trail
    pub fn new() -> Self {
        Self {
            logs: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Add log entry
    pub async fn log(&self, entry: AuditLogEntry) {
        let mut logs = self.logs.write().await;
        logs.push(entry);
    }

    /// Get logs for user
    pub async fn get_user_logs(&self, user_id: &str) -> Vec<AuditLogEntry> {
        let logs = self.logs.read().await;
        logs.iter()
            .filter(|entry| {
                match &entry.event {
                    AuditEvent::LoginSucceeded { user_id: uid, .. } => uid == user_id,
                    AuditEvent::LoginFailed { .. } => false, // Don't include failed logins
                    AuditEvent::TokenRefreshed { user_id: uid, .. } => uid == user_id,
                    AuditEvent::MFAEnrollmentStarted { user_id: uid, .. } => uid == user_id,
                    AuditEvent::MFAEnrollmentCompleted { user_id: uid, .. } => uid == user_id,
                    AuditEvent::MFAVerificationSucceeded { user_id: uid, .. } => uid == user_id,
                    AuditEvent::MFAVerificationFailed { user_id: uid, .. } => uid == user_id,
                    AuditEvent::AccountLocked { user_id: uid, .. } => uid == user_id,
                    AuditEvent::AccountUnlocked { user_id: uid, .. } => uid == user_id,
                    AuditEvent::PasswordChanged { user_id: uid, .. } => uid == user_id,
                    _ => false,
                }
            })
            .cloned()
            .collect()
    }

    /// Get logs within time range
    pub async fn get_logs_in_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<AuditLogEntry> {
        let logs = self.logs.read().await;
        logs.iter()
            .filter(|entry| entry.timestamp >= start && entry.timestamp <= end)
            .cloned()
            .collect()
    }

    /// Get security alerts
    pub async fn get_security_alerts(&self) -> Vec<AuditLogEntry> {
        let logs = self.logs.read().await;
        logs.iter()
            .filter(|entry| matches!(entry.event, AuditEvent::SecurityAlert { .. }))
            .cloned()
            .collect()
    }

    /// Get recent logs
    pub async fn get_recent_logs(&self, limit: usize) -> Vec<AuditLogEntry> {
        let logs = self.logs.read().await;
        logs.iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }
}

impl Default for AuditTrail {
    fn default() -> Self {
        Self::new()
    }
}

/// Audit logger
pub struct AuditLogger {
    trail: Arc<AuditTrail>,
}

impl AuditLogger {
    /// Create new audit logger
    pub fn new() -> Self {
        Self {
            trail: Arc::new(AuditTrail::new()),
        }
    }

    /// Log an event
    pub async fn log(&self, event: AuditEvent) {
        let entry = AuditLogEntry::new(event);
        self.trail.log(entry).await;
    }

    /// Log with context
    pub async fn log_with_context(
        &self,
        event: AuditEvent,
        ip: Option<String>,
        user_agent: Option<String>,
        metadata: Option<serde_json::Value>,
    ) {
        let mut entry = AuditLogEntry::new(event);

        if let Some(ip) = ip {
            entry = entry.with_ip(ip);
        }

        if let Some(ua) = user_agent {
            entry = entry.with_user_agent(ua);
        }

        if let Some(meta) = metadata {
            entry = entry.with_metadata(meta);
        }

        self.trail.log(entry).await;
    }

    /// Get audit trail
    pub fn trail(&self) -> Arc<AuditTrail> {
        self.trail.clone()
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_audit_logging() {
        let logger = AuditLogger::new();

        logger.log(AuditEvent::LoginSucceeded {
            user_id: "user123".to_string(),
            provider: "oidc".to_string(),
            timestamp: Utc::now(),
        }).await;

        let trail = logger.trail();
        let recent = trail.get_recent_logs(10).await;
        assert_eq!(recent.len(), 1);
    }

    #[tokio::test]
    async fn test_user_logs_filter() {
        let trail = AuditTrail::new();

        trail.log(AuditLogEntry::new(AuditEvent::LoginSucceeded {
            user_id: "user123".to_string(),
            provider: "oidc".to_string(),
            timestamp: Utc::now(),
        })).await;

        trail.log(AuditLogEntry::new(AuditEvent::LoginSucceeded {
            user_id: "user456".to_string(),
            provider: "saml".to_string(),
            timestamp: Utc::now(),
        })).await;

        let user_logs = trail.get_user_logs("user123").await;
        assert_eq!(user_logs.len(), 1);
    }
}
