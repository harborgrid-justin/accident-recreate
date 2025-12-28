//! Multi-file archive format (.accuscene)

use crate::error::{CompressionError, Result};
use crate::traits::{Algorithm, CompressionLevel};
use std::collections::HashMap;
use tracing::{debug, trace};

/// Archive magic number "ACSC"
const ARCHIVE_MAGIC: u32 = 0x41435343;

/// Archive format version
const ARCHIVE_VERSION: u32 = 1;

/// Entry in the archive
#[derive(Debug, Clone)]
pub struct ArchiveEntry {
    /// File name/path within archive
    pub name: String,
    /// Uncompressed data
    pub data: Vec<u8>,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

impl ArchiveEntry {
    /// Create a new archive entry
    pub fn new(name: String, data: Vec<u8>) -> Self {
        Self {
            name,
            data,
            metadata: HashMap::new(),
        }
    }

    /// Create with metadata
    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = metadata;
        self
    }

    /// Add metadata key-value pair
    pub fn add_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// AccuScene archive (.accuscene file format)
#[derive(Debug)]
pub struct Archive {
    /// Archive entries
    entries: HashMap<String, ArchiveEntry>,
    /// Global metadata
    metadata: HashMap<String, String>,
    /// Compression algorithm
    algorithm: Algorithm,
    /// Compression level
    level: CompressionLevel,
}

impl Archive {
    /// Create a new empty archive
    pub fn new(algorithm: Algorithm, level: CompressionLevel) -> Self {
        Self {
            entries: HashMap::new(),
            metadata: HashMap::new(),
            algorithm,
            level,
        }
    }

    /// Create with default compression (Zstd, default level)
    pub fn new_default() -> Self {
        Self::new(Algorithm::Zstd, CompressionLevel::Default)
    }

    /// Add an entry to the archive
    pub fn add_entry(&mut self, entry: ArchiveEntry) {
        trace!("Adding entry '{}' ({} bytes)", entry.name, entry.data.len());
        self.entries.insert(entry.name.clone(), entry);
    }

    /// Add a file with name and data
    pub fn add_file(&mut self, name: String, data: Vec<u8>) {
        self.add_entry(ArchiveEntry::new(name, data));
    }

    /// Get an entry by name
    pub fn get_entry(&self, name: &str) -> Option<&ArchiveEntry> {
        self.entries.get(name)
    }

    /// Remove an entry
    pub fn remove_entry(&mut self, name: &str) -> Option<ArchiveEntry> {
        self.entries.remove(name)
    }

    /// List all entry names
    pub fn list_entries(&self) -> Vec<&str> {
        self.entries.keys().map(|s| s.as_str()).collect()
    }

    /// Get number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if archive is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Add global metadata
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// Get global metadata
    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    /// Serialize archive to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        debug!(
            "Serializing archive with {} entries using {:?}",
            self.entries.len(),
            self.algorithm
        );

        let mut bytes = Vec::new();

        // Write header
        bytes.extend_from_slice(&ARCHIVE_MAGIC.to_le_bytes());
        bytes.extend_from_slice(&ARCHIVE_VERSION.to_le_bytes());
        bytes.push(self.algorithm as u8);
        bytes.push(self.level as u8);

        // Write global metadata
        self.write_metadata(&mut bytes, &self.metadata);

        // Write number of entries
        bytes.extend_from_slice(&(self.entries.len() as u32).to_le_bytes());

        // Write each entry
        for entry in self.entries.values() {
            self.write_entry(&mut bytes, entry)?;
        }

