//! Job executor with retry logic and timeout handling.

use crate::error::{JobError, Result};
use crate::job::{Job, JobContext};
use crate::result::JobResult;
use crate::retry::RetryPolicy;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

/// Job executor
pub struct JobExecutor {
    retry_policy: RetryPolicy,
    results: Arc<RwLock<HashMap<String, JobResult>>>,
}

impl JobExecutor {
    /// Create a new job executor
    pub fn new(retry_policy: RetryPolicy) -> Self {
        Self {
            retry_policy,
            results: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Execute a job with retry logic
    pub async fn execute(&self, mut job: Box<dyn Job>, context: Arc<JobContext>) -> Result<JobResult> {
        let job_id = job.id().to_string();
        let max_retries = job.max_retries();
        let timeout = job.timeout_secs();

        tracing::info!(
            job_id = %job_id,
            job_name = %job.name(),
            "Starting job execution"
        );

        let mut attempt = 0;
        let start_time = Instant::now();

        loop {
            attempt += 1;
            let attempt_context = Arc::new(context.as_ref().clone().with_attempt(attempt));

            // Call before_execute hook
            if let Err(e) = job.before_execute(attempt_context.clone()).await {
                tracing::error!(
                    job_id = %job_id,
                    error = %e,
                    "before_execute hook failed"
                );
                return Err(e);
            }

            // Execute the job with optional timeout
            let result = if let Some(timeout_secs) = timeout {
                match tokio::time::timeout(
                    tokio::time::Duration::from_secs(timeout_secs),
                    job.execute(attempt_context.clone()),
                )
                .await
                {
                    Ok(result) => result,
                    Err(_) => Err(JobError::Timeout {
                        duration_secs: timeout_secs,
                    }),
                }
            } else {
                job.execute(attempt_context.clone()).await
            };

            let duration_ms = start_time.elapsed().as_millis() as u64;

            match result {
                Ok(job_result) => {
                    // Call after_execute hook
                    if let Err(e) = job.after_execute(attempt_context, &job_result).await {
                        tracing::error!(
                            job_id = %job_id,
                            error = %e,
                            "after_execute hook failed"
                        );
                    }

                    let final_result = job_result.with_duration(duration_ms);

                    tracing::info!(
                        job_id = %job_id,
                        duration_ms = duration_ms,
                        "Job completed successfully"
                    );

                    // Store result
                    self.results.write().insert(job_id.clone(), final_result.clone());

                    return Ok(final_result);
                }
                Err(error) => {
                    // Call on_failure hook
                    if let Err(e) = job.on_failure(attempt_context, &error).await {
                        tracing::error!(
                            job_id = %job_id,
                            error = %e,
                            "on_failure hook failed"
                        );
                    }

                    // Check if we should retry
                    if self.retry_policy.should_retry(attempt, &error) {
                        if let Some(delay) = self.retry_policy.next_delay(attempt) {
                            tracing::warn!(
                                job_id = %job_id,
                                attempt = attempt,
                                max_retries = max_retries,
                                delay_ms = delay.as_millis(),
                                error = %error,
                                "Job failed, retrying"
                            );

                            tokio::time::sleep(delay).await;
                            continue;
                        }
                    }

                    // No more retries
                    let final_result = JobResult::failure(job_id.clone(), error.to_string())
                        .with_duration(duration_ms);

                    tracing::error!(
                        job_id = %job_id,
                        attempt = attempt,
                        duration_ms = duration_ms,
                        error = %error,
                        "Job failed permanently"
                    );

                    // Store result
                    self.results.write().insert(job_id.clone(), final_result.clone());

                    return Ok(final_result);
                }
            }
        }
    }

    /// Get result for a job
    pub fn get_result(&self, job_id: &str) -> Option<JobResult> {
        self.results.read().get(job_id).cloned()
    }

    /// Clear result for a job
    pub fn clear_result(&self, job_id: &str) -> bool {
        self.results.write().remove(job_id).is_some()
    }

    /// Clear all results
    pub fn clear_all_results(&self) {
        self.results.write().clear();
    }

    /// Get all results
    pub fn get_all_results(&self) -> HashMap<String, JobResult> {
        self.results.read().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::job::PhysicsSimulationJob;
    use crate::retry::RetryPolicy;

    #[tokio::test]
    async fn test_executor_success() {
        let executor = JobExecutor::new(RetryPolicy::none());
        let job = Box::new(PhysicsSimulationJob::new(
            "test-scenario".to_string(),
            serde_json::json!({}),
        ));
        let job_id = job.id().to_string();
        let context = Arc::new(JobContext::new(job_id.clone(), "worker-1".to_string()));

        let result = executor.execute(job, context).await.unwrap();
        assert!(result.is_success());

        let stored_result = executor.get_result(&job_id);
        assert!(stored_result.is_some());
    }

    #[tokio::test]
    async fn test_executor_timeout() {
        use crate::job::{Job, JobContext};
        use async_trait::async_trait;

        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        struct SlowJob {
            id: String,
        }

        #[async_trait]
        impl Job for SlowJob {
            async fn execute(&mut self, _context: Arc<JobContext>) -> Result<JobResult> {
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                Ok(JobResult::success(self.id.clone(), serde_json::json!({})))
            }

            fn id(&self) -> &str {
                &self.id
            }

            fn name(&self) -> &str {
                "slow_job"
            }

            fn timeout_secs(&self) -> Option<u64> {
                Some(1) // 1 second timeout
            }

            fn serialize(&self) -> Result<String> {
                serde_json::to_string(self).map_err(Into::into)
            }

            fn deserialize(data: &str) -> Result<Box<dyn Job>> {
                let job: SlowJob = serde_json::from_str(data)?;
                Ok(Box::new(job))
            }
        }

        let executor = JobExecutor::new(RetryPolicy::none());
        let job = Box::new(SlowJob {
            id: "slow-job-1".to_string(),
        });
        let context = Arc::new(JobContext::new("slow-job-1".to_string(), "worker-1".to_string()));

        let result = executor.execute(job, context).await.unwrap();
        assert!(result.is_failure());
    }
}
