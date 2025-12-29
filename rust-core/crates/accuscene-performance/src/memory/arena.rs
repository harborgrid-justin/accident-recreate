//! Arena allocator for batch processing

use crate::memory::{align_up, MemoryStats};
use bumpalo::Bump;
use parking_lot::Mutex;
use std::sync::Arc;

/// Arena allocator for fast batch allocations
pub struct Arena {
    bump: Bump,
    stats: Arc<Mutex<MemoryStats>>,
}

impl Arena {
    /// Create a new arena
    pub fn new() -> Self {
        Self {
            bump: Bump::new(),
            stats: Arc::new(Mutex::new(MemoryStats::new())),
        }
    }

    /// Create with initial capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            bump: Bump::with_capacity(capacity),
            stats: Arc::new(Mutex::new(MemoryStats::new())),
        }
    }

    /// Allocate a value in the arena
    pub fn alloc<T>(&self, value: T) -> &mut T {
        let size = std::mem::size_of::<T>();
        self.stats.lock().record_allocation(size);
        self.bump.alloc(value)
    }

    /// Allocate a slice in the arena
    pub fn alloc_slice<T: Copy>(&self, slice: &[T]) -> &mut [T] {
        let size = std::mem::size_of::<T>() * slice.len();
        self.stats.lock().record_allocation(size);
        self.bump.alloc_slice_copy(slice)
    }

    /// Allocate uninitialized memory
    pub fn alloc_layout(&self, layout: std::alloc::Layout) -> &mut [u8] {
        self.stats.lock().record_allocation(layout.size());
        unsafe {
            let ptr = self.bump.alloc_layout(layout);
            std::slice::from_raw_parts_mut(ptr.as_ptr(), layout.size())
        }
    }

    /// Reset the arena, freeing all allocations
    pub fn reset(&mut self) {
        self.bump.reset();
        *self.stats.lock() = MemoryStats::new();
    }

    /// Get memory statistics
    pub fn stats(&self) -> MemoryStats {
        self.stats.lock().clone()
    }

    /// Get allocated bytes
    pub fn allocated_bytes(&self) -> usize {
        self.bump.allocated_bytes()
    }

    /// Allocate a string in the arena
    pub fn alloc_str(&self, s: &str) -> &mut str {
        let size = s.len();
        self.stats.lock().record_allocation(size);
        self.bump.alloc_str(s)
    }
}

impl Default for Arena {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe arena allocator
pub struct ArenaAllocator {
    arenas: Arc<Mutex<Vec<Arena>>>,
    chunk_size: usize,
    current_arena: Arc<Mutex<usize>>,
}

impl ArenaAllocator {
    /// Create a new arena allocator
    pub fn new(chunk_size: usize) -> Self {
        Self {
            arenas: Arc::new(Mutex::new(vec![Arena::with_capacity(chunk_size)])),
            chunk_size,
            current_arena: Arc::new(Mutex::new(0)),
        }
    }

    /// Allocate memory from the arena
    pub fn allocate(&self, size: usize, alignment: usize) -> *mut u8 {
        let aligned_size = align_up(size, alignment);

        let mut arenas = self.arenas.lock();
        let current_idx = *self.current_arena.lock();

        // Try to allocate from current arena
        if let Some(arena) = arenas.get(current_idx) {
            let layout = std::alloc::Layout::from_size_align(aligned_size, alignment)
                .expect("Invalid layout");
            let slice = arena.alloc_layout(layout);
            return slice.as_mut_ptr();
        }

        // Need a new arena
        let new_arena = Arena::with_capacity(self.chunk_size.max(aligned_size));
        let layout = std::alloc::Layout::from_size_align(aligned_size, alignment)
            .expect("Invalid layout");
        let slice = new_arena.alloc_layout(layout);
        let ptr = slice.as_mut_ptr();

        arenas.push(new_arena);
        *self.current_arena.lock() = arenas.len() - 1;

        ptr
    }

    /// Reset all arenas
    pub fn reset(&self) {
        let mut arenas = self.arenas.lock();
        for arena in arenas.iter_mut() {
            arena.reset();
        }
        *self.current_arena.lock() = 0;
    }

    /// Get total allocated bytes across all arenas
    pub fn total_allocated(&self) -> usize {
        self.arenas
            .lock()
            .iter()
            .map(|a| a.allocated_bytes())
            .sum()
    }

