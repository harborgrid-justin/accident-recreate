use crate::error::{OfflineError, Result};
use crate::storage::{Storage, StorageRecord};
use crate::sync::SyncOperation;
use crate::versioning::Version;
use async_trait::async_trait;

#[cfg(feature = "rocksdb-backend")]
use rocksdb::{DB, Options, WriteBatch};
use std::path::Path;
use std::sync::Arc;

/// RocksDB storage backend (optimized for mobile and high-performance scenarios)
#[cfg(feature = "rocksdb-backend")]
pub struct RocksDbStorage {
    db: Arc<DB>,
}

#[cfg(feature = "rocksdb-backend")]
impl RocksDbStorage {
    /// Create a new RocksDB storage
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
        opts.set_write_buffer_size(64 * 1024 * 1024); // 64MB
        opts.set_max_write_buffer_number(3);
        opts.set_target_file_size_base(64 * 1024 * 1024);
        opts.increase_parallelism(num_cpus::get() as i32);

        let db = DB::open(&opts, path)
            .map_err(|e| OfflineError::Storage(e.to_string()))?;

        Ok(Self {
            db: Arc::new(db),
        })
    }

    /// Generate key for record
    fn record_key(entity_id: &str, entity_type: &str) -> Vec<u8> {
        format!("rec:{}:{}", entity_type, entity_id).into_bytes()
    }

    /// Generate key for operation
    fn operation_key(operation_id: &str) -> Vec<u8> {
        format!("op:{}", operation_id).into_bytes()
    }

    /// Generate prefix for entity type
    fn entity_type_prefix(entity_type: &str) -> Vec<u8> {
        format!("rec:{}:", entity_type).into_bytes()
    }
}

#[cfg(feature = "rocksdb-backend")]
#[async_trait]
impl Storage for RocksDbStorage {
    async fn init(&self) -> Result<()> {
        Ok(())
    }

    async fn put(&self, record: StorageRecord) -> Result<()> {
        let key = Self::record_key(&record.entity_id, &record.entity_type);
        let value = bincode::serialize(&record)?;

        self.db
            .put(key, value)
            .map_err(|e| OfflineError::Storage(e.to_string()))?;

        Ok(())
    }

    async fn get(&self, entity_id: &str, entity_type: &str) -> Result<Option<StorageRecord>> {
        let key = Self::record_key(entity_id, entity_type);

        match self.db.get(key) {
            Ok(Some(value)) => {
                let record: StorageRecord = bincode::deserialize(&value)?;
                if record.deleted {
                    Ok(None)
                } else {
                    Ok(Some(record))
                }
            }
            Ok(None) => Ok(None),
            Err(e) => Err(OfflineError::Storage(e.to_string())),
        }
    }

    async fn delete(&self, entity_id: &str, entity_type: &str) -> Result<()> {
        if let Some(mut record) = self.get(entity_id, entity_type).await? {
            record.deleted = true;
            record.updated_at = chrono::Utc::now();
            self.put(record).await?;
        }
        Ok(())
    }

    async fn list(&self, entity_type: &str, limit: Option<usize>) -> Result<Vec<StorageRecord>> {
        let prefix = Self::entity_type_prefix(entity_type);
        let iter = self.db.prefix_iterator(&prefix);

        let mut records = Vec::new();
        let mut count = 0;

        for item in iter {
            if let Some(limit) = limit {
                if count >= limit {
                    break;
                }
            }

            match item {
                Ok((_key, value)) => {
                    if let Ok(record) = bincode::deserialize::<StorageRecord>(&value) {
                        if !record.deleted {
                            records.push(record);
                            count += 1;
                        }
                    }
                }
                Err(e) => return Err(OfflineError::Storage(e.to_string())),
            }
        }

        Ok(records)
    }

    async fn store_operation(&self, operation: SyncOperation) -> Result<()> {
        let key = Self::operation_key(&operation.id);
        let value = bincode::serialize(&operation)?;

        self.db
            .put(key, value)
            .map_err(|e| OfflineError::Storage(e.to_string()))?;

        Ok(())
    }

    async fn get_pending_operations(&self) -> Result<Vec<SyncOperation>> {
        let prefix = b"op:";
        let iter = self.db.prefix_iterator(prefix);

        let mut operations = Vec::new();

        for item in iter {
            match item {
                Ok((_key, value)) => {
                    if let Ok(operation) = bincode::deserialize::<SyncOperation>(&value) {
                        operations.push(operation);
                    }
                }
                Err(e) => return Err(OfflineError::Storage(e.to_string())),
            }
        }

        // Sort by priority and queued time
        operations.sort_by(|a, b| {
            b.priority
                .cmp(&a.priority)
                .then_with(|| a.queued_at.cmp(&b.queued_at))
        });

        Ok(operations)
    }

