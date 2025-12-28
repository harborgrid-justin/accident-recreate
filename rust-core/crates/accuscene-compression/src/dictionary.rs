//! Dictionary-based compression for improved ratios on similar data

use crate::algorithms::zstd::ZstdCompressor;
use crate::error::{CompressionError, Result};
use crate::traits::CompressionLevel;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, trace};

/// Dictionary for compression
#[derive(Debug, Clone)]
pub struct CompressionDictionary {
    /// Dictionary data
    data: Vec<u8>,
    /// Dictionary ID for validation
    id: u32,
    /// Metadata about the dictionary
    metadata: DictionaryMetadata,
}

/// Metadata about a compression dictionary
#[derive(Debug, Clone)]
pub struct DictionaryMetadata {
    /// Number of samples used to train
    pub sample_count: usize,
    /// Total size of training data
    pub training_size: usize,
    /// Dictionary size
    pub dict_size: usize,
    /// Creation timestamp
    pub created_at: u64,
    /// Description
    pub description: String,
}

impl CompressionDictionary {
    /// Create a new dictionary
    pub fn new(data: Vec<u8>, metadata: DictionaryMetadata) -> Self {
        // Generate ID from dictionary data
        let id = xxhash_rust::xxh3::xxh3_64(&data) as u32;

        Self { data, id, metadata }
    }

    /// Train a dictionary from sample data
    pub fn train(samples: &[&[u8]], dict_size: usize, description: String) -> Result<Self> {
        debug!(
            "Training dictionary from {} samples, target size: {} bytes",
            samples.len(),
            dict_size
        );

        if samples.is_empty() {
            return Err(CompressionError::Dictionary(
                "No samples provided for training".to_string(),
            ));
        }

        // Use Zstandard's dictionary training
        let data = ZstdCompressor::train_dictionary(samples, dict_size)?;

        let metadata = DictionaryMetadata {
            sample_count: samples.len(),
            training_size: samples.iter().map(|s| s.len()).sum(),
            dict_size: data.len(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            description,
        };

        debug!(
            "Dictionary trained: {} bytes from {} samples",
            data.len(),
            samples.len()
        );

        Ok(Self::new(data, metadata))
    }

    /// Get dictionary data
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get dictionary ID
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Get dictionary metadata
    pub fn metadata(&self) -> &DictionaryMetadata {
        &self.metadata
    }

    /// Compress data using this dictionary
    pub fn compress(&self, data: &[u8], level: CompressionLevel) -> Result<Vec<u8>> {
        trace!("Compressing {} bytes with dictionary {}", data.len(), self.id);

        let compressor = ZstdCompressor::new();
        let level_int = level.to_level(crate::traits::Algorithm::Zstd);

        let mut compressed = compressor.compress_with_dict(data, &self.data, level_int)?;

        // Prepend dictionary ID for validation
        let mut result = Vec::with_capacity(compressed.len() + 4);
        result.extend_from_slice(&self.id.to_le_bytes());
        result.append(&mut compressed);

        Ok(result)
    }

    /// Decompress data using this dictionary
    pub fn decompress(&self, data: &[u8]) -> Result<Vec<u8>> {
        trace!(
            "Decompressing {} bytes with dictionary {}",
            data.len(),
            self.id
        );

        if data.len() < 4 {
            return Err(CompressionError::CorruptedData(
                "Data too small for dictionary ID".to_string(),
            ));
        }

        // Verify dictionary ID
        let stored_id = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        if stored_id != self.id {
            return Err(CompressionError::Dictionary(format!(
                "Dictionary ID mismatch: expected {}, got {}",
                self.id, stored_id
            )));
        }

        let compressor = ZstdCompressor::new();
        compressor.decompress_with_dict(&data[4..], &self.data)
    }

    /// Serialize dictionary to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Write metadata
        bytes.extend_from_slice(&self.id.to_le_bytes());
        bytes.extend_from_slice(&(self.metadata.sample_count as u32).to_le_bytes());
        bytes.extend_from_slice(&(self.metadata.training_size as u64).to_le_bytes());
        bytes.extend_from_slice(&(self.metadata.dict_size as u32).to_le_bytes());
        bytes.extend_from_slice(&self.metadata.created_at.to_le_bytes());

        // Write description
        let desc_bytes = self.metadata.description.as_bytes();
        bytes.extend_from_slice(&(desc_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(desc_bytes);

        // Write dictionary data
        bytes.extend_from_slice(&(self.data.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&self.data);

        bytes
    }

    /// Deserialize dictionary from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 32 {
            return Err(CompressionError::Dictionary(
                "Invalid dictionary data".to_string(),
            ));
        }

        let mut offset = 0;

        // Read metadata
        let id = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
        offset += 4;

        let sample_count = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;

        let training_size = u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap()) as usize;
        offset += 8;

        let dict_size = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;

        let created_at = u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap());
        offset += 8;

