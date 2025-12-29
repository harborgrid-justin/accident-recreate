//! Adaptive compression algorithm selection.
//!
//! Automatically selects the best compression algorithm based on data
//! characteristics and content analysis.
//!
//! # Algorithm Selection
//! - Sparse data (>50% zeros) -> RLE
//! - Highly repetitive (>70% repeats) -> Zstd with dictionary
//! - Sequential/incremental data -> Delta + LZ4
//! - General purpose -> LZ4 or Zstd based on size
//!
//! # Complexity
//! - Analysis: O(n) where n is input size
//! - Compression: Depends on selected algorithm

use crate::compression::{
    delta::DeltaEncoder, lz4_stream::Lz4Stream, rle::RunLengthEncoder,
    zstd_dict::ZstdDictionary, CompressionStats, Compressor,
};
use crate::config::CompressionConfig;
use crate::error::{AlgorithmError, Result};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

const SPARSE_THRESHOLD: f64 = 0.5;
const REPETITIVE_THRESHOLD: f64 = 0.7;
const SEQUENTIAL_THRESHOLD: f64 = 0.6;

/// Compression algorithm type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Algorithm {
    /// No compression.
    None = 0,
    /// LZ4 compression.
    Lz4 = 1,
    /// Zstandard compression.
    Zstd = 2,
    /// Run-length encoding.
    Rle = 3,
    /// Delta encoding + LZ4.
    Delta = 4,
}

impl Algorithm {
    /// Convert from byte tag.
    pub fn from_u8(tag: u8) -> Option<Self> {
        match tag {
            0 => Some(Algorithm::None),
            1 => Some(Algorithm::Lz4),
            2 => Some(Algorithm::Zstd),
            3 => Some(Algorithm::Rle),
            4 => Some(Algorithm::Delta),
            _ => None,
        }
    }
}

/// Data characteristics for algorithm selection.
#[derive(Debug, Clone)]
struct DataProfile {
    sparsity: f64,
    repetitiveness: f64,
    sequentiality: f64,
    entropy: f64,
}

impl DataProfile {
    /// Analyze data to create profile.
    ///
    /// # Complexity
    /// O(n) where n is data size
    fn analyze(data: &[u8]) -> Self {
        if data.is_empty() {
            return Self {
                sparsity: 0.0,
                repetitiveness: 0.0,
                sequentiality: 0.0,
                entropy: 0.0,
            };
        }

        // Calculate sparsity (percentage of zeros)
        let zero_count = data.iter().filter(|&&b| b == 0).count();
        let sparsity = zero_count as f64 / data.len() as f64;

        // Calculate repetitiveness (percentage of repeated consecutive bytes)
        let mut repeat_count = 0;
        for i in 1..data.len() {
            if data[i] == data[i - 1] {
                repeat_count += 1;
            }
        }
        let repetitiveness = repeat_count as f64 / (data.len() - 1).max(1) as f64;

        // Calculate sequentiality (percentage of sequential values)
        let mut sequential_count = 0;
        for i in 1..data.len() {
            let diff = (data[i] as i16 - data[i - 1] as i16).abs();
            if diff <= 1 {
                sequential_count += 1;
            }
        }
        let sequentiality = sequential_count as f64 / (data.len() - 1).max(1) as f64;

        // Calculate Shannon entropy
        let mut counts = [0u32; 256];
        for &byte in data {
            counts[byte as usize] += 1;
        }
        let mut entropy = 0.0;
        for &count in &counts {
            if count > 0 {
                let p = count as f64 / data.len() as f64;
                entropy -= p * p.log2();
            }
        }

        Self {
            sparsity,
            repetitiveness,
            sequentiality,
            entropy,
        }
    }

    /// Select best algorithm based on profile.
    fn select_algorithm(&self, size: usize) -> Algorithm {
        // Very sparse data -> RLE
        if self.sparsity > SPARSE_THRESHOLD {
            return Algorithm::Rle;
        }

        // Highly repetitive data -> Zstd
        if self.repetitiveness > REPETITIVE_THRESHOLD {
            return Algorithm::Zstd;
        }

        // Sequential data -> Delta encoding
        if self.sequentiality > SEQUENTIAL_THRESHOLD {
            return Algorithm::Delta;
        }

        // For small data, use LZ4 for speed
        if size < 4096 {
            return Algorithm::Lz4;
        }

        // For larger data with low entropy, use Zstd
        if self.entropy < 4.0 {
            return Algorithm::Zstd;
        }

        // Default to LZ4
        Algorithm::Lz4
    }
}

/// Adaptive compressor that selects the best algorithm.
pub struct AdaptiveCompressor {
    lz4: Lz4Stream,
    zstd: ZstdDictionary,
    rle: RunLengthEncoder,
    delta: DeltaEncoder,
    config: CompressionConfig,
    stats: Arc<RwLock<CompressionStats>>,
    forced_algorithm: Option<Algorithm>,
}

impl AdaptiveCompressor {
    /// Create a new adaptive compressor.
    pub fn new(config: CompressionConfig) -> Self {
        Self {
            lz4: Lz4Stream::new(config.level.lz4_acceleration()),
            zstd: ZstdDictionary::new(config.clone()),
            rle: RunLengthEncoder::new(),
            delta: DeltaEncoder::new(),
            config,
            stats: Arc::new(RwLock::new(CompressionStats::default())),
            forced_algorithm: None,
        }
    }

    /// Create with default configuration.
    pub fn default() -> Self {
        Self::new(CompressionConfig::default())
    }

