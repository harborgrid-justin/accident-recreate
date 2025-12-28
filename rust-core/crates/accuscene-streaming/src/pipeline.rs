//! Pipeline builder for constructing streaming pipelines.

use crate::backpressure::BackpressureController;
use crate::checkpoint::CheckpointCoordinator;
use crate::config::StreamingConfig;
use crate::error::{Result, StreamingError};
use crate::runtime::StreamingRuntime;
use crate::sink::Sink;
use crate::source::Source;
use crate::state::StateContext;
use crate::stream::DataStream;
use crate::watermark::WatermarkTracker;
use std::sync::Arc;
use tokio::task::JoinHandle;

/// Pipeline builder for creating streaming pipelines
pub struct PipelineBuilder {
    config: StreamingConfig,
    name: String,
}

impl PipelineBuilder {
    /// Create a new pipeline builder
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            config: StreamingConfig::default(),
            name: name.into(),
        }
    }

    /// Set configuration
    pub fn with_config(mut self, config: StreamingConfig) -> Self {
        self.config = config;
        self
    }

    /// Build the pipeline
    pub fn build(self) -> Pipeline {
        Pipeline {
            name: self.name,
            config: self.config,
            runtime: None,
        }
    }
}

/// Streaming pipeline
pub struct Pipeline {
    name: String,
    config: StreamingConfig,
    runtime: Option<StreamingRuntime>,
}

impl Pipeline {
    /// Create a new pipeline
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            config: StreamingConfig::default(),
            runtime: None,
        }
    }

    /// Get pipeline name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get configuration
    pub fn config(&self) -> &StreamingConfig {
        &self.config
    }

    /// Initialize the pipeline
    pub async fn init(&mut self) -> Result<()> {
        let runtime = StreamingRuntime::new(self.config.clone()).await?;
        self.runtime = Some(runtime);
        Ok(())
    }

    /// Execute a source-to-sink pipeline
    pub async fn execute<S, Snk>(
        &mut self,
        mut source: S,
        mut sink: Snk,
    ) -> Result<()>
    where
        S: Source,
        Snk: Sink<S::Item>,
    {
        if self.runtime.is_none() {
            self.init().await?;
        }

        source.start().await?;

        while let Some(item) = source.next().await? {
            sink.write(item).await?;
        }

        sink.flush().await?;
        sink.close().await?;
        source.stop().await?;

        Ok(())
    }

    /// Execute a stream processing function
    pub async fn execute_stream<S, F, Fut>(&mut self, mut source: S, process_fn: F) -> Result<()>
    where
        S: Source,
        F: Fn(S::Item) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<()>> + Send + 'static,
    {
        if self.runtime.is_none() {
            self.init().await?;
        }

        source.start().await?;

        while let Some(item) = source.next().await? {
            process_fn(item).await?;
        }

        source.stop().await?;

        Ok(())
    }

    /// Shutdown the pipeline
    pub async fn shutdown(mut self) -> Result<()> {
        if let Some(runtime) = self.runtime.take() {
            runtime.shutdown().await?;
        }
        Ok(())
    }
}

/// Pipeline job that can be run in the background
pub struct PipelineJob<S, Snk>
where
    S: Source,
    Snk: Sink<S::Item>,
{
    source: S,
    sink: Snk,
    handle: Option<JoinHandle<Result<()>>>,
}

impl<S, Snk> PipelineJob<S, Snk>
where
    S: Source + Send + 'static,
    Snk: Sink<S::Item> + Send + 'static,
    S::Item: Send + 'static,
{
    /// Create a new pipeline job
    pub fn new(source: S, sink: Snk) -> Self {
        Self {
            source,
            sink,
            handle: None,
        }
    }

    /// Start the job
    pub fn start(&mut self) -> Result<()> {
        if self.handle.is_some() {
            return Err(StreamingError::Pipeline("Job already started".to_string()));
        }

        let mut source = self.source.clone();
        let mut sink = self.sink.clone();

        let handle = tokio::spawn(async move {
            source.start().await?;

            while let Some(item) = source.next().await? {
                sink.write(item).await?;
            }

            sink.flush().await?;
            sink.close().await?;
            source.stop().await?;

            Ok(())
        });

        self.handle = Some(handle);
        Ok(())
    }

    /// Wait for the job to complete
    pub async fn wait(self) -> Result<()> {
        if let Some(handle) = self.handle {
            handle.await.map_err(|e| {
                StreamingError::Pipeline(format!("Job execution error: {}", e))
            })??;
        }
        Ok(())
    }
}

/// Multi-stage pipeline that can chain multiple processing stages
pub struct MultiStagePipeline {
    name: String,
    config: StreamingConfig,
    stages: Vec<Box<dyn Fn() -> Result<()> + Send + Sync>>,
}

impl MultiStagePipeline {
    /// Create a new multi-stage pipeline
    pub fn new(name: impl Into<String>, config: StreamingConfig) -> Self {
        Self {
            name: name.into(),
            config,
            stages: Vec::new(),
        }
    }

    /// Add a stage to the pipeline
    pub fn add_stage<F>(&mut self, stage: F)
    where
        F: Fn() -> Result<()> + Send + Sync + 'static,
    {
        self.stages.push(Box::new(stage));
    }

    /// Execute all stages
    pub async fn execute(&self) -> Result<()> {
        for (i, stage) in self.stages.iter().enumerate() {
            tracing::info!("Executing stage {} of {}", i + 1, self.stages.len());
            stage()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sink::channel::ChannelSink;
    use crate::source::iterator::RangeSource;

    #[tokio::test]
    async fn test_pipeline() {
        let source = RangeSource::new(0, 5);
        let (sink, mut rx) = ChannelSink::create();

        let mut pipeline = Pipeline::new("test");
        pipeline.execute(source, sink).await.unwrap();

        let mut results = Vec::new();
        while let Some(item) = rx.recv().await {
            results.push(item);
        }

        assert_eq!(results, vec![0, 1, 2, 3, 4]);
    }
}
