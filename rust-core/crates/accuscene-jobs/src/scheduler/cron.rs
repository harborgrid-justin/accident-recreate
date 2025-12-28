//! Cron-based job scheduling.

use crate::error::{JobError, Result};
use crate::job::Job;
use crate::queue::JobQueue;
use crate::scheduler::{Schedule, ScheduledJob};
use chrono::{DateTime, Utc};
use cron::Schedule as CronSchedule;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

/// Cron-based job scheduler
pub struct CronScheduler {
    scheduled_jobs: Arc<RwLock<HashMap<String, (ScheduledJob, Box<dyn Job>, CronSchedule)>>>,
    queue: Arc<dyn JobQueue>,
}

impl CronScheduler {
    /// Create a new cron scheduler
    pub fn new(queue: Arc<dyn JobQueue>) -> Self {
        Self {
            scheduled_jobs: Arc::new(RwLock::new(HashMap::new())),
            queue,
        }
    }

    /// Schedule a job with a cron expression
    pub async fn schedule_cron(&self, job: Box<dyn Job>, expression: String) -> Result<String> {
        let schedule_id = Uuid::new_v4().to_string();
        let job_id = job.id().to_string();
        let job_name = job.name().to_string();

        // Parse cron expression
        let cron_schedule = CronSchedule::from_str(&expression)
            .map_err(|e| JobError::InvalidCronExpression(format!("Invalid cron expression: {}", e)))?;

        let schedule = Schedule::Cron {
            expression: expression.clone(),
            timezone: None,
        };

        let mut scheduled_job = ScheduledJob::new(
            schedule_id.clone(),
            job_id.clone(),
            job_name,
            schedule,
        );

        // Calculate next run time
        scheduled_job.next_run = cron_schedule.upcoming(Utc).next();

        self.scheduled_jobs.write().insert(
            schedule_id.clone(),
            (scheduled_job, job, cron_schedule),
        );

        tracing::info!(
            schedule_id = %schedule_id,
            job_id = %job_id,
            expression = %expression,
            next_run = ?scheduled_job.next_run,
            "Job scheduled with cron"
        );

        Ok(schedule_id)
    }

    /// Cancel a scheduled job
    pub async fn cancel(&self, schedule_id: &str) -> Result<bool> {
        let removed = self.scheduled_jobs.write().remove(schedule_id).is_some();

        if removed {
            tracing::info!(schedule_id = %schedule_id, "Scheduled job cancelled");
        }

        Ok(removed)
    }

    /// Get next execution time
    pub async fn next_execution(&self, schedule_id: &str) -> Result<Option<DateTime<Utc>>> {
        let scheduled_jobs = self.scheduled_jobs.read();
        Ok(scheduled_jobs
            .get(schedule_id)
            .and_then(|(job, _, _)| job.next_run))
    }

    /// Tick the scheduler - check for jobs that need to run
    pub async fn tick(&self) -> Result<Vec<String>> {
        let now = Utc::now();
        let mut executed_jobs = Vec::new();

        let mut scheduled_jobs = self.scheduled_jobs.write();

        for (schedule_id, (scheduled_job, job, cron_schedule)) in scheduled_jobs.iter_mut() {
            if let Some(next_run) = scheduled_job.next_run {
                if next_run <= now {
                    // Clone the job for execution
                    let job_clone_data = job.serialize()?;
                    let job_clone: Box<dyn Job> = serde_json::from_str(&job_clone_data)?;

                    // Queue the job
                    self.queue.push(job_clone).await?;

                    // Update scheduled job info
                    scheduled_job.last_run = Some(now);
                    scheduled_job.run_count += 1;
                    scheduled_job.next_run = cron_schedule.upcoming(Utc).next();

                    executed_jobs.push(schedule_id.clone());

                    tracing::info!(
                        schedule_id = %schedule_id,
                        job_id = %job.id(),
                        next_run = ?scheduled_job.next_run,
                        "Cron job triggered"
                    );
                }
            }
        }

        Ok(executed_jobs)
    }

    /// Start the scheduler in the background
    pub fn start(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));

            loop {
                interval.tick().await;

                if let Err(e) = self.tick().await {
                    tracing::error!(error = %e, "Cron scheduler tick failed");
                }
            }
        });
    }

    /// List all scheduled jobs
    pub async fn list_scheduled(&self) -> Vec<ScheduledJob> {
        self.scheduled_jobs
            .read()
            .values()
            .map(|(job, _, _)| job.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::job::PhysicsSimulationJob;
    use crate::queue::memory::MemoryQueue;

    #[tokio::test]
    async fn test_cron_scheduler() {
        let queue = Arc::new(MemoryQueue::new());
        let scheduler = CronScheduler::new(queue.clone());

        let job = Box::new(PhysicsSimulationJob::new(
            "test-scenario".to_string(),
            serde_json::json!({}),
        ));

        // Schedule job to run every minute
        let schedule_id = scheduler
            .schedule_cron(job, "* * * * * *".to_string())
            .await
            .unwrap();

        let scheduled_jobs = scheduler.list_scheduled().await;
        assert_eq!(scheduled_jobs.len(), 1);

        let next_run = scheduler.next_execution(&schedule_id).await.unwrap();
        assert!(next_run.is_some());
    }

    #[tokio::test]
    async fn test_cron_cancel() {
        let queue = Arc::new(MemoryQueue::new());
        let scheduler = CronScheduler::new(queue.clone());

        let job = Box::new(PhysicsSimulationJob::new(
            "test-scenario".to_string(),
            serde_json::json!({}),
        ));

        let schedule_id = scheduler
            .schedule_cron(job, "* * * * * *".to_string())
            .await
            .unwrap();

        let cancelled = scheduler.cancel(&schedule_id).await.unwrap();
        assert!(cancelled);

        let scheduled_jobs = scheduler.list_scheduled().await;
        assert_eq!(scheduled_jobs.len(), 0);
    }
}
