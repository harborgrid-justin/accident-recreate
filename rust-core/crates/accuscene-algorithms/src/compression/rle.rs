//! Run-length encoding for sparse data.
//!
//! Compresses data by replacing runs of identical values with a count and value.
//! Highly effective for sparse scene data with many repeated values.
//!
//! # Complexity
//! - Encoding: O(n) where n is input size
//! - Decoding: O(n) where n is output size
//! - Memory: O(1) additional space

use crate::compression::{CompressionStats, Compressor};
use crate::error::{AlgorithmError, Result};
use bytes::{BufMut, BytesMut};
use parking_lot::Mutex;
use std::sync::Arc;

const MAX_RUN_LENGTH: u16 = u16::MAX;

/// Run-length encoder for byte sequences.
///
/// Encodes consecutive runs of identical bytes as (count, value) pairs.
#[derive(Clone)]
pub struct RunLengthEncoder {
    stats: Arc<Mutex<CompressionStats>>,
}

impl RunLengthEncoder {
    /// Create a new run-length encoder.
    pub fn new() -> Self {
        Self {
            stats: Arc::new(Mutex::new(CompressionStats::default())),
        }
    }

    /// Encode data using run-length encoding.
    ///
    /// Format: [count: u16][value: u8] repeated
    ///
    /// # Complexity
    /// O(n) where n is input size
    pub fn encode(&self, input: &[u8]) -> Result<Vec<u8>> {
        if input.is_empty() {
            return Ok(Vec::new());
        }

        let mut output = BytesMut::with_capacity(input.len());
        let mut i = 0;

        while i < input.len() {
            let value = input[i];
            let mut count = 1u16;

            // Count consecutive identical bytes
            while i + count as usize < input.len()
                && input[i + count as usize] == value
                && count < MAX_RUN_LENGTH
            {
                count += 1;
            }

            // Write count and value
            output.put_u16_le(count);
            output.put_u8(value);

            i += count as usize;
        }

        let result = output.to_vec();
        self.stats.lock().record(input.len(), result.len());

        Ok(result)
    }

    /// Decode run-length encoded data.
    ///
    /// # Complexity
    /// O(n) where n is output size
    pub fn decode(&self, input: &[u8]) -> Result<Vec<u8>> {
        if input.is_empty() {
            return Ok(Vec::new());
        }

        if input.len() % 3 != 0 {
            return Err(AlgorithmError::InvalidFormat(
                "RLE encoded data length must be multiple of 3".to_string(),
            ));
        }

        // Estimate output size
        let mut estimated_size = 0usize;
        for chunk in input.chunks_exact(3) {
            let count = u16::from_le_bytes([chunk[0], chunk[1]]) as usize;
            estimated_size += count;
        }

        let mut output = Vec::with_capacity(estimated_size);
        let mut i = 0;

        while i + 2 < input.len() {
            let count = u16::from_le_bytes([input[i], input[i + 1]]) as usize;
            let value = input[i + 2];

            // Expand run
            output.extend(std::iter::repeat(value).take(count));

            i += 3;
        }

        Ok(output)
    }

    /// Encode with byte-level RLE (alternative format for better compression of mixed data).
    ///
    /// Format: For each byte, either:
    /// - If bit 7 = 0: literal byte follows (0xxxxxxx + byte)
    /// - If bit 7 = 1: run follows (1xxxxxxx + value), where xxxxxxx is count-1
    pub fn encode_adaptive(&self, input: &[u8]) -> Result<Vec<u8>> {
        if input.is_empty() {
            return Ok(Vec::new());
        }

        let mut output = BytesMut::with_capacity(input.len());
        let mut i = 0;

        while i < input.len() {
            let value = input[i];
            let mut count = 1usize;

            // Count consecutive identical bytes (max 128)
            while i + count < input.len() && input[i + count] == value && count < 128 {
                count += 1;
            }

            if count >= 3 {
                // Use run encoding
                output.put_u8(0x80 | ((count - 1) as u8));
                output.put_u8(value);
                i += count;
            } else {
                // Use literal encoding
                output.put_u8(value & 0x7F);
                output.put_u8(value);
                i += 1;
            }
        }

        let result = output.to_vec();
        self.stats.lock().record(input.len(), result.len());

        Ok(result)
    }

