//! Alert thresholds and triggers

use chrono::{DateTime, Utc};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use uuid::Uuid;

/// Alert severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertSeverity {
    /// Low severity
    Low,
    /// Medium severity
    Medium,
    /// High severity
    High,
    /// Critical severity
    Critical,
}

impl AlertSeverity {
    /// Convert to string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }
}

impl std::fmt::Display for AlertSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Alert status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertStatus {
    /// Alert is active
    Active,
    /// Alert has been acknowledged
    Acknowledged,
    /// Alert has been resolved
    Resolved,
}

/// Alert threshold type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AlertThreshold {
    /// Value exceeds threshold
    GreaterThan {
        value: f64,
    },
    /// Value is below threshold
    LessThan {
        value: f64,
    },
    /// Value is between range
    Between {
        min: f64,
        max: f64,
    },
    /// Value is outside range
    Outside {
        min: f64,
        max: f64,
    },
    /// Percentage change exceeds threshold
    PercentageChange {
        percentage: f64,
    },
    /// Rate exceeds threshold (value per second)
    Rate {
        value: f64,
    },
}

impl AlertThreshold {
    /// Check if a value triggers the threshold
    pub fn check(&self, current: f64, previous: Option<f64>) -> bool {
        match self {
            Self::GreaterThan { value } => current > *value,
            Self::LessThan { value } => current < *value,
            Self::Between { min, max } => current >= *min && current <= *max,
            Self::Outside { min, max } => current < *min || current > *max,
            Self::PercentageChange { percentage } => {
                if let Some(prev) = previous {
                    if prev == 0.0 {
                        return false;
                    }
                    let change = ((current - prev) / prev).abs() * 100.0;
                    change > *percentage
                } else {
                    false
                }
            }
            Self::Rate { value } => current > *value,
        }
    }

    /// Get a description of the threshold
    pub fn description(&self) -> String {
        match self {
            Self::GreaterThan { value } => format!("greater than {}", value),
            Self::LessThan { value } => format!("less than {}", value),
            Self::Between { min, max } => format!("between {} and {}", min, max),
            Self::Outside { min, max } => format!("outside {} to {}", min, max),
            Self::PercentageChange { percentage } => format!("percentage change > {}%", percentage),
            Self::Rate { value } => format!("rate > {}/s", value),
        }
    }
}

/// Alert rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    /// Rule ID
    pub id: String,

    /// Rule name
    pub name: String,

    /// Metric name to monitor
    pub metric: String,

    /// Alert threshold
    pub threshold: AlertThreshold,

    /// Alert severity
    pub severity: AlertSeverity,

    /// Description
    pub description: String,

    /// Enabled flag
    pub enabled: bool,

    /// Cooldown period in seconds
    pub cooldown_secs: u64,
}

impl AlertRule {
    /// Create a new alert rule
    pub fn new(
        name: impl Into<String>,
        metric: impl Into<String>,
        threshold: AlertThreshold,
        severity: AlertSeverity,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            metric: metric.into(),
            threshold,
            severity,
            description: String::new(),
            enabled: true,
            cooldown_secs: 300, // 5 minutes default
        }
    }

    /// Set the description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Set the cooldown period
    pub fn with_cooldown(mut self, secs: u64) -> Self {
        self.cooldown_secs = secs;
        self
    }

    /// Enable or disable the rule
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if the rule is triggered
    pub fn check(&self, current: f64, previous: Option<f64>) -> bool {
        self.enabled && self.threshold.check(current, previous)
    }
}

