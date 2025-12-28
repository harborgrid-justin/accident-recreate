//! Built-in benchmarking utilities for compression algorithms

use crate::error::Result;
use crate::traits::{Algorithm, CompressionLevel, CompressionStats};
use std::time::Instant;
use tracing::{debug, info};

/// Benchmark results for a single algorithm
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    /// Algorithm tested
    pub algorithm: Algorithm,
    /// Compression level tested
    pub level: CompressionLevel,
    /// Compression statistics
    pub stats: CompressionStats,
}

impl BenchmarkResult {
    /// Format result as a string
    pub fn format(&self) -> String {
        format!(
            "{:10} | {:8} | {:10} bytes → {:10} bytes | Ratio: {:6.2}% | Comp: {:6.1} MB/s | Decomp: {:6.1} MB/s | Savings: {:5.1}%",
            format!("{:?}", self.algorithm),
            format!("{:?}", self.level),
            self.stats.original_size,
            self.stats.compressed_size,
            self.stats.ratio * 100.0,
            self.stats.compression_throughput(),
            self.stats.decompression_throughput(),
            self.stats.savings_percent()
        )
    }
}

/// Benchmark configuration
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    /// Algorithms to test
    pub algorithms: Vec<Algorithm>,
    /// Compression levels to test
    pub levels: Vec<CompressionLevel>,
    /// Number of iterations per test
    pub iterations: usize,
    /// Warmup iterations before measurement
    pub warmup: usize,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            algorithms: vec![
                Algorithm::Lz4,
                Algorithm::Zstd,
                Algorithm::Brotli,
                Algorithm::Deflate,
                Algorithm::Snappy,
            ],
            levels: vec![
                CompressionLevel::Fastest,
                CompressionLevel::Default,
                CompressionLevel::High,
            ],
            iterations: 10,
            warmup: 2,
        }
    }
}

/// Benchmark a single algorithm and level
pub fn benchmark_algorithm(
    data: &[u8],
    algorithm: Algorithm,
    level: CompressionLevel,
    iterations: usize,
    warmup: usize,
) -> Result<BenchmarkResult> {
    debug!(
        "Benchmarking {:?} at {:?} level ({} iterations, {} warmup)",
        algorithm, level, iterations, warmup
    );

    // Warmup
    for _ in 0..warmup {
        let compressed = crate::algorithms::compress(data, algorithm, level)?;
        let _ = crate::algorithms::decompress(&compressed, algorithm)?;
    }

    let mut compression_times = Vec::with_capacity(iterations);
    let mut decompression_times = Vec::with_capacity(iterations);
    let mut compressed_size = 0;

    // Benchmark iterations
    for _ in 0..iterations {
        // Compression
        let start = Instant::now();
        let compressed = crate::algorithms::compress(data, algorithm, level)?;
        let comp_time = start.elapsed();
        compression_times.push(comp_time.as_micros() as u64);
        compressed_size = compressed.len();

        // Decompression
        let start = Instant::now();
        let _ = crate::algorithms::decompress(&compressed, algorithm)?;
        let decomp_time = start.elapsed();
        decompression_times.push(decomp_time.as_micros() as u64);
    }

    // Calculate average times
    let avg_comp_time = compression_times.iter().sum::<u64>() / iterations as u64;
    let avg_decomp_time = decompression_times.iter().sum::<u64>() / iterations as u64;

    let mut stats = CompressionStats::new(data.len(), compressed_size);
    stats.compression_time_ms = avg_comp_time / 1000;
    stats.decompression_time_ms = avg_decomp_time / 1000;
    stats.algorithm = Some(algorithm);

    Ok(BenchmarkResult {
        algorithm,
        level,
        stats,
    })
}

/// Run comprehensive benchmark on data
pub fn benchmark(data: &[u8], config: BenchmarkConfig) -> Result<Vec<BenchmarkResult>> {
    info!(
        "Running compression benchmark on {} bytes of data",
        data.len()
    );

    let mut results = Vec::new();

    for algorithm in config.algorithms {
        for level in &config.levels {
            let result = benchmark_algorithm(
                data,
                algorithm,
                *level,
                config.iterations,
                config.warmup,
            )?;
            results.push(result);
        }
    }

    Ok(results)
}

