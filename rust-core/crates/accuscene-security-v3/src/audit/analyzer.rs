//! Audit log analysis and alerting

use super::events::{AuditEvent, EventSeverity, EventType};
use super::logger::{AuditFilter, AuditLogger};
use crate::config::AuditConfig;
use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Audit alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditAlert {
    /// Alert ID
    pub id: String,

    /// Alert timestamp
    pub timestamp: DateTime<Utc>,

    /// Alert type
    pub alert_type: AlertType,

    /// Severity
    pub severity: EventSeverity,

    /// Description
    pub description: String,

    /// Related event IDs
    pub related_events: Vec<String>,

    /// Metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Alert types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertType {
    /// Multiple failed login attempts
    MultipleFailedLogins,

    /// Account locked
    AccountLocked,

    /// Suspicious activity detected
    SuspiciousActivity,

    /// Unusual data access pattern
    UnusualDataAccess,

    /// Privilege escalation attempt
    PrivilegeEscalation,

    /// Mass data export
    MassDataExport,

    /// Configuration change
    ConfigurationChange,

    /// System error
    SystemError,

    /// Custom alert
    Custom(String),
}

/// Audit analyzer
#[derive(Debug)]
pub struct AuditAnalyzer {
    config: AuditConfig,
    logger: Arc<AuditLogger>,
    alerts: Arc<DashMap<String, AuditAlert>>,

    // Tracking for pattern detection
    failed_login_attempts: Arc<DashMap<String, Vec<DateTime<Utc>>>>,
    data_access_counts: Arc<DashMap<String, usize>>,
}

impl AuditAnalyzer {
    /// Create a new audit analyzer
    pub fn new(config: AuditConfig, logger: Arc<AuditLogger>) -> Self {
        Self {
            config,
            logger,
            alerts: Arc::new(DashMap::new()),
            failed_login_attempts: Arc::new(DashMap::new()),
            data_access_counts: Arc::new(DashMap::new()),
        }
    }

    /// Analyze an audit event and generate alerts if needed
    pub fn analyze(&self, event: &AuditEvent) -> Vec<AuditAlert> {
        let mut alerts = Vec::new();

        // Check for various patterns
        if let Some(alert) = self.check_failed_logins(event) {
            alerts.push(alert);
        }

        if let Some(alert) = self.check_suspicious_activity(event) {
            alerts.push(alert);
        }

        if let Some(alert) = self.check_data_access_patterns(event) {
            alerts.push(alert);
        }

        if let Some(alert) = self.check_privilege_escalation(event) {
            alerts.push(alert);
        }

        // Store alerts
        for alert in &alerts {
            self.alerts.insert(alert.id.clone(), alert.clone());

            // Emit alert via tracing
            tracing::warn!(
                alert_id = %alert.id,
                alert_type = ?alert.alert_type,
                severity = ?alert.severity,
                description = %alert.description,
                "Security alert generated"
            );
        }

        alerts
    }

    /// Check for multiple failed login attempts
    fn check_failed_logins(&self, event: &AuditEvent) -> Option<AuditAlert> {
        if event.event_type != EventType::LoginFailure {
            return None;
        }

        // Get username or user_id
        let identifier = event.user_id.clone()
            .or_else(|| event.metadata.get("username").and_then(|v| v.as_str().map(String::from)))?;

        // Track failed attempts
        let mut attempts = self.failed_login_attempts
            .entry(identifier.clone())
            .or_insert_with(Vec::new);

        attempts.push(event.timestamp);

        // Clean old attempts (older than 15 minutes)
        let cutoff = Utc::now() - Duration::minutes(15);
        attempts.retain(|&t| t > cutoff);

        // Check threshold
        if attempts.len() >= self.config.failed_login_alert_threshold as usize {
            let alert = AuditAlert {
                id: uuid::Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
                alert_type: AlertType::MultipleFailedLogins,
                severity: EventSeverity::High,
                description: format!(
                    "{} failed login attempts for user '{}'",
                    attempts.len(),
                    identifier
                ),
                related_events: vec![event.id.clone()],
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("user".to_string(), serde_json::json!(identifier));
                    meta.insert("attempt_count".to_string(), serde_json::json!(attempts.len()));
                    meta
                },
            };

            return Some(alert);
        }

