//! Memory prefetching hints

/// Prefetch data for reading
#[inline(always)]
pub fn prefetch_read<T>(ptr: *const T) {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        #[cfg(target_feature = "sse")]
        unsafe {
            std::arch::x86_64::_mm_prefetch(ptr as *const i8, std::arch::x86_64::_MM_HINT_T0);
        }
    }

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        // No-op on other architectures
        let _ = ptr;
    }
}

/// Prefetch data for writing
#[inline(always)]
pub fn prefetch_write<T>(ptr: *const T) {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        #[cfg(target_feature = "sse")]
        unsafe {
            std::arch::x86_64::_mm_prefetch(
                ptr as *const i8,
                std::arch::x86_64::_MM_HINT_T0,
            );
        }
    }

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        // No-op on other architectures
        let _ = ptr;
    }
}

/// Prefetch with temporal locality hint
#[inline(always)]
pub fn prefetch_temporal<T>(ptr: *const T) {
    prefetch_read(ptr);
}

/// Prefetch with non-temporal locality hint
#[inline(always)]
pub fn prefetch_non_temporal<T>(ptr: *const T) {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        #[cfg(target_feature = "sse")]
        unsafe {
            std::arch::x86_64::_mm_prefetch(ptr as *const i8, std::arch::x86_64::_MM_HINT_NTA);
        }
    }

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        let _ = ptr;
    }
}

/// Prefetch multiple cache lines
#[inline(always)]
pub fn prefetch_range<T>(ptr: *const T, count: usize) {
    const CACHE_LINE_SIZE: usize = 64;
    let stride = (CACHE_LINE_SIZE / std::mem::size_of::<T>()).max(1);

    for i in (0..count).step_by(stride) {
        unsafe {
            let offset_ptr = ptr.add(i);
            prefetch_read(offset_ptr);
        }
    }
}

/// Prefetch for sequential access
#[inline(always)]
pub fn prefetch_sequential<T>(ptr: *const T, ahead: usize) {
    unsafe {
        let next_ptr = ptr.add(ahead);
        prefetch_read(next_ptr);
    }
}

/// Prefetch slice
#[inline(always)]
pub fn prefetch_slice<T>(slice: &[T]) {
    if !slice.is_empty() {
        prefetch_read(slice.as_ptr());
    }
}

/// Software prefetch for loop optimization
#[macro_export]
macro_rules! prefetch_loop {
    ($data:expr, $index:expr, $ahead:expr) => {
        if $index + $ahead < $data.len() {
            $crate::optimization::prefetch::prefetch_read(&$data[$index + $ahead] as *const _);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefetch_read() {
        let data = vec![1u64, 2, 3, 4, 5];
        prefetch_read(data.as_ptr());
        // Just ensure it doesn't crash
        assert!(true);
    }

    #[test]
    fn test_prefetch_write() {
        let data = vec![1u64, 2, 3, 4, 5];
        prefetch_write(data.as_ptr());
        assert!(true);
    }

    #[test]
    fn test_prefetch_range() {
        let data = vec![0u64; 100];
        prefetch_range(data.as_ptr(), data.len());
        assert!(true);
    }

    #[test]
    fn test_prefetch_sequential() {
        let data = vec![1u64, 2, 3, 4, 5];
        for i in 0..data.len() {
            if i + 2 < data.len() {
                prefetch_sequential(&data[i], 2);
            }
        }
        assert!(true);
    }

    #[test]
    fn test_prefetch_slice() {
        let data = vec![1, 2, 3, 4, 5];
        prefetch_slice(&data);
        assert!(true);
    }
}
