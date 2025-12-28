use crate::error::{OfflineError, Result};
use crate::storage::{Storage, StorageRecord};
use crate::sync::SyncOperation;
use crate::versioning::Version;
use async_trait::async_trait;
use parking_lot::RwLock;
use rusqlite::{params, Connection, OpenFlags};
use std::path::Path;
use std::sync::Arc;

/// SQLite storage backend
pub struct SqliteStorage {
    conn: Arc<RwLock<Connection>>,
}

impl SqliteStorage {
    /// Create a new SQLite storage
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;

        Ok(Self {
            conn: Arc::new(RwLock::new(conn)),
        })
    }

    /// Create in-memory database
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        Ok(Self {
            conn: Arc::new(RwLock::new(conn)),
        })
    }

    /// Configure SQLite for optimal performance
    fn configure(&self) -> Result<()> {
        let conn = self.conn.write();

        // Enable WAL mode for better concurrency
        conn.execute("PRAGMA journal_mode=WAL", [])?;

        // Increase cache size
        conn.execute("PRAGMA cache_size=-10000", [])?; // 10MB cache

        // Normal synchronous mode for balance of safety and speed
        conn.execute("PRAGMA synchronous=NORMAL", [])?;

        // Enable foreign keys
        conn.execute("PRAGMA foreign_keys=ON", [])?;

        // Temp store in memory
        conn.execute("PRAGMA temp_store=MEMORY", [])?;

        Ok(())
    }

    /// Create database schema
    fn create_schema(&self) -> Result<()> {
        let conn = self.conn.write();

        // Records table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS records (
                entity_id TEXT NOT NULL,
                entity_type TEXT NOT NULL,
                data TEXT NOT NULL,
                version TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                deleted INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (entity_id, entity_type)
            )",
            [],
        )?;

        // Index for queries
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_records_type_created
             ON records(entity_type, created_at)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_records_type_updated
             ON records(entity_type, updated_at)",
            [],
        )?;

        // Pending operations table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS pending_operations (
                id TEXT PRIMARY KEY,
                entity_id TEXT NOT NULL,
                entity_type TEXT NOT NULL,
                operation_type TEXT NOT NULL,
                data TEXT NOT NULL,
                version TEXT NOT NULL,
                priority INTEGER NOT NULL,
                queued_at TEXT NOT NULL,
                retry_count INTEGER NOT NULL DEFAULT 0,
                last_error TEXT,
                dependencies TEXT,
                tags TEXT
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_operations_priority
             ON pending_operations(priority DESC, queued_at ASC)",
            [],
        )?;

        // Metadata table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS metadata (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        Ok(())
    }
}

#[async_trait]
impl Storage for SqliteStorage {
    async fn init(&self) -> Result<()> {
        self.configure()?;
        self.create_schema()?;
        Ok(())
    }

    async fn put(&self, record: StorageRecord) -> Result<()> {
        let conn = self.conn.write();

        let data_json = serde_json::to_string(&record.data)?;
        let version_json = serde_json::to_string(&record.version)?;

        conn.execute(
            "INSERT OR REPLACE INTO records
             (entity_id, entity_type, data, version, created_at, updated_at, deleted)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                record.entity_id,
                record.entity_type,
                data_json,
                version_json,
                record.created_at.to_rfc3339(),
                record.updated_at.to_rfc3339(),
                if record.deleted { 1 } else { 0 },
            ],
        )?;

