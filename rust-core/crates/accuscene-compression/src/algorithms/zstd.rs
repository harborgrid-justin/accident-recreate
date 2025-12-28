//! Zstandard compression implementation - Balanced compression speed and ratio

use crate::error::{CompressionError, Result};
use crate::traits::{Algorithm, CompressionLevel, Compressor as CompressorTrait};
use tracing::{debug, trace};
use zstd::bulk::{compress, decompress, Compressor, Decompressor};

/// Zstandard compression implementation
#[derive(Debug, Clone)]
pub struct ZstdCompressor {
    /// Default compression level
    default_level: i32,
}

impl ZstdCompressor {
    /// Create a new Zstandard compressor
    pub fn new() -> Self {
        Self {
            default_level: 3, // Default level for good balance
        }
    }

    /// Create with custom default level
    pub fn with_level(level: i32) -> Self {
        Self {
            default_level: level,
        }
    }

    /// Train a dictionary from sample data
    pub fn train_dictionary(samples: &[&[u8]], dict_size: usize) -> Result<Vec<u8>> {
        trace!("Training Zstandard dictionary with {} samples", samples.len());

        // Concatenate samples into continuous buffer for zstd
        let mut continuous = Vec::new();
        for sample in samples {
            continuous.extend_from_slice(sample);
        }

        zstd::dict::from_continuous(&continuous, &[], dict_size)
            .map_err(|e| CompressionError::Dictionary(e.to_string()))
    }

    /// Compress with dictionary
    pub fn compress_with_dict(&self, data: &[u8], dict: &[u8], level: i32) -> Result<Vec<u8>> {
        trace!("Zstandard compressing {} bytes with dictionary", data.len());

        let mut encoder = zstd::bulk::Compressor::with_dictionary(level, dict)
            .map_err(|e| CompressionError::Zstd(e.to_string()))?;

        encoder.compress(data)
            .map_err(|e| CompressionError::Zstd(e.to_string()))
    }

    /// Decompress with dictionary
    pub fn decompress_with_dict(&self, data: &[u8], dict: &[u8]) -> Result<Vec<u8>> {
        trace!("Zstandard decompressing {} bytes with dictionary", data.len());

        let mut decompressor = Decompressor::with_dictionary(dict)
            .map_err(|e| CompressionError::Zstd(e.to_string()))?;

        decompressor
            .decompress(data, usize::MAX)
            .map_err(|e| CompressionError::Zstd(e.to_string()))
    }
}

impl Default for ZstdCompressor {
    fn default() -> Self {
        Self::new()
    }
}

impl CompressorTrait for ZstdCompressor {
    fn compress(&self, data: &[u8], level: CompressionLevel) -> Result<Vec<u8>> {
        let zstd_level = level.to_level(Algorithm::Zstd);
        trace!("Zstandard compressing {} bytes at level {}", data.len(), zstd_level);

        let compressed = compress(data, zstd_level)
            .map_err(|e| CompressionError::Zstd(e.to_string()))?;

        debug!(
            "Zstandard compressed {} -> {} bytes ({:.2}% ratio)",
            data.len(),
            compressed.len(),
            (compressed.len() as f64 / data.len() as f64) * 100.0
        );

        Ok(compressed)
    }

    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>> {
        trace!("Zstandard decompressing {} bytes", data.len());

        // Use a reasonable upper bound for decompressed size
        decompress(data, usize::MAX)
            .map_err(|e| CompressionError::Zstd(e.to_string()))
    }

    fn algorithm(&self) -> Algorithm {
        Algorithm::Zstd
    }

    fn max_compressed_size(&self, input_size: usize) -> usize {
        zstd::zstd_safe::compress_bound(input_size)
    }

    fn supports_parallel(&self) -> bool {
        true
    }

    fn supports_streaming(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zstd_round_trip() {
        let compressor = ZstdCompressor::new();
        let data = b"Hello, AccuScene! Testing Zstandard compression.".repeat(100);

        let compressed = compressor.compress(&data, CompressionLevel::Default).unwrap();
        let decompressed = compressor.decompress(&compressed).unwrap();

        assert_eq!(data.to_vec(), decompressed);
        assert!(compressed.len() < data.len());
    }

    #[test]
    fn test_zstd_levels() {
        let compressor = ZstdCompressor::new();
        let data = b"Test data for compression level comparison.".repeat(1000);

        let fast = compressor.compress(&data, CompressionLevel::Fastest).unwrap();
        let default = compressor.compress(&data, CompressionLevel::Default).unwrap();
        let high = compressor.compress(&data, CompressionLevel::High).unwrap();

        // Higher compression should generally produce smaller output
        assert!(high.len() <= default.len());
        assert!(default.len() <= fast.len() * 2); // Allow some variance
    }

    #[test]
    fn test_zstd_with_dictionary() {
        let compressor = ZstdCompressor::new();
        let samples = vec![
            b"AccuScene case data sample 1".as_slice(),
            b"AccuScene case data sample 2".as_slice(),
            b"AccuScene case data sample 3".as_slice(),
        ];

        let dict = ZstdCompressor::train_dictionary(&samples, 1024).unwrap();

        let data = b"AccuScene case data for compression";
        let compressed = compressor.compress_with_dict(data, &dict, 3).unwrap();
        let decompressed = compressor.decompress_with_dict(&compressed, &dict).unwrap();

        assert_eq!(data.to_vec(), decompressed);
    }
}
