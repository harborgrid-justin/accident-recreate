//! Channel-based streaming sink.

use crate::error::{Result, StreamingError};
use crate::sink::Sink;
use async_trait::async_trait;
use tokio::sync::mpsc;

/// Channel sink that writes to a tokio channel
pub struct ChannelSink<T> {
    sender: mpsc::UnboundedSender<T>,
}

impl<T: Send + 'static> ChannelSink<T> {
    /// Create a new channel sink
    pub fn new(sender: mpsc::UnboundedSender<T>) -> Self {
        Self { sender }
    }

    /// Create a channel sink with receiver
    pub fn create() -> (Self, mpsc::UnboundedReceiver<T>) {
        let (tx, rx) = mpsc::unbounded_channel();
        (Self::new(tx), rx)
    }
}

#[async_trait]
impl<T: Send + 'static> Sink<T> for ChannelSink<T> {
    async fn write(&mut self, item: T) -> Result<()> {
        self.sender
            .send(item)
            .map_err(|e| StreamingError::Sink(format!("Channel send error: {}", e)))
    }

    async fn flush(&mut self) -> Result<()> {
        // Channels flush automatically
        Ok(())
    }

    async fn close(&mut self) -> Result<()> {
        // Sender will be dropped, closing the channel
        Ok(())
    }
}

/// Bounded channel sink
pub struct BoundedChannelSink<T> {
    sender: mpsc::Sender<T>,
}

impl<T: Send + 'static> BoundedChannelSink<T> {
    /// Create a new bounded channel sink
    pub fn new(sender: mpsc::Sender<T>) -> Self {
        Self { sender }
    }

    /// Create a bounded channel sink with receiver
    pub fn create(capacity: usize) -> (Self, mpsc::Receiver<T>) {
        let (tx, rx) = mpsc::channel(capacity);
        (Self::new(tx), rx)
    }
}

#[async_trait]
impl<T: Send + 'static> Sink<T> for BoundedChannelSink<T> {
    async fn write(&mut self, item: T) -> Result<()> {
        self.sender
            .send(item)
            .await
            .map_err(|e| StreamingError::Sink(format!("Channel send error: {}", e)))
    }

    async fn flush(&mut self) -> Result<()> {
        Ok(())
    }

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }
}

/// Flume channel sink for high-performance scenarios
pub struct FlumeChannelSink<T> {
    sender: flume::Sender<T>,
}

impl<T: Send + 'static> FlumeChannelSink<T> {
    /// Create a new flume channel sink
    pub fn new(sender: flume::Sender<T>) -> Self {
        Self { sender }
    }

    /// Create a bounded flume channel sink
    pub fn create_bounded(capacity: usize) -> (Self, flume::Receiver<T>) {
        let (tx, rx) = flume::bounded(capacity);
        (Self::new(tx), rx)
    }

    /// Create an unbounded flume channel sink
    pub fn create_unbounded() -> (Self, flume::Receiver<T>) {
        let (tx, rx) = flume::unbounded();
        (Self::new(tx), rx)
    }
}

#[async_trait]
impl<T: Send + 'static> Sink<T> for FlumeChannelSink<T> {
    async fn write(&mut self, item: T) -> Result<()> {
        self.sender
            .send_async(item)
            .await
            .map_err(|e| StreamingError::Sink(format!("Flume send error: {}", e)))
    }

    async fn flush(&mut self) -> Result<()> {
        Ok(())
    }

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_channel_sink() {
        let (mut sink, mut rx) = ChannelSink::create();

        sink.write(1).await.unwrap();
        sink.write(2).await.unwrap();
        sink.write(3).await.unwrap();

        assert_eq!(rx.recv().await, Some(1));
        assert_eq!(rx.recv().await, Some(2));
        assert_eq!(rx.recv().await, Some(3));

        sink.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_flume_channel_sink() {
        let (mut sink, rx) = FlumeChannelSink::create_unbounded();

        sink.write(1).await.unwrap();
        sink.write(2).await.unwrap();

        assert_eq!(rx.recv_async().await.unwrap(), 1);
        assert_eq!(rx.recv_async().await.unwrap(), 2);
    }
}
