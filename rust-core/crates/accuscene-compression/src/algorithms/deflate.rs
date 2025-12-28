//! Deflate/gzip compression implementation - Standard compression

use crate::error::{CompressionError, Result};
use crate::traits::{Algorithm, CompressionLevel, Compressor};
use flate2::read::{GzDecoder, GzEncoder};
use flate2::Compression;
use std::io::{Read, Write};
use tracing::{debug, trace};

/// Deflate/gzip compression implementation
#[derive(Debug, Clone)]
pub struct DeflateCompressor;

impl DeflateCompressor {
    /// Create a new Deflate compressor
    pub fn new() -> Self {
        Self
    }

    /// Convert compression level to flate2 Compression
    fn to_flate2_level(&self, level: CompressionLevel) -> Compression {
        let level_int = level.to_level(Algorithm::Deflate);
        Compression::new(level_int as u32)
    }

    /// Compress using raw deflate (no gzip header)
    pub fn compress_raw(&self, data: &[u8], level: CompressionLevel) -> Result<Vec<u8>> {
        use flate2::write::DeflateEncoder;

        trace!("Raw deflate compressing {} bytes", data.len());

        let mut encoder = DeflateEncoder::new(Vec::new(), self.to_flate2_level(level));
        encoder
            .write_all(data)
            .map_err(|e| CompressionError::Deflate(e.to_string()))?;

        encoder
            .finish()
            .map_err(|e| CompressionError::Deflate(e.to_string()))
    }

    /// Decompress raw deflate data
    pub fn decompress_raw(&self, data: &[u8]) -> Result<Vec<u8>> {
        use flate2::read::DeflateDecoder;

        trace!("Raw deflate decompressing {} bytes", data.len());

        let mut decoder = DeflateDecoder::new(data);
        let mut output = Vec::new();

        decoder
            .read_to_end(&mut output)
            .map_err(|e| CompressionError::Deflate(e.to_string()))?;

        Ok(output)
    }
}

impl Default for DeflateCompressor {
    fn default() -> Self {
        Self::new()
    }
}

impl Compressor for DeflateCompressor {
    fn compress(&self, data: &[u8], level: CompressionLevel) -> Result<Vec<u8>> {
        trace!("Deflate compressing {} bytes", data.len());

        let mut encoder = GzEncoder::new(data, self.to_flate2_level(level));
        let mut output = Vec::new();

        encoder
            .read_to_end(&mut output)
            .map_err(|e| CompressionError::Deflate(e.to_string()))?;

        debug!(
            "Deflate compressed {} -> {} bytes ({:.2}% ratio)",
            data.len(),
            output.len(),
            (output.len() as f64 / data.len() as f64) * 100.0
        );

        Ok(output)
    }

    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>> {
        trace!("Deflate decompressing {} bytes", data.len());

        let mut decoder = GzDecoder::new(data);
        let mut output = Vec::new();

        decoder
            .read_to_end(&mut output)
            .map_err(|e| CompressionError::Deflate(e.to_string()))?;

        Ok(output)
    }

    fn algorithm(&self) -> Algorithm {
        Algorithm::Deflate
    }

    fn max_compressed_size(&self, input_size: usize) -> usize {
        // Deflate worst case: roughly input + 0.1% + 12 bytes
        // Add gzip header overhead (~18 bytes)
        input_size + (input_size / 1000) + 30
    }

    fn supports_parallel(&self) -> bool {
        false // Standard deflate is sequential
    }

    fn supports_streaming(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deflate_round_trip() {
        let compressor = DeflateCompressor::new();
        let data = b"Hello, AccuScene! Testing Deflate compression.".repeat(100);

        let compressed = compressor.compress(&data, CompressionLevel::Default).unwrap();
        let decompressed = compressor.decompress(&compressed).unwrap();

        assert_eq!(data.to_vec(), decompressed);
        assert!(compressed.len() < data.len());
    }

    #[test]
    fn test_deflate_raw_round_trip() {
        let compressor = DeflateCompressor::new();
        let data = b"Testing raw deflate without gzip wrapper.".repeat(50);

        let compressed = compressor.compress_raw(&data, CompressionLevel::Default).unwrap();
        let decompressed = compressor.decompress_raw(&compressed).unwrap();

        assert_eq!(data.to_vec(), decompressed);
    }

    #[test]
    fn test_deflate_levels() {
        let compressor = DeflateCompressor::new();
        let data = b"Test data for deflate compression level comparison.".repeat(1000);

        let fastest = compressor.compress(&data, CompressionLevel::Fastest).unwrap();
        let default = compressor.compress(&data, CompressionLevel::Default).unwrap();
        let maximum = compressor.compress(&data, CompressionLevel::Maximum).unwrap();

        // All should decompress correctly
        assert_eq!(data.to_vec(), compressor.decompress(&fastest).unwrap());
        assert_eq!(data.to_vec(), compressor.decompress(&default).unwrap());
        assert_eq!(data.to_vec(), compressor.decompress(&maximum).unwrap());
    }

    #[test]
    fn test_deflate_empty_data() {
        let compressor = DeflateCompressor::new();
        let data = b"";

        let compressed = compressor.compress(data, CompressionLevel::Default).unwrap();
        let decompressed = compressor.decompress(&compressed).unwrap();

        assert_eq!(data.to_vec(), decompressed);
    }
}