        // Read description
        let desc_len = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;

        let description = String::from_utf8(bytes[offset..offset + desc_len].to_vec())
            .map_err(|e| CompressionError::Dictionary(e.to_string()))?;
        offset += desc_len;

        // Read dictionary data
        let data_len = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;

        let data = bytes[offset..offset + data_len].to_vec();

        let metadata = DictionaryMetadata {
            sample_count,
            training_size,
            dict_size,
            created_at,
            description,
        };

        Ok(Self { data, id, metadata })
    }
}

/// Dictionary manager for handling multiple dictionaries
#[derive(Debug, Clone)]
pub struct DictionaryManager {
    dictionaries: Arc<RwLock<HashMap<String, CompressionDictionary>>>,
}

impl DictionaryManager {
    /// Create a new dictionary manager
    pub fn new() -> Self {
        Self {
            dictionaries: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a dictionary
    pub fn add_dictionary(&self, name: String, dict: CompressionDictionary) {
        debug!("Adding dictionary '{}' (ID: {})", name, dict.id());
        self.dictionaries.write().insert(name, dict);
    }

    /// Get a dictionary by name
    pub fn get_dictionary(&self, name: &str) -> Option<CompressionDictionary> {
        self.dictionaries.read().get(name).cloned()
    }

    /// Remove a dictionary
    pub fn remove_dictionary(&self, name: &str) -> bool {
        self.dictionaries.write().remove(name).is_some()
    }

    /// List all dictionary names
    pub fn list_dictionaries(&self) -> Vec<String> {
        self.dictionaries.read().keys().cloned().collect()
    }

    /// Compress with named dictionary
    pub fn compress(
        &self,
        name: &str,
        data: &[u8],
        level: CompressionLevel,
    ) -> Result<Vec<u8>> {
        let dict = self.get_dictionary(name).ok_or_else(|| {
            CompressionError::Dictionary(format!("Dictionary '{}' not found", name))
        })?;

        dict.compress(data, level)
    }

    /// Decompress with named dictionary
    pub fn decompress(&self, name: &str, data: &[u8]) -> Result<Vec<u8>> {
        let dict = self.get_dictionary(name).ok_or_else(|| {
            CompressionError::Dictionary(format!("Dictionary '{}' not found", name))
        })?;

        dict.decompress(data)
    }
}

impl Default for DictionaryManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dictionary_training() {
        let samples = vec![
            b"AccuScene case data version 1.0".as_slice(),
            b"AccuScene case data version 1.1".as_slice(),
            b"AccuScene case data version 1.2".as_slice(),
            b"AccuScene case metadata record".as_slice(),
            b"AccuScene case simulation results".as_slice(),
        ];

        let dict = CompressionDictionary::train(
            &samples,
            1024,
            "Test dictionary for AccuScene cases".to_string(),
        )
        .unwrap();

        assert!(dict.data().len() > 0);
        assert_eq!(dict.metadata().sample_count, 5);
    }

    #[test]
    fn test_dictionary_compression() {
        let samples = vec![
            b"Test data for compression".as_slice(),
            b"Test data for decompression".as_slice(),
            b"Test data for validation".as_slice(),
        ];

        let dict = CompressionDictionary::train(&samples, 512, "Test dict".to_string()).unwrap();

        let data = b"Test data for round trip";
        let compressed = dict.compress(data, CompressionLevel::Default).unwrap();
        let decompressed = dict.decompress(&compressed).unwrap();

        assert_eq!(data.to_vec(), decompressed);
    }

    #[test]
    fn test_dictionary_serialization() {
        let samples = vec![b"Sample 1".as_slice(), b"Sample 2".as_slice()];

        let dict = CompressionDictionary::train(&samples, 256, "Serialization test".to_string())
            .unwrap();

        let bytes = dict.to_bytes();
        let restored = CompressionDictionary::from_bytes(&bytes).unwrap();

        assert_eq!(dict.id(), restored.id());
        assert_eq!(dict.data(), restored.data());
    }

    #[test]
    fn test_dictionary_manager() {
        let manager = DictionaryManager::new();

        let samples = vec![b"Data 1".as_slice(), b"Data 2".as_slice()];
        let dict = CompressionDictionary::train(&samples, 256, "Manager test".to_string()).unwrap();

        manager.add_dictionary("test_dict".to_string(), dict);

        let data = b"Test compression with manager";
        let compressed = manager
            .compress("test_dict", data, CompressionLevel::Default)
            .unwrap();
        let decompressed = manager.decompress("test_dict", &compressed).unwrap();

        assert_eq!(data.to_vec(), decompressed);
    }
}
