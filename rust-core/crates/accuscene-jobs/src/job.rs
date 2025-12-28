//! Core job trait and implementations for the AccuScene job system.

use crate::error::{JobError, Result};
use crate::result::JobResult;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::sync::Arc;
use uuid::Uuid;

/// Job trait that all jobs must implement
#[async_trait]
pub trait Job: Send + Sync + Debug {
    /// Execute the job
    async fn execute(&mut self, context: Arc<JobContext>) -> Result<JobResult>;

    /// Get job ID
    fn id(&self) -> &str;

    /// Get job name/type
    fn name(&self) -> &str;

    /// Get job priority (higher values = higher priority)
    fn priority(&self) -> i32 {
        0
    }

    /// Get maximum retry attempts
    fn max_retries(&self) -> u32 {
        3
    }

    /// Get job timeout in seconds (None = no timeout)
    fn timeout_secs(&self) -> Option<u64> {
        Some(300) // 5 minutes default
    }

    /// Check if job can be retried after this error
    fn can_retry(&self, error: &JobError, attempt: u32) -> bool {
        error.is_retryable() && attempt < self.max_retries()
    }

    /// Called before job execution
    async fn before_execute(&mut self, _context: Arc<JobContext>) -> Result<()> {
        Ok(())
    }

    /// Called after successful job execution
    async fn after_execute(&mut self, _context: Arc<JobContext>, _result: &JobResult) -> Result<()> {
        Ok(())
    }

    /// Called when job fails
    async fn on_failure(&mut self, _context: Arc<JobContext>, _error: &JobError) -> Result<()> {
        Ok(())
    }

    /// Serialize job to JSON
    fn serialize(&self) -> Result<String>;

    /// Deserialize job from JSON
    fn deserialize(data: &str) -> Result<Box<dyn Job>>
    where
        Self: Sized;
}

/// Job execution context
#[derive(Debug, Clone)]
pub struct JobContext {
    pub job_id: String,
    pub attempt: u32,
    pub started_at: DateTime<Utc>,
    pub worker_id: String,
    pub metadata: serde_json::Value,
}

impl JobContext {
    pub fn new(job_id: String, worker_id: String) -> Self {
        Self {
            job_id,
            attempt: 1,
            started_at: Utc::now(),
            worker_id,
            metadata: serde_json::Value::Null,
        }
    }

    pub fn with_attempt(mut self, attempt: u32) -> Self {
        self.attempt = attempt;
        self
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Job state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobState {
    Pending,
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
    Retrying,
}

impl JobState {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            JobState::Completed | JobState::Failed | JobState::Cancelled
        )
    }

    pub fn can_transition_to(&self, new_state: JobState) -> bool {
        match (self, new_state) {
            (JobState::Pending, JobState::Queued) => true,
            (JobState::Queued, JobState::Running) => true,
            (JobState::Running, JobState::Completed) => true,
            (JobState::Running, JobState::Failed) => true,
            (JobState::Running, JobState::Retrying) => true,
            (JobState::Retrying, JobState::Running) => true,
            (JobState::Retrying, JobState::Failed) => true,
            (_, JobState::Cancelled) => !self.is_terminal(),
            _ => false,
        }
    }
}

/// Job metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobMetadata {
    pub id: String,
    pub name: String,
    pub state: JobState,
    pub priority: i32,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub attempt: u32,
    pub max_retries: u32,
    pub timeout_secs: Option<u64>,
    pub worker_id: Option<String>,
    pub tags: Vec<String>,
}

impl JobMetadata {
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            state: JobState::Pending,
            priority: 0,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            attempt: 0,
            max_retries: 3,
            timeout_secs: Some(300),
            worker_id: None,
            tags: Vec::new(),
        }
    }

    pub fn transition_to(&mut self, new_state: JobState) -> Result<()> {
        if !self.state.can_transition_to(new_state) {
            return Err(JobError::InvalidStateTransition {
                from: format!("{:?}", self.state),
                to: format!("{:?}", new_state),
            });
        }
        self.state = new_state;
        Ok(())
    }
}

/// Physics simulation job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsSimulationJob {
    pub id: String,
    pub scenario_id: String,
    pub parameters: serde_json::Value,
    pub metadata: JobMetadata,
}

impl PhysicsSimulationJob {
    pub fn new(scenario_id: String, parameters: serde_json::Value) -> Self {
        let id = Uuid::new_v4().to_string();
        let metadata = JobMetadata::new(id.clone(), "physics_simulation".to_string());
        Self {
            id: id.clone(),
            scenario_id,
            parameters,
            metadata,
        }
    }
}

