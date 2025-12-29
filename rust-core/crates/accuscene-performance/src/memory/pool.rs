//! Object pooling for allocation reduction

use crossbeam::queue::ArrayQueue;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

/// Object pool for reusing allocations
pub struct ObjectPool<T> {
    queue: Arc<ArrayQueue<T>>,
    factory: Arc<dyn Fn() -> T + Send + Sync>,
    capacity: usize,
}

impl<T> ObjectPool<T> {
    /// Create a new object pool
    pub fn new<F>(capacity: usize, factory: F) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        let queue = Arc::new(ArrayQueue::new(capacity));

        // Pre-populate the pool
        for _ in 0..capacity {
            let _ = queue.push(factory());
        }

        Self {
            queue,
            factory: Arc::new(factory),
            capacity,
        }
    }

    /// Get an object from the pool
    pub fn get(&self) -> PooledObject<T> {
        let obj = self
            .queue
            .pop()
            .unwrap_or_else(|| (self.factory)());

        PooledObject {
            obj: Some(obj),
            pool: self.queue.clone(),
        }
    }

    /// Get current pool size
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Check if pool is empty
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Get pool capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Clear the pool
    pub fn clear(&self) {
        while self.queue.pop().is_some() {}
    }
}

impl<T: Clone> Clone for ObjectPool<T> {
    fn clone(&self) -> Self {
        Self {
            queue: self.queue.clone(),
            factory: self.factory.clone(),
            capacity: self.capacity,
        }
    }
}

/// A pooled object that returns to the pool when dropped
pub struct PooledObject<T> {
    obj: Option<T>,
    pool: Arc<ArrayQueue<T>>,
}

impl<T> PooledObject<T> {
    /// Take ownership of the object, preventing it from returning to the pool
    pub fn take(mut self) -> T {
        self.obj.take().expect("Object already taken")
    }
}

impl<T> Deref for PooledObject<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.obj.as_ref().expect("Object already taken")
    }
}

impl<T> DerefMut for PooledObject<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.obj.as_mut().expect("Object already taken")
    }
}

impl<T> Drop for PooledObject<T> {
    fn drop(&mut self) {
        if let Some(obj) = self.obj.take() {
            let _ = self.pool.push(obj); // Ignore if pool is full
        }
    }
}

/// Buffer pool for byte buffers
pub struct BufferPool {
    pool: ObjectPool<Vec<u8>>,
}

impl BufferPool {
    /// Create a new buffer pool
    pub fn new(capacity: usize, buffer_size: usize) -> Self {
        Self {
            pool: ObjectPool::new(capacity, move || Vec::with_capacity(buffer_size)),
        }
    }

    /// Get a buffer from the pool
    pub fn get(&self) -> PooledObject<Vec<u8>> {
        let mut buf = self.pool.get();
        buf.clear(); // Ensure buffer is empty
        buf
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        PoolStats {
            capacity: self.pool.capacity(),
            available: self.pool.len(),
            in_use: self.pool.capacity() - self.pool.len(),
        }
    }
}

/// String pool for string allocations
pub struct StringPool {
    pool: ObjectPool<String>,
}

impl StringPool {
    /// Create a new string pool
    pub fn new(capacity: usize, initial_size: usize) -> Self {
        Self {
            pool: ObjectPool::new(capacity, move || String::with_capacity(initial_size)),
        }
    }

    /// Get a string from the pool
    pub fn get(&self) -> PooledObject<String> {
        let mut s = self.pool.get();
        s.clear(); // Ensure string is empty
        s
    }
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    /// Total pool capacity
    pub capacity: usize,
    /// Available objects in pool
    pub available: usize,
    /// Objects currently in use
    pub in_use: usize,
}

impl PoolStats {
    /// Get utilization ratio (0.0-1.0)
    pub fn utilization(&self) -> f32 {
        self.in_use as f32 / self.capacity.max(1) as f32
    }

    /// Check if pool is exhausted
    pub fn is_exhausted(&self) -> bool {
        self.available == 0
    }
}

/// Generic resettable trait for pool objects
pub trait Resettable {
    /// Reset the object to initial state
    fn reset(&mut self);
}

impl Resettable for Vec<u8> {
    fn reset(&mut self) {
        self.clear();
    }
}

impl Resettable for String {
    fn reset(&mut self) {
        self.clear();
    }
}

/// Pool with automatic reset on return
pub struct ResettablePool<T: Resettable> {
    inner: ObjectPool<T>,
}

impl<T: Resettable> ResettablePool<T> {
    /// Create a new resettable pool
    pub fn new<F>(capacity: usize, factory: F) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        Self {
            inner: ObjectPool::new(capacity, factory),
        }
    }

    /// Get an object from the pool
    pub fn get(&self) -> ResettablePooledObject<T> {
        ResettablePooledObject {
            obj: Some(self.inner.get().take()),
            pool: self.inner.queue.clone(),
        }
    }
}

/// Resettable pooled object
pub struct ResettablePooledObject<T: Resettable> {
    obj: Option<T>,
    pool: Arc<ArrayQueue<T>>,
}

impl<T: Resettable> Deref for ResettablePooledObject<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.obj.as_ref().expect("Object already taken")
    }
}

impl<T: Resettable> DerefMut for ResettablePooledObject<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.obj.as_mut().expect("Object already taken")
    }
}

impl<T: Resettable> Drop for ResettablePooledObject<T> {
    fn drop(&mut self) {
        if let Some(mut obj) = self.obj.take() {
            obj.reset();
            let _ = self.pool.push(obj);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_pool() {
        let pool = ObjectPool::new(10, || Vec::<u8>::with_capacity(1024));

        assert_eq!(pool.len(), 10);
        assert_eq!(pool.capacity(), 10);

        let obj = pool.get();
        assert_eq!(pool.len(), 9);

        drop(obj);
        assert_eq!(pool.len(), 10); // Returned to pool
    }

    #[test]
    fn test_buffer_pool() {
        let pool = BufferPool::new(5, 256);

        let mut buf = pool.get();
        buf.extend_from_slice(b"test");
        assert_eq!(buf.len(), 4);

        drop(buf);

        let buf2 = pool.get();
        assert_eq!(buf2.len(), 0); // Should be cleared
    }

    #[test]
    fn test_string_pool() {
        let pool = StringPool::new(5, 128);

        let mut s = pool.get();
        s.push_str("hello");
        assert_eq!(s.as_str(), "hello");

        drop(s);

        let s2 = pool.get();
        assert!(s2.is_empty()); // Should be cleared
    }

    #[test]
    fn test_pool_stats() {
        let pool = BufferPool::new(10, 256);

        let stats = pool.stats();
        assert_eq!(stats.capacity, 10);
        assert_eq!(stats.available, 10);
        assert_eq!(stats.in_use, 0);

        let _buf1 = pool.get();
        let _buf2 = pool.get();

        let stats = pool.stats();
        assert_eq!(stats.available, 8);
        assert_eq!(stats.in_use, 2);
    }

    #[test]
    fn test_pooled_object_take() {
        let pool = ObjectPool::new(5, || vec![1, 2, 3]);

        let obj = pool.get();
        let vec = obj.take();

        assert_eq!(vec, vec![1, 2, 3]);
        assert_eq!(pool.len(), 4); // Not returned to pool
    }
}
