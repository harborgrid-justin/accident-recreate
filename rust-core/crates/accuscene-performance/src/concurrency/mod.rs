//! Concurrency utilities for high-performance operations

pub mod atomic_batch;
pub mod channel;
pub mod lock_free;
pub mod work_stealing;

pub use atomic_batch::{AtomicBatch, BatchOperation};
pub use channel::{bounded, unbounded, Receiver, Sender};
pub use lock_free::{LockFreeQueue, LockFreeStack};
pub use work_stealing::{WorkStealingPool, WorkStealingTask};

use std::sync::atomic::{AtomicUsize, Ordering};

/// Atomic counter with relaxed ordering for performance
#[derive(Debug, Default)]
pub struct AtomicCounter {
    value: AtomicUsize,
}

impl AtomicCounter {
    /// Create a new counter
    pub fn new(initial: usize) -> Self {
        Self {
            value: AtomicUsize::new(initial),
        }
    }

    /// Increment and return the new value
    pub fn increment(&self) -> usize {
        self.value.fetch_add(1, Ordering::Relaxed) + 1
    }

    /// Decrement and return the new value
    pub fn decrement(&self) -> usize {
        self.value.fetch_sub(1, Ordering::Relaxed).saturating_sub(1)
    }

    /// Add a value and return the new value
    pub fn add(&self, n: usize) -> usize {
        self.value.fetch_add(n, Ordering::Relaxed) + n
    }

    /// Get the current value
    pub fn get(&self) -> usize {
        self.value.load(Ordering::Relaxed)
    }

    /// Set the value
    pub fn set(&self, value: usize) {
        self.value.store(value, Ordering::Relaxed);
    }

    /// Reset to zero
    pub fn reset(&self) {
        self.set(0);
    }

    /// Compare and swap
    pub fn compare_and_swap(&self, current: usize, new: usize) -> Result<usize, usize> {
        self.value
            .compare_exchange(current, new, Ordering::SeqCst, Ordering::SeqCst)
    }
}

/// Spin lock for very short critical sections
pub struct SpinLock<T> {
    lock: AtomicUsize,
    data: std::cell::UnsafeCell<T>,
}

unsafe impl<T: Send> Send for SpinLock<T> {}
unsafe impl<T: Send> Sync for SpinLock<T> {}

impl<T> SpinLock<T> {
    /// Create a new spin lock
    pub fn new(data: T) -> Self {
        Self {
            lock: AtomicUsize::new(0),
            data: std::cell::UnsafeCell::new(data),
        }
    }

    /// Try to acquire the lock
    pub fn try_lock(&self) -> Option<SpinLockGuard<T>> {
        if self
            .lock
            .compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            Some(SpinLockGuard { lock: self })
        } else {
            None
        }
    }

    /// Acquire the lock (spin until available)
    pub fn lock(&self) -> SpinLockGuard<T> {
        while self
            .lock
            .compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            std::hint::spin_loop();
        }
        SpinLockGuard { lock: self }
    }
}

/// Guard for spin lock
pub struct SpinLockGuard<'a, T> {
    lock: &'a SpinLock<T>,
}

impl<'a, T> std::ops::Deref for SpinLockGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.data.get() }
    }
}

impl<'a, T> std::ops::DerefMut for SpinLockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<'a, T> Drop for SpinLockGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.lock.store(0, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atomic_counter() {
        let counter = AtomicCounter::new(0);

        assert_eq!(counter.increment(), 1);
        assert_eq!(counter.increment(), 2);
        assert_eq!(counter.get(), 2);

        counter.set(10);
        assert_eq!(counter.get(), 10);

        counter.reset();
        assert_eq!(counter.get(), 0);
    }

    #[test]
    fn test_spin_lock() {
        let lock = SpinLock::new(42);

        {
            let mut guard = lock.lock();
            *guard = 100;
        }

        {
            let guard = lock.lock();
            assert_eq!(*guard, 100);
        }
    }

    #[test]
    fn test_spin_lock_try() {
        let lock = SpinLock::new(0);

        let guard1 = lock.try_lock();
        assert!(guard1.is_some());

        let guard2 = lock.try_lock();
        assert!(guard2.is_none());

        drop(guard1);

        let guard3 = lock.try_lock();
        assert!(guard3.is_some());
    }
}
