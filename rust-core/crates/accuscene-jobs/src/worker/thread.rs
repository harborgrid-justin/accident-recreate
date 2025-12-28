//! Thread-based worker implementation.

use crate::error::Result;
use crate::executor::JobExecutor;
use crate::job::{Job, JobContext};
use crate::worker::{Worker, WorkerConfig, WorkerState};
use async_trait::async_trait;
use crossbeam_channel::{bounded, Sender};
use parking_lot::RwLock;
use std::sync::Arc;
use std::thread;

/// Thread-based worker
pub struct ThreadWorker {
    config: WorkerConfig,
    state: Arc<RwLock<WorkerState>>,
    executor: Arc<JobExecutor>,
    job_tx: Arc<RwLock<Option<Sender<Box<dyn Job>>>>>,
}

impl ThreadWorker {
    /// Create a new thread worker
    pub fn new(config: WorkerConfig, executor: Arc<JobExecutor>) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(WorkerState::Idle)),
            executor,
            job_tx: Arc::new(RwLock::new(None)),
        }
    }

    /// Start the worker thread
    pub fn start(&self) -> Result<()> {
        let (job_tx, job_rx) = bounded::<Box<dyn Job>>(10);
        *self.job_tx.write() = Some(job_tx);

        let worker_id = self.config.id.clone();
        let state = self.state.clone();
        let executor = self.executor.clone();

        thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            *state.write() = WorkerState::Idle;
            tracing::debug!(worker_id = %worker_id, "Thread worker started");

            while let Ok(mut job) = job_rx.recv() {
                *state.write() = WorkerState::Busy;

                let job_id = job.id().to_string();
                let context = Arc::new(JobContext::new(job_id, worker_id.clone()));

                tracing::info!(
                    worker_id = %worker_id,
                    job_id = %context.job_id,
                    "Thread worker executing job"
                );

                if let Err(e) = runtime.block_on(executor.execute(job, context)) {
                    tracing::error!(
                        worker_id = %worker_id,
                        error = %e,
                        "Thread worker job execution failed"
                    );
                }

                *state.write() = WorkerState::Idle;
            }

            *state.write() = WorkerState::Shutdown;
            tracing::debug!(worker_id = %worker_id, "Thread worker stopped");
        });

        Ok(())
    }

    /// Get worker state
    pub fn state(&self) -> WorkerState {
        *self.state.read()
    }
}

#[async_trait]
impl Worker for ThreadWorker {
    fn id(&self) -> &str {
        &self.config.id
    }

    async fn execute(&self, job: Box<dyn Job>) -> Result<()> {
        if let Some(tx) = self.job_tx.read().as_ref() {
            tx.send(job).map_err(|_| {
                crate::error::JobError::WorkerPoolError("Failed to send job to worker".to_string())
            })?;
        }
        Ok(())
    }

    fn is_available(&self) -> bool {
        matches!(*self.state.read(), WorkerState::Idle)
    }

    async fn shutdown(&self) -> Result<()> {
        drop(self.job_tx.write().take());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::job::PhysicsSimulationJob;
    use crate::retry::RetryPolicy;

    #[tokio::test]
    async fn test_thread_worker() {
        let config = WorkerConfig::new("worker-1".to_string());
        let executor = Arc::new(JobExecutor::new(RetryPolicy::none()));
        let worker = ThreadWorker::new(config, executor);

        worker.start().unwrap();

        // Give the thread time to start
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        assert_eq!(worker.state(), WorkerState::Idle);

        let job = Box::new(PhysicsSimulationJob::new(
            "test-scenario".to_string(),
            serde_json::json!({}),
        ));

        worker.execute(job).await.unwrap();

        // Give the thread time to process
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        worker.shutdown().await.unwrap();
    }
}
