//! Delta compression for incremental saves and efficient version storage

use crate::error::{CompressionError, Result};
use crate::traits::{Algorithm, CompressionLevel};
use rayon::prelude::*;
use std::cmp::min;
use tracing::{debug, trace};

/// Delta compression magic number
const DELTA_MAGIC: u32 = 0x44454C54; // "DELT"

/// Delta compression version
const DELTA_VERSION: u32 = 1;

/// Block size for delta encoding (4KB)
const BLOCK_SIZE: usize = 4096;

/// Delta operation
#[derive(Debug, Clone)]
enum DeltaOp {
    /// Copy bytes from base at offset, length
    Copy { offset: usize, length: usize },
    /// Insert new bytes
    Insert { data: Vec<u8> },
}

/// Delta patch for applying changes
#[derive(Debug, Clone)]
pub struct DeltaPatch {
    /// Base data checksum
    base_checksum: u32,
    /// Target data checksum
    target_checksum: u32,
    /// Delta operations
    operations: Vec<DeltaOp>,
    /// Original target size
    target_size: usize,
}

impl DeltaPatch {
    /// Create a delta patch between base and target
    pub fn create(base: &[u8], target: &[u8]) -> Self {
        trace!(
            "Creating delta patch: base {} bytes, target {} bytes",
            base.len(),
            target.len()
        );

        let base_checksum = crc32fast::hash(base);
        let target_checksum = crc32fast::hash(target);

        let operations = Self::compute_delta(base, target);

        debug!(
            "Delta patch created with {} operations",
            operations.len()
        );

        Self {
            base_checksum,
            target_checksum,
            operations,
            target_size: target.len(),
        }
    }

    /// Apply delta patch to base data
    pub fn apply(&self, base: &[u8]) -> Result<Vec<u8>> {
        trace!("Applying delta patch to {} byte base", base.len());

        // Verify base checksum
        let base_checksum = crc32fast::hash(base);
        if base_checksum != self.base_checksum {
            return Err(CompressionError::ChecksumMismatch {
                expected: self.base_checksum,
                actual: base_checksum,
            });
        }

        let mut result = Vec::with_capacity(self.target_size);

        for op in &self.operations {
            match op {
                DeltaOp::Copy { offset, length } => {
                    if *offset + *length > base.len() {
                        return Err(CompressionError::CorruptedData(
                            "Delta copy operation out of bounds".to_string(),
                        ));
                    }
                    result.extend_from_slice(&base[*offset..*offset + *length]);
                }
                DeltaOp::Insert { data } => {
                    result.extend_from_slice(data);
                }
            }
        }

        // Verify target checksum
        let result_checksum = crc32fast::hash(&result);
        if result_checksum != self.target_checksum {
            return Err(CompressionError::ChecksumMismatch {
                expected: self.target_checksum,
                actual: result_checksum,
            });
        }

        debug!("Delta patch applied successfully: {} bytes", result.len());
        Ok(result)
    }

    /// Compute delta operations using a simple block-based algorithm
    fn compute_delta(base: &[u8], target: &[u8]) -> Vec<DeltaOp> {
        let mut operations = Vec::new();
        let mut target_pos = 0;

        // Build hash table of base blocks for quick matching
        let base_blocks = Self::build_block_index(base);

        while target_pos < target.len() {
            let remaining = target.len() - target_pos;
            let block_size = min(BLOCK_SIZE, remaining);
            let target_block = &target[target_pos..target_pos + block_size];

            // Try to find matching block in base
            if let Some(&base_offset) = base_blocks.get(&Self::hash_block(target_block)) {
                // Verify match and extend
                let match_len = Self::extend_match(base, base_offset, target, target_pos);

                if match_len >= 16 {
                    // Use copy operation for matches >= 16 bytes
                    operations.push(DeltaOp::Copy {
                        offset: base_offset,
                        length: match_len,
                    });
                    target_pos += match_len;
                    continue;
                }
            }

            // No match found, insert new data
            // Look ahead to find next match or end
            let insert_start = target_pos;
            let mut insert_end = target_pos + block_size;

            // Don't insert too much at once
            insert_end = min(insert_end, target_pos + BLOCK_SIZE * 4);
            insert_end = min(insert_end, target.len());

            operations.push(DeltaOp::Insert {
                data: target[insert_start..insert_end].to_vec(),
            });
            target_pos = insert_end;
        }

        Self::optimize_operations(operations)
    }

