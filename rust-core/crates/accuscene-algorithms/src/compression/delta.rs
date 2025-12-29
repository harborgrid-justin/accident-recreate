//! Delta encoding for incremental scene updates.
//!
//! Compresses data by storing differences between consecutive values.
//! Highly effective for time-series data and incremental scene changes.
//!
//! # Complexity
//! - Encoding: O(n) where n is number of values
//! - Decoding: O(n) where n is number of values
//! - Memory: O(1) additional space

use crate::compression::{CompressionStats, Compressor};
use crate::error::{AlgorithmError, Result};
use bytes::{Buf, BufMut, BytesMut};
use parking_lot::Mutex;
use std::sync::Arc;

/// Delta encoding for integer sequences.
///
/// Stores the difference between consecutive values, which is often smaller
/// and more compressible than the original values.
#[derive(Clone)]
pub struct DeltaEncoder {
    stats: Arc<Mutex<CompressionStats>>,
}

impl DeltaEncoder {
    /// Create a new delta encoder.
    pub fn new() -> Self {
        Self {
            stats: Arc::new(Mutex::new(CompressionStats::default())),
        }
    }

    /// Encode i32 values using delta encoding.
    ///
    /// # Complexity
    /// O(n) where n is the number of values
    pub fn encode_i32(&self, values: &[i32]) -> Result<Vec<u8>> {
        if values.is_empty() {
            return Ok(Vec::new());
        }

        let mut output = BytesMut::with_capacity(4 + values.len() * 4);

        // Store first value as-is
        output.put_i32_le(values[0]);

        // Store deltas
        for i in 1..values.len() {
            let delta = values[i].wrapping_sub(values[i - 1]);
            output.put_i32_le(delta);
        }

        let result = output.to_vec();
        self.stats.lock().record(values.len() * 4, result.len());

        Ok(result)
    }

    /// Decode i32 values from delta encoding.
    ///
    /// # Complexity
    /// O(n) where n is the number of values
    pub fn decode_i32(&self, encoded: &[u8]) -> Result<Vec<i32>> {
        if encoded.is_empty() {
            return Ok(Vec::new());
        }

        if encoded.len() % 4 != 0 {
            return Err(AlgorithmError::InvalidFormat(
                "Encoded data length must be multiple of 4".to_string(),
            ));
        }

        let count = encoded.len() / 4;
        let mut result = Vec::with_capacity(count);
        let mut buf = &encoded[..];

        // Read first value
        let mut prev = buf.get_i32_le();
        result.push(prev);

        // Decode deltas
        while buf.has_remaining() {
            let delta = buf.get_i32_le();
            let value = prev.wrapping_add(delta);
            result.push(value);
            prev = value;
        }

        Ok(result)
    }

    /// Encode i64 values using delta encoding.
    ///
    /// # Complexity
    /// O(n) where n is the number of values
    pub fn encode_i64(&self, values: &[i64]) -> Result<Vec<u8>> {
        if values.is_empty() {
            return Ok(Vec::new());
        }

        let mut output = BytesMut::with_capacity(8 + values.len() * 8);

        // Store first value as-is
        output.put_i64_le(values[0]);

        // Store deltas
        for i in 1..values.len() {
            let delta = values[i].wrapping_sub(values[i - 1]);
            output.put_i64_le(delta);
        }

        let result = output.to_vec();
        self.stats.lock().record(values.len() * 8, result.len());

        Ok(result)
    }

    /// Decode i64 values from delta encoding.
    ///
    /// # Complexity
    /// O(n) where n is the number of values
    pub fn decode_i64(&self, encoded: &[u8]) -> Result<Vec<i64>> {
        if encoded.is_empty() {
            return Ok(Vec::new());
        }

        if encoded.len() % 8 != 0 {
            return Err(AlgorithmError::InvalidFormat(
                "Encoded data length must be multiple of 8".to_string(),
            ));
        }

        let count = encoded.len() / 8;
        let mut result = Vec::with_capacity(count);
        let mut buf = &encoded[..];

        // Read first value
        let mut prev = buf.get_i64_le();
        result.push(prev);

        // Decode deltas
        while buf.has_remaining() {
            let delta = buf.get_i64_le();
            let value = prev.wrapping_add(delta);
            result.push(value);
            prev = value;
        }

        Ok(result)
    }

