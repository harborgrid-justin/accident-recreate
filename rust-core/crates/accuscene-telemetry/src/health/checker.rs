//! Health check runner

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Health check trait
#[async_trait]
pub trait HealthCheck: Send + Sync {
    /// Get the name of the health check
    fn name(&self) -> &str;

    /// Perform the health check
    async fn check(&self) -> HealthCheckResult;

    /// Get the timeout for this check in seconds
    fn timeout_secs(&self) -> u64 {
        5
    }
}

/// Result of a health check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub healthy: bool,
    pub message: Option<String>,
    pub details: HashMap<String, String>,
}

impl HealthCheckResult {
    /// Create a healthy result
    pub fn healthy() -> Self {
        Self {
            healthy: true,
            message: None,
            details: HashMap::new(),
        }
    }

    /// Create an unhealthy result
    pub fn unhealthy(message: impl Into<String>) -> Self {
        Self {
            healthy: false,
            message: Some(message.into()),
            details: HashMap::new(),
        }
    }

    /// Add a detail to the result
    pub fn with_detail(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.details.insert(key.into(), value.into());
        self
    }
}

/// Health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub healthy: bool,
    pub timestamp: DateTime<Utc>,
    pub checks: HashMap<String, HealthCheckResult>,
}

impl HealthStatus {
    /// Create a new health status
    pub fn new() -> Self {
        Self {
            healthy: true,
            timestamp: Utc::now(),
            checks: HashMap::new(),
        }
    }

    /// Check if all checks are healthy
    pub fn is_healthy(&self) -> bool {
        self.healthy && self.checks.values().all(|c| c.healthy)
    }

    /// Add a check result
    pub fn add_check(&mut self, name: String, result: HealthCheckResult) {
        if !result.healthy {
            self.healthy = false;
        }
        self.checks.insert(name, result);
    }

    /// Get a summary message
    pub fn summary(&self) -> String {
        if self.is_healthy() {
            "All health checks passed".to_string()
        } else {
            let failed: Vec<&String> = self
                .checks
                .iter()
                .filter(|(_, r)| !r.healthy)
                .map(|(name, _)| name)
                .collect();

            format!("Health checks failed: {}", failed.join(", "))
        }
    }
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self::new()
    }
}

/// Health checker that runs multiple health checks
pub struct HealthChecker {
    checks: Vec<Box<dyn HealthCheck>>,
}

impl HealthChecker {
    /// Create a new health checker
    pub fn new() -> Self {
        Self {
            checks: Vec::new(),
        }
    }

    /// Register a health check
    pub fn register(&mut self, check: Box<dyn HealthCheck>) {
        self.checks.push(check);
    }

    /// Run all health checks
    pub async fn check_all(&self) -> HealthStatus {
        let mut status = HealthStatus::new();

        for check in &self.checks {
            let name = check.name().to_string();
            let timeout = tokio::time::Duration::from_secs(check.timeout_secs());

            let result = match tokio::time::timeout(timeout, check.check()).await {
                Ok(result) => result,
                Err(_) => HealthCheckResult::unhealthy("Health check timed out"),
            };

            status.add_check(name, result);
        }

        status
    }

    /// Run a specific health check by name
    pub async fn check_by_name(&self, name: &str) -> Option<HealthCheckResult> {
        for check in &self.checks {
            if check.name() == name {
                return Some(check.check().await);
            }
        }
        None
    }

    /// Get the number of registered checks
    pub fn count(&self) -> usize {
        self.checks.len()
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Database health check
pub struct DatabaseHealthCheck {
    name: String,
}

impl DatabaseHealthCheck {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

#[async_trait]
impl HealthCheck for DatabaseHealthCheck {
    fn name(&self) -> &str {
        &self.name
    }

    async fn check(&self) -> HealthCheckResult {
        // Simulate database check
        // In real implementation, this would ping the database
        HealthCheckResult::healthy()
            .with_detail("type", "database")
            .with_detail("status", "connected")
    }
}

/// Cache health check
pub struct CacheHealthCheck {
    name: String,
}

impl CacheHealthCheck {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

#[async_trait]
impl HealthCheck for CacheHealthCheck {
    fn name(&self) -> &str {
        &self.name
    }

    async fn check(&self) -> HealthCheckResult {
        // Simulate cache check
        HealthCheckResult::healthy()
            .with_detail("type", "cache")
            .with_detail("status", "available")
    }
}

/// Memory health check
pub struct MemoryHealthCheck {
    threshold_mb: u64,
}

impl MemoryHealthCheck {
    pub fn new(threshold_mb: u64) -> Self {
        Self { threshold_mb }
    }
}

#[async_trait]
impl HealthCheck for MemoryHealthCheck {
    fn name(&self) -> &str {
        "memory"
    }

    async fn check(&self) -> HealthCheckResult {
        // Simulate memory check
        // In real implementation, this would check actual memory usage
        HealthCheckResult::healthy()
            .with_detail("threshold_mb", self.threshold_mb.to_string())
            .with_detail("status", "ok")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_checker() {
        let mut checker = HealthChecker::new();
        checker.register(Box::new(DatabaseHealthCheck::new("db")));
        checker.register(Box::new(CacheHealthCheck::new("cache")));

        let status = checker.check_all().await;
        assert!(status.is_healthy());
        assert_eq!(status.checks.len(), 2);
    }

    #[test]
    fn test_health_check_result() {
        let result = HealthCheckResult::healthy()
            .with_detail("key", "value");

        assert!(result.healthy);
        assert_eq!(result.details.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_health_status() {
        let mut status = HealthStatus::new();
        status.add_check("check1".to_string(), HealthCheckResult::healthy());
        status.add_check("check2".to_string(), HealthCheckResult::unhealthy("Failed"));

        assert!(!status.is_healthy());
    }
}
