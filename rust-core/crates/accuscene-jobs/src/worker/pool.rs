//! Worker pool with dynamic sizing for job execution.

use crate::error::Result;
use crate::executor::JobExecutor;
use crate::job::{Job, JobContext};
use crate::queue::JobQueue;
use crate::worker::Worker;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::Semaphore;
use uuid::Uuid;

/// Worker pool configuration
#[derive(Debug, Clone)]
pub struct WorkerPoolConfig {
    pub min_workers: usize,
    pub max_workers: usize,
    pub scale_up_threshold: f64,
    pub scale_down_threshold: f64,
}

impl Default for WorkerPoolConfig {
    fn default() -> Self {
        Self {
            min_workers: 2,
            max_workers: 10,
            scale_up_threshold: 0.8,
            scale_down_threshold: 0.2,
        }
    }
}

impl WorkerPoolConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_min_workers(mut self, min: usize) -> Self {
        self.min_workers = min;
        self
    }

    pub fn with_max_workers(mut self, max: usize) -> Self {
        self.max_workers = max;
        self
    }
}

/// Worker pool for executing jobs
pub struct WorkerPool {
    config: WorkerPoolConfig,
    active_workers: Arc<AtomicUsize>,
    busy_workers: Arc<AtomicUsize>,
    running: Arc<AtomicBool>,
    semaphore: Arc<Semaphore>,
    executor: Arc<JobExecutor>,
}

impl WorkerPool {
    /// Create a new worker pool
    pub fn new(config: WorkerPoolConfig, executor: Arc<JobExecutor>) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_workers));

        Self {
            config,
            active_workers: Arc::new(AtomicUsize::new(0)),
            busy_workers: Arc::new(AtomicUsize::new(0)),
            running: Arc::new(AtomicBool::new(false)),
            semaphore,
            executor,
        }
    }

    /// Start the worker pool
    pub async fn start(&self, queue: Arc<dyn JobQueue>) -> Result<()> {
        self.running.store(true, Ordering::SeqCst);

        // Start minimum workers
        for _ in 0..self.config.min_workers {
            self.spawn_worker(queue.clone());
        }

        tracing::info!(
            min_workers = self.config.min_workers,
            max_workers = self.config.max_workers,
            "Worker pool started"
        );

        Ok(())
    }

    /// Spawn a new worker
    fn spawn_worker(&self, queue: Arc<dyn JobQueue>) {
        let worker_id = Uuid::new_v4().to_string();
        let active_workers = self.active_workers.clone();
        let busy_workers = self.busy_workers.clone();
        let running = self.running.clone();
        let semaphore = self.semaphore.clone();
        let executor = self.executor.clone();

        active_workers.fetch_add(1, Ordering::SeqCst);

        tokio::spawn(async move {
            tracing::debug!(worker_id = %worker_id, "Worker started");

            while running.load(Ordering::SeqCst) {
                // Acquire semaphore permit
                let permit = match semaphore.acquire().await {
                    Ok(p) => p,
                    Err(_) => break,
                };

                // Try to get a job from the queue
                let job = match queue.pop().await {
                    Ok(Some(job)) => job,
                    Ok(None) => {
                        drop(permit);
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                        continue;
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "Failed to pop job from queue");
                        drop(permit);
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                        continue;
                    }
                };

                let job_id = job.id().to_string();
                busy_workers.fetch_add(1, Ordering::SeqCst);

                tracing::info!(
                    worker_id = %worker_id,
                    job_id = %job_id,
                    "Worker executing job"
                );

                // Execute job
                let context = Arc::new(JobContext::new(job_id.clone(), worker_id.clone()));
                if let Err(e) = executor.execute(job, context).await {
                    tracing::error!(
                        worker_id = %worker_id,
                        job_id = %job_id,
                        error = %e,
                        "Job execution failed"
                    );
                }

                busy_workers.fetch_sub(1, Ordering::SeqCst);
                drop(permit);
            }

            active_workers.fetch_sub(1, Ordering::SeqCst);
            tracing::debug!(worker_id = %worker_id, "Worker stopped");
        });
    }

    /// Get number of active workers
    pub fn active_workers(&self) -> usize {
        self.active_workers.load(Ordering::SeqCst)
    }

    /// Get number of busy workers
    pub fn busy_workers(&self) -> usize {
        self.busy_workers.load(Ordering::SeqCst)
    }

    /// Get number of idle workers
    pub fn idle_workers(&self) -> usize {
        self.active_workers().saturating_sub(self.busy_workers())
    }

    /// Get pool utilization (0.0 to 1.0)
    pub fn utilization(&self) -> f64 {
        let active = self.active_workers();
        if active == 0 {
            0.0
        } else {
            self.busy_workers() as f64 / active as f64
        }
    }

    /// Check if pool should scale up
    pub fn should_scale_up(&self) -> bool {
        let active = self.active_workers();
        active < self.config.max_workers && self.utilization() > self.config.scale_up_threshold
    }

    /// Check if pool should scale down
    pub fn should_scale_down(&self) -> bool {
        let active = self.active_workers();
        active > self.config.min_workers && self.utilization() < self.config.scale_down_threshold
    }

    /// Scale up the pool
    pub fn scale_up(&self, queue: Arc<dyn JobQueue>) {
        let current = self.active_workers();
        if current < self.config.max_workers {
            self.spawn_worker(queue);
            tracing::info!(
                workers = current + 1,
                "Scaled up worker pool"
            );
        }
    }

    /// Shutdown the worker pool
    pub async fn shutdown(&self) -> Result<()> {
        self.running.store(false, Ordering::SeqCst);

        // Wait for workers to finish
        while self.active_workers() > 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        tracing::info!("Worker pool shut down");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::job::PhysicsSimulationJob;
    use crate::queue::memory::MemoryQueue;
    use crate::retry::RetryPolicy;

    #[tokio::test]
    async fn test_worker_pool_basic() {
        let queue = Arc::new(MemoryQueue::new());
        let executor = Arc::new(JobExecutor::new(RetryPolicy::none()));
        let config = WorkerPoolConfig::default().with_min_workers(2).with_max_workers(5);
        let pool = WorkerPool::new(config, executor);

        pool.start(queue.clone()).await.unwrap();

        // Give workers time to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        assert_eq!(pool.active_workers(), 2);

        pool.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_worker_pool_utilization() {
        let queue = Arc::new(MemoryQueue::new());
        let executor = Arc::new(JobExecutor::new(RetryPolicy::none()));
        let config = WorkerPoolConfig::default().with_min_workers(2).with_max_workers(5);
        let pool = WorkerPool::new(config, executor);

        assert_eq!(pool.utilization(), 0.0);

        pool.start(queue.clone()).await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        pool.shutdown().await.unwrap();
    }
}
