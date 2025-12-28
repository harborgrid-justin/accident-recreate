//! Sink connectors for streaming data.

pub mod channel;
pub mod file;
pub mod parquet;
pub mod websocket;

use crate::error::Result;
use async_trait::async_trait;

/// Trait for data sinks
#[async_trait]
pub trait Sink<T>: Send + 'static {
    /// Write an item to the sink
    async fn write(&mut self, item: T) -> Result<()>;

    /// Flush any buffered data
    async fn flush(&mut self) -> Result<()>;

    /// Close the sink
    async fn close(&mut self) -> Result<()>;
}

pub use self::channel::ChannelSink;
pub use self::file::FileSink;
pub use self::parquet::ParquetSink;
pub use self::websocket::WebSocketSink;
