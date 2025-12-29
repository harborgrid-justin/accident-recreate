//! Prefix compression for sorted strings.
//!
//! Compresses sorted strings by storing shared prefixes only once.
//! Highly effective for sorted keys with common prefixes.
//!
//! # Example
//! Input: ["apple", "application", "apply"]
//! Compressed:
//! - "apple" (5 chars)
//! - "appl" (4 chars) + "ication" (7 chars)
//! - "appl" (4 chars) + "y" (1 char)
//!
//! # Complexity
//! - Compress: O(n * m) where n is string count, m is average length
//! - Decompress: O(n * m)
//! - Space: O(total chars - shared prefixes)

use crate::error::{AlgorithmError, Result};
use bytes::{BufMut, BytesMut};

/// Prefix compressor for sorted strings.
pub struct PrefixCompressor;

impl PrefixCompressor {
    /// Compress a sequence of sorted strings.
    ///
    /// Returns compressed bytes with format:
    /// [count: u32][entry1][entry2]...
    /// Each entry: [prefix_len: u8][suffix_len: u16][suffix_bytes]
    ///
    /// # Complexity
    /// O(n * m) where n is string count, m is average length
    pub fn compress(strings: &[String]) -> Result<Vec<u8>> {
        if strings.is_empty() {
            return Ok(vec![]);
        }

        let mut bytes = BytesMut::new();
        bytes.put_u32_le(strings.len() as u32);

        let mut prev = String::new();

        for current in strings {
            // Find common prefix length
            let prefix_len = Self::common_prefix_len(&prev, current);

            if prefix_len > 255 {
                return Err(AlgorithmError::InvalidFormat(
                    "Prefix too long (max 255)".to_string(),
                ));
            }

            let suffix = &current[prefix_len..];

            if suffix.len() > u16::MAX as usize {
                return Err(AlgorithmError::InvalidFormat(
                    "Suffix too long".to_string(),
                ));
            }

            // Write entry
            bytes.put_u8(prefix_len as u8);
            bytes.put_u16_le(suffix.len() as u16);
            bytes.put_slice(suffix.as_bytes());

            prev = current.clone();
        }

        Ok(bytes.to_vec())
    }

    /// Decompress strings from compressed bytes.
    ///
    /// # Complexity
    /// O(n * m) where n is string count, m is average length
    pub fn decompress(bytes: &[u8]) -> Result<Vec<String>> {
        if bytes.is_empty() {
            return Ok(Vec::new());
        }

        if bytes.len() < 4 {
            return Err(AlgorithmError::InvalidFormat(
                "Compressed data too small".to_string(),
            ));
        }

        let count = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
        let mut offset = 4;
        let mut result = Vec::with_capacity(count);
        let mut prev = String::new();

        for _ in 0..count {
            if offset + 3 > bytes.len() {
                return Err(AlgorithmError::InvalidFormat(
                    "Incomplete entry header".to_string(),
                ));
            }

            let prefix_len = bytes[offset] as usize;
            let suffix_len = u16::from_le_bytes([bytes[offset + 1], bytes[offset + 2]]) as usize;
            offset += 3;

            if offset + suffix_len > bytes.len() {
                return Err(AlgorithmError::InvalidFormat(
                    "Incomplete suffix".to_string(),
                ));
            }

            let suffix = &bytes[offset..offset + suffix_len];
            offset += suffix_len;

            // Reconstruct string
            let mut current = String::with_capacity(prefix_len + suffix_len);
            if prefix_len > prev.len() {
                return Err(AlgorithmError::InvalidFormat(format!(
                    "Prefix length {} exceeds previous string length {}",
                    prefix_len,
                    prev.len()
                )));
            }
            current.push_str(&prev[..prefix_len]);
            current.push_str(
                std::str::from_utf8(suffix)
                    .map_err(|_| AlgorithmError::InvalidFormat("Invalid UTF-8".to_string()))?,
            );

            result.push(current.clone());
            prev = current;
        }

        Ok(result)
    }

    /// Compress with additional delta encoding for numeric suffixes.
    pub fn compress_advanced(strings: &[String]) -> Result<Vec<u8>> {
        // For now, use basic compression
        // Could be extended with numeric suffix delta encoding
        Self::compress(strings)
    }

    /// Calculate compression ratio.
    pub fn compression_ratio(strings: &[String]) -> f64 {
        if strings.is_empty() {
            return 1.0;
        }

        let original_size: usize = strings.iter().map(|s| s.len()).sum();
        let compressed = Self::compress(strings).unwrap_or_default();

        if compressed.is_empty() {
            return 1.0;
        }

        original_size as f64 / compressed.len() as f64
    }

