//! Job metrics and statistics tracking.

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// Job metrics collector
#[derive(Clone)]
pub struct JobMetrics {
    stats: Arc<RwLock<JobStatistics>>,
    job_metrics: Arc<RwLock<HashMap<String, JobMetric>>>,
}

impl JobMetrics {
    /// Create a new job metrics collector
    pub fn new() -> Self {
        Self {
            stats: Arc::new(RwLock::new(JobStatistics::default())),
            job_metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record job started
    pub fn record_started(&self, job_id: String, job_name: String) {
        let mut stats = self.stats.write();
        stats.total_jobs += 1;
        stats.running_jobs += 1;

        let metric = JobMetric {
            job_id: job_id.clone(),
            job_name,
            started_at: Utc::now(),
            completed_at: None,
            duration_ms: 0,
            status: JobMetricStatus::Running,
            retry_count: 0,
        };

        self.job_metrics.write().insert(job_id, metric);
    }

    /// Record job completed successfully
    pub fn record_completed(&self, job_id: &str, duration_ms: u64) {
        let mut stats = self.stats.write();
        stats.running_jobs = stats.running_jobs.saturating_sub(1);
        stats.completed_jobs += 1;
        stats.total_duration_ms += duration_ms;

        if let Some(metric) = self.job_metrics.write().get_mut(job_id) {
            metric.completed_at = Some(Utc::now());
            metric.duration_ms = duration_ms;
            metric.status = JobMetricStatus::Completed;
        }

        // Update average duration
        if stats.completed_jobs > 0 {
            stats.average_duration_ms = stats.total_duration_ms / stats.completed_jobs;
        }
    }

    /// Record job failed
    pub fn record_failed(&self, job_id: &str, duration_ms: u64) {
        let mut stats = self.stats.write();
        stats.running_jobs = stats.running_jobs.saturating_sub(1);
        stats.failed_jobs += 1;
        stats.total_duration_ms += duration_ms;

        if let Some(metric) = self.job_metrics.write().get_mut(job_id) {
            metric.completed_at = Some(Utc::now());
            metric.duration_ms = duration_ms;
            metric.status = JobMetricStatus::Failed;
        }
    }

    /// Record job retry
    pub fn record_retry(&self, job_id: &str) {
        let mut stats = self.stats.write();
        stats.total_retries += 1;

        if let Some(metric) = self.job_metrics.write().get_mut(job_id) {
            metric.retry_count += 1;
        }
    }

    /// Record job cancelled
    pub fn record_cancelled(&self, job_id: &str) {
        let mut stats = self.stats.write();
        stats.running_jobs = stats.running_jobs.saturating_sub(1);
        stats.cancelled_jobs += 1;

        if let Some(metric) = self.job_metrics.write().get_mut(job_id) {
            metric.completed_at = Some(Utc::now());
            metric.status = JobMetricStatus::Cancelled;
        }
    }

    /// Get overall statistics
    pub fn get_statistics(&self) -> JobStatistics {
        self.stats.read().clone()
    }

    /// Get metric for a specific job
    pub fn get_job_metric(&self, job_id: &str) -> Option<JobMetric> {
        self.job_metrics.read().get(job_id).cloned()
    }

    /// Get all job metrics
    pub fn get_all_metrics(&self) -> Vec<JobMetric> {
        self.job_metrics.read().values().cloned().collect()
    }

    /// Get metrics for running jobs
    pub fn get_running_jobs(&self) -> Vec<JobMetric> {
        self.job_metrics
            .read()
            .values()
            .filter(|m| matches!(m.status, JobMetricStatus::Running))
            .cloned()
            .collect()
    }

    /// Calculate success rate
    pub fn success_rate(&self) -> f64 {
        let stats = self.stats.read();
        let total_finished = stats.completed_jobs + stats.failed_jobs;
        if total_finished == 0 {
            0.0
        } else {
            stats.completed_jobs as f64 / total_finished as f64
        }
    }

    /// Clear old metrics
    pub fn clear_old_metrics(&self, older_than: Duration) {
        let cutoff = Utc::now() - chrono::Duration::from_std(older_than).unwrap();
        self.job_metrics.write().retain(|_, metric| {
            if let Some(completed_at) = metric.completed_at {
                completed_at > cutoff
            } else {
                true // Keep running jobs
            }
        });
    }

    /// Reset all metrics
    pub fn reset(&self) {
        *self.stats.write() = JobStatistics::default();
        self.job_metrics.write().clear();
    }
}

impl Default for JobMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Overall job statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStatistics {
    pub total_jobs: u64,
    pub running_jobs: u64,
    pub completed_jobs: u64,
    pub failed_jobs: u64,
    pub cancelled_jobs: u64,
    pub total_retries: u64,
    pub total_duration_ms: u64,
    pub average_duration_ms: u64,
}

impl Default for JobStatistics {
    fn default() -> Self {
        Self {
            total_jobs: 0,
            running_jobs: 0,
            completed_jobs: 0,
            failed_jobs: 0,
            cancelled_jobs: 0,
            total_retries: 0,
            total_duration_ms: 0,
            average_duration_ms: 0,
        }
    }
}

/// Individual job metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobMetric {
    pub job_id: String,
    pub job_name: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_ms: u64,
    pub status: JobMetricStatus,
    pub retry_count: u32,
}

/// Job metric status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobMetricStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Metrics aggregator for grouping metrics by job type
pub struct MetricsAggregator {
    metrics: Arc<RwLock<HashMap<String, JobTypeMetrics>>>,
}

impl MetricsAggregator {
    /// Create a new metrics aggregator
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record job execution
    pub fn record(&self, job_name: &str, duration_ms: u64, success: bool) {
        let mut metrics = self.metrics.write();
        let job_metrics = metrics
            .entry(job_name.to_string())
            .or_insert_with(|| JobTypeMetrics::new(job_name.to_string()));

        job_metrics.total_executions += 1;
        job_metrics.total_duration_ms += duration_ms;

        if success {
            job_metrics.successful_executions += 1;
        } else {
            job_metrics.failed_executions += 1;
        }

        // Update min/max
        if duration_ms < job_metrics.min_duration_ms {
            job_metrics.min_duration_ms = duration_ms;
        }
        if duration_ms > job_metrics.max_duration_ms {
            job_metrics.max_duration_ms = duration_ms;
        }

        // Update average
        job_metrics.average_duration_ms =
            job_metrics.total_duration_ms / job_metrics.total_executions;
    }

