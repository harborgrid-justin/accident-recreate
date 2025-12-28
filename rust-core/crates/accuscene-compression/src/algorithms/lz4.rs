//! LZ4 compression implementation - Fast compression with moderate ratio

use crate::error::{CompressionError, Result};
use crate::traits::{Algorithm, CompressionLevel, Compressor};
use lz4_flex::block::{compress_prepend_size, decompress_size_prepended};
use tracing::{debug, trace};

/// LZ4 compression implementation
#[derive(Debug, Clone)]
pub struct Lz4Compressor;

impl Lz4Compressor {
    /// Create a new LZ4 compressor
    pub fn new() -> Self {
        Self
    }

    /// Compress with high compression mode
    pub fn compress_high(&self, data: &[u8]) -> Result<Vec<u8>> {
        trace!("LZ4 high compression of {} bytes", data.len());
        let compressed = lz4_flex::block::compress(data);
        debug!("LZ4 compressed {} -> {} bytes", data.len(), compressed.len());
        Ok(compressed)
    }
}

impl Default for Lz4Compressor {
    fn default() -> Self {
        Self::new()
    }
}

impl Compressor for Lz4Compressor {
    fn compress(&self, data: &[u8], _level: CompressionLevel) -> Result<Vec<u8>> {
        trace!("LZ4 compressing {} bytes", data.len());

        // LZ4 doesn't have compression levels in lz4_flex
        // We use prepend_size variant for self-describing format
        let compressed = compress_prepend_size(data);

        debug!(
            "LZ4 compressed {} -> {} bytes ({:.2}% ratio)",
            data.len(),
            compressed.len(),
            (compressed.len() as f64 / data.len() as f64) * 100.0
        );

        Ok(compressed)
    }

    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>> {
        trace!("LZ4 decompressing {} bytes", data.len());

        decompress_size_prepended(data)
            .map_err(|e| CompressionError::Lz4(e.to_string()))
    }

    fn algorithm(&self) -> Algorithm {
        Algorithm::Lz4
    }

    fn max_compressed_size(&self, input_size: usize) -> usize {
        // LZ4 worst case: input_size + (input_size / 255) + 16
        input_size + (input_size / 255) + 16 + 4 // +4 for size prefix
    }

    fn supports_parallel(&self) -> bool {
        true // LZ4 can compress independent blocks in parallel
    }

    fn supports_streaming(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lz4_round_trip() {
        let compressor = Lz4Compressor::new();
        let data = b"Hello, AccuScene! Testing LZ4 compression.".repeat(100);

        let compressed = compressor.compress(&data, CompressionLevel::Default).unwrap();
        let decompressed = compressor.decompress(&compressed).unwrap();

        assert_eq!(data.to_vec(), decompressed);
        assert!(compressed.len() < data.len());
    }

    #[test]
    fn test_lz4_empty_data() {
        let compressor = Lz4Compressor::new();
        let data = b"";

        let compressed = compressor.compress(data, CompressionLevel::Default).unwrap();
        let decompressed = compressor.decompress(&compressed).unwrap();

        assert_eq!(data.to_vec(), decompressed);
    }
}
