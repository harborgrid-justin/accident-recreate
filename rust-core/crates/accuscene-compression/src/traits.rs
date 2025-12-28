//! Traits for compression and serialization

use crate::error::Result;
use bytes::Bytes;

/// Compression algorithm identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Algorithm {
    /// LZ4 - Fast compression with moderate ratio
    Lz4,
    /// Zstandard - Balanced compression speed and ratio
    Zstd,
    /// Brotli - High compression ratio, slower
    Brotli,
    /// Deflate/gzip - Standard compression
    Deflate,
    /// Snappy - Very fast, moderate ratio, good for streaming
    Snappy,
}

impl Algorithm {
    /// Get algorithm from string identifier
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "lz4" => Some(Algorithm::Lz4),
            "zstd" | "zstandard" => Some(Algorithm::Zstd),
            "brotli" | "br" => Some(Algorithm::Brotli),
            "deflate" | "gzip" | "gz" => Some(Algorithm::Deflate),
            "snappy" | "snap" => Some(Algorithm::Snappy),
            _ => None,
        }
    }

    /// Get string identifier for algorithm
    pub fn as_str(&self) -> &'static str {
        match self {
            Algorithm::Lz4 => "lz4",
            Algorithm::Zstd => "zstd",
            Algorithm::Brotli => "brotli",
            Algorithm::Deflate => "deflate",
            Algorithm::Snappy => "snappy",
        }
    }

    /// Get file extension for algorithm
    pub fn extension(&self) -> &'static str {
        match self {
            Algorithm::Lz4 => "lz4",
            Algorithm::Zstd => "zst",
            Algorithm::Brotli => "br",
            Algorithm::Deflate => "gz",
            Algorithm::Snappy => "snap",
        }
    }
}

/// Compression level configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionLevel {
    /// Fastest compression with lowest ratio
    Fastest,
    /// Fast compression with moderate ratio
    Fast,
    /// Default balanced compression
    Default,
    /// High compression ratio, slower
    High,
    /// Maximum compression ratio, slowest
    Maximum,
    /// Custom level (algorithm-specific)
    Custom(i32),
}

impl CompressionLevel {
    /// Convert to algorithm-specific level
    pub fn to_level(&self, algorithm: Algorithm) -> i32 {
        match (self, algorithm) {
            (CompressionLevel::Fastest, Algorithm::Zstd) => 1,
            (CompressionLevel::Fast, Algorithm::Zstd) => 3,
            (CompressionLevel::Default, Algorithm::Zstd) => 3,
            (CompressionLevel::High, Algorithm::Zstd) => 15,
            (CompressionLevel::Maximum, Algorithm::Zstd) => 22,
            (CompressionLevel::Custom(level), _) => *level,

            (CompressionLevel::Fastest, Algorithm::Brotli) => 1,
            (CompressionLevel::Fast, Algorithm::Brotli) => 4,
            (CompressionLevel::Default, Algorithm::Brotli) => 6,
            (CompressionLevel::High, Algorithm::Brotli) => 9,
            (CompressionLevel::Maximum, Algorithm::Brotli) => 11,

            (CompressionLevel::Fastest, Algorithm::Deflate) => 1,
            (CompressionLevel::Fast, Algorithm::Deflate) => 3,
            (CompressionLevel::Default, Algorithm::Deflate) => 6,
            (CompressionLevel::High, Algorithm::Deflate) => 9,
            (CompressionLevel::Maximum, Algorithm::Deflate) => 9,

            // LZ4 and Snappy don't have configurable levels
            _ => 0,
        }
    }
}

/// Trait for types that can be compressed
pub trait Compressible: Sized {
    /// Compress the data using the specified algorithm and level
    fn compress(&self, algorithm: Algorithm, level: CompressionLevel) -> Result<Vec<u8>>;

    /// Decompress data into this type
    fn decompress(data: &[u8], algorithm: Algorithm) -> Result<Self>;

    /// Estimate the uncompressed size (if known)
    fn estimated_size(&self) -> Option<usize> {
        None
    }
}

/// Trait for compression algorithms
pub trait Compressor: Send + Sync {
    /// Compress data
    fn compress(&self, data: &[u8], level: CompressionLevel) -> Result<Vec<u8>>;

    /// Decompress data
    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>>;

    /// Get algorithm identifier
    fn algorithm(&self) -> Algorithm;

    /// Estimate compressed size (upper bound)
    fn max_compressed_size(&self, input_size: usize) -> usize;

    /// Check if algorithm supports parallel compression
    fn supports_parallel(&self) -> bool {
        false
    }

    /// Check if algorithm supports streaming
    fn supports_streaming(&self) -> bool {
        false
    }
}

/// Trait for streaming compression
#[cfg(feature = "async")]
#[async_trait::async_trait]
pub trait AsyncCompressor: Send + Sync {
    /// Compress data asynchronously
    async fn compress_async(&self, data: Bytes, level: CompressionLevel) -> Result<Bytes>;

    /// Decompress data asynchronously
    async fn decompress_async(&self, data: Bytes) -> Result<Bytes>;
}

/// Compression statistics
#[derive(Debug, Clone, Default)]
pub struct CompressionStats {
    /// Original uncompressed size
    pub original_size: usize,
    /// Compressed size
    pub compressed_size: usize,
    /// Compression ratio (compressed / original)
    pub ratio: f64,
    /// Time taken to compress (milliseconds)
    pub compression_time_ms: u64,
    /// Time taken to decompress (milliseconds)
    pub decompression_time_ms: u64,
    /// Algorithm used
    pub algorithm: Option<Algorithm>,
}

impl CompressionStats {
    /// Create new statistics
    pub fn new(original_size: usize, compressed_size: usize) -> Self {
        let ratio = if original_size > 0 {
            compressed_size as f64 / original_size as f64
        } else {
            0.0
        };

        Self {
            original_size,
            compressed_size,
            ratio,
            compression_time_ms: 0,
            decompression_time_ms: 0,
            algorithm: None,
        }
    }

    /// Calculate savings percentage
    pub fn savings_percent(&self) -> f64 {
        if self.original_size > 0 {
            (1.0 - self.ratio) * 100.0
        } else {
            0.0
        }
    }

    /// Get compression throughput (MB/s)
    pub fn compression_throughput(&self) -> f64 {
        if self.compression_time_ms > 0 {
            (self.original_size as f64 / 1_000_000.0) / (self.compression_time_ms as f64 / 1000.0)
        } else {
            0.0
        }
    }

    /// Get decompression throughput (MB/s)
    pub fn decompression_throughput(&self) -> f64 {
        if self.decompression_time_ms > 0 {
            (self.original_size as f64 / 1_000_000.0) / (self.decompression_time_ms as f64 / 1000.0)
        } else {
            0.0
        }
    }
}
