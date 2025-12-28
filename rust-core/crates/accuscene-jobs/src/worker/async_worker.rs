//! Async worker implementation using tokio.

use crate::error::Result;
use crate::executor::JobExecutor;
use crate::job::{Job, JobContext};
use crate::worker::{Worker, WorkerConfig, WorkerState};
use async_trait::async_trait;
use parking_lot::RwLock;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Async worker using tokio
pub struct AsyncWorker {
    config: WorkerConfig,
    state: Arc<RwLock<WorkerState>>,
    executor: Arc<JobExecutor>,
    shutdown_tx: Arc<RwLock<Option<mpsc::Sender<()>>>>,
}

impl AsyncWorker {
    /// Create a new async worker
    pub fn new(config: WorkerConfig, executor: Arc<JobExecutor>) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(WorkerState::Idle)),
            executor,
            shutdown_tx: Arc::new(RwLock::new(None)),
        }
    }

    /// Start the worker
    pub async fn start(&self) -> Result<()> {
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        *self.shutdown_tx.write() = Some(shutdown_tx);

        let worker_id = self.config.id.clone();
        let state = self.state.clone();

        tokio::spawn(async move {
            *state.write() = WorkerState::Idle;
            tracing::debug!(worker_id = %worker_id, "Async worker started");

            shutdown_rx.recv().await;

            *state.write() = WorkerState::Shutdown;
            tracing::debug!(worker_id = %worker_id, "Async worker stopped");
        });

        Ok(())
    }

    /// Get worker state
    pub fn state(&self) -> WorkerState {
        *self.state.read()
    }
}

#[async_trait]
impl Worker for AsyncWorker {
    fn id(&self) -> &str {
        &self.config.id
    }

    async fn execute(&self, mut job: Box<dyn Job>) -> Result<()> {
        *self.state.write() = WorkerState::Busy;

        let job_id = job.id().to_string();
        let context = Arc::new(JobContext::new(job_id, self.config.id.clone()));

        tracing::info!(
            worker_id = %self.config.id,
            job_id = %context.job_id,
            "Executing job"
        );

        let result = self.executor.execute(job, context).await;

        *self.state.write() = WorkerState::Idle;

        result.map(|_| ())
    }

    fn is_available(&self) -> bool {
        matches!(*self.state.read(), WorkerState::Idle)
    }

    async fn shutdown(&self) -> Result<()> {
        if let Some(tx) = self.shutdown_tx.write().take() {
            let _ = tx.send(()).await;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::job::PhysicsSimulationJob;
    use crate::retry::RetryPolicy;

    #[tokio::test]
    async fn test_async_worker() {
        let config = WorkerConfig::new("worker-1".to_string());
        let executor = Arc::new(JobExecutor::new(RetryPolicy::none()));
        let worker = AsyncWorker::new(config, executor);

        worker.start().await.unwrap();
        assert_eq!(worker.state(), WorkerState::Idle);

        let job = Box::new(PhysicsSimulationJob::new(
            "test-scenario".to_string(),
            serde_json::json!({}),
        ));

        worker.execute(job).await.unwrap();
        assert_eq!(worker.state(), WorkerState::Idle);

        worker.shutdown().await.unwrap();
    }
}
