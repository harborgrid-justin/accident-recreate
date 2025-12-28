//! AccuScene Enterprise Cache System
//!
//! A sophisticated multi-tier caching system for the AccuScene accident recreation platform.
//!
//! # Features
//!
//! - **Multi-tier caching**: L1 (memory), L2 (concurrent), L3 (disk)
//! - **Multiple eviction policies**: LRU, LFU, TTL, Adaptive
//! - **Tag-based invalidation**: Group-based cache invalidation
//! - **Computed values**: Automatic computation and memoization
//! - **Statistics tracking**: Hit rates, eviction rates, utilization
//! - **Partitioning**: Separate caches by data type
//! - **Middleware**: Logging, metrics, validation
//! - **Distributed coordination**: Future support for multi-node caching
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```rust
//! use accuscene_cache::{
//!     backends::memory::MemoryCache,
//!     config::CacheConfig,
//!     key::CacheKey,
//!     value::CacheValue,
//! };
//!
//! // Create a cache
//! let config = CacheConfig::default();
//! let cache = MemoryCache::<String>::new(&config);
//!
//! // Insert and retrieve
//! let key = CacheKey::new("physics", "simulation_1");
//! let value = CacheValue::new("result_data".to_string());
//! // cache.insert(key.clone(), value).unwrap();
//! // let retrieved = cache.get(&key).unwrap();
//! ```
//!
//! ## Tiered Cache
//!
//! ```rust
//! use accuscene_cache::{
//!     backends::tiered::TieredCache,
//!     config::CacheConfig,
//! };
//!
//! let config = CacheConfig::default();
//! // let cache: TieredCache<String> = TieredCache::new(&config).unwrap();
//! ```
//!
//! ## Tagged Cache
//!
//! ```rust
//! use accuscene_cache::{
//!     backends::memory::MemoryCache,
//!     tags::TaggedCache,
//!     key::CacheKey,
//!     value::CacheValue,
//! };
//!
//! let cache = Box::new(MemoryCache::<String>::with_capacity(100));
//! let tagged = TaggedCache::new(cache);
//!
//! // Insert with tags
//! let key = CacheKey::new("user", "123");
//! let value = CacheValue::new("user_data".to_string());
//! let tags = vec!["active".to_string(), "premium".to_string()];
//! // tagged.insert_with_tags(key, value, tags).unwrap();
//!
//! // Invalidate by tag
//! // tagged.invalidate_tag("active").unwrap();
//! ```

pub mod backends;
pub mod computed;
pub mod config;
pub mod distributed;
pub mod error;
pub mod invalidation;
pub mod key;
pub mod middleware;
pub mod partitioning;
pub mod policy;
pub mod preload;
pub mod serialization;
pub mod stats;
pub mod tags;
pub mod value;

// Re-export commonly used types
pub use error::{CacheError, CacheResult};
pub use key::CacheKey;
pub use value::CacheValue;

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::backends::memory::MemoryCache;
    pub use crate::backends::moka::MokaCacheBackend;
    pub use crate::backends::tiered::TieredCache;
    pub use crate::backends::CacheBackend;
    pub use crate::computed::ComputedCache;
    pub use crate::config::{CacheConfig, CacheType, EvictionPolicy};
    pub use crate::error::{CacheError, CacheResult};
    pub use crate::key::{CacheKey, CacheKeyBuilder};
    pub use crate::partitioning::PartitionedCache;
    pub use crate::policy::lru::LruPolicy;
    pub use crate::stats::CacheStats;
    pub use crate::tags::TaggedCache;
    pub use crate::value::CacheValue;
}

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// Cache system information
pub fn info() -> String {
    format!("{} v{}", NAME, VERSION)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
        assert_eq!(NAME, "accuscene-cache");
    }

    #[test]
    fn test_info() {
        let info = info();
        assert!(info.contains("accuscene-cache"));
        assert!(info.contains(VERSION));
    }
}
