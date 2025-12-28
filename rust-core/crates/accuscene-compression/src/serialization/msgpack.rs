//! MessagePack serialization

use crate::error::Result;
use serde::{Deserialize, Serialize};
use tracing::trace;

/// Serialize data to MessagePack format
pub fn serialize<T: Serialize>(data: &T) -> Result<Vec<u8>> {
    trace!("Serializing to MessagePack format");
    rmp_serde::to_vec(data).map_err(Into::into)
}

/// Serialize with named fields (more readable but larger)
pub fn serialize_named<T: Serialize>(data: &T) -> Result<Vec<u8>> {
    trace!("Serializing to MessagePack format with named fields");
    rmp_serde::to_vec_named(data).map_err(Into::into)
}

/// Deserialize data from MessagePack format
pub fn deserialize<'a, T: Deserialize<'a>>(data: &'a [u8]) -> Result<T> {
    trace!("Deserializing from MessagePack format ({} bytes)", data.len());
    rmp_serde::from_slice(data).map_err(Into::into)
}

/// Serialize with compression
pub fn serialize_compressed<T: Serialize>(
    data: &T,
    algorithm: crate::traits::Algorithm,
    level: crate::traits::CompressionLevel,
) -> Result<Vec<u8>> {
    let serialized = serialize(data)?;
    crate::algorithms::compress(&serialized, algorithm, level)
}

/// Decompress and deserialize
pub fn deserialize_compressed<'a, T: Deserialize<'a>>(
    data: &'a [u8],
    algorithm: crate::traits::Algorithm,
) -> Result<T> {
    let decompressed = crate::algorithms::decompress(data, algorithm)?;
    deserialize(&decompressed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct TestStruct {
        id: u32,
        name: String,
        values: Vec<f64>,
        metadata: HashMap<String, String>,
    }

    #[test]
    fn test_msgpack_round_trip() {
        let mut metadata = HashMap::new();
        metadata.insert("version".to_string(), "1.0".to_string());

        let test = TestStruct {
            id: 789,
            name: "MessagePack Test".to_string(),
            values: vec![3.14, 2.71, 1.41],
            metadata,
        };

        let serialized = serialize(&test).unwrap();
        let deserialized: TestStruct = deserialize(&serialized).unwrap();

        assert_eq!(test, deserialized);
    }

    #[test]
    fn test_msgpack_named() {
        let test = TestStruct {
            id: 123,
            name: "Named".to_string(),
            values: vec![1.0],
            metadata: HashMap::new(),
        };

        let serialized = serialize_named(&test).unwrap();
        let deserialized: TestStruct = deserialize(&serialized).unwrap();

        assert_eq!(test, deserialized);
    }

    #[test]
    fn test_msgpack_compressed() {
        let test = TestStruct {
            id: 456,
            name: "Compressed MessagePack".to_string(),
            values: vec![0.0; 1000],
            metadata: HashMap::new(),
        };

        let compressed = serialize_compressed(
            &test,
            crate::traits::Algorithm::Lz4,
            crate::traits::CompressionLevel::Fast,
        )
        .unwrap();

        let deserialized: TestStruct =
            deserialize_compressed(&compressed, crate::traits::Algorithm::Lz4).unwrap();

        assert_eq!(test, deserialized);
    }
}
