//! LRU buffer pool for caching disk pages.
//!
//! Maintains a cache of frequently accessed pages in memory to reduce disk I/O.
//! Uses Least Recently Used (LRU) eviction policy.
//!
//! # Complexity
//! - Get: O(1) average
//! - Put: O(1) average
//! - Evict: O(1)

use crate::config::BufferPoolConfig;
use crate::error::{AlgorithmError, Result};
use crate::storage::page::{Page, PageFile, PageId};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// LRU list node.
struct LruNode {
    page_id: PageId,
    prev: Option<PageId>,
    next: Option<PageId>,
}

/// Buffer pool for caching pages.
///
/// Uses LRU (Least Recently Used) eviction policy to manage cache.
pub struct BufferPool {
    /// Cached pages.
    cache: Arc<RwLock<HashMap<PageId, Arc<RwLock<Page>>>>>,
    /// LRU list for eviction.
    lru: Arc<RwLock<HashMap<PageId, LruNode>>>,
    /// Head of LRU list (most recently used).
    lru_head: Arc<RwLock<Option<PageId>>>,
    /// Tail of LRU list (least recently used).
    lru_tail: Arc<RwLock<Option<PageId>>>,
    /// Page file for disk I/O.
    page_file: PageFile,
    /// Configuration.
    config: BufferPoolConfig,
    /// Statistics.
    hits: Arc<RwLock<u64>>,
    misses: Arc<RwLock<u64>>,
}

impl BufferPool {
    /// Create a new buffer pool.
    pub fn new(page_file: PageFile, config: BufferPoolConfig) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            lru: Arc::new(RwLock::new(HashMap::new())),
            lru_head: Arc::new(RwLock::new(None)),
            lru_tail: Arc::new(RwLock::new(None)),
            page_file,
            config,
            hits: Arc::new(RwLock::new(0)),
            misses: Arc::new(RwLock::new(0)),
        }
    }

    /// Get a page from the buffer pool.
    ///
    /// Loads from disk if not in cache.
    ///
    /// # Complexity
    /// O(1) average (cache hit), O(disk) on miss
    pub fn get_page(&self, page_id: PageId) -> Result<Arc<RwLock<Page>>> {
        // Check cache
        {
            let cache = self.cache.read();
            if let Some(page) = cache.get(&page_id) {
                *self.hits.write() += 1;
                self.mark_accessed(page_id);
                return Ok(Arc::clone(page));
            }
        }

        // Cache miss - load from disk
        *self.misses.write() += 1;
        let page = self.page_file.read_page(page_id)?;

        // Add to cache
        self.put_page(page_id, page)?;

        // Return from cache
        let cache = self.cache.read();
        Ok(Arc::clone(
            cache
                .get(&page_id)
                .ok_or_else(|| AlgorithmError::PageError("Page not in cache".to_string()))?,
        ))
    }

    /// Put a page in the buffer pool.
    fn put_page(&self, page_id: PageId, page: Page) -> Result<()> {
        let mut cache = self.cache.write();

        // Check if we need to evict
        if cache.len() >= self.config.pool_size && !cache.contains_key(&page_id) {
            self.evict_lru()?;
        }

        // Add to cache
        cache.insert(page_id, Arc::new(RwLock::new(page)));

        // Update LRU
        self.mark_accessed(page_id);

        Ok(())
    }

    /// Mark a page as accessed (move to head of LRU).
    fn mark_accessed(&self, page_id: PageId) {
        let mut lru = self.lru.write();
        let mut head = self.lru_head.write();
        let mut tail = self.lru_tail.write();

        // Remove from current position if exists
        if let Some(node) = lru.get(&page_id) {
            if let Some(prev) = node.prev {
                if let Some(prev_node) = lru.get_mut(&prev) {
                    prev_node.next = node.next;
                }
            }
            if let Some(next) = node.next {
                if let Some(next_node) = lru.get_mut(&next) {
                    next_node.prev = node.prev;
                }
            }

            // Update tail if necessary
            if *tail == Some(page_id) {
                *tail = node.prev;
            }
        }

        // Add to head
        let old_head = *head;
        let new_node = LruNode {
            page_id,
            prev: None,
            next: old_head,
        };

        if let Some(old_head_id) = old_head {
            if let Some(old_head_node) = lru.get_mut(&old_head_id) {
                old_head_node.prev = Some(page_id);
            }
        } else {
            // List was empty
            *tail = Some(page_id);
        }

        lru.insert(page_id, new_node);
        *head = Some(page_id);
    }

    /// Evict least recently used page.
    fn evict_lru(&self) -> Result<()> {
        let tail_id = {
            let tail = self.lru_tail.read();
            tail.ok_or_else(|| AlgorithmError::PageError("LRU list empty".to_string()))?
        };

        // Flush if dirty
        {
            let cache = self.cache.read();
            if let Some(page_arc) = cache.get(&tail_id) {
                let mut page = page_arc.write();
                if page.is_dirty() {
                    self.page_file.write_page(&mut page)?;
                }
            }
        }

        // Remove from cache
        self.cache.write().remove(&tail_id);

        // Update LRU list
        let mut lru = self.lru.write();
        let mut tail = self.lru_tail.write();
        let mut head = self.lru_head.write();

        if let Some(node) = lru.remove(&tail_id) {
            *tail = node.prev;

            if let Some(prev_id) = node.prev {
                if let Some(prev_node) = lru.get_mut(&prev_id) {
                    prev_node.next = None;
                }
            } else {
                // List is now empty
                *head = None;
            }
        }

        Ok(())
    }

    /// Flush all dirty pages to disk.
    pub fn flush_all(&self) -> Result<()> {
        let cache = self.cache.read();
        for page_arc in cache.values() {
            let mut page = page_arc.write();
            if page.is_dirty() {
                self.page_file.write_page(&mut page)?;
            }
        }
        self.page_file.flush()?;
        Ok(())
    }

    /// Flush a specific page.
    pub fn flush_page(&self, page_id: PageId) -> Result<()> {
        let cache = self.cache.read();
        if let Some(page_arc) = cache.get(&page_id) {
            let mut page = page_arc.write();
            if page.is_dirty() {
                self.page_file.write_page(&mut page)?;
            }
        }
        Ok(())
    }

    /// Allocate a new page.
    pub fn allocate_page(&self) -> Result<Arc<RwLock<Page>>> {
        let page = self.page_file.allocate_page()?;
        let page_id = page.id();
        self.put_page(page_id, page)?;
        self.get_page(page_id)
    }

    /// Get cache statistics.
    pub fn stats(&self) -> BufferPoolStats {
        let hits = *self.hits.read();
        let misses = *self.misses.read();
        let total = hits + misses;

        BufferPoolStats {
            hits,
            misses,
            hit_rate: if total > 0 {
                hits as f64 / total as f64
            } else {
                0.0
            },
            cache_size: self.cache.read().len(),
            capacity: self.config.pool_size,
        }
    }

    /// Reset statistics.
    pub fn reset_stats(&self) {
        *self.hits.write() = 0;
        *self.misses.write() = 0;
    }

    /// Get number of cached pages.
    pub fn size(&self) -> usize {
        self.cache.read().len()
    }

    /// Clear the buffer pool (flush and remove all pages).
    pub fn clear(&self) -> Result<()> {
        self.flush_all()?;
        self.cache.write().clear();
        self.lru.write().clear();
        *self.lru_head.write() = None;
        *self.lru_tail.write() = None;
        Ok(())
    }
}

