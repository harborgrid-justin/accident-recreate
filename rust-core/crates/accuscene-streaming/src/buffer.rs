//! Buffer implementations for stream processing.

use crate::config::BackpressureStrategy;
use crate::error::{Result, StreamingError};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};

/// A bounded buffer with backpressure support
#[derive(Clone)]
pub struct BoundedBuffer<T> {
    inner: Arc<BoundedBufferInner<T>>,
}

struct BoundedBufferInner<T> {
    buffer: Mutex<VecDeque<T>>,
    capacity: usize,
    semaphore: Semaphore,
    strategy: BackpressureStrategy,
}

impl<T> BoundedBuffer<T> {
    /// Create a new bounded buffer with the given capacity
    pub fn new(capacity: usize, strategy: BackpressureStrategy) -> Self {
        Self {
            inner: Arc::new(BoundedBufferInner {
                buffer: Mutex::new(VecDeque::with_capacity(capacity)),
                capacity,
                semaphore: Semaphore::new(capacity),
                strategy,
            }),
        }
    }

    /// Push an item into the buffer
    pub async fn push(&self, item: T) -> Result<()> {
        match self.inner.strategy {
            BackpressureStrategy::Block => {
                // Acquire a permit (blocks if buffer is full)
                let _permit = self
                    .inner
                    .semaphore
                    .acquire()
                    .await
                    .map_err(|e| StreamingError::Backpressure(e.to_string()))?;

                let mut buffer = self.inner.buffer.lock().await;
                buffer.push_back(item);
                // Permit is dropped here, releasing the slot
                Ok(())
            }
            BackpressureStrategy::DropOldest => {
                let mut buffer = self.inner.buffer.lock().await;
                if buffer.len() >= self.inner.capacity {
                    buffer.pop_front();
                }
                buffer.push_back(item);
                Ok(())
            }
            BackpressureStrategy::DropNewest => {
                let mut buffer = self.inner.buffer.lock().await;
                if buffer.len() < self.inner.capacity {
                    buffer.push_back(item);
                }
                // Drop the new item if buffer is full
                Ok(())
            }
            BackpressureStrategy::Fail => {
                let mut buffer = self.inner.buffer.lock().await;
                if buffer.len() >= self.inner.capacity {
                    return Err(StreamingError::BufferOverflow {
                        capacity: self.inner.capacity,
                    });
                }
                buffer.push_back(item);
                Ok(())
            }
        }
    }

    /// Try to push an item without blocking
    pub fn try_push(&self, item: T) -> Result<()> {
        let buffer = self.inner.buffer.try_lock().map_err(|e| {
            StreamingError::Backpressure(format!("Failed to acquire buffer lock: {}", e))
        })?;

        match self.inner.strategy {
            BackpressureStrategy::Block | BackpressureStrategy::Fail => {
                if buffer.len() >= self.inner.capacity {
                    return Err(StreamingError::BufferOverflow {
                        capacity: self.inner.capacity,
                    });
                }
                drop(buffer);
                // Can't use async semaphore in sync context
                Err(StreamingError::Backpressure(
                    "Buffer full, use async push".to_string(),
                ))
            }
            _ => {
                drop(buffer);
                // For Drop strategies, we'd need to modify the buffer
                Err(StreamingError::Backpressure(
                    "Use async push for this strategy".to_string(),
                ))
            }
        }
    }

    /// Pop an item from the buffer
    pub async fn pop(&self) -> Result<Option<T>> {
        let mut buffer = self.inner.buffer.lock().await;
        let item = buffer.pop_front();

        if item.is_some() {
            // Release a permit for the next push
            self.inner.semaphore.add_permits(1);
        }

        Ok(item)
    }

    /// Try to pop an item without blocking
    pub fn try_pop(&self) -> Result<Option<T>> {
        let mut buffer = self.inner.buffer.try_lock().map_err(|e| {
            StreamingError::Backpressure(format!("Failed to acquire buffer lock: {}", e))
        })?;

        let item = buffer.pop_front();

        if item.is_some() {
            self.inner.semaphore.add_permits(1);
        }

        Ok(item)
    }

    /// Get the current size of the buffer
    pub async fn len(&self) -> usize {
        self.inner.buffer.lock().await.len()
    }

    /// Check if the buffer is empty
    pub async fn is_empty(&self) -> bool {
        self.inner.buffer.lock().await.is_empty()
    }

