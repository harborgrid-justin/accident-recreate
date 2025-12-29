//! Variable-length integer encoding (varint).
//!
//! Encodes integers using fewer bytes for smaller values.
//! Uses the MSB to indicate continuation.
//!
//! # Encoding
//! - 0-127: 1 byte
//! - 128-16383: 2 bytes
//! - 16384-2097151: 3 bytes
//! - etc.
//!
//! # Complexity
//! - Encode: O(1) - max 10 bytes for u64
//! - Decode: O(1) - max 10 iterations for u64

use crate::error::{AlgorithmError, Result};
use bytes::{BufMut, BytesMut};

/// Encode a u64 as varint.
///
/// # Complexity
/// O(1) - at most 10 bytes for u64
pub fn encode_varint(mut value: u64) -> Vec<u8> {
    let mut bytes = BytesMut::with_capacity(10);

    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;

        if value != 0 {
            byte |= 0x80; // Set continuation bit
        }

        bytes.put_u8(byte);

        if value == 0 {
            break;
        }
    }

    bytes.to_vec()
}

/// Decode a varint from bytes.
///
/// Returns (value, bytes_consumed).
///
/// # Complexity
/// O(1) - at most 10 iterations for u64
pub fn decode_varint(bytes: &[u8]) -> Result<(u64, usize)> {
    let mut value = 0u64;
    let mut shift = 0u32;
    let mut consumed = 0usize;

    for &byte in bytes {
        consumed += 1;

        if shift >= 64 {
            return Err(AlgorithmError::InvalidFormat(
                "Varint too long".to_string(),
            ));
        }

        value |= ((byte & 0x7F) as u64) << shift;
        shift += 7;

        if byte & 0x80 == 0 {
            return Ok((value, consumed));
        }
    }

    Err(AlgorithmError::InvalidFormat(
        "Incomplete varint".to_string(),
    ))
}

/// Encode i64 as varint (sign bit preserved).
pub fn encode_varint_i64(value: i64) -> Vec<u8> {
    encode_varint(value as u64)
}

/// Decode i64 from varint.
pub fn decode_varint_i64(bytes: &[u8]) -> Result<(i64, usize)> {
    let (value, consumed) = decode_varint(bytes)?;
    Ok((value as i64, consumed))
}

/// Encode u32 as varint.
pub fn encode_varint_u32(value: u32) -> Vec<u8> {
    encode_varint(value as u64)
}

/// Decode u32 from varint.
pub fn decode_varint_u32(bytes: &[u8]) -> Result<(u32, usize)> {
    let (value, consumed) = decode_varint(bytes)?;
    if value > u32::MAX as u64 {
        return Err(AlgorithmError::InvalidFormat(
            "Varint value too large for u32".to_string(),
        ));
    }
    Ok((value as u32, consumed))
}

/// Encode a sequence of varints.
pub fn encode_varint_sequence(values: &[u64]) -> Vec<u8> {
    let mut bytes = BytesMut::with_capacity(values.len() * 5);
    for &value in values {
        bytes.put_slice(&encode_varint(value));
    }
    bytes.to_vec()
}

/// Decode a sequence of varints.
pub fn decode_varint_sequence(bytes: &[u8], count: usize) -> Result<Vec<u64>> {
    let mut values = Vec::with_capacity(count);
    let mut offset = 0;

    for _ in 0..count {
        if offset >= bytes.len() {
            return Err(AlgorithmError::InvalidFormat(
                "Not enough data for varint sequence".to_string(),
            ));
        }

        let (value, consumed) = decode_varint(&bytes[offset..])?;
        values.push(value);
        offset += consumed;
    }

    Ok(values)
}

/// Calculate encoded size for a value.
pub fn varint_size(mut value: u64) -> usize {
    let mut size = 1;
    while value >= 0x80 {
        size += 1;
        value >>= 7;
    }
    size
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode() {
        let test_values = vec![0, 1, 127, 128, 255, 256, 65535, 65536, u64::MAX];

        for &value in &test_values {
            let encoded = encode_varint(value);
            let (decoded, consumed) = decode_varint(&encoded).unwrap();

            assert_eq!(value, decoded);
            assert_eq!(encoded.len(), consumed);
        }
    }

    #[test]
    fn test_small_values() {
        // Values 0-127 should encode to 1 byte
        for i in 0..128u64 {
            let encoded = encode_varint(i);
            assert_eq!(encoded.len(), 1);
        }
    }

    #[test]
    fn test_varint_size() {
        assert_eq!(varint_size(0), 1);
        assert_eq!(varint_size(127), 1);
        assert_eq!(varint_size(128), 2);
        assert_eq!(varint_size(16383), 2);
        assert_eq!(varint_size(16384), 3);
    }

    #[test]
    fn test_sequence() {
        let values = vec![1, 100, 10000, 1000000];
        let encoded = encode_varint_sequence(&values);
        let decoded = decode_varint_sequence(&encoded, values.len()).unwrap();

        assert_eq!(values, decoded);
    }

    #[test]
    fn test_invalid_varint() {
        let bytes = vec![0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80];
        assert!(decode_varint(&bytes).is_err());
    }
}
