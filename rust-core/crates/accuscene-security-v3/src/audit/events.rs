//! Audit event types and definitions

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Event severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EventSeverity {
    /// Informational events
    Info,
    /// Low severity
    Low,
    /// Medium severity
    Medium,
    /// High severity
    High,
    /// Critical events requiring immediate attention
    Critical,
}

/// Event types for audit logging
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    // Authentication events
    /// User login attempt
    LoginAttempt,
    /// Successful login
    LoginSuccess,
    /// Failed login
    LoginFailure,
    /// User logout
    Logout,
    /// Password changed
    PasswordChanged,
    /// Password reset requested
    PasswordResetRequested,
    /// MFA enrollment
    MfaEnrolled,
    /// MFA disabled
    MfaDisabled,

    // Authorization events
    /// Permission granted
    PermissionGranted,
    /// Permission denied
    PermissionDenied,
    /// Role assigned
    RoleAssigned,
    /// Role removed
    RoleRemoved,
    /// Policy created
    PolicyCreated,
    /// Policy updated
    PolicyUpdated,
    /// Policy deleted
    PolicyDeleted,

    // Data access events
    /// Data accessed
    DataAccessed,
    /// Data created
    DataCreated,
    /// Data updated
    DataUpdated,
    /// Data deleted
    DataDeleted,
    /// Data exported
    DataExported,

    // Security events
    /// Security alert
    SecurityAlert,
    /// Suspicious activity detected
    SuspiciousActivity,
    /// Account locked
    AccountLocked,
    /// Account unlocked
    AccountUnlocked,
    /// Session expired
    SessionExpired,
    /// Token issued
    TokenIssued,
    /// Token revoked
    TokenRevoked,

    // Configuration events
    /// Configuration changed
    ConfigurationChanged,
    /// Settings updated
    SettingsUpdated,

    // System events
    /// System started
    SystemStarted,
    /// System stopped
    SystemStopped,
    /// System error
    SystemError,

    /// Custom event type
    Custom(String),
}

impl EventType {
    /// Get the default severity for this event type
    pub fn default_severity(&self) -> EventSeverity {
        match self {
            Self::LoginSuccess | Self::Logout | Self::DataAccessed => EventSeverity::Info,

            Self::LoginAttempt
            | Self::PasswordChanged
            | Self::MfaEnrolled
            | Self::DataCreated
            | Self::DataUpdated
            | Self::TokenIssued => EventSeverity::Low,

            Self::PermissionDenied
            | Self::PasswordResetRequested
            | Self::DataDeleted
            | Self::SessionExpired
            | Self::ConfigurationChanged => EventSeverity::Medium,

            Self::LoginFailure
            | Self::AccountLocked
            | Self::MfaDisabled
            | Self::DataExported
            | Self::SuspiciousActivity => EventSeverity::High,

            Self::SecurityAlert
            | Self::AccountUnlocked
            | Self::TokenRevoked
            | Self::SystemError => EventSeverity::Critical,

            _ => EventSeverity::Info,
        }
    }

    /// Check if this event type should trigger an alert
    pub fn should_alert(&self) -> bool {
        matches!(
            self,
            Self::LoginFailure
                | Self::SecurityAlert
                | Self::SuspiciousActivity
                | Self::AccountLocked
                | Self::SystemError
        )
    }
}

/// Audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unique event ID
    pub id: String,

    /// Event timestamp
    pub timestamp: DateTime<Utc>,

    /// Event type
    pub event_type: EventType,

    /// Event severity
    pub severity: EventSeverity,

    /// User ID (if applicable)
    pub user_id: Option<String>,

    /// Resource type (if applicable)
    pub resource_type: Option<String>,

    /// Resource ID (if applicable)
    pub resource_id: Option<String>,

    /// Action performed
    pub action: String,

    /// Event outcome (success, failure, etc.)
    pub outcome: String,

    /// IP address
    pub ip_address: Option<String>,

    /// User agent
    pub user_agent: Option<String>,

    /// Session ID
    pub session_id: Option<String>,

    /// Error message (if applicable)
    pub error: Option<String>,

    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,

    /// Hash of previous event (for tamper detection)
    pub previous_hash: Option<String>,

    /// Hash of this event
    pub event_hash: Option<String>,
}

