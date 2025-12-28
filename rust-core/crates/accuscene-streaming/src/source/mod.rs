//! Source connectors for streaming data.

pub mod channel;
pub mod file;
pub mod iterator;
pub mod websocket;

use crate::error::Result;
use crate::stream::DataStream;
use async_trait::async_trait;

/// Trait for data sources
#[async_trait]
pub trait Source: DataStream {
    /// Start the source
    async fn start(&mut self) -> Result<()>;

    /// Stop the source
    async fn stop(&mut self) -> Result<()>;

    /// Check if the source is running
    fn is_running(&self) -> bool;
}

pub use self::channel::ChannelSource;
pub use self::file::FileSource;
pub use self::iterator::IteratorSource;
pub use self::websocket::WebSocketSource;
