//! SQLite-backed persistent job queue implementation.

use crate::error::{JobError, Result};
use crate::job::Job;
use crate::queue::{JobQueue, QueueConfig};
use async_trait::async_trait;
use chrono::Utc;
use parking_lot::Mutex;
use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::Arc;

/// Persistent job queue using SQLite
#[derive(Clone)]
pub struct PersistentQueue {
    conn: Arc<Mutex<Connection>>,
    config: QueueConfig,
}

impl PersistentQueue {
    /// Create a new persistent queue
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        Self::with_config(db_path, QueueConfig::default().with_persistence())
    }

    /// Create a new persistent queue with configuration
    pub fn with_config<P: AsRef<Path>>(db_path: P, config: QueueConfig) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        let queue = Self {
            conn: Arc::new(Mutex::new(conn)),
            config,
        };

        queue.initialize_schema()?;
        Ok(queue)
    }

    /// Initialize database schema
    fn initialize_schema(&self) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS jobs (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                priority INTEGER NOT NULL DEFAULT 0,
                data TEXT NOT NULL,
                state TEXT NOT NULL DEFAULT 'pending',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_jobs_priority ON jobs(priority DESC, created_at ASC)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_jobs_state ON jobs(state)",
            [],
        )?;

        Ok(())
    }

    /// Create an in-memory persistent queue for testing
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let queue = Self {
            conn: Arc::new(Mutex::new(conn)),
            config: QueueConfig::default().with_persistence(),
        };

        queue.initialize_schema()?;
        Ok(queue)
    }
}

#[async_trait]
impl JobQueue for PersistentQueue {
    async fn push(&self, job: Box<dyn Job>) -> Result<()> {
        let job_id = job.id().to_string();
        let job_name = job.name().to_string();
        let priority = job.priority();
        let data = job.serialize()?;
        let now = Utc::now().to_rfc3339();

        let conn = self.conn.lock();

        // Check capacity
        if let Some(max_size) = self.config.max_size {
            let count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM jobs WHERE state = 'pending' OR state = 'queued'",
                [],
                |row| row.get(0),
            )?;

            if count >= max_size as i64 {
                return Err(JobError::QueueFull {
                    capacity: max_size,
                });
            }
        }

        conn.execute(
            "INSERT INTO jobs (id, name, priority, data, state, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, 'queued', ?5, ?6)",
            params![job_id, job_name, priority, data, now, now],
        )?;

        tracing::debug!(job_id = %job_id, "Job persisted to database queue");
        Ok(())
    }

    async fn pop(&self) -> Result<Option<Box<dyn Job>>> {
        let conn = self.conn.lock();

        // Get highest priority job
        let result = conn.query_row(
            "SELECT id, name, data FROM jobs
             WHERE state = 'queued'
             ORDER BY priority DESC, created_at ASC
             LIMIT 1",
            [],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            },
        );

        match result {
            Ok((job_id, job_name, data)) => {
                // Mark as running or delete
                conn.execute("DELETE FROM jobs WHERE id = ?1", params![job_id])?;

                // Deserialize job
                let job: Box<dyn Job> = serde_json::from_str(&data)?;

                tracing::debug!(job_id = %job_id, "Job popped from persistent queue");
                Ok(Some(job))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn peek(&self) -> Result<Option<Box<dyn Job>>> {
        let conn = self.conn.lock();

        let result = conn.query_row(
            "SELECT data FROM jobs
             WHERE state = 'queued'
             ORDER BY priority DESC, created_at ASC
             LIMIT 1",
            [],
            |row| row.get::<_, String>(0),
        );

        match result {
            Ok(data) => {
                let job: Box<dyn Job> = serde_json::from_str(&data)?;
                Ok(Some(job))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn len(&self) -> Result<usize> {
        let conn = self.conn.lock();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM jobs WHERE state = 'queued'",
            [],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }

    async fn clear(&self) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM jobs WHERE state = 'queued'", [])?;
        tracing::info!("Persistent queue cleared");
        Ok(())
    }

    async fn get(&self, job_id: &str) -> Result<Option<Box<dyn Job>>> {
        let conn = self.conn.lock();

        let result = conn.query_row(
            "SELECT data FROM jobs WHERE id = ?1",
            params![job_id],
            |row| row.get::<_, String>(0),
        );

        match result {
            Ok(data) => {
                let job: Box<dyn Job> = serde_json::from_str(&data)?;
                Ok(Some(job))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn remove(&self, job_id: &str) -> Result<bool> {
        let conn = self.conn.lock();
        let affected = conn.execute("DELETE FROM jobs WHERE id = ?1", params![job_id])?;

        if affected > 0 {
            tracing::debug!(job_id = %job_id, "Job removed from persistent queue");
        }

        Ok(affected > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::job::PhysicsSimulationJob;

    #[tokio::test]
    async fn test_persistent_queue_push_pop() {
        let queue = PersistentQueue::in_memory().unwrap();
        let job = Box::new(PhysicsSimulationJob::new(
            "test-scenario".to_string(),
            serde_json::json!({}),
        ));
        let job_id = job.id().to_string();

        queue.push(job).await.unwrap();
        assert_eq!(queue.len().await.unwrap(), 1);

        let popped = queue.pop().await.unwrap();
        assert!(popped.is_some());
        assert_eq!(queue.len().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_persistent_queue_priority() {
        let queue = PersistentQueue::in_memory().unwrap();

        let mut job1 = PhysicsSimulationJob::new("scenario-1".to_string(), serde_json::json!({}));
        job1.metadata.priority = 1;

        let mut job2 = PhysicsSimulationJob::new("scenario-2".to_string(), serde_json::json!({}));
        job2.metadata.priority = 10;

        queue.push(Box::new(job1)).await.unwrap();
        queue.push(Box::new(job2.clone())).await.unwrap();

        let popped = queue.pop().await.unwrap().unwrap();
        // Should get job2 first due to higher priority
        assert_eq!(popped.id(), job2.id());
    }

    #[tokio::test]
    async fn test_persistent_queue_persistence() {
        use tempfile::NamedTempFile;

        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path();

        {
            let queue = PersistentQueue::new(db_path).unwrap();
            let job = Box::new(PhysicsSimulationJob::new(
                "test-scenario".to_string(),
                serde_json::json!({}),
            ));
            queue.push(job).await.unwrap();
        }

        // Reopen queue
        {
            let queue = PersistentQueue::new(db_path).unwrap();
            assert_eq!(queue.len().await.unwrap(), 1);
        }
    }
}
