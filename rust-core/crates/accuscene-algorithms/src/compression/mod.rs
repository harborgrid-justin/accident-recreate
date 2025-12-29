//! Compression algorithms for AccuScene data.
//!
//! This module provides various compression algorithms optimized for different
//! types of scene data:
//! - LZ4 streaming for real-time compression
//! - Zstd with dictionary training for scene data
//! - Delta encoding for incremental updates
//! - Run-length encoding for sparse data
//! - Adaptive algorithm selection

pub mod adaptive;
pub mod delta;
pub mod lz4_stream;
pub mod rle;
pub mod zstd_dict;

pub use adaptive::AdaptiveCompressor;
pub use delta::DeltaEncoder;
pub use lz4_stream::Lz4Stream;
pub use rle::RunLengthEncoder;
pub use zstd_dict::ZstdDictionary;

use crate::error::Result;

/// Trait for compression algorithms.
pub trait Compressor {
    /// Compress data.
    fn compress(&mut self, input: &[u8]) -> Result<Vec<u8>>;

    /// Decompress data.
    fn decompress(&mut self, input: &[u8]) -> Result<Vec<u8>>;

    /// Get compression ratio estimate.
    fn compression_ratio(&self) -> f64;
}

/// Compression statistics.
#[derive(Debug, Clone, Default)]
pub struct CompressionStats {
    /// Total bytes compressed.
    pub bytes_in: u64,
    /// Total bytes after compression.
    pub bytes_out: u64,
    /// Number of compression operations.
    pub operations: u64,
}

impl CompressionStats {
    /// Calculate overall compression ratio.
    pub fn ratio(&self) -> f64 {
        if self.bytes_out == 0 {
            0.0
        } else {
            self.bytes_in as f64 / self.bytes_out as f64
        }
    }

    /// Calculate space savings percentage.
    pub fn savings(&self) -> f64 {
        if self.bytes_in == 0 {
            0.0
        } else {
            (1.0 - (self.bytes_out as f64 / self.bytes_in as f64)) * 100.0
        }
    }

    /// Record a compression operation.
    pub fn record(&mut self, input_size: usize, output_size: usize) {
        self.bytes_in += input_size as u64;
        self.bytes_out += output_size as u64;
        self.operations += 1;
    }

    /// Reset statistics.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}
