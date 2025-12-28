//! Performance profiling

use crate::{PerformanceConfig, Result, TelemetryError};
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

/// Performance profiler
pub struct PerformanceProfiler {
    config: PerformanceConfig,
    sessions: Arc<RwLock<Vec<ProfileSession>>>,
    active_session: Arc<RwLock<Option<ProfileSession>>>,
}

impl PerformanceProfiler {
    /// Create a new performance profiler
    pub fn new(config: &PerformanceConfig) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
            sessions: Arc::new(RwLock::new(Vec::new())),
            active_session: Arc::new(RwLock::new(None)),
        })
    }

    /// Start profiling
    pub fn start(&mut self) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let session = ProfileSession::new("default");
        *self.active_session.write() = Some(session);

        tracing::info!("Performance profiling started");
        Ok(())
    }

    /// Stop profiling
    pub fn stop(&mut self) -> Result<()> {
        if let Some(session) = self.active_session.write().take() {
            self.sessions.write().push(session);
            tracing::info!("Performance profiling stopped");
        }
        Ok(())
    }

    /// Record a measurement
    pub fn record(&self, name: impl Into<String>, duration_ms: f64) {
        if let Some(session) = &mut *self.active_session.write() {
            session.record(name, duration_ms);
        }
    }

    /// Record memory usage
    pub fn record_memory(&self, bytes: u64) {
        if let Some(session) = &mut *self.active_session.write() {
            session.record_memory(bytes);
        }
    }

    /// Get the active session
    pub fn active_session(&self) -> Option<ProfileSession> {
        self.active_session.read().clone()
    }

    /// Get all sessions
    pub fn sessions(&self) -> Vec<ProfileSession> {
        self.sessions.read().clone()
    }

    /// Clear all sessions
    pub fn clear(&self) {
        self.sessions.write().clear();
    }

    /// Get a summary of all sessions
    pub fn summary(&self) -> ProfileSummary {
        let sessions = self.sessions.read();
        ProfileSummary::from_sessions(&sessions)
    }
}

/// Profile session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSession {
    name: String,
    start_time: DateTime<Utc>,
    end_time: Option<DateTime<Utc>>,
    measurements: HashMap<String, Vec<f64>>,
    memory_samples: Vec<u64>,
}

impl ProfileSession {
    /// Create a new profile session
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            start_time: Utc::now(),
            end_time: None,
            measurements: HashMap::new(),
            memory_samples: Vec::new(),
        }
    }

    /// Record a measurement
    pub fn record(&mut self, name: impl Into<String>, duration_ms: f64) {
        self.measurements
            .entry(name.into())
            .or_insert_with(Vec::new)
            .push(duration_ms);
    }

    /// Record memory usage
    pub fn record_memory(&mut self, bytes: u64) {
        self.memory_samples.push(bytes);
    }

    /// End the session
    pub fn end(&mut self) {
        self.end_time = Some(Utc::now());
    }

    /// Get the session name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get all measurements for a metric
    pub fn get_measurements(&self, name: &str) -> Option<&Vec<f64>> {
        self.measurements.get(name)
    }

    /// Get statistics for a metric
    pub fn statistics(&self, name: &str) -> Option<Statistics> {
        self.get_measurements(name)
            .map(|measurements| Statistics::from_samples(measurements))
    }

    /// Get memory statistics
    pub fn memory_statistics(&self) -> Option<Statistics> {
        if self.memory_samples.is_empty() {
            None
        } else {
            let samples: Vec<f64> = self.memory_samples.iter().map(|&x| x as f64).collect();
            Some(Statistics::from_samples(&samples))
        }
    }
}

/// Statistics for performance measurements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Statistics {
    pub count: usize,
    pub sum: f64,
    pub mean: f64,
    pub min: f64,
    pub max: f64,
    pub median: f64,
    pub p95: f64,
    pub p99: f64,
}

