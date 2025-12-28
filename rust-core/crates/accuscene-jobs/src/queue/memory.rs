//! In-memory job queue implementation.

use crate::error::{JobError, Result};
use crate::job::Job;
use crate::queue::{JobQueue, QueueConfig};
use async_trait::async_trait;
use crossbeam_queue::SegQueue;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// In-memory job queue using crossbeam
#[derive(Clone)]
pub struct MemoryQueue {
    queue: Arc<SegQueue<Box<dyn Job>>>,
    jobs: Arc<RwLock<HashMap<String, Box<dyn Job>>>>,
    config: QueueConfig,
}

impl MemoryQueue {
    /// Create a new memory queue
    pub fn new() -> Self {
        Self::with_config(QueueConfig::default())
    }

    /// Create a new memory queue with configuration
    pub fn with_config(config: QueueConfig) -> Self {
        Self {
            queue: Arc::new(SegQueue::new()),
            jobs: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Get current queue size
    fn current_size(&self) -> usize {
        self.jobs.read().len()
    }

    /// Check if queue is at capacity
    fn is_at_capacity(&self) -> bool {
        if let Some(max_size) = self.config.max_size {
            self.current_size() >= max_size
        } else {
            false
        }
    }
}

impl Default for MemoryQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl JobQueue for MemoryQueue {
    async fn push(&self, job: Box<dyn Job>) -> Result<()> {
        if self.is_at_capacity() {
            return Err(JobError::QueueFull {
                capacity: self.config.max_size.unwrap(),
            });
        }

        let job_id = job.id().to_string();

        // Check if job already exists
        if self.jobs.read().contains_key(&job_id) {
            return Err(JobError::AlreadyExists { job_id });
        }

        // Store job for lookup
        let job_clone = unsafe {
            // SAFETY: We're cloning the job data through serialization
            let serialized = job.serialize()?;
            let deserialized: Box<dyn Job> = serde_json::from_str(&serialized)
                .map_err(|e| JobError::SerializationError(e))?;
            deserialized
        };

        self.jobs.write().insert(job_id.clone(), job_clone);
        self.queue.push(job);

        tracing::debug!(job_id = %job_id, "Job pushed to memory queue");
        Ok(())
    }

    async fn pop(&self) -> Result<Option<Box<dyn Job>>> {
        if let Some(job) = self.queue.pop() {
            let job_id = job.id().to_string();
            self.jobs.write().remove(&job_id);
            tracing::debug!(job_id = %job_id, "Job popped from memory queue");
            Ok(Some(job))
        } else {
            Ok(None)
        }
    }

    async fn peek(&self) -> Result<Option<Box<dyn Job>>> {
        // Note: SegQueue doesn't support peek, so we need to pop and re-push
        if let Some(job) = self.queue.pop() {
            let job_id = job.id().to_string();

            // Re-serialize to create a copy
            let serialized = job.serialize()?;
            let job_copy: Box<dyn Job> = serde_json::from_str(&serialized)?;

            // Put it back
            self.queue.push(job);

            Ok(Some(job_copy))
        } else {
            Ok(None)
        }
    }

    async fn len(&self) -> Result<usize> {
        Ok(self.current_size())
    }

    async fn clear(&self) -> Result<()> {
        while self.queue.pop().is_some() {}
        self.jobs.write().clear();
        tracing::info!("Memory queue cleared");
        Ok(())
    }

    async fn get(&self, job_id: &str) -> Result<Option<Box<dyn Job>>> {
        let jobs = self.jobs.read();
        if let Some(job) = jobs.get(job_id) {
            // Clone through serialization
            let serialized = job.serialize()?;
            let job_copy: Box<dyn Job> = serde_json::from_str(&serialized)?;
            Ok(Some(job_copy))
        } else {
            Ok(None)
        }
    }

    async fn remove(&self, job_id: &str) -> Result<bool> {
        // Remove from lookup map
        let removed = self.jobs.write().remove(job_id).is_some();

        // Note: We can't efficiently remove from SegQueue without popping all items
        // In production, you'd want to use a different data structure or mark as deleted

        if removed {
            tracing::debug!(job_id = %job_id, "Job removed from memory queue");
        }

        Ok(removed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::job::{JobContext, PhysicsSimulationJob};

    #[tokio::test]
    async fn test_memory_queue_push_pop() {
        let queue = MemoryQueue::new();
        let job = Box::new(PhysicsSimulationJob::new(
            "test-scenario".to_string(),
            serde_json::json!({}),
        ));
        let job_id = job.id().to_string();

        queue.push(job).await.unwrap();
        assert_eq!(queue.len().await.unwrap(), 1);

        let popped = queue.pop().await.unwrap();
        assert!(popped.is_some());
        assert_eq!(popped.unwrap().id(), job_id);
        assert_eq!(queue.len().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_memory_queue_capacity() {
        let config = QueueConfig::new().with_max_size(2);
        let queue = MemoryQueue::with_config(config);

        let job1 = Box::new(PhysicsSimulationJob::new(
            "scenario-1".to_string(),
            serde_json::json!({}),
        ));
        let job2 = Box::new(PhysicsSimulationJob::new(
            "scenario-2".to_string(),
            serde_json::json!({}),
        ));
        let job3 = Box::new(PhysicsSimulationJob::new(
            "scenario-3".to_string(),
            serde_json::json!({}),
        ));

        queue.push(job1).await.unwrap();
        queue.push(job2).await.unwrap();

        let result = queue.push(job3).await;
        assert!(matches!(result, Err(JobError::QueueFull { .. })));
    }

    #[tokio::test]
    async fn test_memory_queue_get() {
        let queue = MemoryQueue::new();
        let job = Box::new(PhysicsSimulationJob::new(
            "test-scenario".to_string(),
            serde_json::json!({}),
        ));
        let job_id = job.id().to_string();

        queue.push(job).await.unwrap();

        let retrieved = queue.get(&job_id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id(), job_id);
    }
}
