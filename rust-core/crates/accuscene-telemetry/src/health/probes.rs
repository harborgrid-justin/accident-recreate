//! Liveness and readiness probes

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Probe status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProbeStatus {
    /// Probe is passing
    Healthy,
    /// Probe is failing
    Unhealthy,
    /// Probe status is unknown
    Unknown,
}

impl ProbeStatus {
    /// Check if the probe is healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self, ProbeStatus::Healthy)
    }
}

impl Default for ProbeStatus {
    fn default() -> Self {
        ProbeStatus::Unknown
    }
}

/// Liveness probe - indicates if the application is running
#[derive(Debug, Clone)]
pub struct LivenessProbe {
    status: ProbeStatus,
    last_check: Option<DateTime<Utc>>,
    failure_count: u32,
    max_failures: u32,
}

impl LivenessProbe {
    /// Create a new liveness probe
    pub fn new() -> Self {
        Self {
            status: ProbeStatus::Healthy,
            last_check: None,
            failure_count: 0,
            max_failures: 3,
        }
    }

    /// Create with custom max failures
    pub fn with_max_failures(max_failures: u32) -> Self {
        Self {
            status: ProbeStatus::Healthy,
            last_check: None,
            failure_count: 0,
            max_failures,
        }
    }

    /// Update the probe status
    pub fn update(&mut self, healthy: bool) {
        self.last_check = Some(Utc::now());

        if healthy {
            self.status = ProbeStatus::Healthy;
            self.failure_count = 0;
        } else {
            self.failure_count += 1;
            if self.failure_count >= self.max_failures {
                self.status = ProbeStatus::Unhealthy;
            }
        }
    }

    /// Get the current status
    pub fn status(&self) -> ProbeStatus {
        self.status
    }

    /// Check if the probe is healthy
    pub fn is_healthy(&self) -> bool {
        self.status.is_healthy()
    }

    /// Get the last check time
    pub fn last_check(&self) -> Option<DateTime<Utc>> {
        self.last_check
    }

    /// Get the failure count
    pub fn failure_count(&self) -> u32 {
        self.failure_count
    }

    /// Reset the probe
    pub fn reset(&mut self) {
        self.status = ProbeStatus::Healthy;
        self.failure_count = 0;
        self.last_check = None;
    }

    /// Get probe state as JSON-serializable struct
    pub fn state(&self) -> ProbeState {
        ProbeState {
            status: self.status,
            last_check: self.last_check,
            failure_count: self.failure_count,
            probe_type: "liveness".to_string(),
        }
    }
}

impl Default for LivenessProbe {
    fn default() -> Self {
        Self::new()
    }
}

/// Readiness probe - indicates if the application is ready to accept traffic
#[derive(Debug, Clone)]
pub struct ReadinessProbe {
    status: ProbeStatus,
    last_check: Option<DateTime<Utc>>,
    ready: bool,
    dependencies: Vec<String>,
    ready_dependencies: Vec<String>,
}

impl ReadinessProbe {
    /// Create a new readiness probe
    pub fn new() -> Self {
        Self {
            status: ProbeStatus::Unknown,
            last_check: None,
            ready: false,
            dependencies: Vec::new(),
            ready_dependencies: Vec::new(),
        }
    }

    /// Add a dependency
    pub fn add_dependency(&mut self, name: impl Into<String>) {
        self.dependencies.push(name.into());
    }

    /// Mark a dependency as ready
    pub fn mark_dependency_ready(&mut self, name: &str) {
        if !self.ready_dependencies.contains(&name.to_string()) {
            self.ready_dependencies.push(name.to_string());
        }
        self.update_status();
    }

    /// Mark a dependency as not ready
    pub fn mark_dependency_not_ready(&mut self, name: &str) {
        self.ready_dependencies.retain(|d| d != name);
        self.update_status();
    }

