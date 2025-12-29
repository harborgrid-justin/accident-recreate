//! Configuration types for AccuScene algorithms.

use serde::{Deserialize, Serialize};

/// Compression level configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionLevel {
    /// Fastest compression, lower ratio.
    Fast,
    /// Balanced compression and speed.
    Balanced,
    /// Maximum compression, slower.
    Max,
    /// Custom compression level (1-22 for Zstd, 1-17 for LZ4).
    Custom(i32),
}

impl CompressionLevel {
    /// Get Zstd compression level.
    pub fn zstd_level(&self) -> i32 {
        match self {
            CompressionLevel::Fast => 1,
            CompressionLevel::Balanced => 3,
            CompressionLevel::Max => 22,
            CompressionLevel::Custom(level) => (*level).clamp(1, 22),
        }
    }

    /// Get LZ4 acceleration level (higher = faster, lower compression).
    pub fn lz4_acceleration(&self) -> i32 {
        match self {
            CompressionLevel::Fast => 10,
            CompressionLevel::Balanced => 1,
            CompressionLevel::Max => 1,
            CompressionLevel::Custom(level) => (*level).clamp(1, 17),
        }
    }
}

/// Configuration for compression operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    /// Compression level.
    pub level: CompressionLevel,
    /// Enable dictionary training for Zstd.
    pub enable_dictionary: bool,
    /// Maximum dictionary size in bytes.
    pub max_dictionary_size: usize,
    /// Minimum sample count for dictionary training.
    pub min_sample_count: usize,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            level: CompressionLevel::Balanced,
            enable_dictionary: true,
            max_dictionary_size: 100 * 1024, // 100 KB
            min_sample_count: 100,
        }
    }
}

/// Configuration for B+ tree indexing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BTreeConfig {
    /// Maximum keys per node (order).
    pub order: usize,
    /// Enable bulk loading optimization.
    pub bulk_load: bool,
}

impl Default for BTreeConfig {
    fn default() -> Self {
        Self {
            order: 128,
            bulk_load: false,
        }
    }
}

/// Configuration for R-tree spatial indexing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RTreeConfig {
    /// Maximum entries per node.
    pub max_entries: usize,
    /// Minimum entries per node.
    pub min_entries: usize,
    /// Reinsert percentage for optimization.
    pub reinsert_p: f64,
}

impl Default for RTreeConfig {
    fn default() -> Self {
        Self {
            max_entries: 50,
            min_entries: 20,
            reinsert_p: 0.3,
        }
    }
}

/// Configuration for spatial hash grid.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialHashConfig {
    /// Cell size for the hash grid.
    pub cell_size: f64,
    /// Initial capacity per cell.
    pub cell_capacity: usize,
}

impl Default for SpatialHashConfig {
    fn default() -> Self {
        Self {
            cell_size: 1.0,
            cell_capacity: 16,
        }
    }
}

/// Configuration for Bloom filter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BloomConfig {
    /// Expected number of elements.
    pub expected_elements: usize,
    /// False positive probability (0.0-1.0).
    pub false_positive_rate: f64,
}

impl Default for BloomConfig {
    fn default() -> Self {
        Self {
            expected_elements: 10_000,
            false_positive_rate: 0.01,
        }
    }
}

impl BloomConfig {
    /// Calculate optimal bit array size.
    pub fn optimal_bit_size(&self) -> usize {
        let m = -(self.expected_elements as f64 * self.false_positive_rate.ln()
            / (2.0_f64.ln().powi(2)));
        m.ceil() as usize
    }

    /// Calculate optimal number of hash functions.
    pub fn optimal_hash_count(&self) -> usize {
        let k = (self.optimal_bit_size() as f64 / self.expected_elements as f64) * 2.0_f64.ln();
        k.ceil().max(1.0) as usize
    }
}

/// Configuration for Cuckoo filter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CuckooConfig {
    /// Capacity of the filter.
    pub capacity: usize,
    /// Fingerprint size in bits.
    pub fingerprint_bits: usize,
    /// Bucket size.
    pub bucket_size: usize,
    /// Maximum kick attempts.
    pub max_kicks: usize,
}

impl Default for CuckooConfig {
    fn default() -> Self {
        Self {
            capacity: 10_000,
            fingerprint_bits: 16,
            bucket_size: 4,
            max_kicks: 500,
        }
    }
}

/// Configuration for Write-Ahead Log (WAL).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalConfig {
    /// Maximum WAL file size in bytes.
    pub max_file_size: usize,
    /// Sync mode for durability.
    pub sync_mode: SyncMode,
    /// Enable compression for WAL entries.
    pub enable_compression: bool,
}

impl Default for WalConfig {
    fn default() -> Self {
        Self {
            max_file_size: 64 * 1024 * 1024, // 64 MB
            sync_mode: SyncMode::EveryWrite,
            enable_compression: true,
        }
    }
}

/// WAL sync mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncMode {
    /// Sync on every write (safest, slowest).
    EveryWrite,
    /// Sync every N writes.
    EveryN(usize),
    /// Sync on manual flush only.
    Manual,
}

/// Configuration for buffer pool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferPoolConfig {
    /// Number of pages in the pool.
    pub pool_size: usize,
    /// Page size in bytes.
    pub page_size: usize,
    /// Enable prefetching.
    pub enable_prefetch: bool,
}

impl Default for BufferPoolConfig {
    fn default() -> Self {
        Self {
            pool_size: 1024,
            page_size: 4096,
            enable_prefetch: true,
        }
    }
}

/// Configuration for MVCC (Multi-Version Concurrency Control).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MvccConfig {
    /// Maximum number of active versions per key.
    pub max_versions: usize,
    /// Garbage collection threshold.
    pub gc_threshold: usize,
}

impl Default for MvccConfig {
    fn default() -> Self {
        Self {
            max_versions: 10,
            gc_threshold: 100,
        }
    }
}
