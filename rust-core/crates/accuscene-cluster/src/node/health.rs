//! Node health status and monitoring.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Overall health status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Node is healthy
    Healthy,
    /// Node is degraded but operational
    Degraded,
    /// Node is unhealthy
    Unhealthy,
    /// Health status unknown
    Unknown,
}

/// Health check result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    /// Check timestamp
    pub timestamp: DateTime<Utc>,

    /// Overall status
    pub status: HealthStatus,

    /// Individual component checks
    pub checks: Vec<ComponentHealth>,

    /// Response time
    pub response_time: Duration,
}

impl HealthCheck {
    /// Create a new health check.
    pub fn new(status: HealthStatus) -> Self {
        Self {
            timestamp: Utc::now(),
            status,
            checks: Vec::new(),
            response_time: Duration::from_millis(0),
        }
    }

    /// Add a component check.
    pub fn add_check(&mut self, check: ComponentHealth) {
        self.checks.push(check);
    }

    /// Set response time.
    pub fn with_response_time(mut self, duration: Duration) -> Self {
        self.response_time = duration;
        self
    }

    /// Calculate overall status from component checks.
    pub fn calculate_status(&mut self) {
        if self.checks.is_empty() {
            self.status = HealthStatus::Unknown;
            return;
        }

        let has_unhealthy = self.checks.iter().any(|c| c.status == HealthStatus::Unhealthy);
        let has_degraded = self.checks.iter().any(|c| c.status == HealthStatus::Degraded);

        self.status = if has_unhealthy {
            HealthStatus::Unhealthy
        } else if has_degraded {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };
    }
}

/// Individual component health.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Component name
    pub name: String,

    /// Component status
    pub status: HealthStatus,

    /// Status message
    pub message: Option<String>,

    /// Component metrics
    pub metrics: Vec<(String, f64)>,
}

impl ComponentHealth {
    /// Create a healthy component.
    pub fn healthy(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Healthy,
            message: None,
            metrics: Vec::new(),
        }
    }

    /// Create a degraded component.
    pub fn degraded(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Degraded,
            message: Some(message.into()),
            metrics: Vec::new(),
        }
    }

    /// Create an unhealthy component.
    pub fn unhealthy(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Unhealthy,
            message: Some(message.into()),
            metrics: Vec::new(),
        }
    }

    /// Add a metric.
    pub fn with_metric(mut self, key: impl Into<String>, value: f64) -> Self {
        self.metrics.push((key.into(), value));
        self
    }
}

/// Health monitor with failure tracking.
#[derive(Debug)]
pub struct HealthMonitor {
    /// Last successful check
    last_success: Option<DateTime<Utc>>,

    /// Last failed check
    last_failure: Option<DateTime<Utc>>,

    /// Consecutive failures
    consecutive_failures: u32,

    /// Consecutive successes
    consecutive_successes: u32,

    /// Failure threshold
    failure_threshold: u32,

    /// Success threshold
    success_threshold: u32,

    /// Current status
    current_status: HealthStatus,
}

impl HealthMonitor {
    /// Create a new health monitor.
    pub fn new(failure_threshold: u32, success_threshold: u32) -> Self {
        Self {
            last_success: None,
            last_failure: None,
            consecutive_failures: 0,
            consecutive_successes: 0,
            failure_threshold,
            success_threshold,
            current_status: HealthStatus::Unknown,
        }
    }

    /// Record a successful health check.
    pub fn record_success(&mut self) {
        self.last_success = Some(Utc::now());
        self.consecutive_successes += 1;
        self.consecutive_failures = 0;

        if self.consecutive_successes >= self.success_threshold {
            self.current_status = HealthStatus::Healthy;
        }
    }

    /// Record a failed health check.
    pub fn record_failure(&mut self) {
        self.last_failure = Some(Utc::now());
        self.consecutive_failures += 1;
        self.consecutive_successes = 0;

        if self.consecutive_failures >= self.failure_threshold {
            self.current_status = HealthStatus::Unhealthy;
        } else {
            self.current_status = HealthStatus::Degraded;
        }
    }

    /// Get current health status.
    pub fn status(&self) -> HealthStatus {
        self.current_status
    }

    /// Check if node is healthy.
    pub fn is_healthy(&self) -> bool {
        self.current_status == HealthStatus::Healthy
    }

    /// Get consecutive failures.
    pub fn consecutive_failures(&self) -> u32 {
        self.consecutive_failures
    }

    /// Get time since last success.
    pub fn time_since_last_success(&self) -> Option<Duration> {
        self.last_success.map(|t| {
            let now = Utc::now();
            (now - t).to_std().unwrap_or(Duration::from_secs(0))
        })
    }
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new(3, 2)
    }
}