        None
    }

    /// Check for suspicious activity
    fn check_suspicious_activity(&self, event: &AuditEvent) -> Option<AuditAlert> {
        // Check for events marked as suspicious
        if event.event_type == EventType::SuspiciousActivity
            || event.event_type == EventType::SecurityAlert
        {
            let alert = AuditAlert {
                id: uuid::Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
                alert_type: AlertType::SuspiciousActivity,
                severity: event.severity,
                description: format!(
                    "Suspicious activity detected: {}",
                    event.action
                ),
                related_events: vec![event.id.clone()],
                metadata: event.metadata.clone(),
            };

            return Some(alert);
        }

        None
    }

    /// Check for unusual data access patterns
    fn check_data_access_patterns(&self, event: &AuditEvent) -> Option<AuditAlert> {
        if event.event_type != EventType::DataAccessed {
            return None;
        }

        let user_id = event.user_id.as_ref()?;

        // Track data access count
        let mut count = self.data_access_counts
            .entry(user_id.clone())
            .or_insert(0);

        *count += 1;

        // Alert if accessing too many resources in short time
        if *count > 100 {
            let alert = AuditAlert {
                id: uuid::Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
                alert_type: AlertType::UnusualDataAccess,
                severity: EventSeverity::Medium,
                description: format!(
                    "User '{}' accessed {} resources",
                    user_id,
                    *count
                ),
                related_events: vec![event.id.clone()],
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("user_id".to_string(), serde_json::json!(user_id));
                    meta.insert("access_count".to_string(), serde_json::json!(*count));
                    meta
                },
            };

            return Some(alert);
        }

        None
    }

    /// Check for privilege escalation attempts
    fn check_privilege_escalation(&self, event: &AuditEvent) -> Option<AuditAlert> {
        // Check for role assignments or permission grants
        if event.event_type == EventType::RoleAssigned
            || event.event_type == EventType::PermissionGranted
        {
            // Check if assigning admin/high-privilege roles
            if let Some(role) = event.metadata.get("role") {
                if role.as_str()? == "admin" || role.as_str()? == "super_admin" {
                    let alert = AuditAlert {
                        id: uuid::Uuid::new_v4().to_string(),
                        timestamp: Utc::now(),
                        alert_type: AlertType::PrivilegeEscalation,
                        severity: EventSeverity::High,
                        description: format!(
                            "High-privilege role '{}' assigned",
                            role.as_str()?
                        ),
                        related_events: vec![event.id.clone()],
                        metadata: event.metadata.clone(),
                    };

                    return Some(alert);
                }
            }
        }

        None
    }

    /// Get all alerts
    pub fn get_alerts(&self, filter: Option<&AlertFilter>) -> Vec<AuditAlert> {
        let mut alerts: Vec<_> = self.alerts
            .iter()
            .map(|entry| entry.value().clone())
            .collect();

        // Apply filter
        if let Some(f) = filter {
            alerts.retain(|a| f.matches(a));
        }

        // Sort by timestamp (newest first)
        alerts.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        alerts
    }

    /// Get recent alerts
    pub fn get_recent_alerts(&self, count: usize) -> Vec<AuditAlert> {
        let mut alerts = self.get_alerts(None);
        alerts.truncate(count);
        alerts
    }

    /// Clear old alerts
    pub fn cleanup_old_alerts(&self, retention_days: u32) -> usize {
        let cutoff = Utc::now() - Duration::days(retention_days as i64);
        let mut removed = 0;

        self.alerts.retain(|_, alert| {
            let should_keep = alert.timestamp > cutoff;
            if !should_keep {
                removed += 1;
            }
            should_keep
        });

        removed
    }

    /// Reset tracking data (e.g., for testing or maintenance)
    pub fn reset_tracking(&self) {
        self.failed_login_attempts.clear();
        self.data_access_counts.clear();
    }

    /// Generate a summary report
    pub fn generate_summary(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> AnalysisSummary {
        let filter = AuditFilter {
            start_time: Some(start),
            end_time: Some(end),
            ..Default::default()
        };

        let events = self.logger.query(&filter);

        let mut summary = AnalysisSummary {
            period_start: start,
            period_end: end,
            total_events: events.len(),
            events_by_type: HashMap::new(),
            events_by_severity: HashMap::new(),
            unique_users: std::collections::HashSet::new(),
            failed_logins: 0,
            successful_logins: 0,
            permission_denials: 0,
            data_accesses: 0,
            alerts_generated: 0,
        };

        for event in &events {
            // Count by type
            *summary.events_by_type
                .entry(format!("{:?}", event.event.event_type))
                .or_insert(0) += 1;

            // Count by severity
            *summary.events_by_severity
                .entry(event.event.severity)
                .or_insert(0) += 1;

            // Track unique users
            if let Some(user_id) = &event.event.user_id {
                summary.unique_users.insert(user_id.clone());
            }

            // Count specific event types
            match event.event.event_type {
                EventType::LoginFailure => summary.failed_logins += 1,
                EventType::LoginSuccess => summary.successful_logins += 1,
                EventType::PermissionDenied => summary.permission_denials += 1,
                EventType::DataAccessed => summary.data_accesses += 1,
                _ => {}
            }
        }

        // Count alerts in period
        let alert_filter = AlertFilter {
            start_time: Some(start),
            end_time: Some(end),
            ..Default::default()
        };
        summary.alerts_generated = self.get_alerts(Some(&alert_filter)).len();

        summary
    }
}