    /// Build index of block hashes to offsets in base
    fn build_block_index(base: &[u8]) -> std::collections::HashMap<u32, usize> {
        let mut index = std::collections::HashMap::new();

        for offset in 0..base.len().saturating_sub(16) {
            let block_size = min(BLOCK_SIZE, base.len() - offset);
            let block = &base[offset..offset + block_size];
            let hash = Self::hash_block(block);

            // Store only first occurrence of each hash
            index.entry(hash).or_insert(offset);
        }

        index
    }

    /// Hash a block of data
    fn hash_block(block: &[u8]) -> u32 {
        xxhash_rust::xxh3::xxh3_64(block) as u32
    }

    /// Extend match between base and target
    fn extend_match(base: &[u8], base_offset: usize, target: &[u8], target_offset: usize) -> usize {
        let max_len = min(base.len() - base_offset, target.len() - target_offset);
        let mut len = 0;

        while len < max_len && base[base_offset + len] == target[target_offset + len] {
            len += 1;
        }

        len
    }

    /// Optimize operations by merging adjacent inserts
    fn optimize_operations(operations: Vec<DeltaOp>) -> Vec<DeltaOp> {
        if operations.is_empty() {
            return operations;
        }

        let mut optimized = Vec::new();
        let mut current_insert: Option<Vec<u8>> = None;

        for op in operations {
            match op {
                DeltaOp::Insert { data } => {
                    if let Some(ref mut insert_data) = current_insert {
                        insert_data.extend_from_slice(&data);
                    } else {
                        current_insert = Some(data);
                    }
                }
                DeltaOp::Copy { offset, length } => {
                    // Flush any pending insert
                    if let Some(data) = current_insert.take() {
                        optimized.push(DeltaOp::Insert { data });
                    }
                    optimized.push(DeltaOp::Copy { offset, length });
                }
            }
        }

        // Flush final insert
        if let Some(data) = current_insert {
            optimized.push(DeltaOp::Insert { data });
        }

        optimized
    }

    /// Serialize patch to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Write header
        bytes.extend_from_slice(&DELTA_MAGIC.to_le_bytes());
        bytes.extend_from_slice(&DELTA_VERSION.to_le_bytes());
        bytes.extend_from_slice(&self.base_checksum.to_le_bytes());
        bytes.extend_from_slice(&self.target_checksum.to_le_bytes());
        bytes.extend_from_slice(&(self.target_size as u64).to_le_bytes());
        bytes.extend_from_slice(&(self.operations.len() as u32).to_le_bytes());

        // Write operations
        for op in &self.operations {
            match op {
                DeltaOp::Copy { offset, length } => {
                    bytes.push(0); // Copy op code
                    bytes.extend_from_slice(&(*offset as u64).to_le_bytes());
                    bytes.extend_from_slice(&(*length as u32).to_le_bytes());
                }
                DeltaOp::Insert { data } => {
                    bytes.push(1); // Insert op code
                    bytes.extend_from_slice(&(data.len() as u32).to_le_bytes());
                    bytes.extend_from_slice(data);
                }
            }
        }