/// Active alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// Alert ID
    pub id: String,

    /// Rule ID that triggered this alert
    pub rule_id: String,

    /// Alert severity
    pub severity: AlertSeverity,

    /// Alert status
    pub status: AlertStatus,

    /// Alert message
    pub message: String,

    /// Current value
    pub current_value: f64,

    /// Previous value
    pub previous_value: Option<f64>,

    /// Timestamp when alert was triggered
    pub triggered_at: DateTime<Utc>,

    /// Timestamp when alert was acknowledged
    pub acknowledged_at: Option<DateTime<Utc>>,

    /// Timestamp when alert was resolved
    pub resolved_at: Option<DateTime<Utc>>,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl Alert {
    /// Create a new alert
    pub fn new(
        rule: &AlertRule,
        current_value: f64,
        previous_value: Option<f64>,
    ) -> Self {
        let message = format!(
            "{}: {} is {} (current: {}, {})",
            rule.name,
            rule.metric,
            rule.threshold.description(),
            current_value,
            previous_value
                .map(|v| format!("previous: {}", v))
                .unwrap_or_else(|| "no previous value".to_string())
        );

        Self {
            id: Uuid::new_v4().to_string(),
            rule_id: rule.id.clone(),
            severity: rule.severity,
            status: AlertStatus::Active,
            message,
            current_value,
            previous_value,
            triggered_at: Utc::now(),
            acknowledged_at: None,
            resolved_at: None,
            metadata: HashMap::new(),
        }
    }

    /// Acknowledge the alert
    pub fn acknowledge(&mut self) {
        if self.status == AlertStatus::Active {
            self.status = AlertStatus::Acknowledged;
            self.acknowledged_at = Some(Utc::now());
        }
    }

    /// Resolve the alert
    pub fn resolve(&mut self) {
        self.status = AlertStatus::Resolved;
        self.resolved_at = Some(Utc::now());
    }

    /// Check if the alert is active
    pub fn is_active(&self) -> bool {
        self.status == AlertStatus::Active
    }

    /// Get the alert age in seconds
    pub fn age_secs(&self) -> i64 {
        (Utc::now() - self.triggered_at).num_seconds()
    }
}

/// Alert manager
pub struct AlertManager {
    rules: HashMap<String, AlertRule>,
    alerts: Vec<Alert>,
    last_values: HashMap<String, f64>,
    last_trigger_times: HashMap<String, DateTime<Utc>>,
}