    /// Encode f32 values using delta encoding.
    ///
    /// Converts floats to bits and applies delta encoding.
    ///
    /// # Complexity
    /// O(n) where n is the number of values
    pub fn encode_f32(&self, values: &[f32]) -> Result<Vec<u8>> {
        if values.is_empty() {
            return Ok(Vec::new());
        }

        // Convert to bits
        let bits: Vec<i32> = values.iter().map(|&v| v.to_bits() as i32).collect();

        // Apply delta encoding
        self.encode_i32(&bits)
    }

    /// Decode f32 values from delta encoding.
    ///
    /// # Complexity
    /// O(n) where n is the number of values
    pub fn decode_f32(&self, encoded: &[u8]) -> Result<Vec<f32>> {
        let bits = self.decode_i32(encoded)?;
        Ok(bits.iter().map(|&b| f32::from_bits(b as u32)).collect())
    }

    /// Encode f64 values using delta encoding.
    ///
    /// Converts floats to bits and applies delta encoding.
    ///
    /// # Complexity
    /// O(n) where n is the number of values
    pub fn encode_f64(&self, values: &[f64]) -> Result<Vec<u8>> {
        if values.is_empty() {
            return Ok(Vec::new());
        }

        // Convert to bits
        let bits: Vec<i64> = values.iter().map(|&v| v.to_bits() as i64).collect();

        // Apply delta encoding
        self.encode_i64(&bits)
    }

    /// Decode f64 values from delta encoding.
    ///
    /// # Complexity
    /// O(n) where n is the number of values
    pub fn decode_f64(&self, encoded: &[u8]) -> Result<Vec<f64>> {
        let bits = self.decode_i64(encoded)?;
        Ok(bits.iter().map(|&b| f64::from_bits(b as u64)).collect())
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

impl Default for DeltaEncoder {
    fn default() -> Self {
        Self::new()
    }
}

impl Compressor for DeltaEncoder {
    fn compress(&mut self, input: &[u8]) -> Result<Vec<u8>> {
        // Interpret input as i32 array and apply delta encoding
        if input.len() % 4 != 0 {
            return Err(AlgorithmError::InvalidFormat(
                "Input length must be multiple of 4 for delta encoding".to_string(),
            ));
        }

        let mut values = Vec::with_capacity(input.len() / 4);
        let mut buf = input;
        while buf.len() >= 4 {
            values.push(i32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]));
            buf = &buf[4..];
        }

        self.encode_i32(&values)
    }

    fn decompress(&mut self, input: &[u8]) -> Result<Vec<u8>> {
        let values = self.decode_i32(input)?;

        let mut output = BytesMut::with_capacity(values.len() * 4);
        for value in values {
            output.put_i32_le(value);
        }

        Ok(output.to_vec())
    }

    fn compression_ratio(&self) -> f64 {
        self.stats.lock().ratio()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delta_encode_i32() {
        let encoder = DeltaEncoder::new();
        let values = vec![100, 101, 103, 106, 110];

        let encoded = encoder.encode_i32(&values).unwrap();
        let decoded = encoder.decode_i32(&encoded).unwrap();

        assert_eq!(values, decoded);
    }

    #[test]
    fn test_delta_encode_f32() {
        let encoder = DeltaEncoder::new();
        let values = vec![1.0f32, 1.1, 1.2, 1.3, 1.4];

        let encoded = encoder.encode_f32(&values).unwrap();
        let decoded = encoder.decode_f32(&encoded).unwrap();

        assert_eq!(values, decoded);
    }

    #[test]
    fn test_empty_input() {
        let encoder = DeltaEncoder::new();
        let encoded = encoder.encode_i32(&[]).unwrap();
        assert!(encoded.is_empty());
    }

    #[test]
    fn test_sequential_values() {
        let encoder = DeltaEncoder::new();
        let values: Vec<i32> = (0..1000).collect();

        let encoded = encoder.encode_i32(&values).unwrap();
        let decoded = encoder.decode_i32(&encoded).unwrap();

        assert_eq!(values, decoded);
    }
}
