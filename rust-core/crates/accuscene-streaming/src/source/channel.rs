//! Channel-based streaming source.

use crate::error::{Result, StreamingError};
use crate::source::Source;
use crate::stream::DataStream;
use async_trait::async_trait;
use tokio::sync::mpsc;

/// Channel source that reads from a tokio channel
pub struct ChannelSource<T> {
    receiver: mpsc::UnboundedReceiver<T>,
    running: bool,
}

impl<T: Send + 'static> ChannelSource<T> {
    /// Create a new channel source
    pub fn new(receiver: mpsc::UnboundedReceiver<T>) -> Self {
        Self {
            receiver,
            running: false,
        }
    }

    /// Create a channel source with sender
    pub fn create() -> (mpsc::UnboundedSender<T>, Self) {
        let (tx, rx) = mpsc::unbounded_channel();
        (tx, Self::new(rx))
    }
}

#[async_trait]
impl<T: Send + 'static> DataStream for ChannelSource<T> {
    type Item = T;

    async fn next(&mut self) -> Result<Option<Self::Item>> {
        if !self.running {
            return Ok(None);
        }

        Ok(self.receiver.recv().await)
    }

    fn is_complete(&self) -> bool {
        !self.running || self.receiver.is_closed()
    }
}

#[async_trait]
impl<T: Send + 'static> Source for ChannelSource<T> {
    async fn start(&mut self) -> Result<()> {
        self.running = true;
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        self.running = false;
        self.receiver.close();
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running
    }
}

/// Bounded channel source
pub struct BoundedChannelSource<T> {
    receiver: mpsc::Receiver<T>,
    running: bool,
}

impl<T: Send + 'static> BoundedChannelSource<T> {
    /// Create a new bounded channel source
    pub fn new(receiver: mpsc::Receiver<T>) -> Self {
        Self {
            receiver,
            running: false,
        }
    }

    /// Create a bounded channel source with sender
    pub fn create(capacity: usize) -> (mpsc::Sender<T>, Self) {
        let (tx, rx) = mpsc::channel(capacity);
        (tx, Self::new(rx))
    }
}

#[async_trait]
impl<T: Send + 'static> DataStream for BoundedChannelSource<T> {
    type Item = T;

    async fn next(&mut self) -> Result<Option<Self::Item>> {
        if !self.running {
            return Ok(None);
        }

        Ok(self.receiver.recv().await)
    }

    fn is_complete(&self) -> bool {
        !self.running
    }
}

#[async_trait]
impl<T: Send + 'static> Source for BoundedChannelSource<T> {
    async fn start(&mut self) -> Result<()> {
        self.running = true;
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        self.running = false;
        self.receiver.close();
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running
    }
}

/// Flume channel source for high-performance scenarios
pub struct FlumeChannelSource<T> {
    receiver: flume::Receiver<T>,
    running: bool,
}

impl<T: Send + 'static> FlumeChannelSource<T> {
    /// Create a new flume channel source
    pub fn new(receiver: flume::Receiver<T>) -> Self {
        Self {
            receiver,
            running: false,
        }
    }

    /// Create a bounded flume channel source
    pub fn create_bounded(capacity: usize) -> (flume::Sender<T>, Self) {
        let (tx, rx) = flume::bounded(capacity);
        (tx, Self::new(rx))
    }

    /// Create an unbounded flume channel source
    pub fn create_unbounded() -> (flume::Sender<T>, Self) {
        let (tx, rx) = flume::unbounded();
        (tx, Self::new(rx))
    }
}

#[async_trait]
impl<T: Send + 'static> DataStream for FlumeChannelSource<T> {
    type Item = T;

    async fn next(&mut self) -> Result<Option<Self::Item>> {
        if !self.running {
            return Ok(None);
        }

        self.receiver
            .recv_async()
            .await
            .map(Some)
            .map_err(|e| StreamingError::Source(format!("Channel receive error: {}", e)))
    }

    fn is_complete(&self) -> bool {
        !self.running || self.receiver.is_disconnected()
    }
}

#[async_trait]
impl<T: Send + 'static> Source for FlumeChannelSource<T> {
    async fn start(&mut self) -> Result<()> {
        self.running = true;
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        self.running = false;
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_channel_source() {
        let (tx, mut source) = ChannelSource::create();

        source.start().await.unwrap();
        assert!(source.is_running());

        tx.send(1).unwrap();
        tx.send(2).unwrap();
        tx.send(3).unwrap();

        assert_eq!(source.next().await.unwrap(), Some(1));
        assert_eq!(source.next().await.unwrap(), Some(2));
        assert_eq!(source.next().await.unwrap(), Some(3));

        source.stop().await.unwrap();
        assert!(!source.is_running());
    }

    #[tokio::test]
    async fn test_flume_channel_source() {
        let (tx, mut source) = FlumeChannelSource::create_unbounded();

        source.start().await.unwrap();

        tx.send(1).unwrap();
        tx.send(2).unwrap();

        assert_eq!(source.next().await.unwrap(), Some(1));
        assert_eq!(source.next().await.unwrap(), Some(2));
    }
}