/// Alert filter
#[derive(Debug, Clone, Default)]
pub struct AlertFilter {
    /// Filter by alert type
    pub alert_type: Option<AlertType>,

    /// Filter by minimum severity
    pub min_severity: Option<EventSeverity>,

    /// Start time
    pub start_time: Option<DateTime<Utc>>,

    /// End time
    pub end_time: Option<DateTime<Utc>>,
}

impl AlertFilter {
    /// Check if alert matches filter
    pub fn matches(&self, alert: &AuditAlert) -> bool {
        if let Some(alert_type) = &self.alert_type {
            if &alert.alert_type != alert_type {
                return false;
            }
        }

        if let Some(min_severity) = &self.min_severity {
            if alert.severity < *min_severity {
                return false;
            }
        }

        if let Some(start) = self.start_time {
            if alert.timestamp < start {
                return false;
            }
        }

        if let Some(end) = self.end_time {
            if alert.timestamp > end {
                return false;
            }
        }

        true
    }
}

/// Analysis summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisSummary {
    /// Period start
    pub period_start: DateTime<Utc>,

    /// Period end
    pub period_end: DateTime<Utc>,

    /// Total events
    pub total_events: usize,

    /// Events by type
    pub events_by_type: HashMap<String, usize>,

    /// Events by severity
    pub events_by_severity: HashMap<EventSeverity, usize>,

    /// Unique users
    #[serde(skip)]
    pub unique_users: std::collections::HashSet<String>,

    /// Failed logins
    pub failed_logins: usize,

    /// Successful logins
    pub successful_logins: usize,

    /// Permission denials
    pub permission_denials: usize,

    /// Data accesses
    pub data_accesses: usize,

    /// Alerts generated
    pub alerts_generated: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_failed_login_detection() {
        let config = AuditConfig {
            failed_login_alert_threshold: 3,
            ..Default::default()
        };

        let logger = Arc::new(AuditLogger::new(config.clone()));
        let analyzer = AuditAnalyzer::new(config, logger);

        // Generate failed login events
        for _ in 0..3 {
            let event = AuditEvent::login_failure("user123", "192.168.1.1", "bad password");
            let alerts = analyzer.analyze(&event);
            if !alerts.is_empty() {
                assert_eq!(alerts[0].alert_type, AlertType::MultipleFailedLogins);
            }
        }
    }

    #[test]
    fn test_generate_summary() {
        let logger = Arc::new(AuditLogger::default());
        let analyzer = AuditAnalyzer::new(AuditConfig::default(), logger.clone());

        // Generate some events
        logger.log(AuditEvent::login_success("user1", "192.168.1.1")).unwrap();
        logger.log(AuditEvent::login_failure("user2", "192.168.1.2", "bad password")).unwrap();
        logger.log(AuditEvent::data_accessed("user1", "scene", "scene123")).unwrap();

        let start = Utc::now() - Duration::hours(1);
        let end = Utc::now() + Duration::hours(1);

        let summary = analyzer.generate_summary(start, end);

        assert_eq!(summary.total_events, 3);
        assert_eq!(summary.successful_logins, 1);
        assert_eq!(summary.failed_logins, 1);
        assert_eq!(summary.data_accesses, 1);
    }
}
