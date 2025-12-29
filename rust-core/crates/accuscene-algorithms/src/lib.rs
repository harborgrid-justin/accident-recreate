//! AccuScene Algorithms - Advanced compression, indexing, and storage algorithms.
//!
//! This crate provides production-ready implementations of advanced algorithms
//! for the AccuScene Enterprise platform:
//!
//! # Compression
//! - **LZ4 Streaming**: High-speed compression for real-time data
//! - **Zstd Dictionary**: High-ratio compression with dictionary training
//! - **Delta Encoding**: Efficient encoding of incremental changes
//! - **Run-Length Encoding**: Optimal for sparse data
//! - **Adaptive Selection**: Automatic algorithm selection based on data characteristics
//!
//! # Indexing
//! - **B+ Tree**: Disk-friendly sorted index
//! - **R-Tree**: Spatial indexing for 3D scene objects
//! - **Spatial Hash**: Fast collision detection and proximity queries
//! - **Bloom Filter**: Probabilistic existence checks
//! - **Cuckoo Filter**: Space-efficient filter with deletion support
//!
//! # Storage
//! - **Write-Ahead Log (WAL)**: Durable transaction logging
//! - **MVCC**: Multi-version concurrency control for lock-free reads
//! - **Page Management**: Fixed-size page I/O
//! - **Buffer Pool**: LRU caching for hot pages
//!
//! # Encoding
//! - **Varint**: Variable-length integer encoding
//! - **ZigZag**: Efficient encoding for signed integers
//! - **Prefix Compression**: Compact storage for sorted strings
//!
//! # Examples
//!
//! ## Compression
//!
//! ```rust
//! use accuscene_algorithms::compression::{AdaptiveCompressor, Compressor};
//! use accuscene_algorithms::config::CompressionConfig;
//!
//! let mut compressor = AdaptiveCompressor::new(CompressionConfig::default());
//! let data = b"Hello, World!".repeat(100);
//!
//! let compressed = compressor.compress(&data).unwrap();
//! let decompressed = compressor.decompress(&compressed).unwrap();
//!
//! assert_eq!(data.to_vec(), decompressed);
//! ```
//!
//! ## Indexing
//!
//! ```rust
//! use accuscene_algorithms::indexing::{BPlusTree, BoundingBox, Point, RTree};
//! use accuscene_algorithms::config::{BTreeConfig, RTreeConfig};
//!
//! // B+ Tree for sorted data
//! let btree = BPlusTree::new(BTreeConfig::default());
//! btree.insert(42, "value").unwrap();
//! assert_eq!(btree.search(&42), Some("value"));
//!
//! // R-Tree for spatial data
//! let rtree = RTree::new(RTreeConfig::default());
//! let bounds = BoundingBox::new(
//!     Point::new(0.0, 0.0, 0.0),
//!     Point::new(10.0, 10.0, 10.0)
//! );
//! rtree.insert(bounds, "object").unwrap();
//! ```
//!
//! ## Storage
//!
//! ```rust
//! use accuscene_algorithms::storage::{MvccEngine, TransactionId};
//! use accuscene_algorithms::config::MvccConfig;
//!
//! let mvcc = MvccEngine::new(MvccConfig::default());
//!
//! // Transaction 1
//! let txn1 = mvcc.begin();
//! mvcc.write(txn1, "key", "value").unwrap();
//! mvcc.commit(txn1).unwrap();
//!
//! // Transaction 2 (concurrent read)
//! let txn2 = mvcc.begin();
//! let value = mvcc.read(txn2, &"key").unwrap();
//! assert_eq!(value, Some("value"));
//! ```

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

pub mod benchmarks;
pub mod compression;
pub mod config;
pub mod encoding;
pub mod error;
pub mod indexing;
pub mod storage;

// Re-export commonly used types
pub use config::{
    BloomConfig, BTreeConfig, BufferPoolConfig, CompressionConfig, CompressionLevel,
    CuckooConfig, MvccConfig, RTreeConfig, SpatialHashConfig, SyncMode, WalConfig,
};
pub use error::{AlgorithmError, Result};

/// Crate version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Check if optimizations are enabled.
pub const fn is_optimized() -> bool {
    cfg!(not(debug_assertions))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
