//! Recurring job execution.

use crate::error::Result;
use crate::job::Job;
use crate::queue::JobQueue;
use crate::scheduler::{Schedule, ScheduledJob};
use chrono::{DateTime, Duration, Utc};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Recurring job scheduler
pub struct RecurringScheduler {
    scheduled_jobs: Arc<RwLock<HashMap<String, (ScheduledJob, Box<dyn Job>)>>>,
    queue: Arc<dyn JobQueue>,
}

impl RecurringScheduler {
    /// Create a new recurring scheduler
    pub fn new(queue: Arc<dyn JobQueue>) -> Self {
        Self {
            scheduled_jobs: Arc::new(RwLock::new(HashMap::new())),
            queue,
        }
    }

    /// Schedule a job to run at regular intervals
    pub async fn schedule_recurring(
        &self,
        job: Box<dyn Job>,
        interval_secs: u64,
        start_at: Option<DateTime<Utc>>,
        end_at: Option<DateTime<Utc>>,
    ) -> Result<String> {
        let schedule_id = Uuid::new_v4().to_string();
        let job_id = job.id().to_string();
        let job_name = job.name().to_string();

        let schedule = Schedule::Recurring {
            interval_secs,
            start_at,
            end_at,
        };

        let mut scheduled_job = ScheduledJob::new(
            schedule_id.clone(),
            job_id.clone(),
            job_name,
            schedule,
        );

        // Calculate next run time
        let next_run = start_at.unwrap_or_else(|| Utc::now() + Duration::seconds(interval_secs as i64));
        scheduled_job.next_run = Some(next_run);

        self.scheduled_jobs
            .write()
            .insert(schedule_id.clone(), (scheduled_job, job));

        tracing::info!(
            schedule_id = %schedule_id,
            job_id = %job_id,
            interval_secs = interval_secs,
            next_run = %next_run,
            "Job scheduled as recurring"
        );

        Ok(schedule_id)
    }

    /// Schedule a job to run every N seconds
    pub async fn every_seconds(&self, job: Box<dyn Job>, seconds: u64) -> Result<String> {
        self.schedule_recurring(job, seconds, None, None).await
    }

    /// Schedule a job to run every N minutes
    pub async fn every_minutes(&self, job: Box<dyn Job>, minutes: u64) -> Result<String> {
        self.schedule_recurring(job, minutes * 60, None, None).await
    }

    /// Schedule a job to run every N hours
    pub async fn every_hours(&self, job: Box<dyn Job>, hours: u64) -> Result<String> {
        self.schedule_recurring(job, hours * 3600, None, None).await
    }

    /// Cancel a scheduled job
    pub async fn cancel(&self, schedule_id: &str) -> Result<bool> {
        let removed = self.scheduled_jobs.write().remove(schedule_id).is_some();

        if removed {
            tracing::info!(schedule_id = %schedule_id, "Recurring job cancelled");
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

        let mut scheduled_jobs = self.scheduled_jobs.write();

        for (schedule_id, (scheduled_job, job)) in scheduled_jobs.iter_mut() {
            if let Some(next_run) = scheduled_job.next_run {
                if next_run <= now {
                    // Check if job should still run (end_at not reached)
                    if let Schedule::Recurring { interval_secs, end_at, .. } = &scheduled_job.schedule {
                        if let Some(end_time) = end_at {
                            if now >= *end_time {
                                to_remove.push(schedule_id.clone());
                                tracing::info!(
                                    schedule_id = %schedule_id,
                                    "Recurring job ended (reached end_at)"
                                );
                                continue;
                            }
                        }

                        // Clone the job for execution
                        let job_clone_data = job.serialize()?;
                        let job_clone: Box<dyn Job> = serde_json::from_str(&job_clone_data)?;

                        // Queue the job
                        self.queue.push(job_clone).await?;

                        // Update scheduled job info
                        scheduled_job.last_run = Some(now);
                        scheduled_job.run_count += 1;
                        scheduled_job.next_run = Some(now + Duration::seconds(*interval_secs as i64));

                        executed_jobs.push(schedule_id.clone());

                        tracing::info!(
                            schedule_id = %schedule_id,
                            job_id = %job.id(),
                            run_count = scheduled_job.run_count,
                            next_run = ?scheduled_job.next_run,
                            "Recurring job triggered"
                        );
                    }
                }
            }
        }

        // Remove jobs that have ended
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
                    tracing::error!(error = %e, "Recurring scheduler tick failed");
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
    async fn test_recurring_scheduler() {
        let queue = Arc::new(MemoryQueue::new());
        let scheduler = RecurringScheduler::new(queue.clone());

        let job = Box::new(PhysicsSimulationJob::new(
            "test-scenario".to_string(),
            serde_json::json!({}),
        ));

        // Schedule job to run every 2 seconds
        let schedule_id = scheduler.every_seconds(job, 2).await.unwrap();

        let scheduled_jobs = scheduler.list_scheduled().await;
        assert_eq!(scheduled_jobs.len(), 1);

        // Wait and tick
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        scheduler.tick().await.unwrap();

        // Job should be in queue
        assert!(queue.len().await.unwrap() > 0);

        // Scheduled job should still exist (recurring)
        let scheduled_jobs = scheduler.list_scheduled().await;
        assert_eq!(scheduled_jobs.len(), 1);
    }

    #[tokio::test]
    async fn test_recurring_with_end_time() {
        let queue = Arc::new(MemoryQueue::new());
        let scheduler = RecurringScheduler::new(queue.clone());

        let job = Box::new(PhysicsSimulationJob::new(
            "test-scenario".to_string(),
            serde_json::json!({}),
        ));

        let start_at = Utc::now();
        let end_at = start_at + Duration::seconds(1);

        let schedule_id = scheduler
            .schedule_recurring(job, 1, Some(start_at), Some(end_at))
            .await
            .unwrap();

        // Wait past end time
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        scheduler.tick().await.unwrap();

        // Job should be removed after end_at
        let scheduled_jobs = scheduler.list_scheduled().await;
        assert_eq!(scheduled_jobs.len(), 0);
    }
}