    /// Decode adaptive RLE format.
    pub fn decode_adaptive(&self, input: &[u8]) -> Result<Vec<u8>> {
        if input.is_empty() {
            return Ok(Vec::new());
        }

        if input.len() % 2 != 0 {
            return Err(AlgorithmError::InvalidFormat(
                "Adaptive RLE encoded data must have even length".to_string(),
            ));
        }

        let mut output = Vec::with_capacity(input.len());
        let mut i = 0;

        while i + 1 < input.len() {
            let header = input[i];
            let value = input[i + 1];

            if header & 0x80 != 0 {
                // Run encoding
                let count = ((header & 0x7F) as usize) + 1;
                output.extend(std::iter::repeat(value).take(count));
            } else {
                // Literal encoding
                output.push(value);
            }

            i += 2;
        }

        Ok(output)
    }

    /// Estimate compression ratio for given data.
    pub fn estimate_ratio(data: &[u8]) -> f64 {
        if data.is_empty() {
            return 1.0;
        }

        let encoder = Self::new();
        let compressed = encoder.encode(data).unwrap_or_default();

        if compressed.is_empty() {
            return 1.0;
        }

        data.len() as f64 / compressed.len() as f64
    }

    /// Get compression statistics.
    pub fn stats(&self) -> CompressionStats {
        self.stats.lock().clone()
    }

    /// Reset statistics.
    pub fn reset_stats(&self) {
        self.stats.lock().reset();
    }
}

impl Default for RunLengthEncoder {
    fn default() -> Self {
        Self::new()
    }
}

impl Compressor for RunLengthEncoder {
    fn compress(&mut self, input: &[u8]) -> Result<Vec<u8>> {
        self.encode(input)
    }

    fn decompress(&mut self, input: &[u8]) -> Result<Vec<u8>> {
        self.decode(input)
    }

    fn compression_ratio(&self) -> f64 {
        self.stats.lock().ratio()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rle_encode_decode() {
        let encoder = RunLengthEncoder::new();
        let data = b"aaaaaabbbbbcccccdddddeeeee";

        let encoded = encoder.encode(data).unwrap();
        let decoded = encoder.decode(&encoded).unwrap();

        assert_eq!(data.to_vec(), decoded);
    }

    #[test]
    fn test_rle_sparse_data() {
        let encoder = RunLengthEncoder::new();
        let mut data = vec![0u8; 1000];
        data.extend_from_slice(&[1; 500]);
        data.extend_from_slice(&[0; 1000]);

        let encoded = encoder.encode(&data).unwrap();
        assert!(encoded.len() < data.len());

        let decoded = encoder.decode(&encoded).unwrap();
        assert_eq!(data, decoded);
    }

    #[test]
    fn test_adaptive_rle() {
        let encoder = RunLengthEncoder::new();
        let data = b"aaabbbcccdddeeefffggghhhiii";

        let encoded = encoder.encode_adaptive(data).unwrap();
        let decoded = encoder.decode_adaptive(&encoded).unwrap();

        assert_eq!(data.to_vec(), decoded);
    }

    #[test]
    fn test_empty_input() {
        let encoder = RunLengthEncoder::new();
        let encoded = encoder.encode(&[]).unwrap();
        assert!(encoded.is_empty());
    }

    #[test]
    fn test_single_byte() {
        let encoder = RunLengthEncoder::new();
        let data = b"a";

        let encoded = encoder.encode(data).unwrap();
        let decoded = encoder.decode(&encoded).unwrap();

        assert_eq!(data.to_vec(), decoded);
    }
}
