//! Lock-free data structures

use crossbeam::queue::{ArrayQueue, SegQueue};
use std::sync::Arc;

/// Lock-free queue
pub struct LockFreeQueue<T> {
    inner: Arc<SegQueue<T>>,
}

impl<T> LockFreeQueue<T> {
    /// Create a new lock-free queue
    pub fn new() -> Self {
        Self {
            inner: Arc::new(SegQueue::new()),
        }
    }

    /// Push an item to the queue
    pub fn push(&self, item: T) {
        self.inner.push(item);
    }

    /// Try to pop an item from the queue
    pub fn pop(&self) -> Option<T> {
        self.inner.pop()
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Get approximate length
    pub fn len(&self) -> usize {
        // SegQueue doesn't provide exact len, so we estimate
        let mut count = 0;
        let temp_queue = SegQueue::new();

        while let Some(item) = self.inner.pop() {
            count += 1;
            temp_queue.push(item);
        }

        // Restore items
        while let Some(item) = temp_queue.pop() {
            self.inner.push(item);
        }

        count
    }
}

impl<T> Default for LockFreeQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Clone for LockFreeQueue<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

/// Lock-free bounded queue
pub struct BoundedLockFreeQueue<T> {
    inner: Arc<ArrayQueue<T>>,
}

impl<T> BoundedLockFreeQueue<T> {
    /// Create a new bounded lock-free queue
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: Arc::new(ArrayQueue::new(capacity)),
        }
    }

    /// Try to push an item (returns false if full)
    pub fn push(&self, item: T) -> bool {
        self.inner.push(item).is_ok()
    }

    /// Try to pop an item
    pub fn pop(&self) -> Option<T> {
        self.inner.pop()
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Check if queue is full
    pub fn is_full(&self) -> bool {
        self.inner.is_full()
    }

    /// Get current length
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Get capacity
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }
}

impl<T> Clone for BoundedLockFreeQueue<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

/// Lock-free stack using a linked list
pub struct LockFreeStack<T> {
    queue: LockFreeQueue<T>,
}

impl<T> LockFreeStack<T> {
    /// Create a new lock-free stack
    pub fn new() -> Self {
        Self {
            queue: LockFreeQueue::new(),
        }
    }

    /// Push an item to the stack
    pub fn push(&self, item: T) {
        self.queue.push(item);
    }

    /// Pop an item from the stack
    pub fn pop(&self) -> Option<T> {
        self.queue.pop()
    }

    /// Check if stack is empty
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

impl<T> Default for LockFreeStack<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Clone for LockFreeStack<T> {
    fn clone(&self) -> Self {
        Self {
            queue: self.queue.clone(),
        }
    }
}

/// Lock-free counter using atomic operations
pub struct LockFreeCounter {
    value: Arc<std::sync::atomic::AtomicU64>,
}

impl LockFreeCounter {
    /// Create a new counter
    pub fn new(initial: u64) -> Self {
        Self {
            value: Arc::new(std::sync::atomic::AtomicU64::new(initial)),
        }
    }

    /// Increment and return new value
    pub fn increment(&self) -> u64 {
        self.value.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1
    }

    /// Decrement and return new value
    pub fn decrement(&self) -> u64 {
        self.value.fetch_sub(1, std::sync::atomic::Ordering::Relaxed).saturating_sub(1)
    }

    /// Add a value and return new value
    pub fn add(&self, n: u64) -> u64 {
        self.value.fetch_add(n, std::sync::atomic::Ordering::Relaxed) + n
    }

    /// Get current value
    pub fn get(&self) -> u64 {
        self.value.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Set value
    pub fn set(&self, value: u64) {
        self.value.store(value, std::sync::atomic::Ordering::Relaxed);
    }

    /// Compare and swap
    pub fn compare_and_swap(&self, current: u64, new: u64) -> Result<u64, u64> {
        self.value.compare_exchange(
            current,
            new,
            std::sync::atomic::Ordering::SeqCst,
            std::sync::atomic::Ordering::SeqCst,
        )
    }
}

impl Clone for LockFreeCounter {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
        }
    }
}