    /// Get number of arenas
    pub fn arena_count(&self) -> usize {
        self.arenas.lock().len()
    }

    /// Get combined statistics
    pub fn stats(&self) -> MemoryStats {
        let arenas = self.arenas.lock();
        let mut combined = MemoryStats::new();

        for arena in arenas.iter() {
            let stats = arena.stats();
            combined.allocated_bytes += stats.allocated_bytes;
            combined.allocation_count += stats.allocation_count;
            combined.deallocation_count += stats.deallocation_count;
            combined.in_use_bytes += stats.in_use_bytes;
            if stats.peak_bytes > combined.peak_bytes {
                combined.peak_bytes = stats.peak_bytes;
            }
        }

        combined
    }
}

/// Arena-backed vector
pub struct ArenaVec<'a, T> {
    arena: &'a Arena,
    items: Vec<&'a mut T>,
}

impl<'a, T> ArenaVec<'a, T> {
    /// Create a new arena vector
    pub fn new(arena: &'a Arena) -> Self {
        Self {
            arena,
            items: Vec::new(),
        }
    }

    /// Push a value into the arena vector
    pub fn push(&mut self, value: T) {
        let item = self.arena.alloc(value);
        self.items.push(item);
    }

    /// Get the length
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Get an item by index
    pub fn get(&self, index: usize) -> Option<&T> {
        self.items.get(index).map(|r| &**r)
    }

    /// Iterate over items
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.items.iter().map(|r| &**r)
    }
}

/// Typed arena for a specific type
pub struct TypedArena<T> {
    arena: Arena,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> TypedArena<T> {
    /// Create a new typed arena
    pub fn new() -> Self {
        Self {
            arena: Arena::new(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Create with capacity (number of items)
    pub fn with_capacity(capacity: usize) -> Self {
        let bytes = capacity * std::mem::size_of::<T>();
        Self {
            arena: Arena::with_capacity(bytes),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Allocate a value
    pub fn alloc(&self, value: T) -> &mut T {
        self.arena.alloc(value)
    }

    /// Allocate multiple values
    pub fn alloc_slice(&self, values: &[T]) -> &mut [T]
    where
        T: Copy,
    {
        self.arena.alloc_slice(values)
    }

    /// Reset the arena
    pub fn reset(&mut self) {
        self.arena.reset();
    }

    /// Get statistics
    pub fn stats(&self) -> MemoryStats {
        self.arena.stats()
    }
}

impl<T> Default for TypedArena<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_alloc() {
        let arena = Arena::new();

        let x = arena.alloc(42);
        assert_eq!(*x, 42);

        let s = arena.alloc_str("hello");
        assert_eq!(s, "hello");
    }

    #[test]
    fn test_arena_slice() {
        let arena = Arena::new();

        let slice = arena.alloc_slice(&[1, 2, 3, 4, 5]);
        assert_eq!(slice, &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_arena_reset() {
        let mut arena = Arena::new();

        let _x = arena.alloc(100);
        assert!(arena.allocated_bytes() > 0);

        arena.reset();
        assert_eq!(arena.allocated_bytes(), 0);
    }

    #[test]
    fn test_arena_allocator() {
        let allocator = ArenaAllocator::new(4096);

        let _ptr1 = allocator.allocate(100, 8);
        let _ptr2 = allocator.allocate(200, 8);

        assert!(allocator.total_allocated() >= 300);
    }

    #[test]
    fn test_arena_vec() {
        let arena = Arena::new();
        let mut vec = ArenaVec::new(&arena);

        vec.push(1);
        vec.push(2);
        vec.push(3);

        assert_eq!(vec.len(), 3);
        assert_eq!(vec.get(0), Some(&1));
        assert_eq!(vec.get(1), Some(&2));
    }

    #[test]
    fn test_typed_arena() {
        let arena = TypedArena::<i32>::new();

        let x = arena.alloc(10);
        let y = arena.alloc(20);

        assert_eq!(*x, 10);
        assert_eq!(*y, 20);
    }

    #[test]
    fn test_arena_stats() {
        let arena = Arena::new();

        arena.alloc(42u64);
        arena.alloc(100u32);

        let stats = arena.stats();
        assert!(stats.allocated_bytes >= 12); // u64 + u32
        assert_eq!(stats.allocation_count, 2);
    }
}
