//! Job pipelines and workflows for chaining jobs together.

use crate::error::{JobError, Result};
use crate::executor::JobExecutor;
use crate::job::{Job, JobContext};
use crate::result::JobResult;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Job pipeline for executing jobs in sequence
pub struct JobPipeline {
    id: String,
    name: String,
    stages: Vec<PipelineStage>,
    executor: Arc<JobExecutor>,
    results: Arc<RwLock<Vec<JobResult>>>,
}

impl JobPipeline {
    /// Create a new job pipeline
    pub fn new(name: String, executor: Arc<JobExecutor>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            stages: Vec::new(),
            executor,
            results: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Add a stage to the pipeline
    pub fn add_stage(mut self, stage: PipelineStage) -> Self {
        self.stages.push(stage);
        self
    }

    /// Execute the pipeline
    pub async fn execute(&self) -> Result<PipelineResult> {
        let start_time = std::time::Instant::now();
        let mut stage_results = Vec::new();
        let mut pipeline_data = serde_json::Value::Null;

        tracing::info!(
            pipeline_id = %self.id,
            pipeline_name = %self.name,
            stages = self.stages.len(),
            "Starting pipeline execution"
        );

        for (index, stage) in self.stages.iter().enumerate() {
            tracing::info!(
                pipeline_id = %self.id,
                stage_index = index,
                stage_name = %stage.name,
                "Executing pipeline stage"
            );

            let stage_result = self.execute_stage(stage, &pipeline_data).await?;

            if !stage_result.is_success() {
                if stage.fail_fast {
                    tracing::error!(
                        pipeline_id = %self.id,
                        stage_index = index,
                        stage_name = %stage.name,
                        "Stage failed, stopping pipeline (fail_fast=true)"
                    );

                    return Ok(PipelineResult {
                        pipeline_id: self.id.clone(),
                        pipeline_name: self.name.clone(),
                        status: PipelineStatus::Failed,
                        stage_results,
                        duration_ms: start_time.elapsed().as_millis() as u64,
                        error: stage_result.error.clone(),
                    });
                }
            }

            // Pass output to next stage
            if stage.pass_output {
                pipeline_data = stage_result.output.clone();
            }

            stage_results.push(stage_result);
        }

        let all_successful = stage_results.iter().all(|r| r.is_success());
        let status = if all_successful {
            PipelineStatus::Completed
        } else {
            PipelineStatus::PartialSuccess
        };

        tracing::info!(
            pipeline_id = %self.id,
            status = ?status,
            duration_ms = start_time.elapsed().as_millis(),
            "Pipeline execution completed"
        );

        Ok(PipelineResult {
            pipeline_id: self.id.clone(),
            pipeline_name: self.name.clone(),
            status,
            stage_results,
            duration_ms: start_time.elapsed().as_millis() as u64,
            error: None,
        })
    }

    /// Execute a single pipeline stage
    async fn execute_stage(&self, stage: &PipelineStage, input_data: &serde_json::Value) -> Result<JobResult> {
        let job_data = if stage.pass_input && input_data != &serde_json::Value::Null {
            input_data.clone()
        } else {
            stage.job_data.clone()
        };

        // In a real implementation, you would deserialize and create the actual job
        // For now, we'll create a mock result
        let job_id = Uuid::new_v4().to_string();
        let context = Arc::new(JobContext::new(job_id.clone(), "pipeline".to_string()));

        // Simulate execution
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let result = JobResult::success(job_id, serde_json::json!({
            "stage": stage.name,
            "processed": true,
            "input": job_data
        }));

        Ok(result)
    }

    /// Get pipeline ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get pipeline name
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Pipeline stage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStage {
    pub name: String,
    pub job_type: String,
    pub job_data: serde_json::Value,
    pub pass_input: bool,
    pub pass_output: bool,
    pub fail_fast: bool,
    pub retry_on_failure: bool,
}

impl PipelineStage {
    /// Create a new pipeline stage
    pub fn new(name: String, job_type: String) -> Self {
        Self {
            name,
            job_type,
            job_data: serde_json::Value::Null,
            pass_input: false,
            pass_output: true,
            fail_fast: true,
            retry_on_failure: false,
        }
    }

    /// Set job data
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.job_data = data;
        self
    }

    /// Enable input passing
    pub fn with_input(mut self) -> Self {
        self.pass_input = true;
        self
    }

    /// Disable fail fast
    pub fn continue_on_failure(mut self) -> Self {
        self.fail_fast = false;
        self
    }
}

/// Pipeline execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineResult {
    pub pipeline_id: String,
    pub pipeline_name: String,
    pub status: PipelineStatus,
    pub stage_results: Vec<JobResult>,
    pub duration_ms: u64,
    pub error: Option<String>,
}

