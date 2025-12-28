//! Snappy compression implementation - Very fast, moderate ratio, good for streaming

use crate::error::{CompressionError, Result};
use crate::traits::{Algorithm, CompressionLevel, Compressor};
use snap::raw::{max_compress_len, Decoder, Encoder};
use tracing::{debug, trace};

/// Snappy compression implementation
#[derive(Debug, Clone)]
pub struct SnappyCompressor;

impl SnappyCompressor {
    /// Create a new Snappy compressor
    pub fn new() -> Self {
        Self
    }

    /// Compress with framed format (streaming)
    pub fn compress_framed(&self, data: &[u8]) -> Result<Vec<u8>> {
        use snap::write::FrameEncoder;
        use std::io::Write;

        trace!("Snappy framed compressing {} bytes", data.len());

        let mut encoder = FrameEncoder::new(Vec::new());
        encoder
            .write_all(data)
            .map_err(|e| CompressionError::Snappy(e.to_string()))?;

        encoder
            .into_inner()
            .map_err(|e| CompressionError::Snappy(e.to_string()))
    }

    /// Decompress framed format
    pub fn decompress_framed(&self, data: &[u8]) -> Result<Vec<u8>> {
        use snap::read::FrameDecoder;
        use std::io::Read;

        trace!("Snappy framed decompressing {} bytes", data.len());

        let mut decoder = FrameDecoder::new(data);
        let mut output = Vec::new();

        decoder
            .read_to_end(&mut output)
            .map_err(|e| CompressionError::Snappy(e.to_string()))?;

        Ok(output)
    }
}

impl Default for SnappyCompressor {
    fn default() -> Self {
        Self::new()
    }
}

impl Compressor for SnappyCompressor {
    fn compress(&self, data: &[u8], _level: CompressionLevel) -> Result<Vec<u8>> {
        trace!("Snappy compressing {} bytes", data.len());

        // Snappy doesn't have compression levels
        let mut encoder = Encoder::new();
        let compressed = encoder
            .compress_vec(data)
            .map_err(|e| CompressionError::Snappy(e.to_string()))?;

        debug!(
            "Snappy compressed {} -> {} bytes ({:.2}% ratio)",
            data.len(),
            compressed.len(),
            (compressed.len() as f64 / data.len() as f64) * 100.0
        );

        Ok(compressed)
    }

    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>> {
        trace!("Snappy decompressing {} bytes", data.len());

        let mut decoder = Decoder::new();
        decoder
            .decompress_vec(data)
            .map_err(|e| CompressionError::Snappy(e.to_string()))
    }

    fn algorithm(&self) -> Algorithm {
        Algorithm::Snappy
    }

    fn max_compressed_size(&self, input_size: usize) -> usize {
        max_compress_len(input_size)
    }

    fn supports_parallel(&self) -> bool {
        true // Can compress independent blocks in parallel
    }

    fn supports_streaming(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snappy_round_trip() {
        let compressor = SnappyCompressor::new();
        let data = b"Hello, AccuScene! Testing Snappy compression.".repeat(100);

        let compressed = compressor.compress(&data, CompressionLevel::Default).unwrap();
        let decompressed = compressor.decompress(&compressed).unwrap();

        assert_eq!(data.to_vec(), decompressed);
        assert!(compressed.len() < data.len());
    }

    #[test]
    fn test_snappy_framed_round_trip() {
        let compressor = SnappyCompressor::new();
        let data = b"Testing Snappy framed format for streaming.".repeat(50);

        let compressed = compressor.compress_framed(&data).unwrap();
        let decompressed = compressor.decompress_framed(&compressed).unwrap();

        assert_eq!(data.to_vec(), decompressed);
    }

    #[test]
    fn test_snappy_empty_data() {
        let compressor = SnappyCompressor::new();
        let data = b"";

        let compressed = compressor.compress(data, CompressionLevel::Default).unwrap();
        let decompressed = compressor.decompress(&compressed).unwrap();

        assert_eq!(data.to_vec(), decompressed);
    }

    #[test]
    fn test_snappy_large_data() {
        let compressor = SnappyCompressor::new();
        let data = vec![0u8; 1_000_000]; // 1MB of zeros

        let compressed = compressor.compress(&data, CompressionLevel::Default).unwrap();
        let decompressed = compressor.decompress(&compressed).unwrap();

        assert_eq!(data, decompressed);
        assert!(compressed.len() < data.len() / 10); // Should compress very well
    }
}
