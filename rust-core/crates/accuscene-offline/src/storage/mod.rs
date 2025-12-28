pub mod sqlite;

#[cfg(feature = "rocksdb-backend")]
pub mod rocksdb;

use crate::error::Result;
use crate::sync::{SyncOperation, OperationType};
use crate::versioning::Version;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Storage record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageRecord {
    /// Entity ID
    pub entity_id: String,

    /// Entity type
    pub entity_type: String,

    /// Data
    pub data: serde_json::Value,

    /// Version
    pub version: Version,

    /// Created timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Updated timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,

    /// Deleted flag
    pub deleted: bool,
}

/// Storage backend trait
#[async_trait]
pub trait Storage: Send + Sync {
    /// Initialize storage
    async fn init(&self) -> Result<()>;

    /// Store a record
    async fn put(&self, record: StorageRecord) -> Result<()>;

    /// Get a record by ID
    async fn get(&self, entity_id: &str, entity_type: &str) -> Result<Option<StorageRecord>>;

    /// Delete a record
    async fn delete(&self, entity_id: &str, entity_type: &str) -> Result<()>;

    /// List records by type
    async fn list(&self, entity_type: &str, limit: Option<usize>) -> Result<Vec<StorageRecord>>;

    /// Store a pending operation
    async fn store_operation(&self, operation: SyncOperation) -> Result<()>;

    /// Get pending operations
    async fn get_pending_operations(&self) -> Result<Vec<SyncOperation>>;

    /// Mark operation as completed
    async fn mark_operation_completed(&self, operation_id: &str) -> Result<()>;

    /// Get storage size in bytes
    async fn size(&self) -> Result<u64>;

    /// Clear all data
    async fn clear(&self) -> Result<()>;

    /// Vacuum/optimize storage
    async fn vacuum(&self) -> Result<()>;

    /// Get version for entity
    async fn get_version(&self, entity_id: &str, entity_type: &str) -> Result<Option<Version>>;

    /// List all entity IDs of a type
    async fn list_entity_ids(&self, entity_type: &str) -> Result<Vec<String>>;

    /// Batch get records
    async fn batch_get(&self, ids: Vec<(String, String)>) -> Result<Vec<Option<StorageRecord>>>;

    /// Batch put records
    async fn batch_put(&self, records: Vec<StorageRecord>) -> Result<()>;

    /// Query records with filter
    async fn query(
        &self,
        entity_type: &str,
        filter: Option<String>,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<StorageRecord>>;
}

/// In-memory storage for testing
#[cfg(test)]
pub mod memory {
    use super::*;
    use parking_lot::RwLock;
    use std::collections::HashMap;

    pub struct MemoryStorage {
        records: RwLock<HashMap<(String, String), StorageRecord>>,
        operations: RwLock<HashMap<String, SyncOperation>>,
    }

    impl MemoryStorage {
        pub fn new() -> Self {
            Self {
                records: RwLock::new(HashMap::new()),
                operations: RwLock::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl Storage for MemoryStorage {
        async fn init(&self) -> Result<()> {
            Ok(())
        }

        async fn put(&self, record: StorageRecord) -> Result<()> {
            let key = (record.entity_id.clone(), record.entity_type.clone());
            self.records.write().insert(key, record);
            Ok(())
        }

        async fn get(&self, entity_id: &str, entity_type: &str) -> Result<Option<StorageRecord>> {
            let key = (entity_id.to_string(), entity_type.to_string());
            Ok(self.records.read().get(&key).cloned())
        }

        async fn delete(&self, entity_id: &str, entity_type: &str) -> Result<()> {
            let key = (entity_id.to_string(), entity_type.to_string());
            self.records.write().remove(&key);
            Ok(())
        }

        async fn list(&self, entity_type: &str, limit: Option<usize>) -> Result<Vec<StorageRecord>> {
            let records: Vec<_> = self.records.read()
                .values()
                .filter(|r| r.entity_type == entity_type)
                .cloned()
                .collect();

            if let Some(limit) = limit {
                Ok(records.into_iter().take(limit).collect())
            } else {
                Ok(records)
            }
        }

        async fn store_operation(&self, operation: SyncOperation) -> Result<()> {
            self.operations.write().insert(operation.id.clone(), operation);
            Ok(())
        }

        async fn get_pending_operations(&self) -> Result<Vec<SyncOperation>> {
            Ok(self.operations.read().values().cloned().collect())
        }

        async fn mark_operation_completed(&self, operation_id: &str) -> Result<()> {
            self.operations.write().remove(operation_id);
            Ok(())
        }

        async fn size(&self) -> Result<u64> {
            Ok(0)
        }

        async fn clear(&self) -> Result<()> {
            self.records.write().clear();
            self.operations.write().clear();
            Ok(())
        }

        async fn vacuum(&self) -> Result<()> {
            Ok(())
        }

        async fn get_version(&self, entity_id: &str, entity_type: &str) -> Result<Option<Version>> {
            Ok(self.get(entity_id, entity_type).await?.map(|r| r.version))
        }

        async fn list_entity_ids(&self, entity_type: &str) -> Result<Vec<String>> {
            Ok(self.records.read()
                .values()
                .filter(|r| r.entity_type == entity_type)
                .map(|r| r.entity_id.clone())
                .collect())
        }

        async fn batch_get(&self, ids: Vec<(String, String)>) -> Result<Vec<Option<StorageRecord>>> {
            let records = self.records.read();
            Ok(ids.into_iter()
                .map(|key| records.get(&key).cloned())
                .collect())
        }

        async fn batch_put(&self, records: Vec<StorageRecord>) -> Result<()> {
            let mut storage = self.records.write();
            for record in records {
                let key = (record.entity_id.clone(), record.entity_type.clone());
                storage.insert(key, record);
            }
            Ok(())
        }

        async fn query(
            &self,
            entity_type: &str,
            _filter: Option<String>,
            limit: Option<usize>,
            offset: Option<usize>,
        ) -> Result<Vec<StorageRecord>> {
            let mut records: Vec<_> = self.records.read()
                .values()
                .filter(|r| r.entity_type == entity_type)
                .cloned()
                .collect();

            records.sort_by(|a, b| a.created_at.cmp(&b.created_at));

            let offset = offset.unwrap_or(0);
            let records: Vec<_> = records.into_iter().skip(offset).collect();

            if let Some(limit) = limit {
                Ok(records.into_iter().take(limit).collect())
            } else {
                Ok(records)
            }
        }
    }
}
