//! Memory profiling utilities

use crate::profiling::Profiler;
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Instant;

/// Memory snapshot
#[derive(Debug, Clone)]
pub struct MemorySnapshot {
    /// Timestamp
    pub timestamp: Instant,
    /// Allocated bytes
    pub allocated: usize,
    /// Deallocated bytes
    pub deallocated: usize,
    /// Current in-use bytes
    pub in_use: usize,
    /// Peak memory usage
    pub peak: usize,
}

/// Memory profiler
pub struct MemoryProfiler {
    snapshots: Arc<Mutex<Vec<MemorySnapshot>>>,
    active: Arc<Mutex<bool>>,
    current_allocated: Arc<Mutex<usize>>,
    current_deallocated: Arc<Mutex<usize>>,
    current_in_use: Arc<Mutex<usize>>,
    peak_usage: Arc<Mutex<usize>>,
}

impl MemoryProfiler {
    /// Create a new memory profiler
    pub fn new() -> Self {
        Self {
            snapshots: Arc::new(Mutex::new(Vec::new())),
            active: Arc::new(Mutex::new(false)),
            current_allocated: Arc::new(Mutex::new(0)),
            current_deallocated: Arc::new(Mutex::new(0)),
            current_in_use: Arc::new(Mutex::new(0)),
            peak_usage: Arc::new(Mutex::new(0)),
        }
    }

    /// Record an allocation
    pub fn record_alloc(&self, size: usize) {
        if !self.is_active() {
            return;
        }

        *self.current_allocated.lock() += size;
        *self.current_in_use.lock() += size;

        let in_use = *self.current_in_use.lock();
        let mut peak = self.peak_usage.lock();
        if in_use > *peak {
            *peak = in_use;
        }
    }

    /// Record a deallocation
    pub fn record_dealloc(&self, size: usize) {
        if !self.is_active() {
            return;
        }

        *self.current_deallocated.lock() += size;
        let mut in_use = self.current_in_use.lock();
        *in_use = in_use.saturating_sub(size);
    }

    /// Take a snapshot
    pub fn snapshot(&self) -> MemorySnapshot {
        MemorySnapshot {
            timestamp: Instant::now(),
            allocated: *self.current_allocated.lock(),
            deallocated: *self.current_deallocated.lock(),
            in_use: *self.current_in_use.lock(),
            peak: *self.peak_usage.lock(),
        }
    }

    /// Record a snapshot
    pub fn record_snapshot(&self) {
        let snapshot = self.snapshot();
        self.snapshots.lock().push(snapshot);
    }

    /// Get all snapshots
    pub fn snapshots(&self) -> Vec<MemorySnapshot> {
        self.snapshots.lock().clone()
    }

    /// Get current memory usage
    pub fn current_usage(&self) -> usize {
        *self.current_in_use.lock()
    }

    /// Get peak memory usage
    pub fn peak_usage(&self) -> usize {
        *self.peak_usage.lock()
    }

    /// Get total allocated
    pub fn total_allocated(&self) -> usize {
        *self.current_allocated.lock()
    }

    /// Get total deallocated
    pub fn total_deallocated(&self) -> usize {
        *self.current_deallocated.lock()
    }

    /// Print statistics
    pub fn print_stats(&self) {
        println!("\nMemory Profile:");
        println!("Total allocated:   {} bytes", self.total_allocated());
        println!("Total deallocated: {} bytes", self.total_deallocated());
        println!("Current in use:    {} bytes", self.current_usage());
        println!("Peak usage:        {} bytes", self.peak_usage());
    }
}

impl Default for MemoryProfiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for MemoryProfiler {
    fn clone(&self) -> Self {
        Self {
            snapshots: self.snapshots.clone(),
            active: self.active.clone(),
            current_allocated: self.current_allocated.clone(),
            current_deallocated: self.current_deallocated.clone(),
            current_in_use: self.current_in_use.clone(),
            peak_usage: self.peak_usage.clone(),
        }
    }
}

impl Profiler for MemoryProfiler {
    fn start(&mut self) {
        *self.active.lock() = true;
    }

    fn stop(&mut self) {
        *self.active.lock() = false;
    }

    fn reset(&mut self) {
        self.snapshots.lock().clear();
        *self.current_allocated.lock() = 0;
        *self.current_deallocated.lock() = 0;
        *self.current_in_use.lock() = 0;
        *self.peak_usage.lock() = 0;
    }

    fn is_active(&self) -> bool {
        *self.active.lock()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_profiler() {
        let mut profiler = MemoryProfiler::new();
        profiler.start();

        profiler.record_alloc(1024);
        profiler.record_alloc(2048);

        assert_eq!(profiler.total_allocated(), 3072);
        assert_eq!(profiler.current_usage(), 3072);

        profiler.record_dealloc(1024);
        assert_eq!(profiler.current_usage(), 2048);
    }

    #[test]
    fn test_peak_usage() {
        let mut profiler = MemoryProfiler::new();
        profiler.start();

        profiler.record_alloc(1000);
        profiler.record_alloc(2000);
        assert_eq!(profiler.peak_usage(), 3000);

        profiler.record_dealloc(2000);
        assert_eq!(profiler.peak_usage(), 3000); // Peak remains
        assert_eq!(profiler.current_usage(), 1000);
    }

    #[test]
    fn test_snapshots() {
        let mut profiler = MemoryProfiler::new();
        profiler.start();

        profiler.record_alloc(1024);
        profiler.record_snapshot();

        profiler.record_alloc(2048);
        profiler.record_snapshot();

        let snapshots = profiler.snapshots();
        assert_eq!(snapshots.len(), 2);
        assert_eq!(snapshots[0].in_use, 1024);
        assert_eq!(snapshots[1].in_use, 3072);
    }
}