        debug!("Archive serialized: {} bytes", bytes.len());
        Ok(bytes)
    }

    /// Deserialize archive from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        debug!("Deserializing archive from {} bytes", bytes.len());

        if bytes.len() < 16 {
            return Err(CompressionError::Archive(
                "Archive too small".to_string(),
            ));
        }

        let mut offset = 0;

        // Read and verify header
        let magic = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
        offset += 4;

        if magic != ARCHIVE_MAGIC {
            return Err(CompressionError::InvalidMagic {
                expected: ARCHIVE_MAGIC,
                actual: magic,
            });
        }

        let version = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
        offset += 4;

        if version != ARCHIVE_VERSION {
            return Err(CompressionError::UnsupportedVersion(version));
        }

        let algorithm = match bytes[offset] {
            0 => Algorithm::Lz4,
            1 => Algorithm::Zstd,
            2 => Algorithm::Brotli,
            3 => Algorithm::Deflate,
            4 => Algorithm::Snappy,
            _ => {
                return Err(CompressionError::Archive(format!(
                    "Invalid algorithm: {}",
                    bytes[offset]
                )))
            }
        };
        offset += 1;

        let level = match bytes[offset] {
            0 => CompressionLevel::Fastest,
            1 => CompressionLevel::Fast,
            2 => CompressionLevel::Default,
            3 => CompressionLevel::High,
            4 => CompressionLevel::Maximum,
            _ => CompressionLevel::Custom(bytes[offset] as i32),
        };
        offset += 1;

        // Read global metadata
        let (metadata, new_offset) = Self::read_metadata(bytes, offset)?;
        offset = new_offset;

        // Read number of entries
        let entry_count = u32::from_le_bytes(
            bytes[offset..offset + 4].try_into().unwrap()
        ) as usize;
        offset += 4;

        // Read entries
        let mut entries = HashMap::new();
        for _ in 0..entry_count {
            let (entry, new_offset) = Self::read_entry(bytes, offset, algorithm)?;
            offset = new_offset;
            entries.insert(entry.name.clone(), entry);
        }

        debug!("Archive deserialized: {} entries", entries.len());

        Ok(Self {
            entries,
            metadata,
            algorithm,
            level,
        })
    }

    /// Write metadata to bytes
    fn write_metadata(&self, bytes: &mut Vec<u8>, metadata: &HashMap<String, String>) {
        bytes.extend_from_slice(&(metadata.len() as u32).to_le_bytes());

        for (key, value) in metadata {
            let key_bytes = key.as_bytes();
            let value_bytes = value.as_bytes();

            bytes.extend_from_slice(&(key_bytes.len() as u32).to_le_bytes());
            bytes.extend_from_slice(key_bytes);

            bytes.extend_from_slice(&(value_bytes.len() as u32).to_le_bytes());
            bytes.extend_from_slice(value_bytes);
        }
    }

    /// Read metadata from bytes
    fn read_metadata(bytes: &[u8], mut offset: usize) -> Result<(HashMap<String, String>, usize)> {
        let count = u32::from_le_bytes(
            bytes[offset..offset + 4].try_into().unwrap()
        ) as usize;
        offset += 4;

        let mut metadata = HashMap::new();

        for _ in 0..count {
            let key_len = u32::from_le_bytes(
                bytes[offset..offset + 4].try_into().unwrap()
            ) as usize;
            offset += 4;

            let key = String::from_utf8(bytes[offset..offset + key_len].to_vec())
                .map_err(|e| CompressionError::Archive(e.to_string()))?;
            offset += key_len;

            let value_len = u32::from_le_bytes(
                bytes[offset..offset + 4].try_into().unwrap()
            ) as usize;
            offset += 4;

            let value = String::from_utf8(bytes[offset..offset + value_len].to_vec())
                .map_err(|e| CompressionError::Archive(e.to_string()))?;
            offset += value_len;

            metadata.insert(key, value);
        }

        Ok((metadata, offset))
    }

    /// Write entry to bytes
    fn write_entry(&self, bytes: &mut Vec<u8>, entry: &ArchiveEntry) -> Result<()> {
        // Write name
        let name_bytes = entry.name.as_bytes();
        bytes.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(name_bytes);

        // Write metadata
        self.write_metadata(bytes, &entry.metadata);

        // Compress data
        let compressed = crate::algorithms::compress(&entry.data, self.algorithm, self.level)?;

        // Write uncompressed size
        bytes.extend_from_slice(&(entry.data.len() as u64).to_le_bytes());

        // Write compressed size and data
        bytes.extend_from_slice(&(compressed.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&compressed);

        Ok(())
    }

    /// Read entry from bytes
    fn read_entry(bytes: &[u8], mut offset: usize, algorithm: Algorithm) -> Result<(ArchiveEntry, usize)> {
        // Read name
        let name_len = u32::from_le_bytes(
            bytes[offset..offset + 4].try_into().unwrap()
        ) as usize;
        offset += 4;

        let name = String::from_utf8(bytes[offset..offset + name_len].to_vec())
            .map_err(|e| CompressionError::Archive(e.to_string()))?;
        offset += name_len;

        // Read metadata
        let (metadata, new_offset) = Self::read_metadata(bytes, offset)?;
        offset = new_offset;

        // Read uncompressed size
        let _uncompressed_size = u64::from_le_bytes(
            bytes[offset..offset + 8].try_into().unwrap()
        ) as usize;
        offset += 8;

        // Read compressed data
        let compressed_len = u32::from_le_bytes(
            bytes[offset..offset + 4].try_into().unwrap()
        ) as usize;
        offset += 4;

        let compressed = &bytes[offset..offset + compressed_len];
        offset += compressed_len;

        // Decompress data
        let data = crate::algorithms::decompress(compressed, algorithm)?;

        let entry = ArchiveEntry {
            name,
            data,
            metadata,
        };

        Ok((entry, offset))
    }

    /// Write archive to file
    pub fn write_to_file(&self, path: &std::path::Path) -> Result<()> {
        debug!("Writing archive to {:?}", path);
        let bytes = self.to_bytes()?;
        std::fs::write(path, bytes)?;
        Ok(())
    }

    /// Read archive from file
    pub fn read_from_file(path: &std::path::Path) -> Result<Self> {
        debug!("Reading archive from {:?}", path);
        let bytes = std::fs::read(path)?;
        Self::from_bytes(&bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_archive_round_trip() {
        let mut archive = Archive::new_default();

        archive.add_file("file1.txt".to_string(), b"Hello, AccuScene!".to_vec());
        archive.add_file("file2.txt".to_string(), b"This is file 2".to_vec());

        let entry = ArchiveEntry::new("data.bin".to_string(), vec![1, 2, 3, 4, 5])
            .add_metadata("type".to_string(), "binary".to_string());
        archive.add_entry(entry);

        archive.add_metadata("version".to_string(), "1.0".to_string());

        let bytes = archive.to_bytes().unwrap();
        let restored = Archive::from_bytes(&bytes).unwrap();

        assert_eq!(archive.len(), restored.len());
        assert_eq!(
            archive.get_entry("file1.txt").unwrap().data,
            restored.get_entry("file1.txt").unwrap().data
        );
    }

    #[test]
    fn test_archive_algorithms() {
        for algorithm in [
            Algorithm::Lz4,
            Algorithm::Zstd,
            Algorithm::Brotli,
            Algorithm::Deflate,
            Algorithm::Snappy,
        ] {
            let mut archive = Archive::new(algorithm, CompressionLevel::Default);
            archive.add_file("test.txt".to_string(), b"Test data".to_vec());

            let bytes = archive.to_bytes().unwrap();
            let restored = Archive::from_bytes(&bytes).unwrap();

            assert_eq!(archive.len(), restored.len());
        }
    }
}