    /// Get metrics for a job type
    pub fn get_metrics(&self, job_name: &str) -> Option<JobTypeMetrics> {
        self.metrics.read().get(job_name).cloned()
    }

    /// Get all metrics
    pub fn get_all_metrics(&self) -> HashMap<String, JobTypeMetrics> {
        self.metrics.read().clone()
    }
}

impl Default for MetricsAggregator {
    fn default() -> Self {
        Self::new()
    }
}

/// Metrics for a specific job type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobTypeMetrics {
    pub job_name: String,
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub total_duration_ms: u64,
    pub average_duration_ms: u64,
    pub min_duration_ms: u64,
    pub max_duration_ms: u64,
}

impl JobTypeMetrics {
    fn new(job_name: String) -> Self {
        Self {
            job_name,
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            total_duration_ms: 0,
            average_duration_ms: 0,
            min_duration_ms: u64::MAX,
            max_duration_ms: 0,
        }
    }

    /// Calculate success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_executions == 0 {
            0.0
        } else {
            self.successful_executions as f64 / self.total_executions as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_metrics_basic() {
        let metrics = JobMetrics::new();

        metrics.record_started("job-1".to_string(), "test_job".to_string());
        assert_eq!(metrics.get_statistics().total_jobs, 1);
        assert_eq!(metrics.get_statistics().running_jobs, 1);

        metrics.record_completed("job-1", 1000);
        assert_eq!(metrics.get_statistics().completed_jobs, 1);
        assert_eq!(metrics.get_statistics().running_jobs, 0);
    }

    #[test]
    fn test_job_metrics_success_rate() {
        let metrics = JobMetrics::new();

        metrics.record_started("job-1".to_string(), "test".to_string());
        metrics.record_completed("job-1", 100);

        metrics.record_started("job-2".to_string(), "test".to_string());
        metrics.record_failed("job-2", 100);

        assert_eq!(metrics.success_rate(), 0.5);
    }

    #[test]
    fn test_metrics_aggregator() {
        let aggregator = MetricsAggregator::new();

        aggregator.record("physics_simulation", 1000, true);
        aggregator.record("physics_simulation", 1500, true);
        aggregator.record("physics_simulation", 800, false);

        let metrics = aggregator.get_metrics("physics_simulation").unwrap();
        assert_eq!(metrics.total_executions, 3);
        assert_eq!(metrics.successful_executions, 2);
        assert_eq!(metrics.failed_executions, 1);
        assert_eq!(metrics.min_duration_ms, 800);
        assert_eq!(metrics.max_duration_ms, 1500);
    }
}