impl Statistics {
    /// Calculate statistics from samples
    pub fn from_samples(samples: &[f64]) -> Self {
        if samples.is_empty() {
            return Self::default();
        }

        let mut sorted = samples.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let count = samples.len();
        let sum: f64 = samples.iter().sum();
        let mean = sum / count as f64;
        let min = sorted[0];
        let max = sorted[count - 1];
        let median = percentile(&sorted, 50.0);
        let p95 = percentile(&sorted, 95.0);
        let p99 = percentile(&sorted, 99.0);

        Self {
            count,
            sum,
            mean,
            min,
            max,
            median,
            p95,
            p99,
        }
    }
}

impl Default for Statistics {
    fn default() -> Self {
        Self {
            count: 0,
            sum: 0.0,
            mean: 0.0,
            min: 0.0,
            max: 0.0,
            median: 0.0,
            p95: 0.0,
            p99: 0.0,
        }
    }
}

/// Profile summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSummary {
    pub total_sessions: usize,
    pub metrics: HashMap<String, Statistics>,
}

impl ProfileSummary {
    /// Create summary from sessions
    pub fn from_sessions(sessions: &[ProfileSession]) -> Self {
        let mut all_metrics: HashMap<String, Vec<f64>> = HashMap::new();

        for session in sessions {
            for (name, measurements) in &session.measurements {
                all_metrics
                    .entry(name.clone())
                    .or_insert_with(Vec::new)
                    .extend(measurements);
            }
        }

        let metrics = all_metrics
            .into_iter()
            .map(|(name, samples)| (name, Statistics::from_samples(&samples)))
            .collect();

        Self {
            total_sessions: sessions.len(),
            metrics,
        }
    }
}

/// Calculate percentile
fn percentile(sorted_samples: &[f64], p: f64) -> f64 {
    if sorted_samples.is_empty() {
        return 0.0;
    }

    let index = (p / 100.0 * (sorted_samples.len() - 1) as f64).round() as usize;
    sorted_samples[index.min(sorted_samples.len() - 1)]
}

/// Profile scope for automatic timing
pub struct ProfileScope {
    name: String,
    start: Instant,
    profiler: Option<Arc<RwLock<PerformanceProfiler>>>,
}

impl ProfileScope {
    /// Create a new profile scope
    pub fn new(name: impl Into<String>, profiler: Arc<RwLock<PerformanceProfiler>>) -> Self {
        Self {
            name: name.into(),
            start: Instant::now(),
            profiler: Some(profiler),
        }
    }

    /// Create a profile scope without a profiler (for testing)
    pub fn dummy(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            start: Instant::now(),
            profiler: None,
        }
    }
}

impl Drop for ProfileScope {
    fn drop(&mut self) {
        let duration_ms = self.start.elapsed().as_secs_f64() * 1000.0;

        if let Some(profiler) = &self.profiler {
            profiler.read().record(&self.name, duration_ms);
        }

        tracing::trace!("{} took {:.2}ms", self.name, duration_ms);
    }
}

/// Macro to create a profile scope
#[macro_export]
macro_rules! profile_scope {
    ($profiler:expr, $name:expr) => {
        let _profile_scope = $crate::performance::ProfileScope::new($name, $profiler);
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_statistics() {
        let samples = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = Statistics::from_samples(&samples);

        assert_eq!(stats.count, 5);
        assert_eq!(stats.mean, 3.0);
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 5.0);
    }

    #[test]
    fn test_profile_session() {
        let mut session = ProfileSession::new("test");
        session.record("operation1", 10.0);
        session.record("operation1", 20.0);
        session.record("operation2", 30.0);

        let stats = session.statistics("operation1").unwrap();
        assert_eq!(stats.count, 2);
        assert_eq!(stats.mean, 15.0);
    }

    #[test]
    fn test_percentile() {
        let samples = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(percentile(&samples, 50.0), 3.0);
        assert_eq!(percentile(&samples, 100.0), 5.0);
    }
}
