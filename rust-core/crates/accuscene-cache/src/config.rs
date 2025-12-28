//! Cache configuration and settings

use chrono::Duration;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Maximum number of entries in cache
    pub max_entries: usize,

    /// Maximum memory size in bytes
    pub max_memory_bytes: usize,

    /// Default time-to-live for entries
    pub default_ttl: Option<Duration>,

    /// Enable statistics collection
    pub enable_stats: bool,

    /// Enable tracing
    pub enable_tracing: bool,

    /// Eviction policy
    pub eviction_policy: EvictionPolicy,

    /// Cache tier configuration
    pub tier_config: TierConfig,

    /// Disk cache configuration
    pub disk_config: Option<DiskConfig>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 10_000,
            max_memory_bytes: 100 * 1024 * 1024, // 100MB
            default_ttl: Some(Duration::hours(1)),
            enable_stats: true,
            enable_tracing: true,
            eviction_policy: EvictionPolicy::Lru,
            tier_config: TierConfig::default(),
            disk_config: None,
        }
    }
}

/// Eviction policy types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EvictionPolicy {
    /// Least Recently Used
    Lru,
    /// Least Frequently Used
    Lfu,
    /// Time-to-Live based
    Ttl,
    /// Adaptive Replacement Cache
    Arc,
    /// No eviction (cache fills up)
    None,
}

/// Cache tier configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierConfig {
    /// Enable L1 (memory) cache
    pub enable_l1: bool,

    /// L1 cache size
    pub l1_max_entries: usize,

    /// Enable L2 (moka concurrent) cache
    pub enable_l2: bool,

    /// L2 cache size
    pub l2_max_entries: usize,

    /// Enable L3 (disk) cache
    pub enable_l3: bool,

    /// L3 cache size
    pub l3_max_entries: usize,
}

impl Default for TierConfig {
    fn default() -> Self {
        Self {
            enable_l1: true,
            l1_max_entries: 1_000,
            enable_l2: true,
            l2_max_entries: 10_000,
            enable_l3: false,
            l3_max_entries: 100_000,
        }
    }
}

/// Disk cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskConfig {
    /// Path to disk cache directory
    pub cache_dir: PathBuf,

    /// Maximum disk space in bytes
    pub max_disk_bytes: usize,

    /// Compression enabled
    pub enable_compression: bool,

    /// Flush interval in seconds
    pub flush_interval_secs: u64,
}

impl Default for DiskConfig {
    fn default() -> Self {
        Self {
            cache_dir: PathBuf::from("/tmp/accuscene-cache"),
            max_disk_bytes: 1024 * 1024 * 1024, // 1GB
            enable_compression: true,
            flush_interval_secs: 60,
        }
    }
}

/// Cache type classification
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CacheType {
    /// Physics calculation results
    Physics,
    /// Database query results
    Query,
    /// Rendered images/visualizations
    RenderedImage,
    /// User session data
    Session,
    /// Configuration data
    Configuration,
    /// Generic cached data
    Generic,
}

impl CacheType {
    /// Get recommended TTL for cache type
    pub fn default_ttl(&self) -> Option<Duration> {
        match self {
            CacheType::Physics => Some(Duration::hours(24)),
            CacheType::Query => Some(Duration::minutes(30)),
            CacheType::RenderedImage => Some(Duration::hours(6)),
            CacheType::Session => Some(Duration::hours(2)),
            CacheType::Configuration => Some(Duration::hours(12)),
            CacheType::Generic => Some(Duration::hours(1)),
        }
    }

    /// Get recommended max entries for cache type
    pub fn default_max_entries(&self) -> usize {
        match self {
            CacheType::Physics => 5_000,
            CacheType::Query => 10_000,
            CacheType::RenderedImage => 1_000,
            CacheType::Session => 5_000,
            CacheType::Configuration => 500,
            CacheType::Generic => 10_000,
        }
    }
}
