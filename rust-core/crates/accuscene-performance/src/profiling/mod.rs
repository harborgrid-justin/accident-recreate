//! Profiling utilities for performance analysis

pub mod cpu;
pub mod flamegraph;
pub mod latency;
pub mod memory;

pub use cpu::{CpuProfiler, CpuSample};
pub use flamegraph::FlamegraphGenerator;
pub use latency::{LatencyHistogram, LatencyTracker};
pub use memory::{MemoryProfiler, MemorySnapshot};

use std::time::{Duration, Instant};

/// Profiler trait
pub trait Profiler {
    /// Start profiling
    fn start(&mut self);

    /// Stop profiling
    fn stop(&mut self);

    /// Reset profiling data
    fn reset(&mut self);

    /// Check if profiling is active
    fn is_active(&self) -> bool;
}

/// Simple timer for measuring execution time
pub struct Timer {
    start: Instant,
    name: String,
}

impl Timer {
    /// Create a new timer
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            start: Instant::now(),
            name: name.into(),
        }
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Get elapsed time in microseconds
    pub fn elapsed_micros(&self) -> u64 {
        self.elapsed().as_micros() as u64
    }

    /// Get elapsed time in nanoseconds
    pub fn elapsed_nanos(&self) -> u64 {
        self.elapsed().as_nanos() as u64
    }

    /// Print elapsed time
    pub fn print_elapsed(&self) {
        println!("{}: {:?}", self.name, self.elapsed());
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        if std::thread::panicking() {
            return;
        }
        self.print_elapsed();
    }
}

/// Scope guard for automatic timing
#[macro_export]
macro_rules! time_scope {
    ($name:expr) => {
        let _timer = $crate::profiling::Timer::new($name);
    };
}

/// Performance counter
#[derive(Debug, Default)]
pub struct PerfCounter {
    count: u64,
    total_time: Duration,
    min_time: Option<Duration>,
    max_time: Option<Duration>,
}

impl PerfCounter {
    /// Create a new performance counter
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a measurement
    pub fn record(&mut self, duration: Duration) {
        self.count += 1;
        self.total_time += duration;

        if self.min_time.is_none() || Some(duration) < self.min_time {
            self.min_time = Some(duration);
        }

        if self.max_time.is_none() || Some(duration) > self.max_time {
            self.max_time = Some(duration);
        }
    }

    /// Get count
    pub fn count(&self) -> u64 {
        self.count
    }

    /// Get average time
    pub fn avg_time(&self) -> Option<Duration> {
        if self.count > 0 {
            Some(self.total_time / self.count as u32)
        } else {
            None
        }
    }

    /// Get min time
    pub fn min_time(&self) -> Option<Duration> {
        self.min_time
    }

    /// Get max time
    pub fn max_time(&self) -> Option<Duration> {
        self.max_time
    }

    /// Get total time
    pub fn total_time(&self) -> Duration {
        self.total_time
    }

    /// Reset the counter
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_timer() {
        let timer = Timer::new("test");
        thread::sleep(Duration::from_millis(10));
        assert!(timer.elapsed() >= Duration::from_millis(10));
    }

    #[test]
    fn test_perf_counter() {
        let mut counter = PerfCounter::new();

        counter.record(Duration::from_millis(10));
        counter.record(Duration::from_millis(20));
        counter.record(Duration::from_millis(30));

        assert_eq!(counter.count(), 3);
        assert_eq!(counter.min_time(), Some(Duration::from_millis(10)));
        assert_eq!(counter.max_time(), Some(Duration::from_millis(30)));
        assert_eq!(counter.avg_time(), Some(Duration::from_millis(20)));
    }

    #[test]
    fn test_counter_reset() {
        let mut counter = PerfCounter::new();

        counter.record(Duration::from_millis(10));
        assert_eq!(counter.count(), 1);

        counter.reset();
        assert_eq!(counter.count(), 0);
    }
}
