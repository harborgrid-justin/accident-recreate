//! Batch job processing for executing multiple jobs efficiently.

use crate::error::Result;
use crate::executor::JobExecutor;
use crate::job::{Job, JobContext};
use crate::result::{BatchJobResult, JobResult};
use rayon::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

/// Batch job processor
pub struct BatchProcessor {
    executor: Arc<JobExecutor>,
    max_parallelism: usize,
}

impl BatchProcessor {
    /// Create a new batch processor
    pub fn new(executor: Arc<JobExecutor>, max_parallelism: usize) -> Self {
        Self {
            executor,
            max_parallelism,
        }
    }

    /// Process a batch of jobs sequentially
    pub async fn process_sequential(&self, jobs: Vec<Box<dyn Job>>) -> Result<BatchJobResult> {
        let batch_id = Uuid::new_v4().to_string();
        let mut batch_result = BatchJobResult::new(batch_id.clone());

        tracing::info!(
            batch_id = %batch_id,
            job_count = jobs.len(),
            "Starting sequential batch processing"
        );

        for job in jobs {
            let job_id = job.id().to_string();
            let context = Arc::new(JobContext::new(job_id.clone(), "batch-processor".to_string()));

            match self.executor.execute(job, context).await {
                Ok(result) => batch_result.add_result(result),
                Err(e) => {
                    let error_result = JobResult::failure(job_id, e.to_string());
                    batch_result.add_result(error_result);
                }
            }
        }

        batch_result.completed_at = chrono::Utc::now();

        tracing::info!(
            batch_id = %batch_id,
            total = batch_result.total_jobs,
            successful = batch_result.successful,
            failed = batch_result.failed,
            "Sequential batch processing completed"
        );

        Ok(batch_result)
    }

    /// Process a batch of jobs in parallel using async
    pub async fn process_parallel(&self, jobs: Vec<Box<dyn Job>>) -> Result<BatchJobResult> {
        let batch_id = Uuid::new_v4().to_string();
        let mut batch_result = BatchJobResult::new(batch_id.clone());

        tracing::info!(
            batch_id = %batch_id,
            job_count = jobs.len(),
            max_parallelism = self.max_parallelism,
            "Starting parallel batch processing"
        );

        // Create chunks based on max parallelism
        let chunk_size = self.max_parallelism;
        let mut all_results = Vec::new();

        for chunk in jobs.chunks(chunk_size) {
            let mut handles = Vec::new();

            for job in chunk {
                let job_id = job.id().to_string();
                let job_data = job.serialize()?;
                let executor = self.executor.clone();

                let handle = tokio::spawn(async move {
                    let job: Box<dyn Job> = serde_json::from_str(&job_data).unwrap();
                    let context = Arc::new(JobContext::new(job_id.clone(), "batch-processor".to_string()));
                    executor.execute(job, context).await
                });

                handles.push(handle);
            }

            // Wait for chunk to complete
            for handle in handles {
                match handle.await {
                    Ok(Ok(result)) => all_results.push(result),
                    Ok(Err(e)) => {
                        let error_result = JobResult::failure(Uuid::new_v4().to_string(), e.to_string());
                        all_results.push(error_result);
                    }
                    Err(e) => {
                        let error_result = JobResult::failure(
                            Uuid::new_v4().to_string(),
                            format!("Task join error: {}", e),
                        );
                        all_results.push(error_result);
                    }
                }
            }
        }

        for result in all_results {
            batch_result.add_result(result);
        }

        batch_result.completed_at = chrono::Utc::now();

        tracing::info!(
            batch_id = %batch_id,
            total = batch_result.total_jobs,
            successful = batch_result.successful,
            failed = batch_result.failed,
            "Parallel batch processing completed"
        );

        Ok(batch_result)
    }

    /// Process a batch with a custom strategy
    pub async fn process_with_strategy(
        &self,
        jobs: Vec<Box<dyn Job>>,
        strategy: BatchStrategy,
    ) -> Result<BatchJobResult> {
        match strategy {
            BatchStrategy::Sequential => self.process_sequential(jobs).await,
            BatchStrategy::Parallel => self.process_parallel(jobs).await,
            BatchStrategy::FailFast => self.process_fail_fast(jobs).await,
        }
    }

