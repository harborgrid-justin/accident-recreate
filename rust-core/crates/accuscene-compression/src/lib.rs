//! # AccuScene Compression
//!
//! Advanced compression and serialization system for AccuScene Enterprise.
//!
//! ## Features
//!
//! - **Multiple Algorithms**: LZ4, Zstandard, Brotli, Deflate, Snappy
//! - **Adaptive Compression**: Automatically selects the best algorithm
//! - **Streaming Support**: Compress large files efficiently
//! - **Dictionary Compression**: Improved ratios for similar data
//! - **Delta Compression**: Efficient incremental saves
//! - **Archive Format**: Bundle multiple files into .accuscene archives
//! - **Encryption**: Optional AES-256-GCM encryption layer
//! - **Serialization**: Binary, MessagePack, and compact formats
//! - **Benchmarking**: Built-in performance testing
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use accuscene_compression::{compress, decompress, Algorithm, CompressionLevel};
//!
//! let data = b"Hello, AccuScene!";
//!
//! // Simple compression
//! let compressed = compress(data, Algorithm::Zstd, CompressionLevel::Default).unwrap();
//! let decompressed = decompress(&compressed, Algorithm::Zstd).unwrap();
//!
//! assert_eq!(data.to_vec(), decompressed);
//! ```
//!
//! ## Adaptive Compression
//!
//! ```rust,no_run
//! use accuscene_compression::adaptive::{compress_auto, decompress_auto};
//!
//! let data = b"Data to compress";
//! let compressed = compress_auto(data).unwrap();
//! let decompressed = decompress_auto(&compressed).unwrap();
//! ```
//!
//! ## Archive Format
//!
//! ```rust,no_run
//! use accuscene_compression::archive::{Archive, ArchiveEntry};
//! use accuscene_compression::{Algorithm, CompressionLevel};
//!
//! let mut archive = Archive::new(Algorithm::Zstd, CompressionLevel::Default);
//! archive.add_file("data.txt".to_string(), b"File contents".to_vec());
//!
//! let bytes = archive.to_bytes().unwrap();
//! let restored = Archive::from_bytes(&bytes).unwrap();
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod adaptive;
pub mod algorithms;
pub mod archive;
pub mod benchmark;
pub mod delta;
pub mod dictionary;
pub mod encryption;
pub mod error;
pub mod serialization;
pub mod streaming;
pub mod traits;

// Re-export commonly used types
pub use error::{CompressionError, Result};
pub use traits::{
    Algorithm, CompressionLevel, CompressionStats, Compressible, Compressor,
};

#[cfg(feature = "async")]
pub use traits::AsyncCompressor;

/// Compress data using the specified algorithm and level
///
/// # Examples
///
/// ```rust,no_run
/// use accuscene_compression::{compress, Algorithm, CompressionLevel};
///
/// let data = b"Hello, world!";
/// let compressed = compress(data, Algorithm::Zstd, CompressionLevel::Default).unwrap();
/// ```
pub fn compress(data: &[u8], algorithm: Algorithm, level: CompressionLevel) -> Result<Vec<u8>> {
    algorithms::compress(data, algorithm, level)
}

/// Decompress data using the specified algorithm
///
/// # Examples
///
/// ```rust,no_run
/// use accuscene_compression::{compress, decompress, Algorithm, CompressionLevel};
///
/// let data = b"Hello, world!";
/// let compressed = compress(data, Algorithm::Zstd, CompressionLevel::Default).unwrap();
/// let decompressed = decompress(&compressed, Algorithm::Zstd).unwrap();
///
/// assert_eq!(data.to_vec(), decompressed);
/// ```
pub fn decompress(data: &[u8], algorithm: Algorithm) -> Result<Vec<u8>> {
    algorithms::decompress(data, algorithm)
}

/// Compression facade providing a unified interface
pub struct CompressionFacade {
    default_algorithm: Algorithm,
    default_level: CompressionLevel,
}

impl CompressionFacade {
    /// Create a new compression facade with defaults
    pub fn new() -> Self {
        Self {
            default_algorithm: Algorithm::Zstd,
            default_level: CompressionLevel::Default,
        }
    }

    /// Create with custom default algorithm and level
    pub fn with_defaults(algorithm: Algorithm, level: CompressionLevel) -> Self {
        Self {
            default_algorithm: algorithm,
            default_level: level,
        }
    }