impl AuditEvent {
    /// Create a new audit event
    pub fn new(event_type: EventType, action: impl Into<String>) -> Self {
        let severity = event_type.default_severity();

        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type,
            severity,
            user_id: None,
            resource_type: None,
            resource_id: None,
            action: action.into(),
            outcome: "unknown".to_string(),
            ip_address: None,
            user_agent: None,
            session_id: None,
            error: None,
            metadata: HashMap::new(),
            previous_hash: None,
            event_hash: None,
        }
    }

    /// Set user ID
    pub fn with_user(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Set resource
    pub fn with_resource(
        mut self,
        resource_type: impl Into<String>,
        resource_id: impl Into<String>,
    ) -> Self {
        self.resource_type = Some(resource_type.into());
        self.resource_id = Some(resource_id.into());
        self
    }

    /// Set outcome
    pub fn with_outcome(mut self, outcome: impl Into<String>) -> Self {
        self.outcome = outcome.into();
        self
    }

    /// Set IP address
    pub fn with_ip(mut self, ip: impl Into<String>) -> Self {
        self.ip_address = Some(ip.into());
        self
    }

    /// Set user agent
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    /// Set session ID
    pub fn with_session(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    /// Set error
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.error = Some(error.into());
        self
    }

    /// Set severity
    pub fn with_severity(mut self, severity: EventSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Calculate hash of this event
    pub fn calculate_hash(&self) -> String {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();

        // Include all fields except the hash itself
        hasher.update(self.id.as_bytes());
        hasher.update(self.timestamp.to_rfc3339().as_bytes());
        hasher.update(serde_json::to_string(&self.event_type).unwrap_or_default().as_bytes());
        hasher.update(self.action.as_bytes());
        hasher.update(self.outcome.as_bytes());

        if let Some(user_id) = &self.user_id {
            hasher.update(user_id.as_bytes());
        }

        if let Some(prev_hash) = &self.previous_hash {
            hasher.update(prev_hash.as_bytes());
        }

        let result = hasher.finalize();
        hex::encode(result)
    }

    /// Verify the event hash
    pub fn verify_hash(&self) -> bool {
        if let Some(stored_hash) = &self.event_hash {
            let calculated_hash = self.calculate_hash();
            stored_hash == &calculated_hash
        } else {
            false
        }
    }
}

/// Predefined audit events
impl AuditEvent {
    /// Create a login success event
    pub fn login_success(user_id: impl Into<String>, ip: impl Into<String>) -> Self {
        Self::new(EventType::LoginSuccess, "user_login")
            .with_user(user_id)
            .with_ip(ip)
            .with_outcome("success")
    }

    /// Create a login failure event
    pub fn login_failure(username: impl Into<String>, ip: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::new(EventType::LoginFailure, "user_login")
            .with_metadata("username", serde_json::json!(username.into()))
            .with_ip(ip)
            .with_outcome("failure")
            .with_error(reason)
    }

    /// Create a permission denied event
    pub fn permission_denied(
        user_id: impl Into<String>,
        resource_type: impl Into<String>,
        resource_id: impl Into<String>,
        action: impl Into<String>,
    ) -> Self {
        Self::new(EventType::PermissionDenied, action)
            .with_user(user_id)
            .with_resource(resource_type, resource_id)
            .with_outcome("denied")
    }

    /// Create a data access event
    pub fn data_accessed(
        user_id: impl Into<String>,
        resource_type: impl Into<String>,
        resource_id: impl Into<String>,
    ) -> Self {
        Self::new(EventType::DataAccessed, "read")
            .with_user(user_id)
            .with_resource(resource_type, resource_id)
            .with_outcome("success")
    }

    /// Create a security alert event
    pub fn security_alert(message: impl Into<String>, severity: EventSeverity) -> Self {
        Self::new(EventType::SecurityAlert, "security_alert")
            .with_severity(severity)
            .with_metadata("message", serde_json::json!(message.into()))
            .with_outcome("alert")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let event = AuditEvent::new(EventType::LoginSuccess, "user_login")
            .with_user("user123")
            .with_ip("192.168.1.1");

        assert_eq!(event.event_type, EventType::LoginSuccess);
        assert_eq!(event.severity, EventSeverity::Info);
        assert_eq!(event.user_id, Some("user123".to_string()));
    }

    #[test]
    fn test_event_hash() {
        let mut event = AuditEvent::new(EventType::LoginSuccess, "user_login");
        let hash = event.calculate_hash();
        event.event_hash = Some(hash);

        assert!(event.verify_hash());

        // Tamper with the event
        event.action = "modified_action".to_string();
        assert!(!event.verify_hash());
    }

    #[test]
    fn test_event_severity() {
        assert_eq!(EventType::LoginSuccess.default_severity(), EventSeverity::Info);
        assert_eq!(EventType::LoginFailure.default_severity(), EventSeverity::High);
        assert_eq!(EventType::SecurityAlert.default_severity(), EventSeverity::Critical);
    }

    #[test]
    fn test_should_alert() {
        assert!(EventType::LoginFailure.should_alert());
        assert!(EventType::SecurityAlert.should_alert());
        assert!(!EventType::LoginSuccess.should_alert());
    }

    #[test]
    fn test_predefined_events() {
        let login = AuditEvent::login_success("user123", "192.168.1.1");
        assert_eq!(login.outcome, "success");
        assert!(login.user_id.is_some());

        let denied = AuditEvent::permission_denied("user123", "scene", "scene456", "delete");
        assert_eq!(denied.outcome, "denied");
        assert_eq!(denied.event_type, EventType::PermissionDenied);
    }
}
