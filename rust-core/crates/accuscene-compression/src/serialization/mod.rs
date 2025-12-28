//! Serialization formats for compressed data

pub mod binary;
pub mod msgpack;
pub mod compact;

use crate::error::Result;
use serde::{Deserialize, Serialize};

/// Serialization format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SerializationFormat {
    /// Binary format using bincode
    Binary,
    /// MessagePack format
    MessagePack,
    /// Custom compact format
    Compact,
}

impl SerializationFormat {
    /// Get format from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "binary" | "bincode" => Some(SerializationFormat::Binary),
            "messagepack" | "msgpack" | "mp" => Some(SerializationFormat::MessagePack),
            "compact" => Some(SerializationFormat::Compact),
            _ => None,
        }
    }

    /// Get string identifier
    pub fn as_str(&self) -> &'static str {
        match self {
            SerializationFormat::Binary => "binary",
            SerializationFormat::MessagePack => "msgpack",
            SerializationFormat::Compact => "compact",
        }
    }
}

/// Serialize data with the specified format
pub fn serialize<T: Serialize>(data: &T, format: SerializationFormat) -> Result<Vec<u8>> {
    match format {
        SerializationFormat::Binary => binary::serialize(data),
        SerializationFormat::MessagePack => msgpack::serialize(data),
        SerializationFormat::Compact => compact::serialize(data),
    }
}

/// Deserialize data with the specified format
pub fn deserialize<'a, T: Deserialize<'a>>(data: &'a [u8], format: SerializationFormat) -> Result<T> {
    match format {
        SerializationFormat::Binary => binary::deserialize(data),
        SerializationFormat::MessagePack => msgpack::deserialize(data),
        SerializationFormat::Compact => compact::deserialize(data),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestData {
        id: u64,
        name: String,
        values: Vec<f64>,
    }

    #[test]
    fn test_all_formats() {
        let data = TestData {
            id: 42,
            name: "Test".to_string(),
            values: vec![1.0, 2.0, 3.0],
        };

        for format in [
            SerializationFormat::Binary,
            SerializationFormat::MessagePack,
            SerializationFormat::Compact,
        ] {
            let serialized = serialize(&data, format).unwrap();
            let deserialized: TestData = deserialize(&serialized, format).unwrap();
            assert_eq!(data, deserialized);
        }
    }
}