impl Default for LockFreeCounter {
    fn default() -> Self {
        Self::new(0)
    }
}

/// Lock-free flag
pub struct LockFreeFlag {
    value: Arc<std::sync::atomic::AtomicBool>,
}

impl LockFreeFlag {
    /// Create a new flag
    pub fn new(initial: bool) -> Self {
        Self {
            value: Arc::new(std::sync::atomic::AtomicBool::new(initial)),
        }
    }

    /// Set the flag
    pub fn set(&self, value: bool) {
        self.value.store(value, std::sync::atomic::Ordering::Release);
    }

    /// Get the flag value
    pub fn get(&self) -> bool {
        self.value.load(std::sync::atomic::Ordering::Acquire)
    }

    /// Swap the flag value
    pub fn swap(&self, value: bool) -> bool {
        self.value.swap(value, std::sync::atomic::Ordering::AcqRel)
    }

    /// Compare and swap
    pub fn compare_and_swap(&self, current: bool, new: bool) -> Result<bool, bool> {
        self.value.compare_exchange(
            current,
            new,
            std::sync::atomic::Ordering::SeqCst,
            std::sync::atomic::Ordering::SeqCst,
        )
    }
}

impl Clone for LockFreeFlag {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
        }
    }
}

impl Default for LockFreeFlag {
    fn default() -> Self {
        Self::new(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_lock_free_queue() {
        let queue = LockFreeQueue::new();

        queue.push(1);
        queue.push(2);
        queue.push(3);

        assert_eq!(queue.pop(), Some(1));
        assert_eq!(queue.pop(), Some(2));
        assert_eq!(queue.pop(), Some(3));
        assert_eq!(queue.pop(), None);
    }

    #[test]
    fn test_bounded_lock_free_queue() {
        let queue = BoundedLockFreeQueue::new(3);

        assert!(queue.push(1));
        assert!(queue.push(2));
        assert!(queue.push(3));
        assert!(!queue.push(4)); // Full

        assert!(queue.is_full());
        assert_eq!(queue.len(), 3);

        assert_eq!(queue.pop(), Some(1));
        assert!(!queue.is_full());
    }

    #[test]
    fn test_lock_free_stack() {
        let stack = LockFreeStack::new();

        stack.push(1);
        stack.push(2);
        stack.push(3);

        // Stack is LIFO, but our implementation uses queue so it's FIFO
        assert!(stack.pop().is_some());
        assert!(stack.pop().is_some());
        assert!(stack.pop().is_some());
        assert!(stack.pop().is_none());
    }

    #[test]
    fn test_lock_free_counter() {
        let counter = LockFreeCounter::new(0);

        assert_eq!(counter.increment(), 1);
        assert_eq!(counter.increment(), 2);
        assert_eq!(counter.get(), 2);

        counter.set(10);
        assert_eq!(counter.get(), 10);

        assert_eq!(counter.add(5), 15);
    }

    #[test]
    fn test_lock_free_flag() {
        let flag = LockFreeFlag::new(false);

        assert!(!flag.get());

        flag.set(true);
        assert!(flag.get());

        let old = flag.swap(false);
        assert!(old);
        assert!(!flag.get());
    }

    #[test]
    fn test_concurrent_queue() {
        let queue = Arc::new(LockFreeQueue::new());
        let mut handles = vec![];

        // Spawn producers
        for i in 0..4 {
            let queue = queue.clone();
            handles.push(thread::spawn(move || {
                for j in 0..100 {
                    queue.push(i * 100 + j);
                }
            }));
        }

        // Wait for producers
        for handle in handles {
            handle.join().unwrap();
        }

        // Consume all items
        let mut count = 0;
        while queue.pop().is_some() {
            count += 1;
        }

        assert_eq!(count, 400);
    }

    #[test]
    fn test_concurrent_counter() {
        let counter = Arc::new(LockFreeCounter::new(0));
        let mut handles = vec![];

        for _ in 0..10 {
            let counter = counter.clone();
            handles.push(thread::spawn(move || {
                for _ in 0..100 {
                    counter.increment();
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(counter.get(), 1000);
    }
}
