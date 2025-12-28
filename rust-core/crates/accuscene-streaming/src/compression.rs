//! Message compression and decompression utilities.

use crate::error::{Result, StreamingError};
use flate2::read::{GzDecoder, GzEncoder};
use flate2::Compression;
use std::io::Read;

/// Compression level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionLevel {
    /// No compression
    None,
    /// Fast compression (level 1)
    Fast,
    /// Default compression (level 6)
    Default,
    /// Best compression (level 9)
    Best,
}

impl CompressionLevel {
    /// Convert to flate2 compression level
    fn to_flate2(&self) -> Compression {
        match self {
            CompressionLevel::None => Compression::none(),
            CompressionLevel::Fast => Compression::fast(),
            CompressionLevel::Default => Compression::default(),
            CompressionLevel::Best => Compression::best(),
        }
    }
}

/// Compression configuration
#[derive(Debug, Clone)]
pub struct CompressionConfig {
    /// Compression enabled
    pub enabled: bool,
    /// Compression level
    pub level: CompressionLevel,
    /// Minimum message size to compress (bytes)
    pub min_size: usize,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            level: CompressionLevel::Default,
            min_size: 1024, // 1KB
        }
    }
}

impl CompressionConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn with_level(mut self, level: CompressionLevel) -> Self {
        self.level = level;
        self
    }

    pub fn with_min_size(mut self, min_size: usize) -> Self {
        self.min_size = min_size;
        self
    }

    /// Disable compression
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            level: CompressionLevel::None,
            min_size: 0,
        }
    }
}

/// Compressor for compressing messages
pub struct Compressor {
    config: CompressionConfig,
}

impl Compressor {
    /// Create a new compressor
    pub fn new(config: CompressionConfig) -> Self {
        Self { config }
    }

    /// Compress data
    pub fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
        if !self.config.enabled || data.len() < self.config.min_size {
            return Ok(data.to_vec());
        }

        let mut encoder = GzEncoder::new(data, self.config.level.to_flate2());
        let mut compressed = Vec::new();

        encoder
            .read_to_end(&mut compressed)
            .map_err(|e| StreamingError::Compression(e.to_string()))?;

        Ok(compressed)
    }

    /// Compress a string
    pub fn compress_string(&self, data: &str) -> Result<Vec<u8>> {
        self.compress(data.as_bytes())
    }

    /// Check if data should be compressed
    pub fn should_compress(&self, data: &[u8]) -> bool {
        self.config.enabled && data.len() >= self.config.min_size
    }
}

impl Default for Compressor {
    fn default() -> Self {
        Self::new(CompressionConfig::default())
    }
}

/// Decompressor for decompressing messages
pub struct Decompressor;

impl Decompressor {
    /// Create a new decompressor
    pub fn new() -> Self {
        Self
    }

    /// Decompress data
    pub fn decompress(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut decoder = GzDecoder::new(data);
        let mut decompressed = Vec::new();

        decoder
            .read_to_end(&mut decompressed)
            .map_err(|e| StreamingError::Decompression(e.to_string()))?;

        Ok(decompressed)
    }

    /// Decompress to string
    pub fn decompress_string(&self, data: &[u8]) -> Result<String> {
        let decompressed = self.decompress(data)?;
        String::from_utf8(decompressed)
            .map_err(|e| StreamingError::Decompression(e.to_string()))
    }

    /// Try to decompress, return original if not compressed
    pub fn try_decompress(&self, data: &[u8]) -> Vec<u8> {
        self.decompress(data).unwrap_or_else(|_| data.to_vec())
    }
}

impl Default for Decompressor {
    fn default() -> Self {
        Self::new()
    }
}

/// Compression utilities
pub struct CompressionUtils;

impl CompressionUtils {
    /// Calculate compression ratio
    pub fn compression_ratio(original_size: usize, compressed_size: usize) -> f64 {
        if original_size == 0 {
            return 0.0;
        }
        1.0 - (compressed_size as f64 / original_size as f64)
    }

    /// Calculate space saved
    pub fn space_saved(original_size: usize, compressed_size: usize) -> usize {
        original_size.saturating_sub(compressed_size)
    }

