//! Benchmark utilities for algorithm performance testing.

use std::time::{Duration, Instant};

/// Benchmark result.
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    /// Operation name.
    pub name: String,
    /// Number of iterations.
    pub iterations: usize,
    /// Total duration.
    pub total_duration: Duration,
    /// Average duration per iteration.
    pub avg_duration: Duration,
    /// Operations per second.
    pub ops_per_sec: f64,
    /// Throughput in bytes per second (if applicable).
    pub throughput_bps: Option<f64>,
}

impl BenchmarkResult {
    /// Create a new benchmark result.
    pub fn new(name: String, iterations: usize, total_duration: Duration, bytes: Option<usize>) -> Self {
        let avg_duration = total_duration / iterations as u32;
        let ops_per_sec = iterations as f64 / total_duration.as_secs_f64();
        let throughput_bps = bytes.map(|b| b as f64 / total_duration.as_secs_f64());

        Self {
            name,
            iterations,
            total_duration,
            avg_duration,
            ops_per_sec,
            throughput_bps,
        }
    }

    /// Format result as string.
    pub fn format(&self) -> String {
        let mut result = format!(
            "{}: {} iterations in {:?} ({:?} avg, {:.2} ops/sec",
            self.name,
            self.iterations,
            self.total_duration,
            self.avg_duration,
            self.ops_per_sec
        );

        if let Some(throughput) = self.throughput_bps {
            result.push_str(&format!(", {:.2} MB/s", throughput / 1_000_000.0));
        }

        result.push(')');
        result
    }
}

/// Benchmark runner.
pub struct Benchmark;

impl Benchmark {
    /// Run a benchmark with specified iterations.
    pub fn run<F>(name: &str, iterations: usize, mut f: F) -> BenchmarkResult
    where
        F: FnMut(),
    {
        let start = Instant::now();

        for _ in 0..iterations {
            f();
        }

        let duration = start.elapsed();
        BenchmarkResult::new(name.to_string(), iterations, duration, None)
    }

    /// Run a benchmark with throughput measurement.
    pub fn run_throughput<F>(name: &str, iterations: usize, bytes_per_iteration: usize, mut f: F) -> BenchmarkResult
    where
        F: FnMut(),
    {
        let start = Instant::now();

        for _ in 0..iterations {
            f();
        }

        let duration = start.elapsed();
        let total_bytes = iterations * bytes_per_iteration;
        BenchmarkResult::new(name.to_string(), iterations, duration, Some(total_bytes))
    }

    /// Run a benchmark until minimum duration is reached.
    pub fn run_duration<F>(name: &str, min_duration: Duration, mut f: F) -> BenchmarkResult
    where
        F: FnMut(),
    {
        let mut iterations = 0;
        let start = Instant::now();

        while start.elapsed() < min_duration {
            f();
            iterations += 1;
        }

        let duration = start.elapsed();
        BenchmarkResult::new(name.to_string(), iterations, duration, None)
    }

    /// Compare two implementations.
    pub fn compare<F1, F2>(name1: &str, name2: &str, iterations: usize, mut f1: F1, mut f2: F2) -> (BenchmarkResult, BenchmarkResult, f64)
    where
        F1: FnMut(),
        F2: FnMut(),
    {
        let result1 = Self::run(name1, iterations, &mut f1);
        let result2 = Self::run(name2, iterations, &mut f2);

        let speedup = result1.total_duration.as_secs_f64() / result2.total_duration.as_secs_f64();

        (result1, result2, speedup)
    }

    /// Run warmup iterations before benchmarking.
    pub fn run_with_warmup<F>(name: &str, warmup_iterations: usize, iterations: usize, mut f: F) -> BenchmarkResult
    where
        F: FnMut(),
    {
        // Warmup
        for _ in 0..warmup_iterations {
            f();
        }

        // Actual benchmark
        Self::run(name, iterations, f)
    }
}

/// Benchmark suite for running multiple benchmarks.
pub struct BenchmarkSuite {
    name: String,
    results: Vec<BenchmarkResult>,
}

impl BenchmarkSuite {
    /// Create a new benchmark suite.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            results: Vec::new(),
        }
    }

    /// Add a benchmark to the suite.
    pub fn bench<F>(&mut self, name: &str, iterations: usize, f: F)
    where
        F: FnMut(),
    {
        let result = Benchmark::run(name, iterations, f);
        println!("{}", result.format());
        self.results.push(result);
    }

    /// Add a throughput benchmark.
    pub fn bench_throughput<F>(&mut self, name: &str, iterations: usize, bytes_per_iteration: usize, f: F)
    where
        F: FnMut(),
    {
        let result = Benchmark::run_throughput(name, iterations, bytes_per_iteration, f);
        println!("{}", result.format());
        self.results.push(result);
    }

    /// Get all results.
    pub fn results(&self) -> &[BenchmarkResult] {
        &self.results
    }

    /// Print summary.
    pub fn print_summary(&self) {
        println!("\n=== {} Summary ===", self.name);
        for result in &self.results {
            println!("{}", result.format());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_run() {
        let result = Benchmark::run("test", 1000, || {
            let _ = (1..100).sum::<i32>();
        });

        assert_eq!(result.iterations, 1000);
        assert!(result.ops_per_sec > 0.0);
    }

    #[test]
    fn test_benchmark_throughput() {
        let data_size = 1024;
        let result = Benchmark::run_throughput("throughput_test", 100, data_size, || {
            let _ = vec![0u8; data_size];
        });

        assert_eq!(result.iterations, 100);
        assert!(result.throughput_bps.is_some());
    }

    #[test]
    fn test_benchmark_suite() {
        let mut suite = BenchmarkSuite::new("Test Suite");

        suite.bench("add", 1000, || {
            let _ = 1 + 1;
        });

        suite.bench("multiply", 1000, || {
            let _ = 2 * 2;
        });

        assert_eq!(suite.results().len(), 2);
    }
}
