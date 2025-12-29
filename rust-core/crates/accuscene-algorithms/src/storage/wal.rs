//! Write-Ahead Log (WAL) for durability.
//!
//! Ensures data durability by logging all modifications before applying them.
//! Critical for database recovery after crashes.
//!
//! # Complexity
//! - Append: O(1) with buffering
//! - Replay: O(n) where n is log size
//! - Space: O(n) where n is number of operations

use crate::config::{SyncMode, WalConfig};
use crate::error::{AlgorithmError, Result};
use bytes::{BufMut, BytesMut};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Log sequence number.
pub type Lsn = u64;

/// WAL operation type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Operation {
    /// Insert or update a key-value pair.
    Put { key: Vec<u8>, value: Vec<u8> },
    /// Delete a key.
    Delete { key: Vec<u8> },
    /// Begin transaction.
    Begin { txn_id: u64 },
    /// Commit transaction.
    Commit { txn_id: u64 },
    /// Abort transaction.
    Abort { txn_id: u64 },
    /// Checkpoint marker.
    Checkpoint { lsn: Lsn },
}

/// WAL log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Log sequence number (monotonically increasing).
    pub lsn: Lsn,
    /// Operation to log.
    pub operation: Operation,
    /// Checksum for integrity.
    pub checksum: u32,
}

impl LogEntry {
    /// Create a new log entry.
    pub fn new(lsn: Lsn, operation: Operation) -> Self {
        let mut entry = Self {
            lsn,
            operation,
            checksum: 0,
        };
        entry.checksum = entry.compute_checksum();
        entry
    }

    /// Compute checksum for the entry.
    fn compute_checksum(&self) -> u32 {
        let data = bincode::serialize(&(self.lsn, &self.operation)).unwrap();
        seahash::hash(&data) as u32
    }

    /// Verify entry checksum.
    pub fn verify(&self) -> bool {
        self.checksum == self.compute_checksum()
    }

    /// Serialize entry to bytes.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let data = bincode::serialize(self)?;
        let mut bytes = BytesMut::with_capacity(4 + data.len());
        bytes.put_u32_le(data.len() as u32);
        bytes.put_slice(&data);
        Ok(bytes.to_vec())
    }

    /// Deserialize entry from bytes.
    pub fn from_bytes(mut bytes: &[u8]) -> Result<(Self, usize)> {
        if bytes.len() < 4 {
            return Err(AlgorithmError::InvalidFormat(
                "WAL entry too small".to_string(),
            ));
        }

        let len = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
        bytes = &bytes[4..];

        if bytes.len() < len {
            return Err(AlgorithmError::InvalidFormat(
                "WAL entry truncated".to_string(),
            ));
        }

        let entry: LogEntry = bincode::deserialize(&bytes[..len])?;

        if !entry.verify() {
            return Err(AlgorithmError::WalError(
                "Checksum verification failed".to_string(),
            ));
        }

        Ok((entry, 4 + len))
    }
}

/// Write-Ahead Log for durable transaction logging.
pub struct WriteAheadLog {
    file: Arc<Mutex<BufWriter<File>>>,
    path: PathBuf,
    next_lsn: Arc<Mutex<Lsn>>,
    config: WalConfig,
    write_count: Arc<Mutex<usize>>,
}

impl WriteAheadLog {
    /// Create or open a WAL at the specified path.
    pub fn open<P: AsRef<Path>>(path: P, config: WalConfig) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(&path)?;

        let writer = BufWriter::new(file);