    /// Check if compression is worthwhile
    pub fn is_worthwhile(original_size: usize, compressed_size: usize, threshold: f64) -> bool {
        Self::compression_ratio(original_size, compressed_size) >= threshold
    }
}

/// Compression statistics
#[derive(Debug, Clone, Default)]
pub struct CompressionStats {
    /// Total bytes before compression
    pub bytes_in: usize,
    /// Total bytes after compression
    pub bytes_out: usize,
    /// Number of messages compressed
    pub messages_compressed: usize,
    /// Number of messages not compressed
    pub messages_uncompressed: usize,
}

impl CompressionStats {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a compression
    pub fn record_compressed(&mut self, original: usize, compressed: usize) {
        self.bytes_in += original;
        self.bytes_out += compressed;
        self.messages_compressed += 1;
    }

    /// Record an uncompressed message
    pub fn record_uncompressed(&mut self, size: usize) {
        self.bytes_in += size;
        self.bytes_out += size;
        self.messages_uncompressed += 1;
    }

    /// Get compression ratio
    pub fn ratio(&self) -> f64 {
        CompressionUtils::compression_ratio(self.bytes_in, self.bytes_out)
    }

    /// Get space saved
    pub fn space_saved(&self) -> usize {
        CompressionUtils::space_saved(self.bytes_in, self.bytes_out)
    }

    /// Get total messages
    pub fn total_messages(&self) -> usize {
        self.messages_compressed + self.messages_uncompressed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_decompress() {
        let compressor = Compressor::default();
        let decompressor = Decompressor::new();

        let data = "Hello, World! ".repeat(100);
        let compressed = compressor.compress_string(&data).unwrap();
        let decompressed = decompressor.decompress_string(&compressed).unwrap();

        assert_eq!(data, decompressed);
        assert!(compressed.len() < data.len());
    }

    #[test]
    fn test_compression_levels() {
        let data = "Test data ".repeat(100);

        let fast = Compressor::new(
            CompressionConfig::default().with_level(CompressionLevel::Fast),
        );
        let best = Compressor::new(
            CompressionConfig::default().with_level(CompressionLevel::Best),
        );

        let fast_compressed = fast.compress_string(&data).unwrap();
        let best_compressed = best.compress_string(&data).unwrap();

        // Best compression should produce smaller output
        assert!(best_compressed.len() <= fast_compressed.len());
    }

    #[test]
    fn test_min_size_threshold() {
        let config = CompressionConfig::default().with_min_size(1000);
        let compressor = Compressor::new(config);

        let small_data = "Small";
        let compressed = compressor.compress_string(small_data).unwrap();

        // Should not be compressed due to size threshold
        assert_eq!(compressed, small_data.as_bytes());
    }

    #[test]
    fn test_compression_disabled() {
        let config = CompressionConfig::disabled();
        let compressor = Compressor::new(config);

        let data = "Test data ".repeat(100);
        let compressed = compressor.compress_string(&data).unwrap();

        // Should not be compressed
        assert_eq!(compressed, data.as_bytes());
    }

    #[test]
    fn test_compression_ratio() {
        let ratio = CompressionUtils::compression_ratio(1000, 500);
        assert_eq!(ratio, 0.5); // 50% compression

        let ratio = CompressionUtils::compression_ratio(1000, 1000);
        assert_eq!(ratio, 0.0); // No compression
    }

    #[test]
    fn test_compression_stats() {
        let mut stats = CompressionStats::new();

        stats.record_compressed(1000, 500);
        stats.record_compressed(2000, 1000);
        stats.record_uncompressed(100);

        assert_eq!(stats.bytes_in, 3100);
        assert_eq!(stats.bytes_out, 1600);
        assert_eq!(stats.messages_compressed, 2);
        assert_eq!(stats.messages_uncompressed, 1);
        assert_eq!(stats.total_messages(), 3);

        let ratio = stats.ratio();
        assert!(ratio > 0.4 && ratio < 0.6); // Approximately 50% compression
    }

    #[test]
    fn test_should_compress() {
        let config = CompressionConfig::default().with_min_size(100);
        let compressor = Compressor::new(config);

        let small = vec![0u8; 50];
        let large = vec![0u8; 200];

        assert!(!compressor.should_compress(&small));
        assert!(compressor.should_compress(&large));
    }
}
