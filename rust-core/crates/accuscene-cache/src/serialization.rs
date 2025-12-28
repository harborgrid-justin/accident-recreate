//! Cache value serialization

use crate::error::{CacheError, CacheResult};
use serde::{Deserialize, Serialize};

/// Trait for serializing cache values
pub trait CacheSerializer: Send + Sync {
    /// Serialize a value to bytes
    fn serialize<T: Serialize>(&self, value: &T) -> CacheResult<Vec<u8>>;

    /// Deserialize bytes to a value
    fn deserialize<T: for<'de> Deserialize<'de>>(&self, bytes: &[u8]) -> CacheResult<T>;

    /// Serializer name
    fn name(&self) -> &str;
}

/// Bincode serializer (compact binary format)
#[derive(Debug, Clone, Copy)]
pub struct BincodeSerializer;

impl CacheSerializer for BincodeSerializer {
    fn serialize<T: Serialize>(&self, value: &T) -> CacheResult<Vec<u8>> {
        bincode::serialize(value)
            .map_err(|e| CacheError::SerializationError(format!("Bincode error: {}", e)))
    }

    fn deserialize<T: for<'de> Deserialize<'de>>(&self, bytes: &[u8]) -> CacheResult<T> {
        bincode::deserialize(bytes)
            .map_err(|e| CacheError::DeserializationError(format!("Bincode error: {}", e)))
    }

    fn name(&self) -> &str {
        "bincode"
    }
}

/// JSON serializer (human-readable)
#[derive(Debug, Clone, Copy)]
pub struct JsonSerializer;

impl CacheSerializer for JsonSerializer {
    fn serialize<T: Serialize>(&self, value: &T) -> CacheResult<Vec<u8>> {
        serde_json::to_vec(value)
            .map_err(|e| CacheError::SerializationError(format!("JSON error: {}", e)))
    }

    fn deserialize<T: for<'de> Deserialize<'de>>(&self, bytes: &[u8]) -> CacheResult<T> {
        serde_json::from_slice(bytes)
            .map_err(|e| CacheError::DeserializationError(format!("JSON error: {}", e)))
    }

    fn name(&self) -> &str {
        "json"
    }
}

/// Compressed serializer wrapper
#[derive(Debug)]
pub struct CompressedSerializer<S: CacheSerializer> {
    inner: S,
    compression_level: u32,
}

impl<S: CacheSerializer> CompressedSerializer<S> {
    pub fn new(inner: S) -> Self {
        Self {
            inner,
            compression_level: 6, // Default compression level
        }
    }

    pub fn with_level(inner: S, level: u32) -> Self {
        Self {
            inner,
            compression_level: level,
        }
    }
}

impl<S: CacheSerializer> CacheSerializer for CompressedSerializer<S> {
    fn serialize<T: Serialize>(&self, value: &T) -> CacheResult<Vec<u8>> {
        let bytes = self.inner.serialize(value)?;

        // Simple compression using flate2 would go here
        // For now, just return the bytes (compression disabled)
        Ok(bytes)
    }

    fn deserialize<T: for<'de> Deserialize<'de>>(&self, bytes: &[u8]) -> CacheResult<T> {
        // Simple decompression would go here
        // For now, just deserialize directly
        self.inner.deserialize(bytes)
    }

    fn name(&self) -> &str {
        "compressed"
    }
}

/// Get default serializer
pub fn default_serializer() -> BincodeSerializer {
    BincodeSerializer
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestData {
        id: u64,
        name: String,
        values: Vec<f64>,
    }

    #[test]
    fn test_bincode_serializer() {
        let serializer = BincodeSerializer;
        let data = TestData {
            id: 42,
            name: "test".to_string(),
            values: vec![1.0, 2.0, 3.0],
        };

        let bytes = serializer.serialize(&data).unwrap();
        let deserialized: TestData = serializer.deserialize(&bytes).unwrap();

        assert_eq!(data, deserialized);
    }

    #[test]
    fn test_json_serializer() {
        let serializer = JsonSerializer;
        let data = TestData {
            id: 42,
            name: "test".to_string(),
            values: vec![1.0, 2.0, 3.0],
        };

        let bytes = serializer.serialize(&data).unwrap();
        let deserialized: TestData = serializer.deserialize(&bytes).unwrap();

        assert_eq!(data, deserialized);
    }
}
