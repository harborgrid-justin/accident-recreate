//! Health check aggregation
//!
//! This module provides health check functionality for all services,
//! allowing monitoring and alerting on service health.

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::Instant;
use tracing::{debug, warn};

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// Service is healthy
    Healthy,
    /// Service is degraded but operational
    Degraded,
    /// Service is unhealthy
    Unhealthy,
    /// Health status is unknown
    Unknown,
}

/// Health check information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    /// Service name
    pub service: String,

    /// Health status
    pub status: HealthStatus,

    /// Status message
    pub message: Option<String>,

    /// Last check timestamp
    pub checked_at: chrono::DateTime<chrono::Utc>,

    /// Response time in milliseconds
    pub response_time_ms: u64,

    /// Additional details
    pub details: std::collections::HashMap<String, serde_json::Value>,
}

impl HealthCheck {
    /// Create a new health check
    pub fn new(service: String, status: HealthStatus) -> Self {
        Self {
            service,
            status,
            message: None,
            checked_at: chrono::Utc::now(),
            response_time_ms: 0,
            details: std::collections::HashMap::new(),
        }
    }

    /// Set message
    pub fn with_message(mut self, message: String) -> Self {
        self.message = Some(message);
        self
    }

    /// Set response time
    pub fn with_response_time(mut self, response_time_ms: u64) -> Self {
        self.response_time_ms = response_time_ms;
        self
    }

    /// Add detail
    pub fn with_detail(mut self, key: String, value: serde_json::Value) -> Self {
        self.details.insert(key, value);
        self
    }

    /// Check if healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self.status, HealthStatus::Healthy | HealthStatus::Degraded)
    }
}

/// Health checker for aggregating health checks
pub struct HealthChecker {
    checks: Arc<DashMap<String, HealthCheck>>,
    check_interval: Duration,
}

impl HealthChecker {
    /// Create a new health checker
    pub fn new() -> Self {
        Self {
            checks: Arc::new(DashMap::new()),
            check_interval: Duration::from_secs(30),
        }
    }

    /// Create a new health checker with custom interval
    pub fn with_interval(check_interval: Duration) -> Self {
        Self {
            checks: Arc::new(DashMap::new()),
            check_interval,
        }
    }

    /// Register a health check for a service
    pub async fn register_check(&self, service: String) {
        let check = HealthCheck::new(service.clone(), HealthStatus::Unknown);
        self.checks.insert(service.clone(), check);
        debug!("Health check registered for service: {}", service);
    }

    /// Update health check
    pub async fn update_check(&self, check: HealthCheck) {
        let service = check.service.clone();
        let status = check.status.clone();

        self.checks.insert(service.clone(), check);

        match status {
            HealthStatus::Healthy => {
                debug!("Service {} is healthy", service);
            }
            HealthStatus::Degraded => {
                warn!("Service {} is degraded", service);
            }
            HealthStatus::Unhealthy => {
                warn!("Service {} is unhealthy", service);
            }
            HealthStatus::Unknown => {
                debug!("Service {} health is unknown", service);
            }
        }
    }

    /// Perform a health check for a service
    pub async fn check_service(&self, service: &str) -> Option<HealthCheck> {
        let start = Instant::now();

        // Simulate health check (in production, this would actually check the service)
        let status = HealthStatus::Healthy;
        let elapsed = start.elapsed();

        let check = HealthCheck::new(service.to_string(), status)
            .with_message("Service is operational".to_string())
            .with_response_time(elapsed.as_millis() as u64);

        self.update_check(check.clone()).await;

        Some(check)
    }

    /// Get health check for a service
    pub fn get_check(&self, service: &str) -> Option<HealthCheck> {
        self.checks.get(service).map(|entry| entry.clone())
    }

