//! Atomic batch operations for reducing synchronization overhead

use parking_lot::Mutex;
use std::sync::Arc;

/// Batch operation trait
pub trait BatchOperation: Send {
    /// Apply the operation
    fn apply(&mut self);

    /// Merge with another operation (optional)
    fn merge(&mut self, _other: &Self) {}
}

/// Atomic batch processor
pub struct AtomicBatch<T: BatchOperation> {
    pending: Arc<Mutex<Vec<T>>>,
    batch_size: usize,
    executor: Arc<Mutex<Option<Box<dyn Fn(&mut [T]) + Send + Sync>>>>,
}

impl<T: BatchOperation> AtomicBatch<T> {
    /// Create a new atomic batch
    pub fn new(batch_size: usize) -> Self {
        Self {
            pending: Arc::new(Mutex::new(Vec::with_capacity(batch_size))),
            batch_size,
            executor: Arc::new(Mutex::new(None)),
        }
    }

    /// Set the batch executor
    pub fn with_executor<F>(self, executor: F) -> Self
    where
        F: Fn(&mut [T]) + Send + Sync + 'static,
    {
        *self.executor.lock() = Some(Box::new(executor));
        self
    }

    /// Add an operation to the batch
    pub fn add(&self, operation: T) -> bool {
        let mut pending = self.pending.lock();
        pending.push(operation);

        if pending.len() >= self.batch_size {
            self.flush_locked(&mut pending);
            true
        } else {
            false
        }
    }

    /// Flush pending operations
    pub fn flush(&self) {
        let mut pending = self.pending.lock();
        self.flush_locked(&mut pending);
    }

    /// Flush with lock held
    fn flush_locked(&self, pending: &mut Vec<T>) {
        if pending.is_empty() {
            return;
        }

        if let Some(executor) = self.executor.lock().as_ref() {
            executor(pending);
        } else {
            // Default: apply each operation
            for op in pending.iter_mut() {
                op.apply();
            }
        }

        pending.clear();
    }

    /// Get current batch size
    pub fn pending_count(&self) -> usize {
        self.pending.lock().len()
    }

    /// Get batch size limit
    pub fn batch_size(&self) -> usize {
        self.batch_size
    }
}

impl<T: BatchOperation> Clone for AtomicBatch<T> {
    fn clone(&self) -> Self {
        Self {
            pending: self.pending.clone(),
            batch_size: self.batch_size,
            executor: self.executor.clone(),
        }
    }
}

/// Increment operation
#[derive(Debug, Clone)]
pub struct IncrementOp {
    pub target: Arc<parking_lot::Mutex<i64>>,
    pub delta: i64,
}

impl BatchOperation for IncrementOp {
    fn apply(&mut self) {
        *self.target.lock() += self.delta;
    }

    fn merge(&mut self, other: &Self) {
        self.delta += other.delta;
    }
}

/// Write operation
#[derive(Debug, Clone)]
pub struct WriteOp<T: Clone> {
    pub value: T,
}

impl<T: Clone + Send> BatchOperation for WriteOp<T> {
    fn apply(&mut self) {
        // Implementation would write to actual storage
    }
}

/// Batched counter with atomic operations
pub struct BatchedCounter {
    batch: AtomicBatch<IncrementOp>,
    value: Arc<parking_lot::Mutex<i64>>,
}

impl BatchedCounter {
    /// Create a new batched counter
    pub fn new(batch_size: usize) -> Self {
        let value = Arc::new(parking_lot::Mutex::new(0));
        let value_clone = value.clone();

        let batch = AtomicBatch::new(batch_size).with_executor(move |ops: &mut [IncrementOp]| {
            let mut total_delta = 0;
            for op in ops.iter() {
                total_delta += op.delta;
            }
            *value_clone.lock() += total_delta;
        });

        Self { batch, value }
    }

    /// Increment the counter
    pub fn increment(&self, delta: i64) {
        self.batch.add(IncrementOp {
            target: self.value.clone(),
            delta,
        });
    }