/// Print benchmark results in a formatted table
pub fn print_results(results: &[BenchmarkResult]) {
    println!("\n{}", "=".repeat(140));
    println!("COMPRESSION BENCHMARK RESULTS");
    println!("{}", "=".repeat(140));
    println!(
        "{:10} | {:8} | {:23} | {:16} | {:25} | {:9}",
        "Algorithm", "Level", "Size (Original → Compressed)", "Ratio", "Throughput (Comp | Decomp)", "Savings"
    );
    println!("{}", "-".repeat(140));

    for result in results {
        println!("{}", result.format());
    }

    println!("{}", "=".repeat(140));
}

/// Find the best algorithm for the given data and optimization goal
#[derive(Debug, Clone, Copy)]
pub enum OptimizationGoal {
    /// Best compression ratio
    BestRatio,
    /// Fastest compression
    FastestCompression,
    /// Fastest decompression
    FastestDecompression,
    /// Balanced performance
    Balanced,
}

pub fn find_best_algorithm(
    results: &[BenchmarkResult],
    goal: OptimizationGoal,
) -> Option<&BenchmarkResult> {
    match goal {
        OptimizationGoal::BestRatio => results
            .iter()
            .min_by(|a, b| a.stats.ratio.partial_cmp(&b.stats.ratio).unwrap()),
        OptimizationGoal::FastestCompression => results
            .iter()
            .max_by(|a, b| {
                a.stats
                    .compression_throughput()
                    .partial_cmp(&b.stats.compression_throughput())
                    .unwrap()
            }),
        OptimizationGoal::FastestDecompression => results
            .iter()
            .max_by(|a, b| {
                a.stats
                    .decompression_throughput()
                    .partial_cmp(&b.stats.decompression_throughput())
                    .unwrap()
            }),
        OptimizationGoal::Balanced => {
            // Score = (1 - ratio) * comp_throughput * decomp_throughput
            results.iter().max_by(|a, b| {
                let score_a = (1.0 - a.stats.ratio)
                    * a.stats.compression_throughput()
                    * a.stats.decompression_throughput();
                let score_b = (1.0 - b.stats.ratio)
                    * b.stats.compression_throughput()
                    * b.stats.decompression_throughput();
                score_a.partial_cmp(&score_b).unwrap()
            })
        }
    }
}

/// Quick benchmark with default settings
pub fn quick_benchmark(data: &[u8]) -> Result<Vec<BenchmarkResult>> {
    let config = BenchmarkConfig {
        iterations: 5,
        warmup: 1,
        ..Default::default()
    };
    benchmark(data, config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_single() {
        let data = b"Test data for benchmarking compression algorithms".repeat(100);

        let result = benchmark_algorithm(
            &data,
            Algorithm::Lz4,
            CompressionLevel::Default,
            5,
            1,
        )
        .unwrap();

        assert_eq!(result.algorithm, Algorithm::Lz4);
        assert!(result.stats.compressed_size < result.stats.original_size);
        assert!(result.stats.compression_time_ms > 0);
    }

    #[test]
    fn test_benchmark_all() {
        let data = b"Benchmark test data".repeat(50);

        let config = BenchmarkConfig {
            algorithms: vec![Algorithm::Lz4, Algorithm::Zstd],
            levels: vec![CompressionLevel::Fast, CompressionLevel::Default],
            iterations: 3,
            warmup: 1,
        };

        let results = benchmark(&data, config).unwrap();

        assert_eq!(results.len(), 4); // 2 algorithms x 2 levels
    }

    #[test]
    fn test_find_best_ratio() {
        let data = b"Finding best compression ratio".repeat(100);
        let results = quick_benchmark(&data).unwrap();

        let best = find_best_algorithm(&results, OptimizationGoal::BestRatio);
        assert!(best.is_some());

        // Should typically be Brotli or Zstd for best ratio
        let best = best.unwrap();
        assert!(
            best.algorithm == Algorithm::Brotli || best.algorithm == Algorithm::Zstd
        );
    }

    #[test]
    fn test_find_fastest() {
        let data = b"Finding fastest compression".repeat(100);
        let results = quick_benchmark(&data).unwrap();

        let best = find_best_algorithm(&results, OptimizationGoal::FastestCompression);
        assert!(best.is_some());

        // Should typically be LZ4 or Snappy
        let best = best.unwrap();
        assert!(
            best.algorithm == Algorithm::Lz4 || best.algorithm == Algorithm::Snappy
        );
    }
}
