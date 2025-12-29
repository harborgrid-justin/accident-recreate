//! Zstandard compression with dictionary training for scene data.
//!
//! Provides high compression ratios by training dictionaries on sample data.
//! Ideal for repetitive scene structures and similar object types.
//!
//! # Complexity
//! - Compression: O(n) where n is input size
//! - Decompression: O(n) where n is output size
//! - Dictionary training: O(m * k) where m is sample count, k is sample size
//! - Memory: O(dict_size + 128KB) for compression context

use crate::compression::{CompressionStats, Compressor};
use crate::config::CompressionConfig;
use crate::error::{AlgorithmError, Result};
use parking_lot::RwLock;
use std::sync::Arc;

const MAGIC_NUMBER: u32 = 0x5D7EDB00;
const MIN_DICT_SIZE: usize = 256;
const MAX_DICT_SIZE: usize = 2 * 1024 * 1024; // 2 MB

/// Zstandard compressor with dictionary support.
///
/// Maintains a trained dictionary for improved compression of similar data.
/// The dictionary is automatically trained from sample data.
pub struct ZstdDictionary {
    dictionary: Arc<RwLock<Option<Vec<u8>>>>,
    compression_level: i32,
    stats: Arc<RwLock<CompressionStats>>,
    config: CompressionConfig,
}

impl ZstdDictionary {
    /// Create a new Zstd dictionary compressor.
    pub fn new(config: CompressionConfig) -> Self {
        Self {
            dictionary: Arc::new(RwLock::new(None)),
            compression_level: config.level.zstd_level(),
            stats: Arc::new(RwLock::new(CompressionStats::default())),
            config,
        }
    }

    /// Create with default configuration.
    pub fn default() -> Self {
        Self::new(CompressionConfig::default())
    }

    /// Train dictionary from sample data.
    ///
    /// # Arguments
    /// * `samples` - Sample data for training (should be representative)
    /// * `dict_size` - Target dictionary size in bytes
    ///
    /// # Complexity
    /// O(n * m) where n is number of samples, m is average sample size
    pub fn train_dictionary(&self, samples: &[&[u8]], dict_size: usize) -> Result<()> {
        if samples.is_empty() {
            return Err(AlgorithmError::InvalidConfig(
                "No samples provided for dictionary training".to_string(),
            ));
        }

        if samples.len() < self.config.min_sample_count {
            return Err(AlgorithmError::InvalidConfig(format!(
                "Insufficient samples: need at least {}, got {}",
                self.config.min_sample_count,
                samples.len()
            )));
        }

        let dict_size = dict_size.clamp(MIN_DICT_SIZE, MAX_DICT_SIZE);

        // Flatten samples for zstd training
        let total_size: usize = samples.iter().map(|s| s.len()).sum();
        let mut sample_data = Vec::with_capacity(total_size);
        let mut sample_sizes = Vec::with_capacity(samples.len());

        for sample in samples {
            sample_data.extend_from_slice(sample);
            sample_sizes.push(sample.len());
        }

        // Train dictionary using zstd
        let dictionary = zstd::dict::from_continuous(&sample_data, &sample_sizes, dict_size)
            .map_err(|e| AlgorithmError::DictionaryTrainingFailed(e.to_string()))?;

        *self.dictionary.write() = Some(dictionary);

        Ok(())
    }

    /// Compress data using the trained dictionary.
    ///
    /// # Complexity
    /// O(n) where n is input size
    pub fn compress_with_dict(&self, input: &[u8]) -> Result<Vec<u8>> {
        if input.is_empty() {
            return Ok(Vec::new());
        }

        let compressed = if let Some(dict) = self.dictionary.read().as_ref() {
            // Compress with dictionary
            zstd::bulk::compress_using_dict(input, dict, self.compression_level)
                .map_err(|e| AlgorithmError::CompressionFailed(e.to_string()))?
        } else {
            // Compress without dictionary
            zstd::bulk::compress(input, self.compression_level)
                .map_err(|e| AlgorithmError::CompressionFailed(e.to_string()))?
        };

        // Update statistics
        self.stats.write().record(input.len(), compressed.len());

        Ok(compressed)
    }

