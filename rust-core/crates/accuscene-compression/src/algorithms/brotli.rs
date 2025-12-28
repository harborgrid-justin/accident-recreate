//! Brotli compression implementation - High compression ratio, slower

use crate::error::{CompressionError, Result};
use crate::traits::{Algorithm, CompressionLevel, Compressor};
use brotli::{enc::BrotliEncoderParams, BrotliCompress, BrotliDecompress};
use std::io::Cursor;
use tracing::{debug, trace};

/// Brotli compression implementation
#[derive(Debug, Clone)]
pub struct BrotliCompressor {
    /// Window size (10-24, default 22)
    window_size: u32,
}

impl BrotliCompressor {
    /// Create a new Brotli compressor
    pub fn new() -> Self {
        Self {
            window_size: 22, // Default window size
        }
    }

    /// Create with custom window size
    pub fn with_window_size(window_size: u32) -> Self {
        Self { window_size }
    }

    /// Get encoder parameters for compression level
    fn get_params(&self, level: CompressionLevel) -> BrotliEncoderParams {
        let quality = level.to_level(Algorithm::Brotli);
        let mut params = BrotliEncoderParams::default();
        params.quality = quality;
        params.lgwin = self.window_size;
        params
    }
}

impl Default for BrotliCompressor {
    fn default() -> Self {
        Self::new()
    }
}

impl Compressor for BrotliCompressor {
    fn compress(&self, data: &[u8], level: CompressionLevel) -> Result<Vec<u8>> {
        trace!("Brotli compressing {} bytes", data.len());

        let params = self.get_params(level);
        let mut output = Vec::new();
        let mut cursor = Cursor::new(data);

        BrotliCompress(&mut cursor, &mut output, &params)
            .map_err(|e| CompressionError::Brotli(e.to_string()))?;

        debug!(
            "Brotli compressed {} -> {} bytes ({:.2}% ratio)",
            data.len(),
            output.len(),
            (output.len() as f64 / data.len() as f64) * 100.0
        );

        Ok(output)
    }

    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>> {
        trace!("Brotli decompressing {} bytes", data.len());

        let mut output = Vec::new();
        let mut cursor = Cursor::new(data);

        BrotliDecompress(&mut cursor, &mut output)
            .map_err(|e| CompressionError::Brotli(e.to_string()))?;

        Ok(output)
    }

    fn algorithm(&self) -> Algorithm {
        Algorithm::Brotli
    }

    fn max_compressed_size(&self, input_size: usize) -> usize {
        // Brotli worst case is roughly input + 5 bytes per 16KB block
        input_size + ((input_size / 16384) + 1) * 5 + 64
    }

    fn supports_parallel(&self) -> bool {
        false // Standard Brotli doesn't support parallel compression
    }

    fn supports_streaming(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brotli_round_trip() {
        let compressor = BrotliCompressor::new();
        let data = b"Hello, AccuScene! Testing Brotli compression.".repeat(100);

        let compressed = compressor.compress(&data, CompressionLevel::Default).unwrap();
        let decompressed = compressor.decompress(&compressed).unwrap();

        assert_eq!(data.to_vec(), decompressed);
        assert!(compressed.len() < data.len());
    }

    #[test]
    fn test_brotli_levels() {
        let compressor = BrotliCompressor::new();
        let data = b"Test data for Brotli compression level comparison.".repeat(1000);

        let fast = compressor.compress(&data, CompressionLevel::Fast).unwrap();
        let default = compressor.compress(&data, CompressionLevel::Default).unwrap();
        let high = compressor.compress(&data, CompressionLevel::High).unwrap();

        // All should decompress correctly
        assert_eq!(data.to_vec(), compressor.decompress(&fast).unwrap());
        assert_eq!(data.to_vec(), compressor.decompress(&default).unwrap());
        assert_eq!(data.to_vec(), compressor.decompress(&high).unwrap());

        // Higher compression should produce smaller output
        assert!(high.len() <= default.len());
    }

    #[test]
    fn test_brotli_empty_data() {
        let compressor = BrotliCompressor::new();
        let data = b"";

        let compressed = compressor.compress(data, CompressionLevel::Default).unwrap();
        let decompressed = compressor.decompress(&compressed).unwrap();

        assert_eq!(data.to_vec(), decompressed);
    }
}
