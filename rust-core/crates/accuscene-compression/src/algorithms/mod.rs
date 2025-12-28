//! Compression algorithm implementations

pub mod lz4;
pub mod zstd;
pub mod brotli;
pub mod deflate;
pub mod snappy;

use crate::error::Result;
use crate::traits::{Algorithm, CompressionLevel, Compressor};

/// Get a compressor instance for the specified algorithm
pub fn get_compressor(algorithm: Algorithm) -> Box<dyn Compressor> {
    match algorithm {
        Algorithm::Lz4 => Box::new(lz4::Lz4Compressor::new()),
        Algorithm::Zstd => Box::new(zstd::ZstdCompressor::new()),
        Algorithm::Brotli => Box::new(brotli::BrotliCompressor::new()),
        Algorithm::Deflate => Box::new(deflate::DeflateCompressor::new()),
        Algorithm::Snappy => Box::new(snappy::SnappyCompressor::new()),
    }
}

/// Compress data with the specified algorithm and level
pub fn compress(data: &[u8], algorithm: Algorithm, level: CompressionLevel) -> Result<Vec<u8>> {
    let compressor = get_compressor(algorithm);
    compressor.compress(data, level)
}

/// Decompress data with the specified algorithm
pub fn decompress(data: &[u8], algorithm: Algorithm) -> Result<Vec<u8>> {
    let compressor = get_compressor(algorithm);
    compressor.decompress(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_trip_all_algorithms() {
        let test_data = b"Hello, AccuScene! This is test data for compression.".repeat(100);

        for algorithm in [
            Algorithm::Lz4,
            Algorithm::Zstd,
            Algorithm::Brotli,
            Algorithm::Deflate,
            Algorithm::Snappy,
        ] {
            let compressed = compress(&test_data, algorithm, CompressionLevel::Default).unwrap();
            let decompressed = decompress(&compressed, algorithm).unwrap();
            assert_eq!(test_data, decompressed, "Round trip failed for {:?}", algorithm);
        }
    }
}