    /// Compress with default settings
    pub fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
        algorithms::compress(data, self.default_algorithm, self.default_level)
    }

    /// Decompress with default algorithm
    pub fn decompress(&self, data: &[u8]) -> Result<Vec<u8>> {
        algorithms::decompress(data, self.default_algorithm)
    }

    /// Compress with specific algorithm
    pub fn compress_with(
        &self,
        data: &[u8],
        algorithm: Algorithm,
        level: CompressionLevel,
    ) -> Result<Vec<u8>> {
        algorithms::compress(data, algorithm, level)
    }

    /// Get compression statistics
    pub fn get_stats(&self, data: &[u8]) -> Result<CompressionStats> {
        let compressed = self.compress(data)?;
        Ok(CompressionStats::new(data.len(), compressed.len()))
    }

    /// Benchmark all algorithms on the given data
    pub fn benchmark(&self, data: &[u8]) -> Result<Vec<benchmark::BenchmarkResult>> {
        benchmark::quick_benchmark(data)
    }

    /// Create an archive with default settings
    pub fn create_archive(&self) -> archive::Archive {
        archive::Archive::new(self.default_algorithm, self.default_level)
    }

    /// Create a dictionary from samples
    pub fn create_dictionary(
        &self,
        samples: &[&[u8]],
        dict_size: usize,
    ) -> Result<dictionary::CompressionDictionary> {
        dictionary::CompressionDictionary::train(
            samples,
            dict_size,
            "Auto-generated dictionary".to_string(),
        )
    }

    /// Compress and encrypt (if encryption feature is enabled)
    #[cfg(feature = "encryption")]
    pub fn compress_encrypt(&self, data: &[u8], password: &str) -> Result<Vec<u8>> {
        encryption::compress_encrypt(data, password, self.default_algorithm, self.default_level)
    }

    /// Decrypt and decompress (if encryption feature is enabled)
    #[cfg(feature = "encryption")]
    pub fn decrypt_decompress(&self, data: &[u8], password: &str) -> Result<Vec<u8>> {
        encryption::decrypt_decompress(data, password)
    }
}

impl Default for CompressionFacade {
    fn default() -> Self {
        Self::new()
    }
}

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::adaptive::{compress_auto, decompress_auto, AdaptiveCompressor};
    pub use crate::algorithms;
    pub use crate::archive::{Archive, ArchiveEntry};
    pub use crate::benchmark::{benchmark, BenchmarkConfig, BenchmarkResult};
    pub use crate::delta::{compress_delta, decompress_delta, DeltaPatch};
    pub use crate::dictionary::{CompressionDictionary, DictionaryManager};
    pub use crate::error::{CompressionError, Result};
    pub use crate::serialization::{self, SerializationFormat};
    pub use crate::streaming::{compress_stream, decompress_stream};
    pub use crate::traits::{
        Algorithm, CompressionLevel, CompressionStats, Compressible, Compressor,
    };
    pub use crate::{compress, decompress, CompressionFacade};

    #[cfg(feature = "encryption")]
    pub use crate::encryption::{compress_encrypt, decrypt_decompress};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_compression() {
        let data = b"Hello, AccuScene! This is a test of the compression system.";

        for algorithm in [
            Algorithm::Lz4,
            Algorithm::Zstd,
            Algorithm::Brotli,
            Algorithm::Deflate,
            Algorithm::Snappy,
        ] {
            let compressed = compress(data, algorithm, CompressionLevel::Default).unwrap();
            let decompressed = decompress(&compressed, algorithm).unwrap();

            assert_eq!(data.to_vec(), decompressed);
        }
    }

    #[test]
    fn test_compression_facade() {
        let facade = CompressionFacade::new();
        let data = b"Testing the compression facade";

        let compressed = facade.compress(data).unwrap();
        let decompressed = facade.decompress(&compressed).unwrap();

        assert_eq!(data.to_vec(), decompressed);
    }

    #[test]
    fn test_facade_stats() {
        let facade = CompressionFacade::new();
        let data = b"Statistics test data".repeat(100);

        let stats = facade.get_stats(&data).unwrap();

        assert_eq!(stats.original_size, data.len());
        assert!(stats.compressed_size < data.len());
        assert!(stats.ratio < 1.0);
    }

    #[test]
    fn test_large_data() {
        let data = vec![0u8; 1_000_000]; // 1MB of zeros
        let facade = CompressionFacade::new();

        let compressed = facade.compress(&data).unwrap();
        let decompressed = facade.decompress(&compressed).unwrap();

        assert_eq!(data, decompressed);
        assert!(compressed.len() < data.len() / 100); // Should compress extremely well
    }
}