    /// Update the probe status
    pub fn update(&mut self, ready: bool) {
        self.last_check = Some(Utc::now());
        self.ready = ready;
        self.update_status();
    }

    /// Update the status based on dependencies
    fn update_status(&mut self) {
        if self.dependencies.is_empty() {
            self.status = if self.ready {
                ProbeStatus::Healthy
            } else {
                ProbeStatus::Unhealthy
            };
        } else {
            let all_ready = self
                .dependencies
                .iter()
                .all(|d| self.ready_dependencies.contains(d));

            self.status = if all_ready && self.ready {
                ProbeStatus::Healthy
            } else {
                ProbeStatus::Unhealthy
            };
        }
    }

    /// Get the current status
    pub fn status(&self) -> ProbeStatus {
        self.status
    }

    /// Check if the probe is ready
    pub fn is_ready(&self) -> bool {
        self.status.is_healthy()
    }

    /// Get the last check time
    pub fn last_check(&self) -> Option<DateTime<Utc>> {
        self.last_check
    }

    /// Get all dependencies
    pub fn dependencies(&self) -> &[String] {
        &self.dependencies
    }

    /// Get ready dependencies
    pub fn ready_dependencies(&self) -> &[String] {
        &self.ready_dependencies
    }

    /// Get not ready dependencies
    pub fn not_ready_dependencies(&self) -> Vec<&String> {
        self.dependencies
            .iter()
            .filter(|d| !self.ready_dependencies.contains(d))
            .collect()
    }

    /// Reset the probe
    pub fn reset(&mut self) {
        self.status = ProbeStatus::Unknown;
        self.ready = false;
        self.last_check = None;
        self.ready_dependencies.clear();
    }

    /// Get probe state as JSON-serializable struct
    pub fn state(&self) -> ReadinessProbeState {
        ReadinessProbeState {
            status: self.status,
            last_check: self.last_check,
            ready: self.ready,
            dependencies: self.dependencies.clone(),
            ready_dependencies: self.ready_dependencies.clone(),
            not_ready_dependencies: self.not_ready_dependencies()
                .into_iter()
                .cloned()
                .collect(),
        }
    }
}

impl Default for ReadinessProbe {
    fn default() -> Self {
        Self::new()
    }
}

/// Serializable probe state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeState {
    pub status: ProbeStatus,
    pub last_check: Option<DateTime<Utc>>,
    pub failure_count: u32,
    pub probe_type: String,
}

/// Serializable readiness probe state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadinessProbeState {
    pub status: ProbeStatus,
    pub last_check: Option<DateTime<Utc>>,
    pub ready: bool,
    pub dependencies: Vec<String>,
    pub ready_dependencies: Vec<String>,
    pub not_ready_dependencies: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_liveness_probe() {
        let mut probe = LivenessProbe::new();
        assert!(probe.is_healthy());

        probe.update(false);
        assert!(probe.is_healthy()); // Still healthy due to max_failures

        probe.update(false);
        probe.update(false);
        assert!(!probe.is_healthy()); // Now unhealthy

        probe.update(true);
        assert!(probe.is_healthy()); // Recovered
    }

    #[test]
    fn test_readiness_probe() {
        let mut probe = ReadinessProbe::new();
        probe.add_dependency("database");
        probe.add_dependency("cache");

        assert!(!probe.is_ready());

        probe.mark_dependency_ready("database");
        assert!(!probe.is_ready()); // Still missing cache

        probe.mark_dependency_ready("cache");
        probe.update(true);
        assert!(probe.is_ready()); // All dependencies ready

        probe.mark_dependency_not_ready("database");
        assert!(!probe.is_ready()); // Database not ready
    }

    #[test]
    fn test_probe_status() {
        assert!(ProbeStatus::Healthy.is_healthy());
        assert!(!ProbeStatus::Unhealthy.is_healthy());
        assert!(!ProbeStatus::Unknown.is_healthy());
    }
}