    /// Process batch with fail-fast strategy
    async fn process_fail_fast(&self, jobs: Vec<Box<dyn Job>>) -> Result<BatchJobResult> {
        let batch_id = Uuid::new_v4().to_string();
        let mut batch_result = BatchJobResult::new(batch_id.clone());

        tracing::info!(
            batch_id = %batch_id,
            job_count = jobs.len(),
            "Starting fail-fast batch processing"
        );

        for job in jobs {
            let job_id = job.id().to_string();
            let context = Arc::new(JobContext::new(job_id.clone(), "batch-processor".to_string()));

            match self.executor.execute(job, context).await {
                Ok(result) => {
                    let is_failure = result.is_failure();
                    batch_result.add_result(result);

                    if is_failure {
                        tracing::warn!(
                            batch_id = %batch_id,
                            job_id = %job_id,
                            "Job failed, stopping batch (fail-fast)"
                        );
                        break;
                    }
                }
                Err(e) => {
                    let error_result = JobResult::failure(job_id.clone(), e.to_string());
                    batch_result.add_result(error_result);
                    tracing::error!(
                        batch_id = %batch_id,
                        job_id = %job_id,
                        error = %e,
                        "Job execution error, stopping batch (fail-fast)"
                    );
                    break;
                }
            }
        }

        batch_result.completed_at = chrono::Utc::now();

        Ok(batch_result)
    }
}

/// Batch processing strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchStrategy {
    Sequential,
    Parallel,
    FailFast,
}

/// Batch job builder
pub struct BatchJobBuilder {
    jobs: Vec<Box<dyn Job>>,
    strategy: BatchStrategy,
}

impl BatchJobBuilder {
    /// Create a new batch job builder
    pub fn new() -> Self {
        Self {
            jobs: Vec::new(),
            strategy: BatchStrategy::Parallel,
        }
    }

    /// Add a job to the batch
    pub fn add_job(mut self, job: Box<dyn Job>) -> Self {
        self.jobs.push(job);
        self
    }

    /// Set batch strategy
    pub fn with_strategy(mut self, strategy: BatchStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Build and execute the batch
    pub async fn execute(self, processor: &BatchProcessor) -> Result<BatchJobResult> {
        processor.process_with_strategy(self.jobs, self.strategy).await
    }
}

impl Default for BatchJobBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::job::PhysicsSimulationJob;
    use crate::retry::RetryPolicy;

    #[tokio::test]
    async fn test_batch_sequential() {
        let executor = Arc::new(JobExecutor::new(RetryPolicy::none()));
        let processor = BatchProcessor::new(executor, 4);

        let jobs: Vec<Box<dyn Job>> = (0..5)
            .map(|i| {
                Box::new(PhysicsSimulationJob::new(
                    format!("scenario-{}", i),
                    serde_json::json!({}),
                )) as Box<dyn Job>
            })
            .collect();

        let result = processor.process_sequential(jobs).await.unwrap();
        assert_eq!(result.total_jobs, 5);
        assert_eq!(result.successful, 5);
    }

    #[tokio::test]
    async fn test_batch_parallel() {
        let executor = Arc::new(JobExecutor::new(RetryPolicy::none()));
        let processor = BatchProcessor::new(executor, 4);

        let jobs: Vec<Box<dyn Job>> = (0..5)
            .map(|i| {
                Box::new(PhysicsSimulationJob::new(
                    format!("scenario-{}", i),
                    serde_json::json!({}),
                )) as Box<dyn Job>
            })
            .collect();

        let result = processor.process_parallel(jobs).await.unwrap();
        assert_eq!(result.total_jobs, 5);
        assert_eq!(result.successful, 5);
    }

    #[tokio::test]
    async fn test_batch_builder() {
        let executor = Arc::new(JobExecutor::new(RetryPolicy::none()));
        let processor = BatchProcessor::new(executor, 4);

        let result = BatchJobBuilder::new()
            .add_job(Box::new(PhysicsSimulationJob::new(
                "scenario-1".to_string(),
                serde_json::json!({}),
            )))
            .add_job(Box::new(PhysicsSimulationJob::new(
                "scenario-2".to_string(),
                serde_json::json!({}),
            )))
            .with_strategy(BatchStrategy::Sequential)
            .execute(&processor)
            .await
            .unwrap();

        assert_eq!(result.total_jobs, 2);
    }
}
