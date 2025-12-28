//! Binary serialization using bincode

use crate::error::Result;
use serde::{Deserialize, Serialize};
use tracing::trace;

/// Serialize data to binary format using bincode
pub fn serialize<T: Serialize>(data: &T) -> Result<Vec<u8>> {
    trace!("Serializing to binary format");
    bincode::serialize(data).map_err(Into::into)
}

/// Deserialize data from binary format
pub fn deserialize<'a, T: Deserialize<'a>>(data: &'a [u8]) -> Result<T> {
    trace!("Deserializing from binary format ({} bytes)", data.len());
    bincode::deserialize(data).map_err(Into::into)
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

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct TestStruct {
        id: u32,
        name: String,
        data: Vec<u8>,
    }

    #[test]
    fn test_binary_round_trip() {
        let test = TestStruct {
            id: 123,
            name: "AccuScene".to_string(),
            data: vec![1, 2, 3, 4, 5],
        };

        let serialized = serialize(&test).unwrap();
        let deserialized: TestStruct = deserialize(&serialized).unwrap();

        assert_eq!(test, deserialized);
    }

    #[test]
    fn test_compressed_round_trip() {
        let test = TestStruct {
            id: 456,
            name: "Compressed".to_string(),
            data: vec![0u8; 1000],
        };

        let compressed = serialize_compressed(
            &test,
            crate::traits::Algorithm::Zstd,
            crate::traits::CompressionLevel::Default,
        )
        .unwrap();

        let deserialized: TestStruct =
            deserialize_compressed(&compressed, crate::traits::Algorithm::Zstd).unwrap();

        assert_eq!(test, deserialized);
    }
}
