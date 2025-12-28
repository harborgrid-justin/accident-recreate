//! Worker implementations for job execution.

pub mod async_worker;
pub mod pool;
pub mod thread;

use crate::error::Result;
use crate::job::Job;
use async_trait::async_trait;

/// Worker trait for job execution
#[async_trait]
pub trait Worker: Send + Sync {
    /// Get worker ID
    fn id(&self) -> &str;

    /// Execute a job
    async fn execute(&self, job: Box<dyn Job>) -> Result<()>;

    /// Check if worker is available
    fn is_available(&self) -> bool;

    /// Shutdown the worker
    async fn shutdown(&self) -> Result<()>;
}

/// Worker state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkerState {
    Idle,
    Busy,
    Shutdown,
}

/// Worker configuration
#[derive(Debug, Clone)]
pub struct WorkerConfig {
    pub id: String,
    pub max_concurrent_jobs: usize,
}

impl WorkerConfig {
    pub fn new(id: String) -> Self {
        Self {
            id,
            max_concurrent_jobs: 1,
        }
    }

    pub fn with_max_concurrent_jobs(mut self, max: usize) -> Self {
        self.max_concurrent_jobs = max;
        self
    }
}
