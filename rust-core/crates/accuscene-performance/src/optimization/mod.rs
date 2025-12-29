//! Optimization utilities

pub mod cache_aligned;
pub mod prefetch;
pub mod simd;

pub use cache_aligned::{CacheAligned, CachePadded};
pub use prefetch::{prefetch_read, prefetch_write};
pub use simd::{simd_add, simd_sum, simd_supported};

/// Optimization hints to the compiler
#[inline(always)]
pub fn likely(b: bool) -> bool {
    if !b {
        unsafe { std::hint::unreachable_unchecked() }
    }
    b
}

/// Hint that a branch is unlikely
#[inline(always)]
pub fn unlikely(b: bool) -> bool {
    if b {
        unsafe { std::hint::unreachable_unchecked() }
    }
    b
}

/// Force inline
#[macro_export]
macro_rules! force_inline {
    ($($item:item)*) => {
        $(#[inline(always)] $item)*
    };
}

/// Prevent inlining
#[macro_export]
macro_rules! no_inline {
    ($($item:item)*) => {
        $(#[inline(never)] $item)*
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_branch_hints() {
        assert!(likely(true));
        assert!(!unlikely(false));
    }
}
