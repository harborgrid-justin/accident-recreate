//! Delayed job execution.

use crate::error::Result;
use crate::job::Job;
use crate::queue::JobQueue;
use crate::scheduler::{Schedule, ScheduledJob};
use chrono::{DateTime, Duration, Utc};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Delayed job scheduler
pub struct DelayedScheduler {
    scheduled_jobs: Arc<RwLock<HashMap<String, (ScheduledJob, Box<dyn Job>)>>>,
    queue: Arc<dyn JobQueue>,
}

impl DelayedScheduler {
    /// Create a new delayed scheduler
    pub fn new(queue: Arc<dyn JobQueue>) -> Self {
        Self {
            scheduled_jobs: Arc::new(RwLock::new(HashMap::new())),
            queue,
        }
    }

    /// Schedule a job to run after a delay
    pub async fn schedule_delayed(&self, job: Box<dyn Job>, delay_secs: u64) -> Result<String> {
        let schedule_id = Uuid::new_v4().to_string();
        let job_id = job.id().to_string();
        let job_name = job.name().to_string();

        let schedule = Schedule::Delayed { delay_secs };

        let mut scheduled_job = ScheduledJob::new(
            schedule_id.clone(),
            job_id.clone(),
            job_name,
            schedule,
        );

        // Calculate next run time
        let next_run = Utc::now() + Duration::seconds(delay_secs as i64);
        scheduled_job.next_run = Some(next_run);

        self.scheduled_jobs
            .write()
            .insert(schedule_id.clone(), (scheduled_job, job));

        tracing::info!(
            schedule_id = %schedule_id,
            job_id = %job_id,
            delay_secs = delay_secs,
            next_run = %next_run,
            "Job scheduled with delay"
        );

        Ok(schedule_id)
    }

    /// Schedule a job to run at a specific time
    pub async fn schedule_at(&self, job: Box<dyn Job>, run_at: DateTime<Utc>) -> Result<String> {
        let schedule_id = Uuid::new_v4().to_string();
        let job_id = job.id().to_string();
        let job_name = job.name().to_string();

        let schedule = Schedule::Once(run_at);

        let mut scheduled_job = ScheduledJob::new(
            schedule_id.clone(),
            job_id.clone(),
            job_name,
            schedule,
        );

        scheduled_job.next_run = Some(run_at);

        self.scheduled_jobs
            .write()
            .insert(schedule_id.clone(), (scheduled_job, job));

        tracing::info!(
            schedule_id = %schedule_id,
            job_id = %job_id,
            run_at = %run_at,
            "Job scheduled to run at specific time"
        );

        Ok(schedule_id)
    }

    /// Cancel a scheduled job
    pub async fn cancel(&self, schedule_id: &str) -> Result<bool> {
        let removed = self.scheduled_jobs.write().remove(schedule_id).is_some();

        if removed {
            tracing::info!(schedule_id = %schedule_id, "Delayed job cancelled");
        }

        Ok(removed)
    }

    /// Get next execution time
    pub async fn next_execution(&self, schedule_id: &str) -> Result<Option<DateTime<Utc>>> {
        let scheduled_jobs = self.scheduled_jobs.read();
        Ok(scheduled_jobs
            .get(schedule_id)
            .and_then(|(job, _)| job.next_run))
    }

    /// Tick the scheduler - check for jobs that need to run
    pub async fn tick(&self) -> Result<Vec<String>> {
        let now = Utc::now();
        let mut executed_jobs = Vec::new();
        let mut to_remove = Vec::new();

        {
            let scheduled_jobs = self.scheduled_jobs.read();

            for (schedule_id, (scheduled_job, job)) in scheduled_jobs.iter() {
                if let Some(next_run) = scheduled_job.next_run {
                    if next_run <= now {
                        // Clone the job for execution
                        let job_clone_data = job.serialize()?;
                        let job_clone: Box<dyn Job> = serde_json::from_str(&job_clone_data)?;

                        // Queue the job
                        self.queue.push(job_clone).await?;

                        executed_jobs.push(schedule_id.clone());
                        to_remove.push(schedule_id.clone());

                        tracing::info!(
                            schedule_id = %schedule_id,
                            job_id = %job.id(),
                            "Delayed job triggered"
                        );
                    }
                }
            }
        }

        // Remove one-time jobs after execution
        let mut scheduled_jobs = self.scheduled_jobs.write();
        for schedule_id in to_remove {
            scheduled_jobs.remove(&schedule_id);
        }

        Ok(executed_jobs)
    }

    /// Start the scheduler in the background
    pub fn start(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));

            loop {
                interval.tick().await;

                if let Err(e) = self.tick().await {
                    tracing::error!(error = %e, "Delayed scheduler tick failed");
                }
            }
        });
    }

    /// List all scheduled jobs
    pub async fn list_scheduled(&self) -> Vec<ScheduledJob> {
        self.scheduled_jobs
            .read()
            .values()
            .map(|(job, _)| job.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::job::PhysicsSimulationJob;
    use crate::queue::memory::MemoryQueue;

    #[tokio::test]
    async fn test_delayed_scheduler() {
        let queue = Arc::new(MemoryQueue::new());
        let scheduler = DelayedScheduler::new(queue.clone());

        let job = Box::new(PhysicsSimulationJob::new(
            "test-scenario".to_string(),
            serde_json::json!({}),
        ));

        // Schedule job to run in 1 second
        let schedule_id = scheduler.schedule_delayed(job, 1).await.unwrap();

        let scheduled_jobs = scheduler.list_scheduled().await;
        assert_eq!(scheduled_jobs.len(), 1);

        // Job should not be in queue yet
        assert_eq!(queue.len().await.unwrap(), 0);

        // Wait for job to be scheduled
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        scheduler.tick().await.unwrap();

        // Job should now be in queue
        assert_eq!(queue.len().await.unwrap(), 1);

        // Scheduled job should be removed after execution
        let scheduled_jobs = scheduler.list_scheduled().await;
        assert_eq!(scheduled_jobs.len(), 0);
    }

    #[tokio::test]
    async fn test_schedule_at() {
        let queue = Arc::new(MemoryQueue::new());
        let scheduler = DelayedScheduler::new(queue.clone());

        let job = Box::new(PhysicsSimulationJob::new(
            "test-scenario".to_string(),
            serde_json::json!({}),
        ));

        let run_at = Utc::now() + Duration::seconds(1);
        let schedule_id = scheduler.schedule_at(job, run_at).await.unwrap();

        let next_run = scheduler.next_execution(&schedule_id).await.unwrap();
        assert!(next_run.is_some());
    }
}
