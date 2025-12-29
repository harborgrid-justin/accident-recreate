//! CPU profiling utilities

use crate::profiling::Profiler;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// CPU sample
#[derive(Debug, Clone)]
pub struct CpuSample {
    /// Function name
    pub name: String,
    /// Time spent
    pub duration: Duration,
    /// Timestamp
    pub timestamp: Instant,
}

/// CPU profiler
pub struct CpuProfiler {
    samples: Arc<Mutex<Vec<CpuSample>>>,
    active: Arc<Mutex<bool>>,
    start_time: Arc<Mutex<Option<Instant>>>,
}

impl CpuProfiler {
    /// Create a new CPU profiler
    pub fn new() -> Self {
        Self {
            samples: Arc::new(Mutex::new(Vec::new())),
            active: Arc::new(Mutex::new(false)),
            start_time: Arc::new(Mutex::new(None)),
        }
    }

    /// Record a sample
    pub fn record(&self, name: String, duration: Duration) {
        if !self.is_active() {
            return;
        }

        let sample = CpuSample {
            name,
            duration,
            timestamp: Instant::now(),
        };

        self.samples.lock().push(sample);
    }

    /// Get all samples
    pub fn samples(&self) -> Vec<CpuSample> {
        self.samples.lock().clone()
    }

    /// Get aggregated statistics
    pub fn stats(&self) -> HashMap<String, CpuStats> {
        let samples = self.samples.lock();
        let mut stats_map: HashMap<String, CpuStats> = HashMap::new();

        for sample in samples.iter() {
            let stats = stats_map.entry(sample.name.clone()).or_insert_with(CpuStats::new);
            stats.record(sample.duration);
        }

        stats_map
    }

    /// Print statistics
    pub fn print_stats(&self) {
        let stats = self.stats();

        println!("\nCPU Profile:");
        println!("{:<40} {:>10} {:>12} {:>12} {:>12}", "Function", "Count", "Total (ms)", "Avg (µs)", "Max (µs)");
        println!("{}", "-".repeat(90));

        let mut entries: Vec<_> = stats.iter().collect();
        entries.sort_by(|a, b| b.1.total.cmp(&a.1.total));

        for (name, stat) in entries {
            println!(
                "{:<40} {:>10} {:>12.2} {:>12.2} {:>12.2}",
                name,
                stat.count,
                stat.total.as_secs_f64() * 1000.0,
                stat.avg().as_micros(),
                stat.max.as_micros()
            );
        }
    }

    /// Create a scoped profiler
    pub fn scope(&self, name: impl Into<String>) -> CpuProfileScope {
        CpuProfileScope {
            profiler: self.clone(),
            name: name.into(),
            start: Instant::now(),
        }
    }
}

impl Default for CpuProfiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for CpuProfiler {
    fn clone(&self) -> Self {
        Self {
            samples: self.samples.clone(),
            active: self.active.clone(),
            start_time: self.start_time.clone(),
        }
    }
}

impl Profiler for CpuProfiler {
    fn start(&mut self) {
        *self.active.lock() = true;
        *self.start_time.lock() = Some(Instant::now());
    }

    fn stop(&mut self) {
        *self.active.lock() = false;
    }

    fn reset(&mut self) {
        self.samples.lock().clear();
        *self.start_time.lock() = None;
    }

    fn is_active(&self) -> bool {
        *self.active.lock()
    }
}

/// CPU statistics
#[derive(Debug, Clone)]
pub struct CpuStats {
    pub count: u64,
    pub total: Duration,
    pub min: Duration,
    pub max: Duration,
}

impl CpuStats {
    fn new() -> Self {
        Self {
            count: 0,
            total: Duration::ZERO,
            min: Duration::MAX,
            max: Duration::ZERO,
        }
    }

    fn record(&mut self, duration: Duration) {
        self.count += 1;
        self.total += duration;
        self.min = self.min.min(duration);
        self.max = self.max.max(duration);
    }

    fn avg(&self) -> Duration {
        if self.count > 0 {
            self.total / self.count as u32
        } else {
            Duration::ZERO
        }
    }
}

/// Scoped CPU profiler
pub struct CpuProfileScope {
    profiler: CpuProfiler,
    name: String,
    start: Instant,
}

impl Drop for CpuProfileScope {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        self.profiler.record(self.name.clone(), duration);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_cpu_profiler() {
        let mut profiler = CpuProfiler::new();
        profiler.start();

        profiler.record("test_fn".to_string(), Duration::from_millis(10));
        profiler.record("test_fn".to_string(), Duration::from_millis(20));

        let stats = profiler.stats();
        assert_eq!(stats.get("test_fn").unwrap().count, 2);
    }

    #[test]
    fn test_cpu_profile_scope() {
        let mut profiler = CpuProfiler::new();
        profiler.start();

        {
            let _scope = profiler.scope("test_scope");
            thread::sleep(Duration::from_millis(10));
        }

        let samples = profiler.samples();
        assert_eq!(samples.len(), 1);
        assert_eq!(samples[0].name, "test_scope");
    }
}
