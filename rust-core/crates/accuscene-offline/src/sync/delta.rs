use crate::config::CompressionAlgorithm;
use crate::error::{OfflineError, Result};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

/// Delta-encoded data for bandwidth optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delta {
    /// Base version hash
    pub base_hash: String,

    /// Target version hash
    pub target_hash: String,

    /// Compressed delta data
    pub data: Vec<u8>,

    /// Compression algorithm used
    pub compression: CompressionAlgorithm,

    /// Uncompressed size
    pub uncompressed_size: usize,

    /// Compressed size
    pub compressed_size: usize,

    /// Checksum for integrity verification
    pub checksum: String,
}

impl Delta {
    /// Calculate compression ratio
    pub fn compression_ratio(&self) -> f64 {
        if self.uncompressed_size == 0 {
            return 0.0;
        }
        self.compressed_size as f64 / self.uncompressed_size as f64
    }

    /// Verify delta integrity
    pub fn verify(&self) -> bool {
        let computed_checksum = format!("{:x}", blake3::hash(&self.data));
        computed_checksum == self.checksum
    }
}

/// Delta encoding engine for bandwidth optimization
pub struct DeltaEncoder {
    compression: CompressionAlgorithm,
}

impl DeltaEncoder {
    /// Create a new delta encoder
    pub fn new(compression: CompressionAlgorithm) -> Self {
        Self { compression }
    }

    /// Encode delta between base and target
    pub fn encode(&self, base: &[u8], target: &[u8]) -> Result<Delta> {
        // Compute binary diff
        let diff = self.compute_binary_diff(base, target);

        // Compress the diff
        let compressed = self.compress(&diff)?;

        // Calculate hashes
        let base_hash = format!("{:x}", blake3::hash(base));
        let target_hash = format!("{:x}", blake3::hash(target));
        let checksum = format!("{:x}", blake3::hash(&compressed));

        Ok(Delta {
            base_hash,
            target_hash,
            data: compressed.clone(),
            compression: self.compression,
            uncompressed_size: diff.len(),
            compressed_size: compressed.len(),
            checksum,
        })
    }

    /// Decode delta and apply to base to get target
    pub fn decode(&self, base: &[u8], delta: &Delta) -> Result<Vec<u8>> {
        // Verify delta integrity
        if !delta.verify() {
            return Err(OfflineError::DataCorruption(
                "Delta checksum mismatch".to_string()
            ));
        }

        // Verify base hash
        let base_hash = format!("{:x}", blake3::hash(base));
        if base_hash != delta.base_hash {
            return Err(OfflineError::DataCorruption(
                format!("Base hash mismatch: expected {}, got {}", delta.base_hash, base_hash)
            ));
        }

        // Decompress delta
        let diff = self.decompress(&delta.data, delta.compression)?;

        // Apply binary diff
        let target = self.apply_binary_diff(base, &diff);

        // Verify target hash
        let target_hash = format!("{:x}", blake3::hash(&target));
        if target_hash != delta.target_hash {
            return Err(OfflineError::DataCorruption(
                format!("Target hash mismatch: expected {}, got {}", delta.target_hash, target_hash)
            ));
        }

        Ok(target)
    }

    /// Compute binary diff (simple implementation - can be improved with vcdiff/xdelta)
    fn compute_binary_diff(&self, base: &[u8], target: &[u8]) -> Vec<u8> {
        // For simplicity, use a simple format:
        // [operation_type: u8][length: u32][data: bytes]
        // operation_type: 0 = copy from base, 1 = insert new data

        let mut diff = Vec::new();

        if base == target {
            // No changes
            return diff;
        }

        // Simple implementation: just store the full target
        // A more sophisticated approach would use rolling hash (like rsync)
        diff.push(1u8); // insert operation
        diff.extend(&(target.len() as u32).to_le_bytes());
        diff.extend_from_slice(target);

        diff
    }

    /// Apply binary diff to base
    fn apply_binary_diff(&self, base: &[u8], diff: &[u8]) -> Vec<u8> {
        if diff.is_empty() {
            return base.to_vec();
        }

        let mut result = Vec::new();
        let mut pos = 0;

        while pos < diff.len() {
            let op_type = diff[pos];
            pos += 1;

            if pos + 4 > diff.len() {
                break;
            }

            let length = u32::from_le_bytes([diff[pos], diff[pos + 1], diff[pos + 2], diff[pos + 3]]) as usize;
            pos += 4;

            match op_type {
                0 => {
                    // Copy from base
                    if pos + 4 > diff.len() {
                        break;
                    }
                    let offset = u32::from_le_bytes([diff[pos], diff[pos + 1], diff[pos + 2], diff[pos + 3]]) as usize;
                    pos += 4;

                    if offset + length <= base.len() {
                        result.extend_from_slice(&base[offset..offset + length]);
                    }
                }
                1 => {
                    // Insert new data
                    if pos + length <= diff.len() {
                        result.extend_from_slice(&diff[pos..pos + length]);
                        pos += length;
                    }
                }
                _ => break,
            }
        }

        result
    }