        bytes
    }

    /// Deserialize patch from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 32 {
            return Err(CompressionError::CorruptedData(
                "Delta patch too small".to_string(),
            ));
        }

        let mut offset = 0;

        // Read and verify header
        let magic = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
        offset += 4;

        if magic != DELTA_MAGIC {
            return Err(CompressionError::InvalidMagic {
                expected: DELTA_MAGIC,
                actual: magic,
            });
        }

        let version = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
        offset += 4;

        if version != DELTA_VERSION {
            return Err(CompressionError::UnsupportedVersion(version));
        }

        let base_checksum = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
        offset += 4;

        let target_checksum = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
        offset += 4;

        let target_size = u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap()) as usize;
        offset += 8;

        let op_count = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;

        // Read operations
        let mut operations = Vec::with_capacity(op_count);

        for _ in 0..op_count {
            let op_code = bytes[offset];
            offset += 1;

            match op_code {
                0 => {
                    // Copy operation
                    let copy_offset = u64::from_le_bytes(
                        bytes[offset..offset + 8].try_into().unwrap()
                    ) as usize;
                    offset += 8;

                    let length = u32::from_le_bytes(
                        bytes[offset..offset + 4].try_into().unwrap()
                    ) as usize;
                    offset += 4;

                    operations.push(DeltaOp::Copy {
                        offset: copy_offset,
                        length,
                    });
                }
                1 => {
                    // Insert operation
                    let length = u32::from_le_bytes(
                        bytes[offset..offset + 4].try_into().unwrap()
                    ) as usize;
                    offset += 4;

                    let data = bytes[offset..offset + length].to_vec();
                    offset += length;

                    operations.push(DeltaOp::Insert { data });
                }
                _ => {
                    return Err(CompressionError::CorruptedData(format!(
                        "Invalid delta operation code: {}",
                        op_code
                    )));
                }
            }
        }

        Ok(Self {
            base_checksum,
            target_checksum,
            operations,
            target_size,
        })
    }
}

/// Compress a delta patch
pub fn compress_delta(base: &[u8], target: &[u8], algorithm: Algorithm) -> Result<Vec<u8>> {
    let patch = DeltaPatch::create(base, target);
    let patch_bytes = patch.to_bytes();

    debug!(
        "Delta patch size: {} bytes ({}% of target)",
        patch_bytes.len(),
        (patch_bytes.len() as f64 / target.len() as f64) * 100.0
    );

    // Compress the delta patch
    crate::algorithms::compress(&patch_bytes, algorithm, CompressionLevel::Default)
}

/// Decompress and apply a delta patch
pub fn decompress_delta(base: &[u8], compressed_delta: &[u8], algorithm: Algorithm) -> Result<Vec<u8>> {
    let patch_bytes = crate::algorithms::decompress(compressed_delta, algorithm)?;
    let patch = DeltaPatch::from_bytes(&patch_bytes)?;
    patch.apply(base)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delta_patch_identical() {
        let data = b"Hello, AccuScene!";
        let patch = DeltaPatch::create(data, data);
        let result = patch.apply(data).unwrap();
        assert_eq!(data.to_vec(), result);
    }

    #[test]
    fn test_delta_patch_append() {
        let base = b"Hello, AccuScene!";
        let target = b"Hello, AccuScene! Welcome to delta compression.";

        let patch = DeltaPatch::create(base, target);
        let result = patch.apply(base).unwrap();

        assert_eq!(target.to_vec(), result);
    }

    #[test]
    fn test_delta_patch_modification() {
        let base = b"The quick brown fox jumps over the lazy dog";
        let target = b"The quick brown cat jumps over the lazy dog";

        let patch = DeltaPatch::create(base, target);
        let result = patch.apply(base).unwrap();

        assert_eq!(target.to_vec(), result);
    }

    #[test]
    fn test_delta_serialization() {
        let base = b"Base version of the data";
        let target = b"Base version of the data with modifications";

        let patch = DeltaPatch::create(base, target);
        let bytes = patch.to_bytes();
        let restored = DeltaPatch::from_bytes(&bytes).unwrap();

        let result = restored.apply(base).unwrap();
        assert_eq!(target.to_vec(), result);
    }

    #[test]
    fn test_compressed_delta() {
        let base = b"Original case data version 1.0".repeat(100);
        let mut target = base.clone();
        target.extend_from_slice(b" with new simulation results");

        let compressed = compress_delta(&base, &target, Algorithm::Zstd).unwrap();
        let restored = decompress_delta(&base, &compressed, Algorithm::Zstd).unwrap();

        assert_eq!(target, restored);
        assert!(compressed.len() < target.len());
    }
}
