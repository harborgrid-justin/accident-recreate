//! Data sink abstractions for streaming

use crate::error::Result;
use crate::streaming::StreamItem;
use parking_lot::Mutex;
use std::future::Future;
use std::io::Write;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;

/// Trait for data sinks
pub trait Sink: Send + Sync {
    /// Send an item to the sink
    fn send(&mut self, item: StreamItem) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>;

    /// Flush any buffered data
    fn flush(&mut self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>;

    /// Close the sink
    fn close(&mut self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async { Ok(()) })
    }
}

/// Vector-based sink that collects items
pub struct VectorSink {
    items: Arc<Mutex<Vec<StreamItem>>>,
    capacity: Option<usize>,
}

impl VectorSink {
    /// Create a new vector sink
    pub fn new() -> Self {
        Self {
            items: Arc::new(Mutex::new(Vec::new())),
            capacity: None,
        }
    }

    /// Create with capacity limit
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            items: Arc::new(Mutex::new(Vec::with_capacity(capacity))),
            capacity: Some(capacity),
        }
    }

    /// Get collected items
    pub fn items(&self) -> Vec<StreamItem> {
        self.items.lock().clone()
    }

    /// Get item count
    pub fn len(&self) -> usize {
        self.items.lock().len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.items.lock().is_empty()
    }

    /// Clear collected items
    pub fn clear(&self) {
        self.items.lock().clear();
    }
}

impl Default for VectorSink {
    fn default() -> Self {
        Self::new()
    }
}

impl Sink for VectorSink {
    fn send(&mut self, item: StreamItem) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move {
            let mut items = self.items.lock();

            if let Some(cap) = self.capacity {
                if items.len() >= cap {
                    return Err(crate::error::PerformanceError::BufferFull {
                        capacity: cap,
                        size: items.len() + 1,
                    });
                }
            }

            items.push(item);
            Ok(())
        })
    }

    fn flush(&mut self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async { Ok(()) })
    }
}

/// File-based sink that writes to disk
pub struct FileSink {
    path: PathBuf,
    writer: Arc<Mutex<Option<std::fs::File>>>,
    buffer_size: usize,
    bytes_written: Arc<Mutex<usize>>,
}

impl FileSink {
    /// Create a new file sink
    pub fn new(path: PathBuf) -> Result<Self> {
        Ok(Self {
            path,
            writer: Arc::new(Mutex::new(None)),
            buffer_size: 8192,
            bytes_written: Arc::new(Mutex::new(0)),
        })
    }

    /// Create with custom buffer size
    pub fn with_buffer_size(path: PathBuf, buffer_size: usize) -> Result<Self> {
        Ok(Self {
            path,
            writer: Arc::new(Mutex::new(None)),
            buffer_size,
            bytes_written: Arc::new(Mutex::new(0)),
        })
    }

    /// Get bytes written
    pub fn bytes_written(&self) -> usize {
        *self.bytes_written.lock()
    }

    /// Open the file for writing
    fn open(&self) -> Result<()> {
        let mut writer = self.writer.lock();
        if writer.is_none() {
            let file = std::fs::File::create(&self.path)?;
            *writer = Some(file);
        }
        Ok(())
    }
}

impl Sink for FileSink {
    fn send(&mut self, item: StreamItem) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move {
            self.open()?;

            let mut writer = self.writer.lock();
            if let Some(file) = writer.as_mut() {
                file.write_all(&item.data)?;
                *self.bytes_written.lock() += item.size;
            }

            Ok(())
        })
    }

    fn flush(&mut self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move {
            let mut writer = self.writer.lock();
            if let Some(file) = writer.as_mut() {
                file.flush()?;
            }
            Ok(())
        })
    }

    fn close(&mut self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move {
            self.flush().await?;
            *self.writer.lock() = None;
            Ok(())
        })
    }
}

/// Null sink that discards all data (useful for benchmarking)
pub struct NullSink {
    items_received: Arc<Mutex<usize>>,
    bytes_received: Arc<Mutex<usize>>,
}

impl NullSink {
    /// Create a new null sink
    pub fn new() -> Self {
        Self {
            items_received: Arc::new(Mutex::new(0)),
            bytes_received: Arc::new(Mutex::new(0)),
        }
    }

    /// Get number of items received
    pub fn items_received(&self) -> usize {
        *self.items_received.lock()
    }

    /// Get bytes received
    pub fn bytes_received(&self) -> usize {
        *self.bytes_received.lock()
    }
}

impl Default for NullSink {
    fn default() -> Self {
        Self::new()
    }
}

impl Sink for NullSink {
    fn send(&mut self, item: StreamItem) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move {
            *self.items_received.lock() += 1;
            *self.bytes_received.lock() += item.size;
            Ok(())
        })
    }

    fn flush(&mut self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async { Ok(()) })
    }
}

/// Batching sink that accumulates items before sending
pub struct BatchingSink<S: Sink> {
    inner: S,
    batch: Vec<StreamItem>,
    batch_size: usize,
}

impl<S: Sink> BatchingSink<S> {
    /// Create a new batching sink
    pub fn new(inner: S, batch_size: usize) -> Self {
        Self {
            inner,
            batch: Vec::with_capacity(batch_size),
            batch_size,
        }
    }

    /// Flush the current batch
    async fn flush_batch(&mut self) -> Result<()> {
        for item in self.batch.drain(..) {
            self.inner.send(item).await?;
        }
        Ok(())
    }
}

impl<S: Sink> Sink for BatchingSink<S> {
    fn send(&mut self, item: StreamItem) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move {
            self.batch.push(item);

            if self.batch.len() >= self.batch_size {
                self.flush_batch().await?;
            }

            Ok(())
        })
    }

    fn flush(&mut self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move {
            self.flush_batch().await?;
            self.inner.flush().await
        })
    }

    fn close(&mut self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move {
            self.flush().await?;
            self.inner.close().await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

    #[tokio::test]
    async fn test_vector_sink() {
        let mut sink = VectorSink::new();
        let item = StreamItem::new(Bytes::from("test"), 0);

        sink.send(item).await.unwrap();
        assert_eq!(sink.len(), 1);

        sink.flush().await.unwrap();
        assert_eq!(sink.len(), 1);
    }

    #[tokio::test]
    async fn test_vector_sink_capacity() {
        let mut sink = VectorSink::with_capacity(2);

        let item1 = StreamItem::new(Bytes::from("a"), 0);
        let item2 = StreamItem::new(Bytes::from("b"), 1);
        let item3 = StreamItem::new(Bytes::from("c"), 2);

        sink.send(item1).await.unwrap();
        sink.send(item2).await.unwrap();

        let result = sink.send(item3).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_null_sink() {
        let mut sink = NullSink::new();
        let item = StreamItem::new(Bytes::from("test"), 0);

        sink.send(item).await.unwrap();
        assert_eq!(sink.items_received(), 1);
        assert!(sink.bytes_received() > 0);
    }

    #[tokio::test]
    async fn test_batching_sink() {
        let inner = VectorSink::new();
        let mut sink = BatchingSink::new(inner, 3);

        for i in 0..5 {
            let item = StreamItem::new(Bytes::from(format!("item{}", i)), i);
            sink.send(item).await.unwrap();
        }

        sink.flush().await.unwrap();
    }
}