    /// Get current value (forces flush)
    pub fn get(&self) -> i64 {
        self.batch.flush();
        *self.value.lock()
    }

    /// Flush pending operations
    pub fn flush(&self) {
        self.batch.flush();
    }
}

/// Batched write buffer
pub struct BatchedWriteBuffer<T: Clone + Send> {
    batch: AtomicBatch<WriteOp<T>>,
    buffer: Arc<Mutex<Vec<T>>>,
}

impl<T: Clone + Send + 'static> BatchedWriteBuffer<T> {
    /// Create a new batched write buffer
    pub fn new(batch_size: usize) -> Self {
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let buffer_clone = buffer.clone();

        let batch = AtomicBatch::new(batch_size).with_executor(move |ops: &mut [WriteOp<T>]| {
            let mut buf = buffer_clone.lock();
            for op in ops.iter() {
                buf.push(op.value.clone());
            }
        });

        Self { batch, buffer }
    }

    /// Write a value
    pub fn write(&self, value: T) {
        self.batch.add(WriteOp { value });
    }

    /// Get all written values (forces flush)
    pub fn get_all(&self) -> Vec<T> {
        self.batch.flush();
        self.buffer.lock().clone()
    }

    /// Clear the buffer
    pub fn clear(&self) {
        self.batch.flush();
        self.buffer.lock().clear();
    }

    /// Flush pending writes
    pub fn flush(&self) {
        self.batch.flush();
    }
}

/// Coalescing batch that merges similar operations
pub struct CoalescingBatch<T: BatchOperation + PartialEq> {
    pending: Arc<Mutex<Vec<T>>>,
    batch_size: usize,
}

impl<T: BatchOperation + PartialEq> CoalescingBatch<T> {
    /// Create a new coalescing batch
    pub fn new(batch_size: usize) -> Self {
        Self {
            pending: Arc::new(Mutex::new(Vec::new())),
            batch_size,
        }
    }

    /// Add an operation (will merge if similar operation exists)
    pub fn add(&self, operation: T) -> bool {
        let mut pending = self.pending.lock();

        // Try to merge with existing operation
        for existing in pending.iter_mut() {
            if existing == &operation {
                existing.merge(&operation);
                return false;
            }
        }

        // No merge possible, add new operation
        pending.push(operation);

        if pending.len() >= self.batch_size {
            for op in pending.iter_mut() {
                op.apply();
            }
            pending.clear();
            true
        } else {
            false
        }
    }

    /// Flush all pending operations
    pub fn flush(&self) {
        let mut pending = self.pending.lock();
        for op in pending.iter_mut() {
            op.apply();
        }
        pending.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batched_counter() {
        let counter = BatchedCounter::new(5);

        for _ in 0..10 {
            counter.increment(1);
        }

        assert_eq!(counter.get(), 10);
    }

    #[test]
    fn test_batched_write_buffer() {
        let buffer = BatchedWriteBuffer::new(3);

        buffer.write(1);
        buffer.write(2);
        buffer.write(3);
        buffer.write(4);

        let values = buffer.get_all();
        assert_eq!(values, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_atomic_batch_flush() {
        let executed = Arc::new(Mutex::new(0));
        let executed_clone = executed.clone();

        let batch = AtomicBatch::new(10).with_executor(move |ops: &mut [IncrementOp]| {
            *executed_clone.lock() += ops.len();
        });

        let value = Arc::new(parking_lot::Mutex::new(0));

        for _ in 0..5 {
            batch.add(IncrementOp {
                target: value.clone(),
                delta: 1,
            });
        }

        assert_eq!(*executed.lock(), 0); // Not flushed yet

        batch.flush();
        assert_eq!(*executed.lock(), 5); // Flushed
    }

    #[test]
    fn test_batch_auto_flush() {
        let counter = BatchedCounter::new(3);

        counter.increment(1);
        counter.increment(1);
        counter.increment(1); // This should trigger auto-flush

        // Small sleep to ensure batch is processed
        std::thread::sleep(std::time::Duration::from_millis(10));

        let value = counter.get();
        assert_eq!(value, 3);
    }
}