    /// Get the capacity of the buffer
    pub fn capacity(&self) -> usize {
        self.inner.capacity
    }

    /// Get the current fill ratio (0.0 to 1.0)
    pub async fn fill_ratio(&self) -> f64 {
        let len = self.len().await;
        len as f64 / self.inner.capacity as f64
    }
}

/// An unbounded buffer (uses VecDeque with dynamic growth)
#[derive(Clone)]
pub struct UnboundedBuffer<T> {
    inner: Arc<Mutex<VecDeque<T>>>,
}

impl<T> UnboundedBuffer<T> {
    /// Create a new unbounded buffer
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    /// Push an item into the buffer
    pub async fn push(&self, item: T) {
        let mut buffer = self.inner.lock().await;
        buffer.push_back(item);
    }

    /// Pop an item from the buffer
    pub async fn pop(&self) -> Option<T> {
        let mut buffer = self.inner.lock().await;
        buffer.pop_front()
    }

    /// Get the current size of the buffer
    pub async fn len(&self) -> usize {
        self.inner.lock().await.len()
    }

    /// Check if the buffer is empty
    pub async fn is_empty(&self) -> bool {
        self.inner.lock().await.is_empty()
    }
}

impl<T> Default for UnboundedBuffer<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Ring buffer for fixed-size circular storage
pub struct RingBuffer<T> {
    buffer: Vec<Option<T>>,
    capacity: usize,
    head: usize,
    tail: usize,
    size: usize,
}

impl<T> RingBuffer<T> {
    /// Create a new ring buffer with the given capacity
    pub fn new(capacity: usize) -> Self {
        let mut buffer = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            buffer.push(None);
        }

        Self {
            buffer,
            capacity,
            head: 0,
            tail: 0,
            size: 0,
        }
    }

    /// Push an item into the ring buffer
    pub fn push(&mut self, item: T) -> Option<T> {
        let old_item = self.buffer[self.tail].take();

        self.buffer[self.tail] = Some(item);
        self.tail = (self.tail + 1) % self.capacity;

        if self.size == self.capacity {
            self.head = (self.head + 1) % self.capacity;
            old_item
        } else {
            self.size += 1;
            None
        }
    }

    /// Pop an item from the ring buffer
    pub fn pop(&mut self) -> Option<T> {
        if self.size == 0 {
            return None;
        }

        let item = self.buffer[self.head].take();
        self.head = (self.head + 1) % self.capacity;
        self.size -= 1;

        item
    }

    /// Get the current size
    pub fn len(&self) -> usize {
        self.size
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Check if the buffer is full
    pub fn is_full(&self) -> bool {
        self.size == self.capacity
    }

    /// Get the capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bounded_buffer_block() {
        let buffer = BoundedBuffer::new(3, BackpressureStrategy::Block);

        buffer.push(1).await.unwrap();
        buffer.push(2).await.unwrap();
        buffer.push(3).await.unwrap();

        assert_eq!(buffer.len().await, 3);

        assert_eq!(buffer.pop().await.unwrap(), Some(1));
        assert_eq!(buffer.pop().await.unwrap(), Some(2));

        buffer.push(4).await.unwrap();

        assert_eq!(buffer.len().await, 2);
    }

    #[tokio::test]
    async fn test_unbounded_buffer() {
        let buffer = UnboundedBuffer::new();

        buffer.push(1).await;
        buffer.push(2).await;
        buffer.push(3).await;

        assert_eq!(buffer.len().await, 3);

        assert_eq!(buffer.pop().await, Some(1));
        assert_eq!(buffer.pop().await, Some(2));
        assert_eq!(buffer.pop().await, Some(3));
        assert_eq!(buffer.pop().await, None);
    }

    #[test]
    fn test_ring_buffer() {
        let mut buffer = RingBuffer::new(3);

        assert_eq!(buffer.push(1), None);
        assert_eq!(buffer.push(2), None);
        assert_eq!(buffer.push(3), None);
        assert_eq!(buffer.len(), 3);

        // Overflow: oldest item (1) is replaced
        assert_eq!(buffer.push(4), Some(1));
        assert_eq!(buffer.len(), 3);

        assert_eq!(buffer.pop(), Some(2));
        assert_eq!(buffer.pop(), Some(3));
        assert_eq!(buffer.pop(), Some(4));
        assert_eq!(buffer.pop(), None);
    }
}
