//! ZigZag encoding for signed integers.
//!
//! Maps signed integers to unsigned integers so that numbers with small
//! absolute values have small encoded values.
//!
//! # Mapping
//! - 0 -> 0
//! - -1 -> 1
//! - 1 -> 2
//! - -2 -> 3
//! - 2 -> 4
//! - etc.
//!
//! # Complexity
//! - Encode: O(1)
//! - Decode: O(1)

/// Encode i32 using ZigZag encoding.
///
/// # Complexity
/// O(1)
pub fn zigzag_encode_i32(value: i32) -> u32 {
    ((value << 1) ^ (value >> 31)) as u32
}

/// Decode i32 from ZigZag encoding.
///
/// # Complexity
/// O(1)
pub fn zigzag_decode_i32(value: u32) -> i32 {
    ((value >> 1) as i32) ^ -((value & 1) as i32)
}

/// Encode i64 using ZigZag encoding.
///
/// # Complexity
/// O(1)
pub fn zigzag_encode_i64(value: i64) -> u64 {
    ((value << 1) ^ (value >> 63)) as u64
}

/// Decode i64 from ZigZag encoding.
///
/// # Complexity
/// O(1)
pub fn zigzag_decode_i64(value: u64) -> i64 {
    ((value >> 1) as i64) ^ -((value & 1) as i64)
}

/// Generic ZigZag encode (works with any signed integer).
pub fn zigzag_encode<T: Into<i64>>(value: T) -> u64 {
    zigzag_encode_i64(value.into())
}

/// Generic ZigZag decode.
pub fn zigzag_decode(value: u64) -> i64 {
    zigzag_decode_i64(value)
}

/// Encode i32 sequence using ZigZag.
pub fn zigzag_encode_sequence_i32(values: &[i32]) -> Vec<u32> {
    values.iter().map(|&v| zigzag_encode_i32(v)).collect()
}

/// Decode i32 sequence from ZigZag.
pub fn zigzag_decode_sequence_i32(values: &[u32]) -> Vec<i32> {
    values.iter().map(|&v| zigzag_decode_i32(v)).collect()
}

/// Encode i64 sequence using ZigZag.
pub fn zigzag_encode_sequence_i64(values: &[i64]) -> Vec<u64> {
    values.iter().map(|&v| zigzag_encode_i64(v)).collect()
}

/// Decode i64 sequence from ZigZag.
pub fn zigzag_decode_sequence_i64(values: &[u64]) -> Vec<i64> {
    values.iter().map(|&v| zigzag_decode_i64(v)).collect()
}

/// Combine ZigZag and varint encoding for i32.
pub fn encode_zigzag_varint_i32(value: i32) -> Vec<u8> {
    use crate::encoding::varint::encode_varint;
    encode_varint(zigzag_encode_i32(value) as u64)
}

/// Decode ZigZag varint for i32.
pub fn decode_zigzag_varint_i32(bytes: &[u8]) -> crate::error::Result<(i32, usize)> {
    use crate::encoding::varint::decode_varint;
    let (value, consumed) = decode_varint(bytes)?;
    Ok((zigzag_decode_i32(value as u32), consumed))
}

/// Combine ZigZag and varint encoding for i64.
pub fn encode_zigzag_varint_i64(value: i64) -> Vec<u8> {
    use crate::encoding::varint::encode_varint;
    encode_varint(zigzag_encode_i64(value))
}

/// Decode ZigZag varint for i64.
pub fn decode_zigzag_varint_i64(bytes: &[u8]) -> crate::error::Result<(i64, usize)> {
    use crate::encoding::varint::decode_varint;
    let (value, consumed) = decode_varint(bytes)?;
    Ok((zigzag_decode_i64(value), consumed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zigzag_i32() {
        let test_values = vec![0, -1, 1, -2, 2, -100, 100, i32::MIN, i32::MAX];

        for &value in &test_values {
            let encoded = zigzag_encode_i32(value);
            let decoded = zigzag_decode_i32(encoded);
            assert_eq!(value, decoded);
        }
    }

    #[test]
    fn test_zigzag_i64() {
        let test_values = vec![0, -1, 1, -2, 2, -100, 100, i64::MIN, i64::MAX];

        for &value in &test_values {
            let encoded = zigzag_encode_i64(value);
            let decoded = zigzag_decode_i64(encoded);
            assert_eq!(value, decoded);
        }
    }

    #[test]
    fn test_zigzag_mapping() {
        assert_eq!(zigzag_encode_i32(0), 0);
        assert_eq!(zigzag_encode_i32(-1), 1);
        assert_eq!(zigzag_encode_i32(1), 2);
        assert_eq!(zigzag_encode_i32(-2), 3);
        assert_eq!(zigzag_encode_i32(2), 4);
    }

    #[test]
    fn test_zigzag_sequence() {
        let values = vec![-5, -4, -3, -2, -1, 0, 1, 2, 3, 4, 5];
        let encoded = zigzag_encode_sequence_i32(&values);
        let decoded = zigzag_decode_sequence_i32(&encoded);
        assert_eq!(values, decoded);
    }

    #[test]
    fn test_zigzag_varint() {
        let test_values = vec![-1000, -100, -10, -1, 0, 1, 10, 100, 1000];

        for &value in &test_values {
            let encoded = encode_zigzag_varint_i32(value);
            let (decoded, consumed) = decode_zigzag_varint_i32(&encoded).unwrap();
            assert_eq!(value, decoded);
            assert_eq!(encoded.len(), consumed);
        }
    }

    #[test]
    fn test_small_negative_values() {
        // Small negative values should encode compactly
        for i in -64..64 {
            let encoded = encode_zigzag_varint_i32(i);
            assert!(encoded.len() <= 2);
        }
    }
}
