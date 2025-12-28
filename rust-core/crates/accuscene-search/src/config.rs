//! Search engine configuration

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    /// Path to the index directory
    pub index_path: PathBuf,

    /// Number of search threads
    pub num_threads: usize,

    /// Maximum number of search results
    pub max_results: usize,

    /// Enable fuzzy matching
    pub enable_fuzzy: bool,

    /// Fuzzy match distance (Levenshtein)
    pub fuzzy_distance: u8,

    /// Minimum fuzzy match prefix length
    pub fuzzy_prefix_length: usize,

    /// Enable query suggestions
    pub enable_suggestions: bool,

    /// Maximum suggestions to return
    pub max_suggestions: usize,

    /// Enable result highlighting
    pub enable_highlighting: bool,

    /// Maximum highlight snippet length
    pub highlight_snippet_length: usize,

    /// Index writer heap size (bytes)
    pub writer_heap_size: usize,

    /// Batch commit interval (seconds)
    pub commit_interval_secs: u64,

    /// Search timeout (milliseconds)
    pub search_timeout_ms: u64,

    /// BM25 parameters
    pub bm25_config: BM25Config,

    /// Facet configuration
    pub facet_config: FacetConfig,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            index_path: PathBuf::from("./data/search_index"),
            num_threads: num_cpus::get(),
            max_results: 1000,
            enable_fuzzy: true,
            fuzzy_distance: 2,
            fuzzy_prefix_length: 2,
            enable_suggestions: true,
            max_suggestions: 10,
            enable_highlighting: true,
            highlight_snippet_length: 150,
            writer_heap_size: 128 * 1024 * 1024, // 128 MB
            commit_interval_secs: 30,
            search_timeout_ms: 5000,
            bm25_config: BM25Config::default(),
            facet_config: FacetConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BM25Config {
    /// k1 parameter (term frequency saturation)
    pub k1: f32,

    /// b parameter (length normalization)
    pub b: f32,

    /// Minimum term frequency
    pub min_term_freq: u32,
}

impl Default for BM25Config {
    fn default() -> Self {
        Self {
            k1: 1.2,
            b: 0.75,
            min_term_freq: 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacetConfig {
    /// Maximum facet values per field
    pub max_facet_values: usize,

    /// Minimum facet count to include
    pub min_facet_count: u64,

    /// Enable facet caching
    pub enable_caching: bool,

    /// Facet cache TTL (seconds)
    pub cache_ttl_secs: u64,
}

impl Default for FacetConfig {
    fn default() -> Self {
        Self {
            max_facet_values: 100,
            min_facet_count: 1,
            enable_caching: true,
            cache_ttl_secs: 300,
        }
    }
}

impl SearchConfig {
    pub fn builder() -> SearchConfigBuilder {
        SearchConfigBuilder::default()
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.num_threads == 0 {
            return Err("num_threads must be greater than 0".to_string());
        }

        if self.max_results == 0 {
            return Err("max_results must be greater than 0".to_string());
        }

        if self.fuzzy_distance > 3 {
            return Err("fuzzy_distance should not exceed 3".to_string());
        }

        if self.writer_heap_size < 1024 * 1024 {
            return Err("writer_heap_size must be at least 1 MB".to_string());
        }

        if self.bm25_config.k1 <= 0.0 {
            return Err("BM25 k1 must be positive".to_string());
        }

        if self.bm25_config.b < 0.0 || self.bm25_config.b > 1.0 {
            return Err("BM25 b must be between 0 and 1".to_string());
        }

        Ok(())
    }
}

#[derive(Default)]
pub struct SearchConfigBuilder {
    config: SearchConfig,
}

impl SearchConfigBuilder {
    pub fn index_path(mut self, path: PathBuf) -> Self {
        self.config.index_path = path;
        self
    }

    pub fn num_threads(mut self, threads: usize) -> Self {
        self.config.num_threads = threads;
        self
    }

    pub fn max_results(mut self, max: usize) -> Self {
        self.config.max_results = max;
        self
    }

    pub fn enable_fuzzy(mut self, enabled: bool) -> Self {
        self.config.enable_fuzzy = enabled;
        self
    }

    pub fn fuzzy_distance(mut self, distance: u8) -> Self {
        self.config.fuzzy_distance = distance;
        self
    }

    pub fn bm25_k1(mut self, k1: f32) -> Self {
        self.config.bm25_config.k1 = k1;
        self
    }

    pub fn bm25_b(mut self, b: f32) -> Self {
        self.config.bm25_config.b = b;
        self
    }

    pub fn build(self) -> Result<SearchConfig, String> {
        self.config.validate()?;
        Ok(self.config)
    }
}

// Add num_cpus for CPU detection
mod num_cpus {
    pub fn get() -> usize {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
    }
}