/// Buffer pool statistics.
#[derive(Debug, Clone)]
pub struct BufferPoolStats {
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
    pub cache_size: usize,
    pub capacity: usize,
}

impl Clone for BufferPool {
    fn clone(&self) -> Self {
        Self {
            cache: Arc::new(RwLock::new(self.cache.read().clone())),
            lru: Arc::new(RwLock::new(self.lru.read().clone())),
            lru_head: Arc::new(RwLock::new(*self.lru_head.read())),
            lru_tail: Arc::new(RwLock::new(*self.lru_tail.read())),
            page_file: self.page_file.clone(),
            config: self.config.clone(),
            hits: Arc::new(RwLock::new(*self.hits.read())),
            misses: Arc::new(RwLock::new(*self.misses.read())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_buffer_pool() {
        let temp_file = NamedTempFile::new().unwrap();
        let page_file = PageFile::open(temp_file.path()).unwrap();
        let config = BufferPoolConfig {
            pool_size: 3,
            page_size: 4096,
            enable_prefetch: false,
        };

        let pool = BufferPool::new(page_file, config);

        // Allocate pages
        let page1 = pool.allocate_page().unwrap();
        let page2 = pool.allocate_page().unwrap();
        let page3 = pool.allocate_page().unwrap();

        let id1 = page1.read().id();
        let id2 = page2.read().id();
        let id3 = page3.read().id();

        // Access pages
        pool.get_page(id1).unwrap();
        pool.get_page(id2).unwrap();
        pool.get_page(id3).unwrap();

        let stats = pool.stats();
        assert!(stats.hits > 0 || stats.misses > 0);
    }

    #[test]
    fn test_eviction() {
        let temp_file = NamedTempFile::new().unwrap();
        let page_file = PageFile::open(temp_file.path()).unwrap();
        let config = BufferPoolConfig {
            pool_size: 2,
            page_size: 4096,
            enable_prefetch: false,
        };

        let pool = BufferPool::new(page_file, config);

        // Allocate 3 pages (should evict one)
        let _page1 = pool.allocate_page().unwrap();
        let _page2 = pool.allocate_page().unwrap();
        let _page3 = pool.allocate_page().unwrap();

        // Cache should not exceed capacity
        assert!(pool.size() <= 2);
    }
}