    /// Get all health checks
    pub fn get_all_checks(&self) -> Vec<HealthCheck> {
        self.checks
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get overall health status
    pub fn overall_health(&self) -> OverallHealth {
        let all_checks = self.get_all_checks();

        if all_checks.is_empty() {
            return OverallHealth {
                status: HealthStatus::Unknown,
                total_services: 0,
                healthy_services: 0,
                degraded_services: 0,
                unhealthy_services: 0,
                unknown_services: 0,
                checks: vec![],
            };
        }

        let total = all_checks.len();
        let healthy = all_checks
            .iter()
            .filter(|c| c.status == HealthStatus::Healthy)
            .count();
        let degraded = all_checks
            .iter()
            .filter(|c| c.status == HealthStatus::Degraded)
            .count();
        let unhealthy = all_checks
            .iter()
            .filter(|c| c.status == HealthStatus::Unhealthy)
            .count();
        let unknown = all_checks
            .iter()
            .filter(|c| c.status == HealthStatus::Unknown)
            .count();

        // Determine overall status
        let status = if unhealthy > 0 {
            HealthStatus::Unhealthy
        } else if degraded > 0 {
            HealthStatus::Degraded
        } else if unknown > 0 {
            HealthStatus::Unknown
        } else {
            HealthStatus::Healthy
        };

        OverallHealth {
            status,
            total_services: total,
            healthy_services: healthy,
            degraded_services: degraded,
            unhealthy_services: unhealthy,
            unknown_services: unknown,
            checks: all_checks,
        }
    }

    /// Check if all services are healthy
    pub fn all_healthy(&self) -> bool {
        let overall = self.overall_health();
        overall.status == HealthStatus::Healthy
    }

    /// Get unhealthy services
    pub fn unhealthy_services(&self) -> Vec<String> {
        self.checks
            .iter()
            .filter(|entry| {
                matches!(
                    entry.value().status,
                    HealthStatus::Unhealthy | HealthStatus::Degraded
                )
            })
            .map(|entry| entry.key().clone())
            .collect()
    }

    /// Remove a health check
    pub fn remove_check(&self, service: &str) {
        self.checks.remove(service);
        debug!("Health check removed for service: {}", service);
    }

    /// Clear all health checks
    pub fn clear_checks(&self) {
        self.checks.clear();
        debug!("All health checks cleared");
    }

    /// Get check interval
    pub fn check_interval(&self) -> Duration {
        self.check_interval
    }

    /// Start periodic health checks
    pub async fn start_periodic_checks(&self) {
        let checks = Arc::clone(&self.checks);
        let interval = self.check_interval;

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);

            loop {
                interval_timer.tick().await;

                // Check all registered services
                for entry in checks.iter() {
                    let service = entry.key().clone();
                    let start = Instant::now();

                    // Simulate health check
                    let status = HealthStatus::Healthy;
                    let elapsed = start.elapsed();

                    let check = HealthCheck::new(service.clone(), status)
                        .with_message("Periodic check passed".to_string())
                        .with_response_time(elapsed.as_millis() as u64);

                    checks.insert(service, check);
                }
            }
        });
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Overall health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverallHealth {
    /// Overall health status
    pub status: HealthStatus,

    /// Total number of services
    pub total_services: usize,

    /// Number of healthy services
    pub healthy_services: usize,

    /// Number of degraded services
    pub degraded_services: usize,

    /// Number of unhealthy services
    pub unhealthy_services: usize,

    /// Number of services with unknown health
    pub unknown_services: usize,

    /// All health checks
    pub checks: Vec<HealthCheck>,
}

impl OverallHealth {
    /// Check if the system is healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self.status, HealthStatus::Healthy)
    }

    /// Check if the system is operational
    pub fn is_operational(&self) -> bool {
        matches!(
            self.status,
            HealthStatus::Healthy | HealthStatus::Degraded
        )
    }

    /// Get health percentage
    pub fn health_percentage(&self) -> f64 {
        if self.total_services == 0 {
            return 100.0;
        }
        (self.healthy_services as f64 / self.total_services as f64) * 100.0
    }
}
