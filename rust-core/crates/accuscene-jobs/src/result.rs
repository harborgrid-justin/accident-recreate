//! Job result handling for the AccuScene job system.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Job execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResult {
    pub job_id: String,
    pub status: JobResultStatus,
    pub output: serde_json::Value,
    pub error: Option<String>,
    pub completed_at: DateTime<Utc>,
    pub duration_ms: u64,
    pub metadata: serde_json::Value,
}

impl JobResult {
    /// Create a successful job result
    pub fn success(job_id: String, output: serde_json::Value) -> Self {
        Self {
            job_id,
            status: JobResultStatus::Success,
            output,
            error: None,
            completed_at: Utc::now(),
            duration_ms: 0,
            metadata: serde_json::Value::Null,
        }
    }

    /// Create a failed job result
    pub fn failure(job_id: String, error: String) -> Self {
        Self {
            job_id,
            status: JobResultStatus::Failure,
            output: serde_json::Value::Null,
            error: Some(error),
            completed_at: Utc::now(),
            duration_ms: 0,
            metadata: serde_json::Value::Null,
        }
    }

    /// Create a partial success result
    pub fn partial(job_id: String, output: serde_json::Value, error: String) -> Self {
        Self {
            job_id,
            status: JobResultStatus::Partial,
            output,
            error: Some(error),
            completed_at: Utc::now(),
            duration_ms: 0,
            metadata: serde_json::Value::Null,
        }
    }

    /// Set the duration
    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = duration_ms;
        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// Check if the result is successful
    pub fn is_success(&self) -> bool {
        matches!(self.status, JobResultStatus::Success)
    }

    /// Check if the result is a failure
    pub fn is_failure(&self) -> bool {
        matches!(self.status, JobResultStatus::Failure)
    }

    /// Check if the result is partial
    pub fn is_partial(&self) -> bool {
        matches!(self.status, JobResultStatus::Partial)
    }
}

/// Job result status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobResultStatus {
    Success,
    Failure,
    Partial,
}

/// Aggregated results for batch jobs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchJobResult {
    pub batch_id: String,
    pub total_jobs: usize,
    pub successful: usize,
    pub failed: usize,
    pub partial: usize,
    pub results: Vec<JobResult>,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
}

impl BatchJobResult {
    pub fn new(batch_id: String) -> Self {
        let now = Utc::now();
        Self {
            batch_id,
            total_jobs: 0,
            successful: 0,
            failed: 0,
            partial: 0,
            results: Vec::new(),
            started_at: now,
            completed_at: now,
        }
    }

    pub fn add_result(&mut self, result: JobResult) {
        match result.status {
            JobResultStatus::Success => self.successful += 1,
            JobResultStatus::Failure => self.failed += 1,
            JobResultStatus::Partial => self.partial += 1,
        }
        self.results.push(result);
        self.total_jobs = self.results.len();
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_jobs == 0 {
            0.0
        } else {
            self.successful as f64 / self.total_jobs as f64
        }
    }

    pub fn is_complete(&self) -> bool {
        self.successful + self.failed + self.partial == self.total_jobs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_result_creation() {
        let success = JobResult::success("job-1".to_string(), serde_json::json!({"data": 123}));
        assert!(success.is_success());
        assert!(!success.is_failure());

        let failure = JobResult::failure("job-2".to_string(), "error occurred".to_string());
        assert!(failure.is_failure());
        assert!(!failure.is_success());
    }

    #[test]
    fn test_batch_result() {
        let mut batch = BatchJobResult::new("batch-1".to_string());

        batch.add_result(JobResult::success("job-1".to_string(), serde_json::json!({})));
        batch.add_result(JobResult::failure("job-2".to_string(), "error".to_string()));

        assert_eq!(batch.total_jobs, 2);
        assert_eq!(batch.successful, 1);
        assert_eq!(batch.failed, 1);
        assert_eq!(batch.success_rate(), 0.5);
    }
}
