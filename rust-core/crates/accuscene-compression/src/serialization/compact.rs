//! Custom compact format optimized for physics data

use crate::error::{CompressionError, Result};
use serde::{Deserialize, Serialize};
use tracing::trace;

/// Serialize data to compact format
///
/// This is a wrapper around bincode for now, but can be customized
/// for specific physics data structures in the future
pub fn serialize<T: Serialize>(data: &T) -> Result<Vec<u8>> {
    trace!("Serializing to compact format");

    bincode::serialize(data)
        .map_err(|e| CompressionError::Serialization(e.to_string()))
}

/// Deserialize data from compact format
pub fn deserialize<'a, T: Deserialize<'a>>(data: &'a [u8]) -> Result<T> {
    trace!("Deserializing from compact format ({} bytes)", data.len());

    bincode::deserialize(data)
        .map_err(|e| CompressionError::Deserialization(e.to_string()))
}

/// Serialize vector of f32 values (optimized for physics data)
pub fn serialize_f32_vec(values: &[f32]) -> Result<Vec<u8>> {
    trace!("Serializing {} f32 values in compact format", values.len());

    let mut bytes = Vec::with_capacity(4 + values.len() * 4);

    // Write length
    bytes.extend_from_slice(&(values.len() as u32).to_le_bytes());

    // Write values
    for &value in values {
        bytes.extend_from_slice(&value.to_le_bytes());
    }

    Ok(bytes)
}

/// Deserialize vector of f32 values
pub fn deserialize_f32_vec(data: &[u8]) -> Result<Vec<f32>> {
    if data.len() < 4 {
        return Err(CompressionError::Deserialization(
            "Data too small for f32 vector".to_string(),
        ));
    }

    let length = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;

    if data.len() < 4 + length * 4 {
        return Err(CompressionError::Deserialization(
            "Insufficient data for f32 vector".to_string(),
        ));
    }

    let mut values = Vec::with_capacity(length);
    let mut offset = 4;

    for _ in 0..length {
        let bytes = [
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ];
        values.push(f32::from_le_bytes(bytes));
        offset += 4;
    }

    Ok(values)
}

/// Serialize vector of f64 values
pub fn serialize_f64_vec(values: &[f64]) -> Result<Vec<u8>> {
    trace!("Serializing {} f64 values in compact format", values.len());

    let mut bytes = Vec::with_capacity(4 + values.len() * 8);

    // Write length
    bytes.extend_from_slice(&(values.len() as u32).to_le_bytes());

    // Write values
    for &value in values {
        bytes.extend_from_slice(&value.to_le_bytes());
    }

    Ok(bytes)
}

/// Deserialize vector of f64 values
pub fn deserialize_f64_vec(data: &[u8]) -> Result<Vec<f64>> {
    if data.len() < 4 {
        return Err(CompressionError::Deserialization(
            "Data too small for f64 vector".to_string(),
        ));
    }

    let length = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;

    if data.len() < 4 + length * 8 {
        return Err(CompressionError::Deserialization(
            "Insufficient data for f64 vector".to_string(),
        ));
    }

    let mut values = Vec::with_capacity(length);
    let mut offset = 4;

    for _ in 0..length {
        let bytes = [
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
        ];
        values.push(f64::from_le_bytes(bytes));
        offset += 8;
    }

    Ok(values)
}

/// Compact vector3 (x, y, z) serialization
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn to_bytes(&self) -> [u8; 12] {
        let mut bytes = [0u8; 12];
        bytes[0..4].copy_from_slice(&self.x.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.y.to_le_bytes());
        bytes[8..12].copy_from_slice(&self.z.to_le_bytes());
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 12 {
            return Err(CompressionError::Deserialization(
                "Insufficient data for Vector3".to_string(),
            ));
        }

        Ok(Self {
            x: f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            y: f32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
            z: f32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]),
        })
    }
}

/// Serialize array of Vector3
pub fn serialize_vector3_array(vectors: &[Vector3]) -> Result<Vec<u8>> {
    trace!("Serializing {} Vector3 values", vectors.len());

    let mut bytes = Vec::with_capacity(4 + vectors.len() * 12);
    bytes.extend_from_slice(&(vectors.len() as u32).to_le_bytes());

    for vector in vectors {
        bytes.extend_from_slice(&vector.to_bytes());
    }

    Ok(bytes)
}

/// Deserialize array of Vector3
pub fn deserialize_vector3_array(data: &[u8]) -> Result<Vec<Vector3>> {
    if data.len() < 4 {
        return Err(CompressionError::Deserialization(
            "Data too small for Vector3 array".to_string(),
        ));
    }

    let length = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;

    if data.len() < 4 + length * 12 {
        return Err(CompressionError::Deserialization(
            "Insufficient data for Vector3 array".to_string(),
        ));
    }

    let mut vectors = Vec::with_capacity(length);
    let mut offset = 4;

    for _ in 0..length {
        vectors.push(Vector3::from_bytes(&data[offset..offset + 12])?);
        offset += 12;
    }

    Ok(vectors)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f32_vec_round_trip() {
        let values = vec![1.0f32, 2.5, 3.14, -4.2, 0.0];

        let serialized = serialize_f32_vec(&values).unwrap();
        let deserialized = deserialize_f32_vec(&serialized).unwrap();

        assert_eq!(values, deserialized);
    }

    #[test]
    fn test_f64_vec_round_trip() {
        let values = vec![1.0f64, 2.5, 3.14159265359, -4.2, 0.0];

        let serialized = serialize_f64_vec(&values).unwrap();
        let deserialized = deserialize_f64_vec(&serialized).unwrap();

        assert_eq!(values, deserialized);
    }

    #[test]
    fn test_vector3_round_trip() {
        let v = Vector3::new(1.0, 2.0, 3.0);
        let bytes = v.to_bytes();
        let restored = Vector3::from_bytes(&bytes).unwrap();

        assert_eq!(v, restored);
    }

    #[test]
    fn test_vector3_array_round_trip() {
        let vectors = vec![
            Vector3::new(1.0, 2.0, 3.0),
            Vector3::new(-1.0, -2.0, -3.0),
            Vector3::new(0.0, 0.0, 0.0),
        ];

        let serialized = serialize_vector3_array(&vectors).unwrap();
        let deserialized = deserialize_vector3_array(&serialized).unwrap();

        assert_eq!(vectors, deserialized);
    }
}
