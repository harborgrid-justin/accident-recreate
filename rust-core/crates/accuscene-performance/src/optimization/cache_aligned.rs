//! Cache-line alignment utilities

use std::ops::{Deref, DerefMut};

/// Cache line size (typically 64 bytes on modern CPUs)
pub const CACHE_LINE_SIZE: usize = 64;

/// Cache-aligned data structure
#[repr(align(64))]
#[derive(Debug, Clone, Copy)]
pub struct CacheAligned<T> {
    data: T,
}

impl<T> CacheAligned<T> {
    /// Create a new cache-aligned value
    pub const fn new(data: T) -> Self {
        Self { data }
    }

    /// Get a reference to the inner value
    pub fn get(&self) -> &T {
        &self.data
    }

    /// Get a mutable reference to the inner value
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.data
    }

    /// Unwrap the inner value
    pub fn into_inner(self) -> T {
        self.data
    }
}

impl<T> Deref for CacheAligned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for CacheAligned<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<T: Default> Default for CacheAligned<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

/// Cache-padded data structure to prevent false sharing
#[repr(align(64))]
#[derive(Debug)]
pub struct CachePadded<T> {
    data: T,
    _padding: [u8; 0], // Zero-sized padding, alignment is handled by repr
}

impl<T> CachePadded<T> {
    /// Create a new cache-padded value
    pub const fn new(data: T) -> Self {
        Self {
            data,
            _padding: [],
        }
    }

    /// Get a reference to the inner value
    pub fn get(&self) -> &T {
        &self.data
    }

    /// Get a mutable reference to the inner value
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.data
    }

    /// Unwrap the inner value
    pub fn into_inner(self) -> T {
        self.data
    }
}

impl<T> Deref for CachePadded<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for CachePadded<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<T: Default> Default for CachePadded<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: Clone> Clone for CachePadded<T> {
    fn clone(&self) -> Self {
        Self::new(self.data.clone())
    }
}

/// Array of cache-aligned values
pub struct AlignedArray<T, const N: usize> {
    data: [CacheAligned<T>; N],
}

impl<T, const N: usize> AlignedArray<T, N> {
    /// Create a new aligned array
    pub fn new(data: [T; N]) -> Self
    where
        T: Copy,
    {
        // Safe because CacheAligned is repr(transparent) with alignment
        let aligned_data = data.map(CacheAligned::new);
        Self { data: aligned_data }
    }

    /// Get a reference to an element
    pub fn get(&self, index: usize) -> Option<&T> {
        self.data.get(index).map(|a| a.get())
    }

    /// Get a mutable reference to an element
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.data.get_mut(index).map(|a| a.get_mut())
    }

    /// Iterate over elements
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.data.iter().map(|a| a.get())
    }

    /// Iterate over elements mutably
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.data.iter_mut().map(|a| a.get_mut())
    }
}

/// Check if a pointer is cache-aligned
pub fn is_cache_aligned<T>(ptr: *const T) -> bool {
    (ptr as usize) % CACHE_LINE_SIZE == 0
}

/// Allocate cache-aligned memory
pub fn alloc_aligned<T: Default>(count: usize) -> Vec<CacheAligned<T>> {
    (0..count)
        .map(|_| CacheAligned::new(T::default()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_aligned() {
        let aligned = CacheAligned::new(42u64);
        assert_eq!(*aligned, 42);

        let ptr = &aligned as *const _ as usize;
        assert_eq!(ptr % 64, 0, "Should be 64-byte aligned");
    }

    #[test]
    fn test_cache_padded() {
        let padded = CachePadded::new(100u64);
        assert_eq!(*padded, 100);

        let ptr = &padded as *const _ as usize;
        assert_eq!(ptr % 64, 0, "Should be 64-byte aligned");
    }

    #[test]
    fn test_aligned_array() {
        let array = AlignedArray::new([1, 2, 3, 4, 5]);

        assert_eq!(array.get(0), Some(&1));
        assert_eq!(array.get(4), Some(&5));
        assert_eq!(array.get(5), None);
    }

    #[test]
    fn test_cache_aligned_size() {
        use std::mem;

        // Size should be at least the size of T, but aligned to cache line
        assert!(mem::size_of::<CacheAligned<u64>>() >= mem::size_of::<u64>());
        assert_eq!(mem::align_of::<CacheAligned<u64>>(), 64);
    }

    #[test]
    fn test_is_cache_aligned() {
        let aligned = CacheAligned::new(0u64);
        let ptr = &aligned as *const _;
        assert!(is_cache_aligned(ptr));
    }

    #[test]
    fn test_alloc_aligned() {
        let vec = alloc_aligned::<u64>(10);
        assert_eq!(vec.len(), 10);

        for item in &vec {
            let ptr = item as *const _ as usize;
            assert_eq!(ptr % 64, 0);
        }
    }
}