    async fn mark_operation_completed(&self, operation_id: &str) -> Result<()> {
        let key = Self::operation_key(operation_id);
        self.db
            .delete(key)
            .map_err(|e| OfflineError::Storage(e.to_string()))?;
        Ok(())
    }

    async fn size(&self) -> Result<u64> {
        // Approximate size
        let mut size = 0u64;

        if let Some(live_files) = self.db.live_files() {
            for file in live_files {
                size += file.size as u64;
            }
        }

        Ok(size)
    }

    async fn clear(&self) -> Result<()> {
        // Delete all keys
        let iter = self.db.iterator(rocksdb::IteratorMode::Start);

        let mut batch = WriteBatch::default();
        for item in iter {
            if let Ok((key, _)) = item {
                batch.delete(key);
            }
        }

        self.db
            .write(batch)
            .map_err(|e| OfflineError::Storage(e.to_string()))?;

        Ok(())
    }

    async fn vacuum(&self) -> Result<()> {
        // Compact entire database
        self.db
            .compact_range::<&[u8], &[u8]>(None, None);
        Ok(())
    }

    async fn get_version(&self, entity_id: &str, entity_type: &str) -> Result<Option<Version>> {
        Ok(self.get(entity_id, entity_type).await?.map(|r| r.version))
    }

    async fn list_entity_ids(&self, entity_type: &str) -> Result<Vec<String>> {
        let prefix = Self::entity_type_prefix(entity_type);
        let iter = self.db.prefix_iterator(&prefix);

        let mut ids = Vec::new();

        for item in iter {
            match item {
                Ok((_key, value)) => {
                    if let Ok(record) = bincode::deserialize::<StorageRecord>(&value) {
                        if !record.deleted {
                            ids.push(record.entity_id);
                        }
                    }
                }
                Err(e) => return Err(OfflineError::Storage(e.to_string())),
            }
        }

        Ok(ids)
    }

    async fn batch_get(&self, ids: Vec<(String, String)>) -> Result<Vec<Option<StorageRecord>>> {
        let mut results = Vec::new();

        for (entity_id, entity_type) in ids {
            results.push(self.get(&entity_id, &entity_type).await?);
        }

        Ok(results)
    }

    async fn batch_put(&self, records: Vec<StorageRecord>) -> Result<()> {
        let mut batch = WriteBatch::default();

        for record in records {
            let key = Self::record_key(&record.entity_id, &record.entity_type);
            let value = bincode::serialize(&record)?;
            batch.put(key, value);
        }

        self.db
            .write(batch)
            .map_err(|e| OfflineError::Storage(e.to_string()))?;

        Ok(())
    }

    async fn query(
        &self,
        entity_type: &str,
        _filter: Option<String>,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<StorageRecord>> {
        let prefix = Self::entity_type_prefix(entity_type);
        let iter = self.db.prefix_iterator(&prefix);

        let mut records = Vec::new();
        let offset = offset.unwrap_or(0);
        let mut count = 0;
        let mut skipped = 0;

        for item in iter {
            match item {
                Ok((_key, value)) => {
                    if let Ok(record) = bincode::deserialize::<StorageRecord>(&value) {
                        if !record.deleted {
                            if skipped < offset {
                                skipped += 1;
                                continue;
                            }

                            records.push(record);
                            count += 1;

                            if let Some(limit) = limit {
                                if count >= limit {
                                    break;
                                }
                            }
                        }
                    }
                }
                Err(e) => return Err(OfflineError::Storage(e.to_string())),
            }
        }

        Ok(records)
    }
}

// Stub implementation when rocksdb feature is disabled
#[cfg(not(feature = "rocksdb-backend"))]
pub struct RocksDbStorage;

#[cfg(not(feature = "rocksdb-backend"))]
impl RocksDbStorage {
    pub fn new<P: AsRef<Path>>(_path: P) -> Result<Self> {
        Err(OfflineError::InvalidOperation(
            "RocksDB backend not enabled. Enable 'rocksdb-backend' feature".to_string()
        ))
    }
}

mod num_cpus {
    pub fn get() -> usize {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
    }
}
