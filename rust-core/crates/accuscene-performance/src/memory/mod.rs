//! Memory management utilities for high-performance operations

pub mod arena;
pub mod pool;
pub mod slab;
pub mod zero_copy;

pub use arena::{Arena, ArenaAllocator};
pub use pool::{BufferPool, ObjectPool, PooledObject, StringPool};
pub use slab::{Slab, SlabAllocator};
pub use zero_copy::{ZeroCopyBuffer, ZeroCopySlice};

/// Memory alignment utilities
#[inline]
pub fn align_up(size: usize, alignment: usize) -> usize {
    (size + alignment - 1) & !(alignment - 1)
}

/// Check if a value is aligned
#[inline]
pub fn is_aligned(value: usize, alignment: usize) -> bool {
    value & (alignment - 1) == 0
}

/// Cache line size (typically 64 bytes on modern CPUs)
pub const CACHE_LINE_SIZE: usize = 64;

/// Align to cache line
#[inline]
pub fn cache_line_align(size: usize) -> usize {
    align_up(size, CACHE_LINE_SIZE)
}

/// Memory statistics
#[derive(Debug, Clone, Default)]
pub struct MemoryStats {
    /// Total bytes allocated
    pub allocated_bytes: usize,
    /// Total allocations
    pub allocation_count: usize,
    /// Total deallocations
    pub deallocation_count: usize,
    /// Current memory in use
    pub in_use_bytes: usize,
    /// Peak memory usage
    pub peak_bytes: usize,
}

impl MemoryStats {
    /// Create new memory statistics
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an allocation
    pub fn record_allocation(&mut self, size: usize) {
        self.allocated_bytes += size;
        self.allocation_count += 1;
        self.in_use_bytes += size;

        if self.in_use_bytes > self.peak_bytes {
            self.peak_bytes = self.in_use_bytes;
        }
    }

    /// Record a deallocation
    pub fn record_deallocation(&mut self, size: usize) {
        self.deallocation_count += 1;
        self.in_use_bytes = self.in_use_bytes.saturating_sub(size);
    }

    /// Get fragmentation ratio (0.0-1.0)
    pub fn fragmentation_ratio(&self) -> f32 {
        if self.allocated_bytes == 0 {
            0.0
        } else {
            1.0 - (self.in_use_bytes as f32 / self.allocated_bytes as f32)
        }
    }

    /// Get allocation efficiency (0.0-1.0)
    pub fn allocation_efficiency(&self) -> f32 {
        if self.peak_bytes == 0 {
            1.0
        } else {
            self.in_use_bytes as f32 / self.peak_bytes as f32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_align_up() {
        assert_eq!(align_up(1, 8), 8);
        assert_eq!(align_up(8, 8), 8);
        assert_eq!(align_up(9, 8), 16);
        assert_eq!(align_up(15, 8), 16);
    }

    #[test]
    fn test_is_aligned() {
        assert!(is_aligned(0, 8));
        assert!(is_aligned(8, 8));
        assert!(is_aligned(16, 8));
        assert!(!is_aligned(1, 8));
        assert!(!is_aligned(9, 8));
    }

    #[test]
    fn test_cache_line_align() {
        assert_eq!(cache_line_align(1), 64);
        assert_eq!(cache_line_align(64), 64);
        assert_eq!(cache_line_align(65), 128);
    }

    #[test]
    fn test_memory_stats() {
        let mut stats = MemoryStats::new();

        stats.record_allocation(1024);
        assert_eq!(stats.allocated_bytes, 1024);
        assert_eq!(stats.in_use_bytes, 1024);
        assert_eq!(stats.allocation_count, 1);

        stats.record_deallocation(512);
        assert_eq!(stats.in_use_bytes, 512);
        assert_eq!(stats.deallocation_count, 1);
    }
}
