//! Streaming runtime for managing execution.

use crate::backpressure::BackpressureController;
use crate::checkpoint::{CheckpointCoordinator, CheckpointStorage, MemoryCheckpointStorage};
use crate::config::StreamingConfig;
use crate::error::{Result, StreamingError};
use crate::state::{MemoryStateBackend, StateContext};
use crate::watermark::WatermarkTracker;
use std::sync::Arc;
use tokio::runtime::Runtime as TokioRuntime;
use tokio::task::JoinHandle;
use tracing::{debug, info};

/// Streaming runtime
pub struct StreamingRuntime {
    config: StreamingConfig,
    state_context: Arc<StateContext>,
    checkpoint_coordinator: Arc<CheckpointCoordinator>,
    watermark_tracker: Arc<WatermarkTracker>,
    backpressure_controller: Option<Arc<BackpressureController>>,
    checkpoint_task: Option<JoinHandle<()>>,
}

impl StreamingRuntime {
    /// Create a new streaming runtime
    pub async fn new(config: StreamingConfig) -> Result<Self> {
        info!("Initializing streaming runtime");

        // Create state context
        let state_backend = Arc::new(MemoryStateBackend::new());
        let state_context = Arc::new(StateContext::new(state_backend));

        // Create checkpoint coordinator
        let checkpoint_storage: Arc<dyn CheckpointStorage> =
            Arc::new(MemoryCheckpointStorage::new());
        let checkpoint_coordinator = Arc::new(CheckpointCoordinator::new(
            checkpoint_storage,
            config.checkpoint.max_retained,
        ));

        // Create watermark tracker
        let watermark_tracker = Arc::new(WatermarkTracker::new());

        // Create backpressure controller if configured
        let backpressure_controller = if config.buffer.bounded {
            Some(Arc::new(BackpressureController::new(
                config.buffer.default_capacity,
                config.backpressure.clone(),
            )))
        } else {
            None
        };

        let mut runtime = Self {
            config,
            state_context,
            checkpoint_coordinator,
            watermark_tracker,
            backpressure_controller,
            checkpoint_task: None,
        };

        // Start checkpoint task if enabled
        if runtime.config.checkpoint.enabled {
            runtime.start_checkpoint_task().await?;
        }

        info!("Streaming runtime initialized");
        Ok(runtime)
    }

    /// Start periodic checkpointing
    async fn start_checkpoint_task(&mut self) -> Result<()> {
        let coordinator = self.checkpoint_coordinator.clone();
        let interval = self.config.checkpoint.interval;

        let handle = tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            loop {
                interval_timer.tick().await;

                match coordinator.trigger_checkpoint().await {
                    Ok(id) => {
                        debug!("Checkpoint {} triggered", id);
                        // In a real implementation, we would collect state and complete the checkpoint
                    }
                    Err(e) => {
                        tracing::error!("Failed to trigger checkpoint: {}", e);
                    }
                }
            }
        });

        self.checkpoint_task = Some(handle);
        Ok(())
    }

    /// Get state context
    pub fn state_context(&self) -> Arc<StateContext> {
        self.state_context.clone()
    }

    /// Get checkpoint coordinator
    pub fn checkpoint_coordinator(&self) -> Arc<CheckpointCoordinator> {
        self.checkpoint_coordinator.clone()
    }

    /// Get watermark tracker
    pub fn watermark_tracker(&self) -> Arc<WatermarkTracker> {
        self.watermark_tracker.clone()
    }

    /// Get backpressure controller
    pub fn backpressure_controller(&self) -> Option<Arc<BackpressureController>> {
        self.backpressure_controller.clone()
    }

    /// Shutdown the runtime
    pub async fn shutdown(self) -> Result<()> {
        info!("Shutting down streaming runtime");

        // Stop checkpoint task
        if let Some(handle) = self.checkpoint_task {
            handle.abort();
        }

        info!("Streaming runtime shutdown complete");
        Ok(())
    }
}

/// Metrics for the streaming runtime
#[derive(Debug, Clone, Default)]
pub struct RuntimeMetrics {
    /// Number of items processed
    pub items_processed: u64,
    /// Number of errors
    pub errors: u64,
    /// Current backpressure level
    pub backpressure_level: f64,
    /// Number of checkpoints completed
    pub checkpoints_completed: u64,
}

impl RuntimeMetrics {
    /// Create new metrics
    pub fn new() -> Self {
        Self::default()
    }

    /// Record item processed
    pub fn record_item(&mut self) {
        self.items_processed += 1;
    }

    /// Record error
    pub fn record_error(&mut self) {
        self.errors += 1;
    }

    /// Update backpressure level
    pub fn update_backpressure(&mut self, level: f64) {
        self.backpressure_level = level;
    }

    /// Record checkpoint completion
    pub fn record_checkpoint(&mut self) {
        self.checkpoints_completed += 1;
    }
}

/// Runtime builder for custom runtime configuration
pub struct RuntimeBuilder {
    config: StreamingConfig,
}

impl RuntimeBuilder {
    /// Create a new runtime builder
    pub fn new() -> Self {
        Self {
            config: StreamingConfig::default(),
        }
    }

    /// Set configuration
    pub fn with_config(mut self, config: StreamingConfig) -> Self {
        self.config = config;
        self
    }

    /// Enable checkpointing
    pub fn with_checkpointing(mut self, enabled: bool) -> Self {
        self.config.checkpoint.enabled = enabled;
        self
    }

    /// Set checkpoint interval
    pub fn with_checkpoint_interval(mut self, interval: std::time::Duration) -> Self {
        self.config.checkpoint.interval = interval;
        self
    }

    /// Set buffer capacity
    pub fn with_buffer_capacity(mut self, capacity: usize) -> Self {
        self.config.buffer.default_capacity = capacity;
        self
    }

    /// Build the runtime
    pub async fn build(self) -> Result<StreamingRuntime> {
        StreamingRuntime::new(self.config).await
    }
}

impl Default for RuntimeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_runtime_creation() {
        let runtime = StreamingRuntime::new(StreamingConfig::default())
            .await
            .unwrap();

        assert!(runtime.state_context().backend().get(b"test").await.unwrap().is_none());

        runtime.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_runtime_builder() {
        let runtime = RuntimeBuilder::new()
            .with_checkpointing(true)
            .with_buffer_capacity(1000)
            .build()
            .await
            .unwrap();

        runtime.shutdown().await.unwrap();
    }
}
