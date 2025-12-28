//! Job scheduling implementations.

pub mod cron;
pub mod delayed;
pub mod recurring;

use crate::error::Result;
use crate::job::Job;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Job scheduler trait
#[async_trait]
pub trait JobScheduler: Send + Sync {
    /// Schedule a job
    async fn schedule(&self, job: Box<dyn Job>, schedule: Schedule) -> Result<String>;

    /// Cancel a scheduled job
    async fn cancel(&self, schedule_id: &str) -> Result<bool>;

    /// Get next execution time for a scheduled job
    async fn next_execution(&self, schedule_id: &str) -> Result<Option<DateTime<Utc>>>;

    /// List all scheduled jobs
    async fn list_scheduled(&self) -> Result<Vec<ScheduledJob>>;
}

/// Schedule type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Schedule {
    /// Execute once at a specific time
    Once(DateTime<Utc>),

    /// Execute after a delay
    Delayed {
        delay_secs: u64,
    },

    /// Execute at regular intervals
    Recurring {
        interval_secs: u64,
        start_at: Option<DateTime<Utc>>,
        end_at: Option<DateTime<Utc>>,
    },

    /// Execute based on cron expression
    Cron {
        expression: String,
        timezone: Option<String>,
    },
}

/// Scheduled job information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledJob {
    pub schedule_id: String,
    pub job_id: String,
    pub job_name: String,
    pub schedule: Schedule,
    pub created_at: DateTime<Utc>,
    pub next_run: Option<DateTime<Utc>>,
    pub last_run: Option<DateTime<Utc>>,
    pub run_count: u64,
}

impl ScheduledJob {
    pub fn new(schedule_id: String, job_id: String, job_name: String, schedule: Schedule) -> Self {
        Self {
            schedule_id,
            job_id,
            job_name,
            schedule,
            created_at: Utc::now(),
            next_run: None,
            last_run: None,
            run_count: 0,
        }
    }
}
