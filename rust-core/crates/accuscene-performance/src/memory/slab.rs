//! Slab allocator for fixed-size objects

use crate::error::{PerformanceError, Result};
use crate::memory::MemoryStats;
use parking_lot::Mutex;
use std::sync::Arc;

/// Slab allocator for fixed-size objects
pub struct Slab {
    object_size: usize,
    capacity: usize,
    free_list: Vec<usize>,
    memory: Vec<u8>,
    stats: Arc<Mutex<MemoryStats>>,
}

impl Slab {
    /// Create a new slab allocator
    pub fn new(object_size: usize, capacity: usize) -> Self {
        let total_size = object_size * capacity;
        let memory = vec![0u8; total_size];

        // Initialize free list
        let free_list: Vec<usize> = (0..capacity).collect();

        let slab = Self {
            object_size,
            capacity,
            free_list,
            memory,
            stats: Arc::new(Mutex::new(MemoryStats::new())),
        };

        slab.stats.lock().allocated_bytes = total_size;

        slab
    }

    /// Allocate an object from the slab
    pub fn allocate(&mut self) -> Result<SlabHandle> {
        if let Some(index) = self.free_list.pop() {
            let offset = index * self.object_size;
            self.stats.lock().record_allocation(self.object_size);

            Ok(SlabHandle {
                index,
                offset,
                size: self.object_size,
            })
        } else {
            Err(PerformanceError::AllocationFailed(
                "Slab exhausted".to_string(),
            ))
        }
    }

    /// Deallocate an object
    pub fn deallocate(&mut self, handle: SlabHandle) {
        if handle.index < self.capacity {
            self.free_list.push(handle.index);
            self.stats.lock().record_deallocation(self.object_size);
        }
    }

    /// Get a mutable slice for a handle
    pub fn get_mut(&mut self, handle: &SlabHandle) -> Option<&mut [u8]> {
        if handle.offset + handle.size <= self.memory.len() {
            Some(&mut self.memory[handle.offset..handle.offset + handle.size])
        } else {
            None
        }
    }

    /// Get a slice for a handle
    pub fn get(&self, handle: &SlabHandle) -> Option<&[u8]> {
        if handle.offset + handle.size <= self.memory.len() {
            Some(&self.memory[handle.offset..handle.offset + handle.size])
        } else {
            None
        }
    }

    /// Get available objects
    pub fn available(&self) -> usize {
        self.free_list.len()
    }

    /// Get capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get object size
    pub fn object_size(&self) -> usize {
        self.object_size
    }

    /// Check if slab is full
    pub fn is_full(&self) -> bool {
        self.free_list.is_empty()
    }

    /// Get statistics
    pub fn stats(&self) -> MemoryStats {
        self.stats.lock().clone()
    }

    /// Reset the slab
    pub fn reset(&mut self) {
        self.free_list = (0..self.capacity).collect();
        self.memory.fill(0);
        *self.stats.lock() = MemoryStats::new();
        self.stats.lock().allocated_bytes = self.object_size * self.capacity;
    }
}

/// Handle to an allocated slab object
#[derive(Debug, Clone, Copy)]
pub struct SlabHandle {
    index: usize,
    offset: usize,
    size: usize,
}

impl SlabHandle {
    /// Get the index
    pub fn index(&self) -> usize {
        self.index
    }

    /// Get the offset
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Get the size
    pub fn size(&self) -> usize {
        self.size
    }
}

/// Multi-slab allocator for different object sizes
pub struct SlabAllocator {
    slabs: Vec<Mutex<Slab>>,
    object_sizes: Vec<usize>,
}

impl SlabAllocator {
    /// Create a new slab allocator
    pub fn new() -> Self {
        Self {
            slabs: Vec::new(),
            object_sizes: Vec::new(),
        }
    }

    /// Add a slab for a specific object size
    pub fn add_slab(&mut self, object_size: usize, capacity: usize) {
        self.slabs.push(Mutex::new(Slab::new(object_size, capacity)));
        self.object_sizes.push(object_size);
    }

    /// Allocate from the appropriate slab
    pub fn allocate(&self, size: usize) -> Result<(usize, SlabHandle)> {
        // Find the smallest slab that fits
        for (slab_idx, &obj_size) in self.object_sizes.iter().enumerate() {
            if size <= obj_size {
                let mut slab = self.slabs[slab_idx].lock();
                let handle = slab.allocate()?;
                return Ok((slab_idx, handle));
            }
        }

        Err(PerformanceError::AllocationFailed(format!(
            "No slab available for size {}",
            size
        )))
    }