impl PipelineResult {
    /// Check if pipeline completed successfully
    pub fn is_success(&self) -> bool {
        matches!(self.status, PipelineStatus::Completed)
    }

    /// Get failed stages
    pub fn failed_stages(&self) -> Vec<&JobResult> {
        self.stage_results.iter().filter(|r| r.is_failure()).collect()
    }
}

/// Pipeline status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PipelineStatus {
    Pending,
    Running,
    Completed,
    PartialSuccess,
    Failed,
    Cancelled,
}

/// Parallel pipeline for executing jobs concurrently
pub struct ParallelPipeline {
    id: String,
    name: String,
    jobs: Vec<Box<dyn Job>>,
    executor: Arc<JobExecutor>,
}

impl ParallelPipeline {
    /// Create a new parallel pipeline
    pub fn new(name: String, executor: Arc<JobExecutor>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            jobs: Vec::new(),
            executor,
        }
    }

    /// Add a job to the pipeline
    pub fn add_job(mut self, job: Box<dyn Job>) -> Self {
        self.jobs.push(job);
        self
    }

    /// Execute all jobs in parallel
    pub async fn execute(&self) -> Result<Vec<JobResult>> {
        tracing::info!(
            pipeline_id = %self.id,
            pipeline_name = %self.name,
            jobs = self.jobs.len(),
            "Starting parallel pipeline execution"
        );

        let mut handles = Vec::new();

        for job in &self.jobs {
            let job_id = job.id().to_string();
            let job_data = job.serialize()?;
            let executor = self.executor.clone();

            let handle = tokio::spawn(async move {
                let job: Box<dyn Job> = serde_json::from_str(&job_data).unwrap();
                let context = Arc::new(JobContext::new(job_id, "parallel-pipeline".to_string()));
                executor.execute(job, context).await
            });

            handles.push(handle);
        }

        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(Ok(result)) => results.push(result),
                Ok(Err(e)) => return Err(e),
                Err(e) => {
                    return Err(JobError::PipelineError(format!(
                        "Task join error: {}",
                        e
                    )))
                }
            }
        }

        tracing::info!(
            pipeline_id = %self.id,
            results = results.len(),
            "Parallel pipeline execution completed"
        );

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::retry::RetryPolicy;

    #[tokio::test]
    async fn test_pipeline_creation() {
        let executor = Arc::new(JobExecutor::new(RetryPolicy::none()));
        let pipeline = JobPipeline::new("test-pipeline".to_string(), executor)
            .add_stage(
                PipelineStage::new("stage1".to_string(), "physics_simulation".to_string())
                    .with_data(serde_json::json!({"test": 1})),
            )
            .add_stage(
                PipelineStage::new("stage2".to_string(), "report_generation".to_string())
                    .with_input(),
            );

        assert_eq!(pipeline.name(), "test-pipeline");
    }

    #[tokio::test]
    async fn test_pipeline_execution() {
        let executor = Arc::new(JobExecutor::new(RetryPolicy::none()));
        let pipeline = JobPipeline::new("test-pipeline".to_string(), executor)
            .add_stage(
                PipelineStage::new("stage1".to_string(), "physics_simulation".to_string())
                    .with_data(serde_json::json!({"test": 1})),
            );

        let result = pipeline.execute().await.unwrap();
        assert!(result.is_success());
    }
}
