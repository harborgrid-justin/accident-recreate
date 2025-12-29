//! SIMD-optimized operations

/// Check if SIMD is supported
pub fn simd_supported() -> bool {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        is_x86_feature_detected!("sse2")
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        false
    }
}

/// SIMD vector addition
pub fn simd_add(a: &[f32], b: &[f32], result: &mut [f32]) {
    assert_eq!(a.len(), b.len());
    assert_eq!(a.len(), result.len());

    #[cfg(target_feature = "simd128")]
    {
        simd_add_impl(a, b, result);
    }

    #[cfg(not(target_feature = "simd128"))]
    {
        scalar_add(a, b, result);
    }
}

/// Scalar fallback for addition
fn scalar_add(a: &[f32], b: &[f32], result: &mut [f32]) {
    for i in 0..a.len() {
        result[i] = a[i] + b[i];
    }
}

/// SIMD implementation for addition
#[cfg(target_feature = "simd128")]
fn simd_add_impl(a: &[f32], b: &[f32], result: &mut [f32]) {
    // Placeholder for actual SIMD implementation
    // In production, use std::simd or platform-specific intrinsics
    scalar_add(a, b, result);
}

/// SIMD vector sum
pub fn simd_sum(values: &[f32]) -> f32 {
    #[cfg(target_feature = "simd128")]
    {
        simd_sum_impl(values)
    }

    #[cfg(not(target_feature = "simd128"))]
    {
        scalar_sum(values)
    }
}

/// Scalar sum
fn scalar_sum(values: &[f32]) -> f32 {
    values.iter().sum()
}

/// SIMD sum implementation
#[cfg(target_feature = "simd128")]
fn simd_sum_impl(values: &[f32]) -> f32 {
    // Placeholder for actual SIMD implementation
    scalar_sum(values)
}

/// SIMD vector multiplication
pub fn simd_mul(a: &[f32], b: &[f32], result: &mut [f32]) {
    assert_eq!(a.len(), b.len());
    assert_eq!(a.len(), result.len());

    for i in 0..a.len() {
        result[i] = a[i] * b[i];
    }
}

/// SIMD dot product
pub fn simd_dot(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());

    let mut sum = 0.0;
    for i in 0..a.len() {
        sum += a[i] * b[i];
    }
    sum
}

/// SIMD vector scale
pub fn simd_scale(values: &[f32], scalar: f32, result: &mut [f32]) {
    assert_eq!(values.len(), result.len());

    for i in 0..values.len() {
        result[i] = values[i] * scalar;
    }
}

/// SIMD find minimum
pub fn simd_min(values: &[f32]) -> f32 {
    values.iter().copied().fold(f32::INFINITY, f32::min)
}

/// SIMD find maximum
pub fn simd_max(values: &[f32]) -> f32 {
    values.iter().copied().fold(f32::NEG_INFINITY, f32::max)
}

/// SIMD operations for i32
pub mod i32_ops {
    /// Sum i32 values
    pub fn sum(values: &[i32]) -> i32 {
        values.iter().sum()
    }

    /// Add two i32 slices
    pub fn add(a: &[i32], b: &[i32], result: &mut [i32]) {
        assert_eq!(a.len(), b.len());
        assert_eq!(a.len(), result.len());

        for i in 0..a.len() {
            result[i] = a[i] + b[i];
        }
    }

    /// Find minimum
    pub fn min(values: &[i32]) -> i32 {
        *values.iter().min().unwrap_or(&0)
    }

    /// Find maximum
    pub fn max(values: &[i32]) -> i32 {
        *values.iter().max().unwrap_or(&0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simd_add() {
        let a = vec![1.0, 2.0, 3.0, 4.0];
        let b = vec![5.0, 6.0, 7.0, 8.0];
        let mut result = vec![0.0; 4];

        simd_add(&a, &b, &mut result);

        assert_eq!(result, vec![6.0, 8.0, 10.0, 12.0]);
    }

    #[test]
    fn test_simd_sum() {
        let values = vec![1.0, 2.0, 3.0, 4.0];
        let sum = simd_sum(&values);
        assert_eq!(sum, 10.0);
    }

    #[test]
    fn test_simd_mul() {
        let a = vec![2.0, 3.0, 4.0];
        let b = vec![5.0, 6.0, 7.0];
        let mut result = vec![0.0; 3];

        simd_mul(&a, &b, &mut result);

        assert_eq!(result, vec![10.0, 18.0, 28.0]);
    }

    #[test]
    fn test_simd_dot() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];

        let dot = simd_dot(&a, &b);
        assert_eq!(dot, 32.0); // 1*4 + 2*5 + 3*6 = 32
    }

    #[test]
    fn test_simd_scale() {
        let values = vec![1.0, 2.0, 3.0];
        let mut result = vec![0.0; 3];

        simd_scale(&values, 2.0, &mut result);

        assert_eq!(result, vec![2.0, 4.0, 6.0]);
    }

    #[test]
    fn test_simd_min_max() {
        let values = vec![3.0, 1.0, 4.0, 1.0, 5.0];

        assert_eq!(simd_min(&values), 1.0);
        assert_eq!(simd_max(&values), 5.0);
    }
}