    /// Compress data
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
        match self.compression {
            CompressionAlgorithm::None => Ok(data.to_vec()),

            CompressionAlgorithm::Gzip => {
                let mut encoder = flate2::write::GzEncoder::new(
                    Vec::new(),
                    flate2::Compression::best(),
                );
                encoder.write_all(data)?;
                Ok(encoder.finish()?)
            }

            CompressionAlgorithm::Lz4 => {
                let mut encoder = lz4::EncoderBuilder::new()
                    .level(4)
                    .build(Vec::new())?;
                encoder.write_all(data)?;
                let (compressed, result) = encoder.finish();
                result?;
                Ok(compressed)
            }

            CompressionAlgorithm::Zstd => {
                // Fallback to gzip if zstd not available
                let mut encoder = flate2::write::GzEncoder::new(
                    Vec::new(),
                    flate2::Compression::best(),
                );
                encoder.write_all(data)?;
                Ok(encoder.finish()?)
            }
        }
    }

    /// Decompress data
    fn decompress(&self, data: &[u8], algorithm: CompressionAlgorithm) -> Result<Vec<u8>> {
        match algorithm {
            CompressionAlgorithm::None => Ok(data.to_vec()),

            CompressionAlgorithm::Gzip => {
                let mut decoder = flate2::read::GzDecoder::new(data);
                let mut decompressed = Vec::new();
                decoder.read_to_end(&mut decompressed)?;
                Ok(decompressed)
            }

            CompressionAlgorithm::Lz4 => {
                let mut decoder = lz4::Decoder::new(data)?;
                let mut decompressed = Vec::new();
                decoder.read_to_end(&mut decompressed)?;
                Ok(decompressed)
            }

            CompressionAlgorithm::Zstd => {
                // Fallback to gzip
                let mut decoder = flate2::read::GzDecoder::new(data);
                let mut decompressed = Vec::new();
                decoder.read_to_end(&mut decompressed)?;
                Ok(decompressed)
            }
        }
    }
}

impl Default for DeltaEncoder {
    fn default() -> Self {
        Self::new(CompressionAlgorithm::Lz4)
    }
}

/// Delta sync manager for efficient data transfer
pub struct DeltaSyncManager {
    encoder: DeltaEncoder,
    cache_size: usize,
    version_cache: std::sync::Arc<parking_lot::RwLock<lru::LruCache<String, Vec<u8>>>>,
}

impl DeltaSyncManager {
    /// Create a new delta sync manager
    pub fn new(compression: CompressionAlgorithm, cache_size: usize) -> Self {
        Self {
            encoder: DeltaEncoder::new(compression),
            cache_size,
            version_cache: std::sync::Arc::new(parking_lot::RwLock::new(
                lru::LruCache::new(std::num::NonZeroUsize::new(cache_size).unwrap())
            )),
        }
    }

    /// Cache a version
    pub fn cache_version(&self, hash: String, data: Vec<u8>) {
        let mut cache = self.version_cache.write();
        cache.put(hash, data);
    }

    /// Get cached version
    pub fn get_cached_version(&self, hash: &str) -> Option<Vec<u8>> {
        let mut cache = self.version_cache.write();
        cache.get(hash).cloned()
    }

    /// Create delta from base to target
    pub fn create_delta(&self, base_hash: &str, target: &[u8]) -> Result<Option<Delta>> {
        // Try to get base from cache
        if let Some(base) = self.get_cached_version(base_hash) {
            let delta = self.encoder.encode(&base, target)?;

            // Cache target for future deltas
            let target_hash = delta.target_hash.clone();
            self.cache_version(target_hash, target.to_vec());

            // Only use delta if it's smaller than full data
            if delta.compressed_size < target.len() {
                Ok(Some(delta))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// Apply delta to get target
    pub fn apply_delta(&self, delta: &Delta) -> Result<Vec<u8>> {
        // Get base from cache
        let base = self.get_cached_version(&delta.base_hash)
            .ok_or_else(|| OfflineError::InvalidState(
                format!("Base version {} not in cache", delta.base_hash)
            ))?;

        let target = self.encoder.decode(&base, delta)?;

        // Cache target
        self.cache_version(delta.target_hash.clone(), target.clone());

        Ok(target)
    }

    /// Clear version cache
    pub fn clear_cache(&self) {
        let mut cache = self.version_cache.write();
        cache.clear();
    }
}

use lru;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delta_encoding() {
        let encoder = DeltaEncoder::new(CompressionAlgorithm::Lz4);

        let base = b"Hello, world!";
        let target = b"Hello, Rust world!";

        let delta = encoder.encode(base, target).unwrap();
        assert!(delta.verify());

        let decoded = encoder.decode(base, &delta).unwrap();
        assert_eq!(decoded, target);
    }

    #[test]
    fn test_compression_ratio() {
        let encoder = DeltaEncoder::new(CompressionAlgorithm::Lz4);

        let base = vec![0u8; 1000];
        let mut target = base.clone();
        target[500] = 1;

        let delta = encoder.encode(&base, &target).unwrap();
        assert!(delta.compression_ratio() < 1.0);
    }

    #[test]
    fn test_delta_sync_manager() {
        let manager = DeltaSyncManager::new(CompressionAlgorithm::Lz4, 10);

        let base = b"Hello, world!";
        let base_hash = format!("{:x}", blake3::hash(base));

        manager.cache_version(base_hash.clone(), base.to_vec());

        let target = b"Hello, Rust world!";
        let delta = manager.create_delta(&base_hash, target).unwrap();

        if let Some(delta) = delta {
            let decoded = manager.apply_delta(&delta).unwrap();
            assert_eq!(decoded, target);
        }
    }
}
