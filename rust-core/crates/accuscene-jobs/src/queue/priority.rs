//! Priority-based job queue implementation.

use crate::error::{JobError, Result};
use crate::job::Job;
use crate::queue::{JobQueue, QueueConfig};
use async_trait::async_trait;
use parking_lot::RwLock;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::sync::Arc;

/// Priority queue entry
#[derive(Debug)]
struct PriorityJob {
    job: Box<dyn Job>,
    priority: i32,
    sequence: u64,
}

impl PartialEq for PriorityJob {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.sequence == other.sequence
    }
}

impl Eq for PriorityJob {}

impl PartialOrd for PriorityJob {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PriorityJob {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority comes first
        // If priorities are equal, earlier sequence comes first
        match self.priority.cmp(&other.priority) {
            Ordering::Equal => other.sequence.cmp(&self.sequence),
            other => other,
        }
    }
}

/// Priority-based job queue
#[derive(Clone)]
pub struct PriorityQueue {
    heap: Arc<RwLock<BinaryHeap<PriorityJob>>>,
    jobs: Arc<RwLock<HashMap<String, Box<dyn Job>>>>,
    sequence: Arc<RwLock<u64>>,
    config: QueueConfig,
}

impl PriorityQueue {
    /// Create a new priority queue
    pub fn new() -> Self {
        Self::with_config(QueueConfig::default().with_priority())
    }

    /// Create a new priority queue with configuration
    pub fn with_config(config: QueueConfig) -> Self {
        Self {
            heap: Arc::new(RwLock::new(BinaryHeap::new())),
            jobs: Arc::new(RwLock::new(HashMap::new())),
            sequence: Arc::new(RwLock::new(0)),
            config,
        }
    }

    /// Get next sequence number
    fn next_sequence(&self) -> u64 {
        let mut seq = self.sequence.write();
        *seq += 1;
        *seq
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

impl Default for PriorityQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl JobQueue for PriorityQueue {
    async fn push(&self, job: Box<dyn Job>) -> Result<()> {
        if self.is_at_capacity() {
            return Err(JobError::QueueFull {
                capacity: self.config.max_size.unwrap(),
            });
        }

        let job_id = job.id().to_string();
        let priority = job.priority();

        // Check if job already exists
        if self.jobs.read().contains_key(&job_id) {
            return Err(JobError::AlreadyExists { job_id });
        }

        // Clone job for lookup map
        let serialized = job.serialize()?;
        let job_clone: Box<dyn Job> = serde_json::from_str(&serialized)?;

        self.jobs.write().insert(job_id.clone(), job_clone);

        let sequence = self.next_sequence();
        self.heap.write().push(PriorityJob {
            job,
            priority,
            sequence,
        });

        tracing::debug!(
            job_id = %job_id,
            priority = priority,
            sequence = sequence,
            "Job pushed to priority queue"
        );

        Ok(())
    }

    async fn pop(&self) -> Result<Option<Box<dyn Job>>> {
        if let Some(priority_job) = self.heap.write().pop() {
            let job_id = priority_job.job.id().to_string();
            self.jobs.write().remove(&job_id);

            tracing::debug!(
                job_id = %job_id,
                priority = priority_job.priority,
                "Job popped from priority queue"
            );

            Ok(Some(priority_job.job))
        } else {
            Ok(None)
        }
    }

    async fn peek(&self) -> Result<Option<Box<dyn Job>>> {
        if let Some(priority_job) = self.heap.read().peek() {
            let serialized = priority_job.job.serialize()?;
            let job_copy: Box<dyn Job> = serde_json::from_str(&serialized)?;
            Ok(Some(job_copy))
        } else {
            Ok(None)
        }
    }

    async fn len(&self) -> Result<usize> {
        Ok(self.current_size())
    }

    async fn clear(&self) -> Result<()> {
        self.heap.write().clear();
        self.jobs.write().clear();
        *self.sequence.write() = 0;
        tracing::info!("Priority queue cleared");
        Ok(())
    }

    async fn get(&self, job_id: &str) -> Result<Option<Box<dyn Job>>> {
        let jobs = self.jobs.read();
        if let Some(job) = jobs.get(job_id) {
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

        // Note: BinaryHeap doesn't support efficient removal by value
        // In production, you'd want to either:
        // 1. Use a custom heap implementation
        // 2. Mark jobs as deleted and skip them on pop
        // 3. Periodically rebuild the heap

        if removed {
            tracing::debug!(job_id = %job_id, "Job removed from priority queue (lazy)");
        }

        Ok(removed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::job::PhysicsSimulationJob;

    #[tokio::test]
    async fn test_priority_queue_ordering() {
        let queue = PriorityQueue::new();

        let mut job1 = PhysicsSimulationJob::new("scenario-1".to_string(), serde_json::json!({}));
        job1.metadata.priority = 1;

        let mut job2 = PhysicsSimulationJob::new("scenario-2".to_string(), serde_json::json!({}));
        job2.metadata.priority = 10;

        let mut job3 = PhysicsSimulationJob::new("scenario-3".to_string(), serde_json::json!({}));
        job3.metadata.priority = 5;

        let job2_id = job2.id().to_string();
        let job3_id = job3.id().to_string();
        let job1_id = job1.id().to_string();

        queue.push(Box::new(job1)).await.unwrap();
        queue.push(Box::new(job2)).await.unwrap();
        queue.push(Box::new(job3)).await.unwrap();

        // Should pop in priority order: 10, 5, 1
        assert_eq!(queue.pop().await.unwrap().unwrap().id(), job2_id);
        assert_eq!(queue.pop().await.unwrap().unwrap().id(), job3_id);
        assert_eq!(queue.pop().await.unwrap().unwrap().id(), job1_id);
    }

    #[tokio::test]
    async fn test_priority_queue_fifo_same_priority() {
        let queue = PriorityQueue::new();

        let job1 = PhysicsSimulationJob::new("scenario-1".to_string(), serde_json::json!({}));
        let job2 = PhysicsSimulationJob::new("scenario-2".to_string(), serde_json::json!({}));

        let job1_id = job1.id().to_string();
        let job2_id = job2.id().to_string();

        queue.push(Box::new(job1)).await.unwrap();
        queue.push(Box::new(job2)).await.unwrap();

        // Should maintain FIFO order for same priority
        assert_eq!(queue.pop().await.unwrap().unwrap().id(), job1_id);
        assert_eq!(queue.pop().await.unwrap().unwrap().id(), job2_id);
    }

    #[tokio::test]
    async fn test_priority_queue_capacity() {
        let config = QueueConfig::new().with_priority().with_max_size(2);
        let queue = PriorityQueue::with_config(config);

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
}
