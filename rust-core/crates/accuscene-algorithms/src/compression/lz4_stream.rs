//! LZ4 streaming compression for real-time scene data.
//!
//! Provides high-speed compression/decompression suitable for real-time applications.
//! LZ4 is optimized for speed over compression ratio.
//!
//! # Complexity
//! - Compression: O(n) where n is input size
//! - Decompression: O(n) where n is output size
//! - Memory: O(64KB) for compression dictionary

use crate::compression::{CompressionStats, Compressor};
use crate::error::{AlgorithmError, Result};
use bytes::{BufMut, BytesMut};
use parking_lot::Mutex;
use std::sync::Arc;

const MAGIC_NUMBER: u32 = 0x184D2204;
const HEADER_SIZE: usize = 12;

/// LZ4 streaming compressor.
///
/// Maintains a compression context for efficient streaming compression
/// of continuous data streams.
#[derive(Clone)]
pub struct Lz4Stream {
    stats: Arc<Mutex<CompressionStats>>,
    acceleration: i32,
}

impl Lz4Stream {
    /// Create a new LZ4 stream compressor.
    ///
    /// # Arguments
    /// * `acceleration` - Acceleration factor (1-17, higher = faster but lower ratio)
    pub fn new(acceleration: i32) -> Self {
        Self {
            stats: Arc::new(Mutex::new(CompressionStats::default())),
            acceleration: acceleration.clamp(1, 17),
        }
    }

    /// Create with default settings.
    pub fn default() -> Self {
        Self::new(1)
    }

    /// Compress a block with frame format.
    ///
    /// Returns compressed data with LZ4 frame header for self-contained blocks.
    pub fn compress_block(&self, input: &[u8]) -> Result<Vec<u8>> {
        if input.is_empty() {
            return Ok(Vec::new());
        }

        // Compress the data
        let compressed = lz4_flex::compress_prepend_size(input);

        // Create frame with magic number and size info
        let mut output = BytesMut::with_capacity(HEADER_SIZE + compressed.len());
        output.put_u32_le(MAGIC_NUMBER);
        output.put_u32_le(input.len() as u32);
        output.put_u32_le(compressed.len() as u32);
        output.put_slice(&compressed);

        let result = output.to_vec();

        // Update statistics
        self.stats.lock().record(input.len(), result.len());

        Ok(result)
    }

    /// Decompress a block with frame format.
    pub fn decompress_block(&self, input: &[u8]) -> Result<Vec<u8>> {
        if input.is_empty() {
            return Ok(Vec::new());
        }

        if input.len() < HEADER_SIZE {
            return Err(AlgorithmError::InvalidFormat(
                "Input too small for LZ4 frame".to_string(),
            ));
        }

        // Parse header
        let magic = u32::from_le_bytes([input[0], input[1], input[2], input[3]]);
        if magic != MAGIC_NUMBER {
            return Err(AlgorithmError::InvalidFormat(format!(
                "Invalid LZ4 magic number: 0x{:08X}",
                magic
            )));
        }

        let uncompressed_size = u32::from_le_bytes([input[4], input[5], input[6], input[7]]) as usize;
        let compressed_size = u32::from_le_bytes([input[8], input[9], input[10], input[11]]) as usize;

        if input.len() < HEADER_SIZE + compressed_size {
            return Err(AlgorithmError::InvalidFormat(
                "Input smaller than declared compressed size".to_string(),
            ));
        }

        // Decompress
        let compressed_data = &input[HEADER_SIZE..HEADER_SIZE + compressed_size];
        let decompressed = lz4_flex::decompress_size_prepended(compressed_data)
            .map_err(|e| AlgorithmError::DecompressionFailed(e.to_string()))?;

        if decompressed.len() != uncompressed_size {
            return Err(AlgorithmError::InvalidFormat(format!(
                "Decompressed size mismatch: expected {}, got {}",
                uncompressed_size,
                decompressed.len()
            )));
        }

        Ok(decompressed)
    }

    /// Get compression statistics.
    pub fn stats(&self) -> CompressionStats {
        self.stats.lock().clone()
    }

    /// Reset statistics.
    pub fn reset_stats(&self) {
        self.stats.lock().reset();
    }

    /// Estimate compression ratio for given data.
    pub fn estimate_ratio(data: &[u8]) -> f64 {
        if data.is_empty() {
            return 1.0;
        }
        let compressed = lz4_flex::compress_prepend_size(data);
        data.len() as f64 / compressed.len() as f64
    }
}

impl Compressor for Lz4Stream {
    fn compress(&mut self, input: &[u8]) -> Result<Vec<u8>> {
        self.compress_block(input)
    }

    fn decompress(&mut self, input: &[u8]) -> Result<Vec<u8>> {
        self.decompress_block(input)
    }

    fn compression_ratio(&self) -> f64 {
        self.stats.lock().ratio()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_decompress() {
        let stream = Lz4Stream::new(1);
        let data = b"Hello, World! This is a test of LZ4 compression.".repeat(10);

        let compressed = stream.compress_block(&data).unwrap();
        assert!(compressed.len() < data.len());

        let decompressed = stream.decompress_block(&compressed).unwrap();
        assert_eq!(data, decompressed.as_slice());
    }

    #[test]
    fn test_empty_data() {
        let stream = Lz4Stream::new(1);
        let compressed = stream.compress_block(&[]).unwrap();
        assert!(compressed.is_empty());
    }

    #[test]
    fn test_statistics() {
        let stream = Lz4Stream::new(1);
        let data = b"Test data";

        stream.compress_block(data).unwrap();
        let stats = stream.stats();

        assert_eq!(stats.bytes_in, data.len() as u64);
        assert!(stats.bytes_out > 0);
        assert_eq!(stats.operations, 1);
    }
}