    /// Deallocate from a specific slab
    pub fn deallocate(&self, slab_idx: usize, handle: SlabHandle) {
        if let Some(slab) = self.slabs.get(slab_idx) {
            slab.lock().deallocate(handle);
        }
    }

    /// Get mutable access to allocated memory
    pub fn get_mut(&self, slab_idx: usize, handle: &SlabHandle) -> Option<Vec<u8>> {
        if let Some(slab) = self.slabs.get(slab_idx) {
            slab.lock()
                .get(handle)
                .map(|slice| slice.to_vec())
        } else {
            None
        }
    }

    /// Get total statistics across all slabs
    pub fn total_stats(&self) -> MemoryStats {
        let mut total = MemoryStats::new();

        for slab in &self.slabs {
            let stats = slab.lock().stats();
            total.allocated_bytes += stats.allocated_bytes;
            total.allocation_count += stats.allocation_count;
            total.deallocation_count += stats.deallocation_count;
            total.in_use_bytes += stats.in_use_bytes;
            if stats.peak_bytes > total.peak_bytes {
                total.peak_bytes = stats.peak_bytes;
            }
        }

        total
    }

    /// Get number of slabs
    pub fn slab_count(&self) -> usize {
        self.slabs.len()
    }
}

impl Default for SlabAllocator {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for slab allocator with common sizes
pub struct SlabAllocatorBuilder {
    configs: Vec<(usize, usize)>,
}

impl SlabAllocatorBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            configs: Vec::new(),
        }
    }

    /// Add a slab configuration
    pub fn add_slab(mut self, object_size: usize, capacity: usize) -> Self {
        self.configs.push((object_size, capacity));
        self
    }

    /// Add common power-of-2 sizes
    pub fn with_power_of_2_sizes(mut self, max_size: usize, capacity: usize) -> Self {
        let mut size = 16;
        while size <= max_size {
            self.configs.push((size, capacity));
            size *= 2;
        }
        self
    }

    /// Build the allocator
    pub fn build(self) -> SlabAllocator {
        let mut allocator = SlabAllocator::new();

        // Sort by size
        let mut configs = self.configs;
        configs.sort_by_key(|(size, _)| *size);

        for (size, capacity) in configs {
            allocator.add_slab(size, capacity);
        }

        allocator
    }
}

impl Default for SlabAllocatorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slab_allocate() {
        let mut slab = Slab::new(64, 10);

        assert_eq!(slab.available(), 10);
        assert!(!slab.is_full());

        let handle = slab.allocate().unwrap();
        assert_eq!(slab.available(), 9);
        assert_eq!(handle.size(), 64);
    }

    #[test]
    fn test_slab_deallocate() {
        let mut slab = Slab::new(64, 10);

        let handle = slab.allocate().unwrap();
        assert_eq!(slab.available(), 9);

        slab.deallocate(handle);
        assert_eq!(slab.available(), 10);
    }

    #[test]
    fn test_slab_full() {
        let mut slab = Slab::new(32, 2);

        slab.allocate().unwrap();
        slab.allocate().unwrap();

        assert!(slab.is_full());
        assert!(slab.allocate().is_err());
    }

    #[test]
    fn test_slab_get_mut() {
        let mut slab = Slab::new(64, 10);
        let handle = slab.allocate().unwrap();

        let slice = slab.get_mut(&handle).unwrap();
        slice[0] = 42;
        slice[1] = 99;

        let slice_read = slab.get(&handle).unwrap();
        assert_eq!(slice_read[0], 42);
        assert_eq!(slice_read[1], 99);
    }

    #[test]
    fn test_slab_allocator() {
        let mut allocator = SlabAllocator::new();
        allocator.add_slab(64, 10);
        allocator.add_slab(128, 10);
        allocator.add_slab(256, 10);

        let (slab_idx, handle) = allocator.allocate(100).unwrap();
        assert_eq!(slab_idx, 1); // Should use 128-byte slab

        allocator.deallocate(slab_idx, handle);
    }

    #[test]
    fn test_slab_allocator_builder() {
        let allocator = SlabAllocatorBuilder::new()
            .add_slab(64, 10)
            .add_slab(128, 10)
            .with_power_of_2_sizes(1024, 5)
            .build();

        assert!(allocator.slab_count() > 2);
    }

    #[test]
    fn test_slab_reset() {
        let mut slab = Slab::new(64, 5);

        slab.allocate().unwrap();
        slab.allocate().unwrap();
        assert_eq!(slab.available(), 3);

        slab.reset();
        assert_eq!(slab.available(), 5);
    }
}