#[async_trait]
impl Job for PhysicsSimulationJob {
    async fn execute(&mut self, context: Arc<JobContext>) -> Result<JobResult> {
        tracing::info!(
            job_id = %self.id,
            scenario_id = %self.scenario_id,
            "Executing physics simulation job"
        );

        // Simulate physics calculation
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let result = serde_json::json!({
            "scenario_id": self.scenario_id,
            "status": "completed",
            "results": {
                "trajectories": [],
                "impact_points": [],
                "forces": []
            }
        });

        Ok(JobResult::success(self.id.clone(), result))
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        "physics_simulation"
    }

    fn priority(&self) -> i32 {
        self.metadata.priority
    }

    fn max_retries(&self) -> u32 {
        self.metadata.max_retries
    }

    fn timeout_secs(&self) -> Option<u64> {
        self.metadata.timeout_secs
    }

    fn serialize(&self) -> Result<String> {
        serde_json::to_string(self).map_err(Into::into)
    }

    fn deserialize(data: &str) -> Result<Box<dyn Job>> {
        let job: PhysicsSimulationJob = serde_json::from_str(data)?;
        Ok(Box::new(job))
    }
}

/// Report generation job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportGenerationJob {
    pub id: String,
    pub report_type: String,
    pub data_source: String,
    pub parameters: serde_json::Value,
    pub metadata: JobMetadata,
}

impl ReportGenerationJob {
    pub fn new(report_type: String, data_source: String, parameters: serde_json::Value) -> Self {
        let id = Uuid::new_v4().to_string();
        let metadata = JobMetadata::new(id.clone(), "report_generation".to_string());
        Self {
            id: id.clone(),
            report_type,
            data_source,
            parameters,
            metadata,
        }
    }
}

#[async_trait]
impl Job for ReportGenerationJob {
    async fn execute(&mut self, context: Arc<JobContext>) -> Result<JobResult> {
        tracing::info!(
            job_id = %self.id,
            report_type = %self.report_type,
            "Executing report generation job"
        );

        // Simulate report generation
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        let result = serde_json::json!({
            "report_id": Uuid::new_v4().to_string(),
            "type": self.report_type,
            "status": "generated",
            "url": format!("/reports/{}.pdf", self.id)
        });

        Ok(JobResult::success(self.id.clone(), result))
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        "report_generation"
    }

    fn serialize(&self) -> Result<String> {
        serde_json::to_string(self).map_err(Into::into)
    }

    fn deserialize(data: &str) -> Result<Box<dyn Job>> {
        let job: ReportGenerationJob = serde_json::from_str(data)?;
        Ok(Box::new(job))
    }
}

/// Data export job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataExportJob {
    pub id: String,
    pub export_format: String,
    pub query: String,
    pub destination: String,
    pub metadata: JobMetadata,
}

impl DataExportJob {
    pub fn new(export_format: String, query: String, destination: String) -> Self {
        let id = Uuid::new_v4().to_string();
        let metadata = JobMetadata::new(id.clone(), "data_export".to_string());
        Self {
            id: id.clone(),
            export_format,
            query,
            destination,
            metadata,
        }
    }
}

#[async_trait]
impl Job for DataExportJob {
    async fn execute(&mut self, context: Arc<JobContext>) -> Result<JobResult> {
        tracing::info!(
            job_id = %self.id,
            format = %self.export_format,
            "Executing data export job"
        );

        // Simulate data export
        tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;

        let result = serde_json::json!({
            "export_id": self.id,
            "format": self.export_format,
            "destination": self.destination,
            "records_exported": 1000,
            "status": "completed"
        });

        Ok(JobResult::success(self.id.clone(), result))
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        "data_export"
    }

    fn serialize(&self) -> Result<String> {
        serde_json::to_string(self).map_err(Into::into)
    }

    fn deserialize(data: &str) -> Result<Box<dyn Job>> {
        let job: DataExportJob = serde_json::from_str(data)?;
        Ok(Box::new(job))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_state_transitions() {
        assert!(JobState::Pending.can_transition_to(JobState::Queued));
        assert!(JobState::Queued.can_transition_to(JobState::Running));
        assert!(JobState::Running.can_transition_to(JobState::Completed));
        assert!(!JobState::Completed.can_transition_to(JobState::Running));
    }

    #[tokio::test]
    async fn test_physics_simulation_job() {
        let mut job = PhysicsSimulationJob::new(
            "test-scenario".to_string(),
            serde_json::json!({"test": true}),
        );
        let context = Arc::new(JobContext::new(job.id.to_string(), "worker-1".to_string()));
        let result = job.execute(context).await;
        assert!(result.is_ok());
    }
}