    /// Force use of specific algorithm (disables adaptive selection).
    pub fn force_algorithm(&mut self, algorithm: Algorithm) {
        self.forced_algorithm = Some(algorithm);
    }

    /// Clear forced algorithm (re-enable adaptive selection).
    pub fn clear_forced_algorithm(&mut self) {
        self.forced_algorithm = None;
    }

    /// Compress data with adaptive algorithm selection.
    ///
    /// Returns (compressed_data, algorithm_used).
    pub fn compress_adaptive(&mut self, input: &[u8]) -> Result<(Vec<u8>, Algorithm)> {
        if input.is_empty() {
            return Ok((Vec::new(), Algorithm::None));
        }

        // Select algorithm
        let algorithm = if let Some(forced) = self.forced_algorithm {
            forced
        } else {
            let profile = DataProfile::analyze(input);
            profile.select_algorithm(input.len())
        };

        // Compress with selected algorithm
        let compressed = match algorithm {
            Algorithm::None => input.to_vec(),
            Algorithm::Lz4 => self.lz4.compress_block(input)?,
            Algorithm::Zstd => self.zstd.compress_with_dict(input)?,
            Algorithm::Rle => self.rle.encode(input)?,
            Algorithm::Delta => self.delta.encode_i32(
                &input
                    .chunks_exact(4)
                    .map(|c| i32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                    .collect::<Vec<_>>(),
            )?,
        };

        // Add algorithm tag
        let mut result = Vec::with_capacity(1 + compressed.len());
        result.push(algorithm as u8);
        result.extend_from_slice(&compressed);

        self.stats.lock().record(input.len(), result.len());

        Ok((result, algorithm))
    }

    /// Decompress data that was compressed with adaptive selection.
    pub fn decompress_adaptive(&mut self, input: &[u8]) -> Result<Vec<u8>> {
        if input.is_empty() {
            return Ok(Vec::new());
        }

        if input.len() < 1 {
            return Err(AlgorithmError::InvalidFormat(
                "Compressed data too small".to_string(),
            ));
        }

        // Read algorithm tag
        let algorithm = Algorithm::from_u8(input[0]).ok_or_else(|| {
            AlgorithmError::InvalidFormat(format!("Unknown algorithm tag: {}", input[0]))
        })?;

        let compressed = &input[1..];

        // Decompress with appropriate algorithm
        let decompressed = match algorithm {
            Algorithm::None => compressed.to_vec(),
            Algorithm::Lz4 => self.lz4.decompress_block(compressed)?,
            Algorithm::Zstd => self.zstd.decompress_with_dict(compressed)?,
            Algorithm::Rle => self.rle.decode(compressed)?,
            Algorithm::Delta => {
                let values = self.delta.decode_i32(compressed)?;
                let mut result = Vec::with_capacity(values.len() * 4);
                for value in values {
                    result.extend_from_slice(&value.to_le_bytes());
                }
                result
            }
        };

        Ok(decompressed)
    }

    /// Train Zstd dictionary from samples.
    pub fn train_dictionary(&self, samples: &[&[u8]], dict_size: usize) -> Result<()> {
        self.zstd.train_dictionary(samples, dict_size)
    }

    /// Get compression statistics.
    pub fn stats(&self) -> CompressionStats {
        self.stats.read().clone()
    }

    /// Reset statistics.
    pub fn reset_stats(&self) {
        self.stats.write().reset();
    }
}

impl Clone for AdaptiveCompressor {
    fn clone(&self) -> Self {
        Self {
            lz4: self.lz4.clone(),
            zstd: self.zstd.clone(),
            rle: self.rle.clone(),
            delta: self.delta.clone(),
            config: self.config.clone(),
            stats: Arc::new(RwLock::new(self.stats.read().clone())),
            forced_algorithm: self.forced_algorithm,
        }
    }
}

impl Compressor for AdaptiveCompressor {
    fn compress(&mut self, input: &[u8]) -> Result<Vec<u8>> {
        let (compressed, _) = self.compress_adaptive(input)?;
        Ok(compressed)
    }

    fn decompress(&mut self, input: &[u8]) -> Result<Vec<u8>> {
        self.decompress_adaptive(input)
    }

    fn compression_ratio(&self) -> f64 {
        self.stats.read().ratio()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adaptive_compression() {
        let mut compressor = AdaptiveCompressor::default();
        let data = b"Hello, World! This is a test.".repeat(10);

        let (compressed, _) = compressor.compress_adaptive(&data).unwrap();
        assert!(compressed.len() < data.len());

        let decompressed = compressor.decompress_adaptive(&compressed).unwrap();
        assert_eq!(data, decompressed.as_slice());
    }

    #[test]
    fn test_sparse_data_selection() {
        let mut compressor = AdaptiveCompressor::default();
        let mut data = vec![0u8; 10000];
        data[100] = 1;
        data[200] = 1;

        let (compressed, algorithm) = compressor.compress_adaptive(&data).unwrap();
        assert_eq!(algorithm, Algorithm::Rle);

        let decompressed = compressor.decompress_adaptive(&compressed).unwrap();
        assert_eq!(data, decompressed);
    }

    #[test]
    fn test_forced_algorithm() {
        let mut compressor = AdaptiveCompressor::default();
        compressor.force_algorithm(Algorithm::Lz4);

        let data = vec![0u8; 1000];
        let (_, algorithm) = compressor.compress_adaptive(&data).unwrap();
        assert_eq!(algorithm, Algorithm::Lz4);
    }
}
