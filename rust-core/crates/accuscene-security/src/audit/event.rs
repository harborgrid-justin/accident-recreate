//! Audit event types
//!
//! Defines structured audit events for all security-relevant activities.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Event ID (unique)
    pub id: String,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Event type
    pub event_type: EventType,
    /// Severity level
    pub severity: EventSeverity,
    /// User who initiated the action
    pub user_id: Option<String>,
    /// Session ID
    pub session_id: Option<String>,
    /// IP address
    pub ip_address: Option<String>,
    /// Resource affected
    pub resource: Option<ResourceInfo>,
    /// Action performed
    pub action: String,
    /// Result of the action
    pub result: EventResult,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
    /// Request ID for correlation
    pub request_id: Option<String>,
}

impl AuditEvent {
    /// Create a new audit event
    pub fn new(event_type: EventType, action: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            event_type,
            severity: EventSeverity::Info,
            user_id: None,
            session_id: None,
            ip_address: None,
            resource: None,
            action,
            result: EventResult::Success,
            error: None,
            metadata: HashMap::new(),
            request_id: None,
        }
    }

    /// Set user ID
    pub fn with_user(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    /// Set session ID
    pub fn with_session(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }

    /// Set IP address
    pub fn with_ip(mut self, ip_address: String) -> Self {
        self.ip_address = Some(ip_address);
        self
    }

    /// Set resource
    pub fn with_resource(mut self, resource: ResourceInfo) -> Self {
        self.resource = Some(resource);
        self
    }

    /// Set result
    pub fn with_result(mut self, result: EventResult) -> Self {
        self.result = result;
        self
    }

    /// Set error
    pub fn with_error(mut self, error: String) -> Self {
        self.error = Some(error);
        self.result = EventResult::Failure;
        self
    }

    /// Set severity
    pub fn with_severity(mut self, severity: EventSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Add metadata
    pub fn add_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Set request ID
    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }
}

/// Event type categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventType {
    // Authentication events
    AuthLogin,
    AuthLogout,
    AuthMfa,
    AuthPasswordChange,
    AuthPasswordReset,
    AuthSso,
    AuthTokenIssued,
    AuthTokenRefreshed,
    AuthTokenRevoked,
    AuthSessionCreated,
    AuthSessionExpired,
    AuthSessionInvalidated,

    // Authorization events
    AuthzPermissionCheck,
    AuthzAccessGranted,
    AuthzAccessDenied,
    AuthzRoleAssigned,
    AuthzRoleRevoked,
    AuthzPolicyEvaluated,

    // Data access events
    DataRead,
    DataCreated,
    DataModified,
    DataDeleted,
    DataExported,

    // Case events
    CaseCreated,
    CaseModified,
    CaseDeleted,
    CaseAssigned,
    CaseClosed,
    CaseReopened,

    // Evidence events
    EvidenceAdded,
    EvidenceModified,
    EvidenceDeleted,
    EvidenceAccessed,
    EvidenceChainUpdated,

    // Report events
    ReportCreated,
    ReportModified,
    ReportDeleted,
    ReportPublished,
    ReportShared,
    ReportExported,

    // Configuration events
    ConfigModified,
    SettingsChanged,

    // Security events
    SecurityThreatDetected,
    SecurityRateLimitExceeded,
    SecurityBruteForceDetected,
    SecurityAnomalyDetected,
    SecurityPolicyViolation,

    // Audit events
    AuditLogAccessed,
    AuditLogExported,

    // System events
    SystemStartup,
    SystemShutdown,
    SystemError,
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Event severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EventSeverity {
    Debug = 0,
    Info = 1,
    Warning = 2,
    Error = 3,
    Critical = 4,
}

impl std::fmt::Display for EventSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventSeverity::Debug => write!(f, "DEBUG"),
            EventSeverity::Info => write!(f, "INFO"),
            EventSeverity::Warning => write!(f, "WARNING"),
            EventSeverity::Error => write!(f, "ERROR"),
            EventSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Event result
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventResult {
    Success,
    Failure,
    Partial,
}

impl std::fmt::Display for EventResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventResult::Success => write!(f, "SUCCESS"),
            EventResult::Failure => write!(f, "FAILURE"),
            EventResult::Partial => write!(f, "PARTIAL"),
        }
    }
}

/// Resource information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceInfo {
    /// Resource type (e.g., "case", "evidence", "report")
    pub resource_type: String,
    /// Resource ID
    pub resource_id: String,
    /// Resource name/description
    pub name: Option<String>,
}

impl ResourceInfo {
    /// Create new resource info
    pub fn new(resource_type: impl Into<String>, resource_id: impl Into<String>) -> Self {
        Self {
            resource_type: resource_type.into(),
            resource_id: resource_id.into(),
            name: None,
        }
    }

    /// Set resource name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_event_creation() {
        let event = AuditEvent::new(EventType::AuthLogin, "user.login".to_string())
            .with_user("user123".to_string())
            .with_ip("192.168.1.1".to_string())
            .with_result(EventResult::Success);

        assert_eq!(event.event_type, EventType::AuthLogin);
        assert_eq!(event.user_id, Some("user123".to_string()));
        assert_eq!(event.result, EventResult::Success);
    }

    #[test]
    fn test_resource_info() {
        let resource = ResourceInfo::new("case", "case-123")
            .with_name("Traffic Accident on I-95");

        assert_eq!(resource.resource_type, "case");
        assert_eq!(resource.resource_id, "case-123");
        assert_eq!(resource.name, Some("Traffic Accident on I-95".to_string()));
    }

    #[test]
    fn test_event_severity_ordering() {
        assert!(EventSeverity::Critical > EventSeverity::Error);
        assert!(EventSeverity::Error > EventSeverity::Warning);
        assert!(EventSeverity::Warning > EventSeverity::Info);
        assert!(EventSeverity::Info > EventSeverity::Debug);
    }

    #[test]
    fn test_event_with_metadata() {
        let event = AuditEvent::new(EventType::CaseCreated, "case.create".to_string())
            .add_metadata("case_type".to_string(), "traffic".to_string())
            .add_metadata("priority".to_string(), "high".to_string());

        assert_eq!(event.metadata.len(), 2);
        assert_eq!(event.metadata.get("case_type"), Some(&"traffic".to_string()));
    }
}
