//! Timing utilities for performance measurement

use std::time::{Duration, Instant};

/// A timer for measuring elapsed time
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

    /// Start a new timer
    pub fn start(name: impl Into<String>) -> Self {
        Self::new(name)
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Get elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> f64 {
        self.elapsed().as_secs_f64() * 1000.0
    }

    /// Get elapsed time in microseconds
    pub fn elapsed_us(&self) -> f64 {
        self.elapsed().as_secs_f64() * 1_000_000.0
    }

    /// Get elapsed time in seconds
    pub fn elapsed_secs(&self) -> f64 {
        self.elapsed().as_secs_f64()
    }

    /// Reset the timer
    pub fn reset(&mut self) {
        self.start = Instant::now();
    }

    /// Stop the timer and return elapsed time in milliseconds
    pub fn stop(self) -> f64 {
        let elapsed = self.elapsed_ms();
        tracing::debug!("{} took {:.2}ms", self.name, elapsed);
        elapsed
    }

    /// Get the timer name
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// A timing guard that logs elapsed time when dropped
pub struct TimingGuard {
    name: String,
    start: Instant,
    log_on_drop: bool,
}

impl TimingGuard {
    /// Create a new timing guard
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            start: Instant::now(),
            log_on_drop: true,
        }
    }

    /// Create without logging on drop
    pub fn silent(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            start: Instant::now(),
            log_on_drop: false,
        }
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Get elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> f64 {
        self.elapsed().as_secs_f64() * 1000.0
    }

    /// Stop and consume the guard, returning elapsed time
    pub fn stop(mut self) -> f64 {
        self.log_on_drop = false;
        self.elapsed_ms()
    }
}

impl Drop for TimingGuard {
    fn drop(&mut self) {
        if self.log_on_drop {
            let elapsed = self.elapsed_ms();
            tracing::info!("{} completed in {:.2}ms", self.name, elapsed);
        }
    }
}

/// Time a closure and return the result and elapsed time
pub fn time<F, R>(name: &str, f: F) -> (R, Duration)
where
    F: FnOnce() -> R,
{
    let start = Instant::now();
    let result = f();
    let elapsed = start.elapsed();

    tracing::debug!("{} took {:.2}ms", name, elapsed.as_secs_f64() * 1000.0);

    (result, elapsed)
}

/// Time an async closure and return the result and elapsed time
pub async fn time_async<F, R>(name: &str, f: F) -> (R, Duration)
where
    F: std::future::Future<Output = R>,
{
    let start = Instant::now();
    let result = f.await;
    let elapsed = start.elapsed();

    tracing::debug!("{} took {:.2}ms", name, elapsed.as_secs_f64() * 1000.0);

    (result, elapsed)
}

/// Macro to time a block of code
#[macro_export]
macro_rules! timed {
    ($name:expr, $block:block) => {{
        let _timer = $crate::timing::TimingGuard::new($name);
        $block
    }};
}

/// Macro to time a block and get the elapsed time
#[macro_export]
macro_rules! timed_result {
    ($name:expr, $block:block) => {{
        let timer = $crate::timing::TimingGuard::silent($name);
        let result = $block;
        let elapsed = timer.stop();
        (result, elapsed)
    }};
}

/// A stopwatch for measuring multiple intervals
pub struct Stopwatch {
    start: Instant,
    laps: Vec<Duration>,
}

impl Stopwatch {
    /// Create a new stopwatch
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            laps: Vec::new(),
        }
    }

    /// Start the stopwatch (or reset it)
    pub fn start() -> Self {
        Self::new()
    }

    /// Record a lap time
    pub fn lap(&mut self) -> Duration {
        let elapsed = self.start.elapsed();
        self.laps.push(elapsed);
        elapsed
    }

    /// Get the current elapsed time
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Get all lap times
    pub fn laps(&self) -> &[Duration] {
        &self.laps
    }

    /// Get the number of laps
    pub fn lap_count(&self) -> usize {
        self.laps.len()
    }

    /// Reset the stopwatch
    pub fn reset(&mut self) {
        self.start = Instant::now();
        self.laps.clear();
    }

    /// Get lap intervals (time between laps)
    pub fn lap_intervals(&self) -> Vec<Duration> {
        let mut intervals = Vec::new();
        let mut last = Duration::ZERO;

        for &lap in &self.laps {
            intervals.push(lap - last);
            last = lap;
        }

        intervals
    }
}

impl Default for Stopwatch {
    fn default() -> Self {
        Self::new()
    }
}

/// Format a duration as a human-readable string
pub fn format_duration(duration: Duration) -> String {
    let total_ms = duration.as_secs_f64() * 1000.0;

    if total_ms < 1.0 {
        format!("{:.2}µs", total_ms * 1000.0)
    } else if total_ms < 1000.0 {
        format!("{:.2}ms", total_ms)
    } else {
        format!("{:.2}s", total_ms / 1000.0)
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
        assert!(timer.elapsed_ms() >= 10.0);
    }

    #[test]
    fn test_timing_guard() {
        let guard = TimingGuard::new("test");
        thread::sleep(Duration::from_millis(10));
        let elapsed = guard.stop();
        assert!(elapsed >= 10.0);
    }

    #[test]
    fn test_time_function() {
        let (result, duration) = time("test", || {
            thread::sleep(Duration::from_millis(10));
            42
        });

        assert_eq!(result, 42);
        assert!(duration.as_millis() >= 10);
    }

    #[test]
    fn test_stopwatch() {
        let mut stopwatch = Stopwatch::new();

        thread::sleep(Duration::from_millis(10));
        stopwatch.lap();

        thread::sleep(Duration::from_millis(10));
        stopwatch.lap();

        assert_eq!(stopwatch.lap_count(), 2);
        assert!(stopwatch.elapsed().as_millis() >= 20);

        let intervals = stopwatch.lap_intervals();
        assert_eq!(intervals.len(), 2);
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_micros(500)), "0.50µs");
        assert_eq!(format_duration(Duration::from_millis(50)), "50.00ms");
        assert_eq!(format_duration(Duration::from_secs(2)), "2000.00ms");
    }
}