impl AlertManager {
    /// Create a new alert manager
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
            alerts: Vec::new(),
            last_values: HashMap::new(),
            last_trigger_times: HashMap::new(),
        }
    }

    /// Register an alert rule
    pub fn register_rule(&mut self, rule: AlertRule) {
        self.rules.insert(rule.id.clone(), rule);
    }

    /// Remove an alert rule
    pub fn remove_rule(&mut self, rule_id: &str) {
        self.rules.remove(rule_id);
    }

    /// Get all rules
    pub fn rules(&self) -> Vec<&AlertRule> {
        self.rules.values().collect()
    }

    /// Get a rule by ID
    pub fn get_rule(&self, rule_id: &str) -> Option<&AlertRule> {
        self.rules.get(rule_id)
    }

    /// Check a metric value against all rules
    pub fn check_metric(&mut self, metric: &str, value: f64) {
        let previous = self.last_values.get(metric).copied();

        for rule in self.rules.values() {
            if rule.metric == metric && rule.check(value, previous) {
                // Check cooldown
                if let Some(last_trigger) = self.last_trigger_times.get(&rule.id) {
                    let elapsed = (Utc::now() - *last_trigger).num_seconds();
                    if elapsed < rule.cooldown_secs as i64 {
                        continue; // Still in cooldown
                    }
                }

                // Trigger alert
                let alert = Alert::new(rule, value, previous);
                self.trigger_alert(alert);
                self.last_trigger_times.insert(rule.id.clone(), Utc::now());
            }
        }

        self.last_values.insert(metric.to_string(), value);
    }

    /// Trigger an alert
    fn trigger_alert(&mut self, alert: Alert) {
        tracing::warn!(
            alert_id = %alert.id,
            severity = %alert.severity,
            "Alert triggered: {}",
            alert.message
        );
        self.alerts.push(alert);
    }

    /// Get all active alerts
    pub fn active_alerts(&self) -> Vec<&Alert> {
        self.alerts.iter().filter(|a| a.is_active()).collect()
    }

    /// Get all alerts
    pub fn alerts(&self) -> &[Alert] {
        &self.alerts
    }

    /// Acknowledge an alert
    pub fn acknowledge_alert(&mut self, alert_id: &str) {
        if let Some(alert) = self.alerts.iter_mut().find(|a| a.id == alert_id) {
            alert.acknowledge();
            tracing::info!(alert_id = %alert_id, "Alert acknowledged");
        }
    }

    /// Resolve an alert
    pub fn resolve_alert(&mut self, alert_id: &str) {
        if let Some(alert) = self.alerts.iter_mut().find(|a| a.id == alert_id) {
            alert.resolve();
            tracing::info!(alert_id = %alert_id, "Alert resolved");
        }
    }

    /// Clear resolved alerts older than specified seconds
    pub fn clear_old_resolved(&mut self, age_secs: i64) {
        self.alerts.retain(|alert| {
            if alert.status == AlertStatus::Resolved {
                alert.age_secs() < age_secs
            } else {
                true
            }
        });
    }

    /// Get alert statistics
    pub fn statistics(&self) -> AlertStatistics {
        let total = self.alerts.len();
        let active = self.alerts.iter().filter(|a| a.is_active()).count();
        let acknowledged = self
            .alerts
            .iter()
            .filter(|a| a.status == AlertStatus::Acknowledged)
            .count();
        let resolved = self
            .alerts
            .iter()
            .filter(|a| a.status == AlertStatus::Resolved)
            .count();

        let by_severity = self
            .alerts
            .iter()
            .filter(|a| a.is_active())
            .fold(HashMap::new(), |mut acc, alert| {
                *acc.entry(alert.severity).or_insert(0) += 1;
                acc
            });

        AlertStatistics {
            total,
            active,
            acknowledged,
            resolved,
            by_severity,
        }
    }
}

impl Default for AlertManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Alert statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertStatistics {
    pub total: usize,
    pub active: usize,
    pub acknowledged: usize,
    pub resolved: usize,
    pub by_severity: HashMap<AlertSeverity, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_threshold() {
        let threshold = AlertThreshold::GreaterThan { value: 100.0 };
        assert!(threshold.check(150.0, None));
        assert!(!threshold.check(50.0, None));

        let threshold = AlertThreshold::Between {
            min: 10.0,
            max: 20.0,
        };
        assert!(threshold.check(15.0, None));
        assert!(!threshold.check(25.0, None));
    }

    #[test]
    fn test_alert_manager() {
        let mut manager = AlertManager::new();

        let rule = AlertRule::new(
            "high_cpu",
            "cpu_usage",
            AlertThreshold::GreaterThan { value: 80.0 },
            AlertSeverity::High,
        );

        manager.register_rule(rule);

        // Should not trigger
        manager.check_metric("cpu_usage", 50.0);
        assert_eq!(manager.active_alerts().len(), 0);

        // Should trigger
        manager.check_metric("cpu_usage", 90.0);
        assert_eq!(manager.active_alerts().len(), 1);

        // Acknowledge alert
        let alert_id = manager.alerts()[0].id.clone();
        manager.acknowledge_alert(&alert_id);
        assert_eq!(manager.active_alerts().len(), 0);
    }

    #[test]
    fn test_alert_cooldown() {
        let mut manager = AlertManager::new();

        let rule = AlertRule::new(
            "test",
            "metric",
            AlertThreshold::GreaterThan { value: 10.0 },
            AlertSeverity::Low,
        )
        .with_cooldown(60);

        manager.register_rule(rule);

        // First trigger
        manager.check_metric("metric", 20.0);
        assert_eq!(manager.alerts().len(), 1);

        // Second trigger (should be blocked by cooldown)
        manager.check_metric("metric", 30.0);
        assert_eq!(manager.alerts().len(), 1);
    }
}
