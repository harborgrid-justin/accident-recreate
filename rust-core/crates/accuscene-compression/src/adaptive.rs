//! Adaptive compression - automatically selects the best algorithm

use crate::error::Result;
use crate::traits::{Algorithm, CompressionLevel, CompressionStats};
use rayon::prelude::*;
use std::time::Instant;
use tracing::{debug, trace};

/// Compression goal for adaptive selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionGoal {
    /// Minimize compressed size (best ratio)
    MinSize,
    /// Minimize compression time (fastest)
    MinTime,
    /// Balance between size and time
    Balanced,
    /// Minimize decompression time
    FastDecompression,
}

/// Adaptive compressor that selects the best algorithm
pub struct AdaptiveCompressor {
    goal: CompressionGoal,
    sample_size: usize,
    algorithms: Vec<Algorithm>,
}

impl AdaptiveCompressor {
    /// Create a new adaptive compressor
    pub fn new(goal: CompressionGoal) -> Self {
        Self {
            goal,
            sample_size: 4096, // Sample first 4KB to test algorithms
            algorithms: vec![
                Algorithm::Lz4,
                Algorithm::Zstd,
                Algorithm::Brotli,
                Algorithm::Deflate,
                Algorithm::Snappy,
            ],
        }
    }

    /// Create with custom sample size
    pub fn with_sample_size(mut self, sample_size: usize) -> Self {
        self.sample_size = sample_size;
        self
    }

    /// Create with specific algorithms to test
    pub fn with_algorithms(mut self, algorithms: Vec<Algorithm>) -> Self {
        self.algorithms = algorithms;
        self
    }

    /// Select the best algorithm for the given data
    pub fn select_algorithm(&self, data: &[u8], level: CompressionLevel) -> Result<Algorithm> {
        // For small data, use sample as is
        let sample = if data.len() <= self.sample_size {
            data
        } else {
            &data[..self.sample_size]
        };

        debug!(
            "Testing {} algorithms on {} byte sample",
            self.algorithms.len(),
            sample.len()
        );

        // Test all algorithms in parallel
        let results: Vec<(Algorithm, CompressionStats)> = self
            .algorithms
            .par_iter()
            .filter_map(|&algo| {
                let stats = self.benchmark_algorithm(sample, algo, level).ok()?;
                Some((algo, stats))
            })
            .collect();

        if results.is_empty() {
            // Fallback to LZ4 if all fail
            return Ok(Algorithm::Lz4);
        }

        // Select based on goal
        let best = match self.goal {
            CompressionGoal::MinSize => results
                .iter()
                .min_by(|a, b| a.1.compressed_size.cmp(&b.1.compressed_size))
                .unwrap(),
            CompressionGoal::MinTime => results
                .iter()
                .min_by(|a, b| a.1.compression_time_ms.cmp(&b.1.compression_time_ms))
                .unwrap(),
            CompressionGoal::Balanced => {
                // Score = (ratio * 0.5) + (time_normalized * 0.5)
                results
                    .iter()
                    .min_by(|a, b| {
                        let score_a = self.balanced_score(&a.1);
                        let score_b = self.balanced_score(&b.1);
                        score_a.partial_cmp(&score_b).unwrap()
                    })
                    .unwrap()
            }
            CompressionGoal::FastDecompression => {
                // Prefer Snappy, LZ4, Zstd in that order
                results
                    .iter()
                    .find(|(algo, _)| *algo == Algorithm::Snappy)
                    .or_else(|| results.iter().find(|(algo, _)| *algo == Algorithm::Lz4))
                    .or_else(|| results.iter().find(|(algo, _)| *algo == Algorithm::Zstd))
                    .unwrap_or(&results[0])
            }
        };

        debug!(
            "Selected {:?} (ratio: {:.2}%, time: {}ms)",
            best.0,
            best.1.ratio * 100.0,
            best.1.compression_time_ms
        );

        Ok(best.0)
    }

    /// Compress with automatic algorithm selection
    pub fn compress(&self, data: &[u8], level: CompressionLevel) -> Result<Vec<u8>> {
        let algorithm = self.select_algorithm(data, level)?;
        trace!("Compressing {} bytes with {:?}", data.len(), algorithm);

        let mut compressed = crate::algorithms::compress(data, algorithm, level)?;

        // Prepend algorithm identifier (1 byte)
        let mut result = Vec::with_capacity(compressed.len() + 1);
        result.push(algorithm as u8);
        result.append(&mut compressed);

        Ok(result)
    }

    /// Decompress data (algorithm is embedded in the data)
    pub fn decompress(data: &[u8]) -> Result<Vec<u8>> {
        if data.is_empty() {
            return Err(crate::error::CompressionError::CorruptedData(
                "Empty data".to_string(),
            ));
        }

        let algorithm = match data[0] {
            0 => Algorithm::Lz4,
            1 => Algorithm::Zstd,
            2 => Algorithm::Brotli,
            3 => Algorithm::Deflate,
            4 => Algorithm::Snappy,
            _ => {
                return Err(crate::error::CompressionError::InvalidAlgorithm(format!(
                    "Unknown algorithm ID: {}",
                    data[0]
                )))
            }
        };

        trace!("Decompressing with {:?}", algorithm);
        crate::algorithms::decompress(&data[1..], algorithm)
    }