    /// Find common prefix length between two strings.
    fn common_prefix_len(a: &str, b: &str) -> usize {
        a.chars()
            .zip(b.chars())
            .take_while(|(ca, cb)| ca == cb)
            .count()
    }

    /// Compress byte sequences (not necessarily UTF-8).
    pub fn compress_bytes(sequences: &[Vec<u8>]) -> Result<Vec<u8>> {
        if sequences.is_empty() {
            return Ok(vec![]);
        }

        let mut bytes = BytesMut::new();
        bytes.put_u32_le(sequences.len() as u32);

        let mut prev = Vec::new();

        for current in sequences {
            // Find common prefix length
            let prefix_len = prev
                .iter()
                .zip(current.iter())
                .take_while(|(a, b)| a == b)
                .count();

            if prefix_len > 255 {
                return Err(AlgorithmError::InvalidFormat(
                    "Prefix too long (max 255)".to_string(),
                ));
            }

            let suffix = &current[prefix_len..];

            if suffix.len() > u16::MAX as usize {
                return Err(AlgorithmError::InvalidFormat(
                    "Suffix too long".to_string(),
                ));
            }

            // Write entry
            bytes.put_u8(prefix_len as u8);
            bytes.put_u16_le(suffix.len() as u16);
            bytes.put_slice(suffix);

            prev = current.clone();
        }

        Ok(bytes.to_vec())
    }

    /// Decompress byte sequences.
    pub fn decompress_bytes(bytes: &[u8]) -> Result<Vec<Vec<u8>>> {
        if bytes.is_empty() {
            return Ok(Vec::new());
        }

        if bytes.len() < 4 {
            return Err(AlgorithmError::InvalidFormat(
                "Compressed data too small".to_string(),
            ));
        }

        let count = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
        let mut offset = 4;
        let mut result = Vec::with_capacity(count);
        let mut prev = Vec::new();

        for _ in 0..count {
            if offset + 3 > bytes.len() {
                return Err(AlgorithmError::InvalidFormat(
                    "Incomplete entry header".to_string(),
                ));
            }

            let prefix_len = bytes[offset] as usize;
            let suffix_len = u16::from_le_bytes([bytes[offset + 1], bytes[offset + 2]]) as usize;
            offset += 3;

            if offset + suffix_len > bytes.len() {
                return Err(AlgorithmError::InvalidFormat(
                    "Incomplete suffix".to_string(),
                ));
            }

            let suffix = &bytes[offset..offset + suffix_len];
            offset += suffix_len;

            // Reconstruct sequence
            let mut current = Vec::with_capacity(prefix_len + suffix_len);
            if prefix_len > prev.len() {
                return Err(AlgorithmError::InvalidFormat(format!(
                    "Prefix length {} exceeds previous sequence length {}",
                    prefix_len,
                    prev.len()
                )));
            }
            current.extend_from_slice(&prev[..prefix_len]);
            current.extend_from_slice(suffix);

            result.push(current.clone());
            prev = current;
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_decompress() {
        let strings = vec![
            "apple".to_string(),
            "application".to_string(),
            "apply".to_string(),
            "banana".to_string(),
            "band".to_string(),
        ];

        let compressed = PrefixCompressor::compress(&strings).unwrap();
        let decompressed = PrefixCompressor::decompress(&compressed).unwrap();

        assert_eq!(strings, decompressed);
    }

    #[test]
    fn test_compression_ratio() {
        let strings = vec![
            "user_12345_data".to_string(),
            "user_12346_data".to_string(),
            "user_12347_data".to_string(),
            "user_12348_data".to_string(),
        ];

        let ratio = PrefixCompressor::compression_ratio(&strings);
        assert!(ratio > 1.0); // Should achieve some compression
    }

    #[test]
    fn test_empty_input() {
        let strings: Vec<String> = vec![];
        let compressed = PrefixCompressor::compress(&strings).unwrap();
        assert!(compressed.is_empty());
    }

    #[test]
    fn test_single_string() {
        let strings = vec!["hello".to_string()];
        let compressed = PrefixCompressor::compress(&strings).unwrap();
        let decompressed = PrefixCompressor::decompress(&compressed).unwrap();
        assert_eq!(strings, decompressed);
    }

    #[test]
    fn test_bytes_compression() {
        let sequences = vec![
            vec![1, 2, 3, 4, 5],
            vec![1, 2, 3, 6, 7],
            vec![1, 2, 3, 8, 9],
        ];

        let compressed = PrefixCompressor::compress_bytes(&sequences).unwrap();
        let decompressed = PrefixCompressor::decompress_bytes(&compressed).unwrap();

        assert_eq!(sequences, decompressed);
    }
}
