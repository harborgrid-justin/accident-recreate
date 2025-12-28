//! Job queue implementations for AccuScene.

pub mod memory;
pub mod persistent;
pub mod priority;

use crate::error::Result;
use crate::job::Job;
use async_trait::async_trait;

/// Job queue trait
#[async_trait]
pub trait JobQueue: Send + Sync {
    /// Push a job to the queue
    async fn push(&self, job: Box<dyn Job>) -> Result<()>;

    /// Pop a job from the queue
    async fn pop(&self) -> Result<Option<Box<dyn Job>>>;

    /// Peek at the next job without removing it
    async fn peek(&self) -> Result<Option<Box<dyn Job>>>;

    /// Get the number of jobs in the queue
    async fn len(&self) -> Result<usize>;

    /// Check if the queue is empty
    async fn is_empty(&self) -> Result<bool> {
        Ok(self.len().await? == 0)
    }

    /// Clear all jobs from the queue
    async fn clear(&self) -> Result<()>;

    /// Get job by ID
    async fn get(&self, job_id: &str) -> Result<Option<Box<dyn Job>>>;

    /// Remove job by ID
    async fn remove(&self, job_id: &str) -> Result<bool>;
}

/// Queue configuration
#[derive(Debug, Clone)]
pub struct QueueConfig {
    pub max_size: Option<usize>,
    pub persistence_enabled: bool,
    pub priority_enabled: bool,
}

impl Default for QueueConfig {
    fn default() -> Self {
        Self {
            max_size: None,
            persistence_enabled: false,
            priority_enabled: false,
        }
    }
}

impl QueueConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_size(mut self, max_size: usize) -> Self {
        self.max_size = Some(max_size);
        self
    }

    pub fn with_persistence(mut self) -> Self {
        self.persistence_enabled = true;
        self
    }

    pub fn with_priority(mut self) -> Self {
        self.priority_enabled = true;
        self
    }
}