    /// Benchmark an algorithm
    fn benchmark_algorithm(
        &self,
        data: &[u8],
        algorithm: Algorithm,
        level: CompressionLevel,
    ) -> Result<CompressionStats> {
        let start = Instant::now();
        let compressed = crate::algorithms::compress(data, algorithm, level)?;
        let compression_time = start.elapsed();

        let start = Instant::now();
        let _ = crate::algorithms::decompress(&compressed, algorithm)?;
        let decompression_time = start.elapsed();

        let mut stats = CompressionStats::new(data.len(), compressed.len());
        stats.compression_time_ms = compression_time.as_millis() as u64;
        stats.decompression_time_ms = decompression_time.as_millis() as u64;
        stats.algorithm = Some(algorithm);

        Ok(stats)
    }

    /// Calculate balanced score (lower is better)
    fn balanced_score(&self, stats: &CompressionStats) -> f64 {
        // Normalize ratio (0-1, where 1 is best)
        let ratio_score = stats.ratio;

        // Normalize time (assume max 1000ms for sample)
        let time_score = (stats.compression_time_ms as f64) / 1000.0;

        // Combined score (lower is better)
        (ratio_score * 0.5) + (time_score * 0.5)
    }
}

impl Default for AdaptiveCompressor {
    fn default() -> Self {
        Self::new(CompressionGoal::Balanced)
    }
}

/// Quick compression with automatic algorithm selection
pub fn compress_auto(data: &[u8]) -> Result<Vec<u8>> {
    let compressor = AdaptiveCompressor::new(CompressionGoal::Balanced);
    compressor.compress(data, CompressionLevel::Default)
}

/// Quick decompression
pub fn decompress_auto(data: &[u8]) -> Result<Vec<u8>> {
    AdaptiveCompressor::decompress(data)
}

/// Analyze data and recommend an algorithm
pub fn analyze_data(data: &[u8]) -> Result<(Algorithm, String)> {
    let compressor = AdaptiveCompressor::new(CompressionGoal::Balanced);

    // Calculate entropy as a simple heuristic
    let entropy = calculate_entropy(data);

    let recommendation = if entropy > 7.5 {
        (
            Algorithm::Snappy,
            "High entropy data - use fast compression".to_string(),
        )
    } else if entropy < 3.0 {
        (
            Algorithm::Brotli,
            "Low entropy data - use high ratio compression".to_string(),
        )
    } else {
        let algo = compressor.select_algorithm(data, CompressionLevel::Default)?;
        (algo, format!("Selected {:?} based on benchmarks", algo))
    };

    Ok(recommendation)
}

/// Calculate Shannon entropy of data
fn calculate_entropy(data: &[u8]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }

    let mut freq = [0usize; 256];
    for &byte in data {
        freq[byte as usize] += 1;
    }

    let len = data.len() as f64;
    let mut entropy = 0.0;

    for &count in &freq {
        if count > 0 {
            let p = count as f64 / len;
            entropy -= p * p.log2();
        }
    }

    entropy
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adaptive_compression() {
        let compressor = AdaptiveCompressor::new(CompressionGoal::Balanced);
        let data = b"Hello, AccuScene! Testing adaptive compression.".repeat(100);

        let compressed = compressor.compress(&data, CompressionLevel::Default).unwrap();
        let decompressed = AdaptiveCompressor::decompress(&compressed).unwrap();

        assert_eq!(data.to_vec(), decompressed);
    }

    #[test]
    fn test_auto_compress() {
        let data = b"Quick test of automatic compression.".repeat(50);

        let compressed = compress_auto(&data).unwrap();
        let decompressed = decompress_auto(&compressed).unwrap();

        assert_eq!(data.to_vec(), decompressed);
    }

    #[test]
    fn test_entropy_calculation() {
        // Low entropy (all zeros)
        let low_entropy = vec![0u8; 1000];
        let entropy = calculate_entropy(&low_entropy);
        assert!(entropy < 1.0);

        // High entropy (random-ish)
        let high_entropy: Vec<u8> = (0..1000).map(|i| (i % 256) as u8).collect();
        let entropy = calculate_entropy(&high_entropy);
        assert!(entropy > 5.0);
    }

    #[test]
    fn test_algorithm_selection() {
        let compressor = AdaptiveCompressor::new(CompressionGoal::MinSize);

        // Test with compressible data
        let compressible = b"AAAAAABBBBBBCCCCCC".repeat(100);
        let algo = compressor
            .select_algorithm(&compressible, CompressionLevel::Default)
            .unwrap();
        assert!(algo == Algorithm::Brotli || algo == Algorithm::Zstd);

        // Test with less compressible data
        let random: Vec<u8> = (0..1000).map(|i| (i % 256) as u8).collect();
        let algo = compressor
            .select_algorithm(&random, CompressionLevel::Default)
            .unwrap();
        // Should select a fast algorithm for less compressible data
        assert!(
            algo == Algorithm::Lz4 || algo == Algorithm::Snappy || algo == Algorithm::Zstd
        );
    }
}