        Ok(Self {
            file: Arc::new(Mutex::new(writer)),
            path,
            next_lsn: Arc::new(Mutex::new(0)),
            config,
            write_count: Arc::new(Mutex::new(0)),
        })
    }

    /// Append a log entry.
    ///
    /// # Complexity
    /// O(1) with buffering
    pub fn append(&self, operation: Operation) -> Result<Lsn> {
        let lsn = {
            let mut next_lsn = self.next_lsn.lock();
            let lsn = *next_lsn;
            *next_lsn += 1;
            lsn
        };

        let entry = LogEntry::new(lsn, operation);
        let bytes = entry.to_bytes()?;

        {
            let mut file = self.file.lock();
            file.write_all(&bytes)?;

            // Handle sync mode
            match self.config.sync_mode {
                SyncMode::EveryWrite => {
                    file.flush()?;
                    file.get_ref().sync_all()?;
                }
                SyncMode::EveryN(n) => {
                    let mut count = self.write_count.lock();
                    *count += 1;
                    if *count >= n {
                        file.flush()?;
                        file.get_ref().sync_all()?;
                        *count = 0;
                    }
                }
                SyncMode::Manual => {
                    // No automatic sync
                }
            }
        }

        Ok(lsn)
    }

    /// Manually flush and sync to disk.
    pub fn flush(&self) -> Result<()> {
        let mut file = self.file.lock();
        file.flush()?;
        file.get_ref().sync_all()?;
        Ok(())
    }

    /// Replay the log, calling callback for each entry.
    ///
    /// # Complexity
    /// O(n) where n is number of log entries
    pub fn replay<F>(&self, mut callback: F) -> Result<()>
    where
        F: FnMut(&LogEntry) -> Result<()>,
    {
        let mut file = File::open(&self.path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        let mut offset = 0;
        while offset < buffer.len() {
            match LogEntry::from_bytes(&buffer[offset..]) {
                Ok((entry, size)) => {
                    callback(&entry)?;
                    offset += size;

                    // Update LSN
                    let mut next_lsn = self.next_lsn.lock();
                    *next_lsn = entry.lsn + 1;
                }
                Err(e) => {
                    // Truncated entry at end of file is OK during recovery
                    if offset + 4 > buffer.len() {
                        break;
                    }
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    /// Write a checkpoint marker.
    pub fn checkpoint(&self) -> Result<Lsn> {
        let lsn = *self.next_lsn.lock();
        self.append(Operation::Checkpoint { lsn })
    }

    /// Truncate the log (dangerous - should only be used after checkpoint).
    pub fn truncate(&self) -> Result<()> {
        let mut file = self.file.lock();
        file.flush()?;
        drop(file);

        // Reopen file truncated
        let new_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.path)?;

        *self.file.lock() = BufWriter::new(new_file);
        *self.next_lsn.lock() = 0;
        *self.write_count.lock() = 0;

        Ok(())
    }

    /// Get current LSN.
    pub fn current_lsn(&self) -> Lsn {
        *self.next_lsn.lock()
    }

    /// Get WAL file size.
    pub fn size(&self) -> Result<u64> {
        Ok(std::fs::metadata(&self.path)?.len())
    }
}

impl Clone for WriteAheadLog {
    fn clone(&self) -> Self {
        Self {
            file: self.file.clone(),
            path: self.path.clone(),
            next_lsn: self.next_lsn.clone(),
            config: self.config.clone(),
            write_count: self.write_count.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_append_and_replay() {
        let temp_file = NamedTempFile::new().unwrap();
        let wal = WriteAheadLog::open(temp_file.path(), WalConfig::default()).unwrap();

        // Append some entries
        wal.append(Operation::Put {
            key: b"key1".to_vec(),
            value: b"value1".to_vec(),
        })
        .unwrap();

        wal.append(Operation::Delete {
            key: b"key2".to_vec(),
        })
        .unwrap();

        wal.flush().unwrap();

        // Replay
        let mut count = 0;
        wal.replay(|_entry| {
            count += 1;
            Ok(())
        })
        .unwrap();

        assert_eq!(count, 2);
    }

    #[test]
    fn test_checkpoint() {
        let temp_file = NamedTempFile::new().unwrap();
        let wal = WriteAheadLog::open(temp_file.path(), WalConfig::default()).unwrap();

        wal.append(Operation::Put {
            key: b"key".to_vec(),
            value: b"value".to_vec(),
        })
        .unwrap();

        let checkpoint_lsn = wal.checkpoint().unwrap();
        assert!(checkpoint_lsn > 0);
    }
}