        Ok(())
    }

    async fn get(&self, entity_id: &str, entity_type: &str) -> Result<Option<StorageRecord>> {
        let conn = self.conn.read();

        let mut stmt = conn.prepare(
            "SELECT entity_id, entity_type, data, version, created_at, updated_at, deleted
             FROM records
             WHERE entity_id = ?1 AND entity_type = ?2 AND deleted = 0",
        )?;

        let result = stmt.query_row(params![entity_id, entity_type], |row| {
            let data_json: String = row.get(2)?;
            let version_json: String = row.get(3)?;
            let created_at: String = row.get(4)?;
            let updated_at: String = row.get(5)?;

            Ok(StorageRecord {
                entity_id: row.get(0)?,
                entity_type: row.get(1)?,
                data: serde_json::from_str(&data_json).unwrap(),
                version: serde_json::from_str(&version_json).unwrap(),
                created_at: chrono::DateTime::parse_from_rfc3339(&created_at)
                    .unwrap()
                    .with_timezone(&chrono::Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at)
                    .unwrap()
                    .with_timezone(&chrono::Utc),
                deleted: false,
            })
        });

        match result {
            Ok(record) => Ok(Some(record)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn delete(&self, entity_id: &str, entity_type: &str) -> Result<()> {
        let conn = self.conn.write();

        conn.execute(
            "UPDATE records SET deleted = 1, updated_at = ?1
             WHERE entity_id = ?2 AND entity_type = ?3",
            params![chrono::Utc::now().to_rfc3339(), entity_id, entity_type],
        )?;

        Ok(())
    }

    async fn list(&self, entity_type: &str, limit: Option<usize>) -> Result<Vec<StorageRecord>> {
        let conn = self.conn.read();

        let limit_clause = limit.map(|l| format!(" LIMIT {}", l)).unwrap_or_default();

        let query = format!(
            "SELECT entity_id, entity_type, data, version, created_at, updated_at, deleted
             FROM records
             WHERE entity_type = ?1 AND deleted = 0
             ORDER BY created_at DESC{}",
            limit_clause
        );

        let mut stmt = conn.prepare(&query)?;

        let records = stmt
            .query_map(params![entity_type], |row| {
                let data_json: String = row.get(2)?;
                let version_json: String = row.get(3)?;
                let created_at: String = row.get(4)?;
                let updated_at: String = row.get(5)?;

                Ok(StorageRecord {
                    entity_id: row.get(0)?,
                    entity_type: row.get(1)?,
                    data: serde_json::from_str(&data_json).unwrap(),
                    version: serde_json::from_str(&version_json).unwrap(),
                    created_at: chrono::DateTime::parse_from_rfc3339(&created_at)
                        .unwrap()
                        .with_timezone(&chrono::Utc),
                    updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at)
                        .unwrap()
                        .with_timezone(&chrono::Utc),
                    deleted: false,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(records)
    }

    async fn store_operation(&self, operation: SyncOperation) -> Result<()> {
        let conn = self.conn.write();

        let data_json = serde_json::to_string(&operation.data)?;
        let version_json = serde_json::to_string(&operation.version)?;
        let dependencies_json = serde_json::to_string(&operation.dependencies)?;
        let tags_json = serde_json::to_string(&operation.tags)?;
        let operation_type = format!("{:?}", operation.operation_type);

        conn.execute(
            "INSERT OR REPLACE INTO pending_operations
             (id, entity_id, entity_type, operation_type, data, version, priority,
              queued_at, retry_count, last_error, dependencies, tags)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                operation.id,
                operation.entity_id,
                operation.entity_type,
                operation_type,
                data_json,
                version_json,
                operation.priority as i32,
                operation.queued_at.to_rfc3339(),
                operation.retry_count,
                operation.last_error,
                dependencies_json,
                tags_json,
            ],
        )?;

        Ok(())
    }

    async fn get_pending_operations(&self) -> Result<Vec<SyncOperation>> {
        let conn = self.conn.read();

        let mut stmt = conn.prepare(
            "SELECT id, entity_id, entity_type, operation_type, data, version, priority,
                    queued_at, retry_count, last_error, dependencies, tags
             FROM pending_operations
             ORDER BY priority DESC, queued_at ASC",
        )?;

        let operations = stmt
            .query_map([], |row| {
                let data_json: String = row.get(4)?;
                let version_json: String = row.get(5)?;
                let queued_at: String = row.get(7)?;
                let dependencies_json: String = row.get(10)?;
                let tags_json: String = row.get(11)?;

                // This is a simplified version - in production you'd properly parse operation_type
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    data_json,
                    version_json,
                    row.get::<_, i32>(6)?,
                    queued_at,
                    row.get::<_, u32>(8)?,
                    row.get::<_, Option<String>>(9)?,
                    dependencies_json,
                    tags_json,
                ))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        // Convert to SyncOperation (simplified)
        Ok(Vec::new()) // TODO: Implement proper deserialization
    }

    async fn mark_operation_completed(&self, operation_id: &str) -> Result<()> {
        let conn = self.conn.write();

        conn.execute(
            "DELETE FROM pending_operations WHERE id = ?1",
            params![operation_id],
        )?;

        Ok(())
    }

    async fn size(&self) -> Result<u64> {
        let conn = self.conn.read();

        let size: i64 = conn.query_row("PRAGMA page_count", [], |row| row.get(0))?;
        let page_size: i64 = conn.query_row("PRAGMA page_size", [], |row| row.get(0))?;

        Ok((size * page_size) as u64)
    }

    async fn clear(&self) -> Result<()> {
        let conn = self.conn.write();

        conn.execute("DELETE FROM records", [])?;
        conn.execute("DELETE FROM pending_operations", [])?;
        conn.execute("DELETE FROM metadata", [])?;

        Ok(())
    }

    async fn vacuum(&self) -> Result<()> {
        let conn = self.conn.write();
        conn.execute("VACUUM", [])?;
        Ok(())
    }

    async fn get_version(&self, entity_id: &str, entity_type: &str) -> Result<Option<Version>> {
        Ok(self.get(entity_id, entity_type).await?.map(|r| r.version))
    }

    async fn list_entity_ids(&self, entity_type: &str) -> Result<Vec<String>> {
        let conn = self.conn.read();

        let mut stmt = conn.prepare(
            "SELECT entity_id FROM records WHERE entity_type = ?1 AND deleted = 0",
        )?;

        let ids = stmt
            .query_map(params![entity_type], |row| row.get(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;

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
        let conn = self.conn.write();
        let tx = conn.unchecked_transaction()?;

        for record in records {
            let data_json = serde_json::to_string(&record.data)?;
            let version_json = serde_json::to_string(&record.version)?;

            tx.execute(
                "INSERT OR REPLACE INTO records
                 (entity_id, entity_type, data, version, created_at, updated_at, deleted)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    record.entity_id,
                    record.entity_type,
                    data_json,
                    version_json,
                    record.created_at.to_rfc3339(),
                    record.updated_at.to_rfc3339(),
                    if record.deleted { 1 } else { 0 },
                ],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    async fn query(
        &self,
        entity_type: &str,
        _filter: Option<String>,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<StorageRecord>> {
        let conn = self.conn.read();

        let limit_clause = limit.map(|l| format!(" LIMIT {}", l)).unwrap_or_default();
        let offset_clause = offset.map(|o| format!(" OFFSET {}", o)).unwrap_or_default();

        let query = format!(
            "SELECT entity_id, entity_type, data, version, created_at, updated_at, deleted
             FROM records
             WHERE entity_type = ?1 AND deleted = 0
             ORDER BY created_at DESC{}{}",
            limit_clause, offset_clause
        );

        let mut stmt = conn.prepare(&query)?;

        let records = stmt
            .query_map(params![entity_type], |row| {
                let data_json: String = row.get(2)?;
                let version_json: String = row.get(3)?;
                let created_at: String = row.get(4)?;
                let updated_at: String = row.get(5)?;

                Ok(StorageRecord {
                    entity_id: row.get(0)?,
                    entity_type: row.get(1)?,
                    data: serde_json::from_str(&data_json).unwrap(),
                    version: serde_json::from_str(&version_json).unwrap(),
                    created_at: chrono::DateTime::parse_from_rfc3339(&created_at)
                        .unwrap()
                        .with_timezone(&chrono::Utc),
                    updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at)
                        .unwrap()
                        .with_timezone(&chrono::Utc),
                    deleted: false,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(records)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::versioning::VectorClock;

    fn create_test_record() -> StorageRecord {
        let version = Version {
            clock: VectorClock::new(),
            node_id: "test-node".to_string(),
            timestamp: chrono::Utc::now(),
            content_hash: "test-hash".to_string(),
        };

        StorageRecord {
            entity_id: "entity-1".to_string(),
            entity_type: "accident".to_string(),
            data: serde_json::json!({"test": "data"}),
            version,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            deleted: false,
        }
    }

    #[tokio::test]
    async fn test_sqlite_storage() {
        let storage = SqliteStorage::in_memory().unwrap();
        storage.init().await.unwrap();

        let record = create_test_record();
        storage.put(record.clone()).await.unwrap();

        let retrieved = storage
            .get(&record.entity_id, &record.entity_type)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(retrieved.entity_id, record.entity_id);
        assert_eq!(retrieved.data, record.data);
    }
}
