//! AccuScene Jobs - Multi-threaded job processing and task queue system
//!
//! This crate provides a robust job processing system with the following features:
//! - Multiple queue implementations (in-memory, persistent, priority-based)
//! - Configurable worker pools with dynamic scaling
//! - Job scheduling (cron, delayed, recurring)
//! - Retry strategies with exponential backoff
//! - Progress tracking and metrics
//! - Job pipelines and workflows
//! - Batch processing
//! - Rate limiting and throttling
//! - Job state persistence
//!
//! # Examples
//!
//! ## Basic Job Execution
//!
//! ```rust,no_run
//! use accuscene_jobs::prelude::*;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a queue and executor
//! let queue = Arc::new(MemoryQueue::new());
//! let executor = Arc::new(JobExecutor::new(RetryPolicy::exponential(3)));
//!
//! // Create and start worker pool
//! let pool_config = WorkerPoolConfig::default()
//!     .with_min_workers(2)
//!     .with_max_workers(10);
//! let pool = WorkerPool::new(pool_config, executor);
//! pool.start(queue.clone()).await?;
//!
//! // Queue a job
//! let job = Box::new(PhysicsSimulationJob::new(
//!     "scenario-1".to_string(),
//!     serde_json::json!({"velocity": 50}),
//! ));
//! queue.push(job).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Scheduled Jobs
//!
//! ```rust,no_run
//! use accuscene_jobs::prelude::*;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let queue = Arc::new(MemoryQueue::new());
//! let scheduler = Arc::new(CronScheduler::new(queue.clone()));
//!
//! // Schedule a job to run every hour
//! let job = Box::new(ReportGenerationJob::new(
//!     "daily_report".to_string(),
//!     "database".to_string(),
//!     serde_json::json!({}),
//! ));
//!
//! scheduler.schedule_cron(job, "0 * * * * *".to_string()).await?;
//! scheduler.clone().start();
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]

// Re-export accuscene-core
pub use accuscene_core;

// Public modules
pub mod batch;
pub mod error;
pub mod executor;
pub mod job;
pub mod metrics;
pub mod persistence;
pub mod pipeline;
pub mod progress;
pub mod queue;
pub mod result;
pub mod retry;
pub mod scheduler;
pub mod throttle;
pub mod worker;

/// Prelude module for convenient imports
pub mod prelude {
    //! Convenient re-exports of commonly used types

    pub use crate::batch::{BatchJobBuilder, BatchProcessor, BatchStrategy};
    pub use crate::error::{JobError, Result};
    pub use crate::executor::JobExecutor;
    pub use crate::job::{
        DataExportJob, Job, JobContext, JobMetadata, JobState, PhysicsSimulationJob,
        ReportGenerationJob,
    };
    pub use crate::metrics::{JobMetrics, JobStatistics, MetricsAggregator};
    pub use crate::persistence::{JobEvent, JobPersistence};
    pub use crate::pipeline::{JobPipeline, ParallelPipeline, PipelineStage, PipelineStatus};
    pub use crate::progress::{JobProgress, ProgressReporter, ProgressTracker};
    pub use crate::queue::{
        memory::MemoryQueue, persistent::PersistentQueue, priority::PriorityQueue, JobQueue,
        QueueConfig,
    };
    pub use crate::result::{BatchJobResult, JobResult, JobResultStatus};
    pub use crate::retry::{
        ExponentialBackoff, FibonacciBackoff, FixedRetry, LinearBackoff, RetryPolicy,
        RetryStrategy,
    };
    pub use crate::scheduler::{
        cron::CronScheduler, delayed::DelayedScheduler, recurring::RecurringScheduler, Schedule,
        ScheduledJob,
    };
    pub use crate::throttle::{JobThrottler, RateLimitConfig, RateLimiter, SlidingWindowLimiter};
    pub use crate::worker::{
        async_worker::AsyncWorker, pool::WorkerPool, pool::WorkerPoolConfig, thread::ThreadWorker,
        Worker, WorkerConfig, WorkerState,
    };
}

// Version information
/// Crate version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Crate name
pub const NAME: &str = env!("CARGO_PKG_NAME");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(VERSION, "0.1.5");
    }

    #[test]
    fn test_name() {
        assert_eq!(NAME, "accuscene-jobs");
    }
}