    /// Decompress data using the trained dictionary.
    ///
    /// # Complexity
    /// O(n) where n is output size
    pub fn decompress_with_dict(&self, input: &[u8]) -> Result<Vec<u8>> {
        if input.is_empty() {
            return Ok(Vec::new());
        }

        let decompressed = if let Some(dict) = self.dictionary.read().as_ref() {
            // Decompress with dictionary
            zstd::bulk::decompress_using_dict(input, 10 * input.len(), dict)
                .map_err(|e| AlgorithmError::DecompressionFailed(e.to_string()))?
        } else {
            // Decompress without dictionary
            zstd::bulk::decompress(input, 10 * input.len())
                .map_err(|e| AlgorithmError::DecompressionFailed(e.to_string()))?
        };

        Ok(decompressed)
    }

    /// Check if dictionary is trained.
    pub fn has_dictionary(&self) -> bool {
        self.dictionary.read().is_some()
    }

    /// Get dictionary size in bytes.
    pub fn dictionary_size(&self) -> usize {
        self.dictionary
            .read()
            .as_ref()
            .map(|d| d.len())
            .unwrap_or(0)
    }

    /// Get compression statistics.
    pub fn stats(&self) -> CompressionStats {
        self.stats.read().clone()
    }

    /// Reset statistics.
    pub fn reset_stats(&self) {
        self.stats.write().reset();
    }

    /// Clear the trained dictionary.
    pub fn clear_dictionary(&self) {
        *self.dictionary.write() = None;
    }
}

impl Clone for ZstdDictionary {
    fn clone(&self) -> Self {
        Self {
            dictionary: Arc::new(RwLock::new(self.dictionary.read().clone())),
            compression_level: self.compression_level,
            stats: Arc::new(RwLock::new(self.stats.read().clone())),
            config: self.config.clone(),
        }
    }
}

impl Compressor for ZstdDictionary {
    fn compress(&mut self, input: &[u8]) -> Result<Vec<u8>> {
        self.compress_with_dict(input)
    }

    fn decompress(&mut self, input: &[u8]) -> Result<Vec<u8>> {
        self.decompress_with_dict(input)
    }

    fn compression_ratio(&self) -> f64 {
        self.stats.read().ratio()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_decompress_no_dict() {
        let compressor = ZstdDictionary::default();
        let data = b"Hello, World! This is a test of Zstd compression.".repeat(10);

        let compressed = compressor.compress_with_dict(&data).unwrap();
        assert!(compressed.len() < data.len());

        let decompressed = compressor.decompress_with_dict(&compressed).unwrap();
        assert_eq!(data, decompressed.as_slice());
    }

    #[test]
    fn test_dictionary_training() {
        let compressor = ZstdDictionary::default();

        let samples: Vec<Vec<u8>> = (0..200)
            .map(|i| format!("Sample data {}: This is repetitive content", i).into_bytes())
            .collect();
        let sample_refs: Vec<&[u8]> = samples.iter().map(|s| s.as_slice()).collect();

        compressor.train_dictionary(&sample_refs, 4096).unwrap();
        assert!(compressor.has_dictionary());
        assert!(compressor.dictionary_size() > 0);
    }

    #[test]
    fn test_compress_with_dictionary() {
        let compressor = ZstdDictionary::default();

        // Train dictionary
        let samples: Vec<Vec<u8>> = (0..200)
            .map(|i| format!("Sample data {}: This is repetitive content", i).into_bytes())
            .collect();
        let sample_refs: Vec<&[u8]> = samples.iter().map(|s| s.as_slice()).collect();
        compressor.train_dictionary(&sample_refs, 4096).unwrap();

        // Compress similar data
        let data = b"Sample data 999: This is repetitive content".repeat(5);
        let compressed = compressor.compress_with_dict(&data).unwrap();
        let decompressed = compressor.decompress_with_dict(&compressed).unwrap();

        assert_eq!(data, decompressed.as_slice());
    }
}
