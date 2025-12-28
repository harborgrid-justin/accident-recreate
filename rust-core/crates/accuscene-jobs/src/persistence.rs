//! Job state persistence for recovery and auditing.

use crate::error::Result;
use crate::job::{JobMetadata, JobState};
use crate::result::JobResult;
use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;

/// Job persistence layer
pub struct JobPersistence {
    conn: Arc<Mutex<Connection>>,
}

impl JobPersistence {
    /// Create a new job persistence layer
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        let persistence = Self {
            conn: Arc::new(Mutex::new(conn)),
        };

        persistence.initialize_schema()?;
        Ok(persistence)
    }

    /// Create an in-memory persistence layer for testing
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let persistence = Self {
            conn: Arc::new(Mutex::new(conn)),
        };

        persistence.initialize_schema()?;
        Ok(persistence)
    }

    /// Initialize database schema
    fn initialize_schema(&self) -> Result<()> {
        let conn = self.conn.lock();

        // Job metadata table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS job_metadata (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                state TEXT NOT NULL,
                priority INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                started_at TEXT,
                completed_at TEXT,
                attempt INTEGER NOT NULL,
                max_retries INTEGER NOT NULL,
                timeout_secs INTEGER,
                worker_id TEXT,
                tags TEXT
            )",
            [],
        )?;

        // Job results table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS job_results (
                job_id TEXT PRIMARY KEY,
                status TEXT NOT NULL,
                output TEXT NOT NULL,
                error TEXT,
                completed_at TEXT NOT NULL,
                duration_ms INTEGER NOT NULL,
                metadata TEXT
            )",
            [],
        )?;

        // Job history table for audit trail
        conn.execute(
            "CREATE TABLE IF NOT EXISTS job_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                job_id TEXT NOT NULL,
                event_type TEXT NOT NULL,
                event_data TEXT NOT NULL,
                timestamp TEXT NOT NULL
            )",
            [],
        )?;

        // Create indices
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_job_metadata_state ON job_metadata(state)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_job_history_job_id ON job_history(job_id)",
            [],
        )?;

        Ok(())
    }

    /// Save job metadata
    pub fn save_metadata(&self, metadata: &JobMetadata) -> Result<()> {
        let conn = self.conn.lock();

        let tags_json = serde_json::to_string(&metadata.tags)?;

        conn.execute(
            "INSERT OR REPLACE INTO job_metadata
             (id, name, state, priority, created_at, started_at, completed_at,
              attempt, max_retries, timeout_secs, worker_id, tags)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                metadata.id,
                metadata.name,
                format!("{:?}", metadata.state),
                metadata.priority,
                metadata.created_at.to_rfc3339(),
                metadata.started_at.map(|t| t.to_rfc3339()),
                metadata.completed_at.map(|t| t.to_rfc3339()),
                metadata.attempt,
                metadata.max_retries,
                metadata.timeout_secs,
                metadata.worker_id,
                tags_json,
            ],
        )?;

        Ok(())
    }

    /// Load job metadata
    pub fn load_metadata(&self, job_id: &str) -> Result<Option<JobMetadata>> {
        let conn = self.conn.lock();

        let result = conn.query_row(
            "SELECT id, name, state, priority, created_at, started_at, completed_at,
                    attempt, max_retries, timeout_secs, worker_id, tags
             FROM job_metadata WHERE id = ?1",
            params![job_id],
            |row| {
                let state_str: String = row.get(2)?;
                let state = match state_str.as_str() {
                    "Pending" => JobState::Pending,
                    "Queued" => JobState::Queued,
                    "Running" => JobState::Running,
                    "Completed" => JobState::Completed,
                    "Failed" => JobState::Failed,
                    "Cancelled" => JobState::Cancelled,
                    "Retrying" => JobState::Retrying,
                    _ => JobState::Pending,
                };

                let tags_json: String = row.get(11)?;
                let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();

                Ok(JobMetadata {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    state,
                    priority: row.get(3)?,
                    created_at: row.get::<_, String>(4)?.parse().unwrap(),
                    started_at: row.get::<_, Option<String>>(5)?.and_then(|s| s.parse().ok()),
                    completed_at: row.get::<_, Option<String>>(6)?.and_then(|s| s.parse().ok()),
                    attempt: row.get(7)?,
                    max_retries: row.get(8)?,
                    timeout_secs: row.get(9)?,
                    worker_id: row.get(10)?,
                    tags,
                })
            },
        );

        match result {
            Ok(metadata) => Ok(Some(metadata)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Save job result
    pub fn save_result(&self, result: &JobResult) -> Result<()> {
        let conn = self.conn.lock();

        conn.execute(
            "INSERT OR REPLACE INTO job_results
             (job_id, status, output, error, completed_at, duration_ms, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                result.job_id,
                format!("{:?}", result.status),
                serde_json::to_string(&result.output)?,
                result.error,
                result.completed_at.to_rfc3339(),
                result.duration_ms as i64,
                serde_json::to_string(&result.metadata)?,
            ],
        )?;

        Ok(())
    }

    /// Load job result
    pub fn load_result(&self, job_id: &str) -> Result<Option<JobResult>> {
        let conn = self.conn.lock();

        let result = conn.query_row(
            "SELECT job_id, status, output, error, completed_at, duration_ms, metadata
             FROM job_results WHERE job_id = ?1",
            params![job_id],
            |row| {
                let status_str: String = row.get(1)?;
                let status = match status_str.as_str() {
                    "Success" => crate::result::JobResultStatus::Success,
                    "Failure" => crate::result::JobResultStatus::Failure,
                    "Partial" => crate::result::JobResultStatus::Partial,
                    _ => crate::result::JobResultStatus::Failure,
                };

                Ok(JobResult {
                    job_id: row.get(0)?,
                    status,
                    output: serde_json::from_str(&row.get::<_, String>(2)?).unwrap(),
                    error: row.get(3)?,
                    completed_at: row.get::<_, String>(4)?.parse().unwrap(),
                    duration_ms: row.get::<_, i64>(5)? as u64,
                    metadata: serde_json::from_str(&row.get::<_, String>(6)?).unwrap(),
                })
            },
        );

        match result {
            Ok(job_result) => Ok(Some(job_result)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Record job event in history
    pub fn record_event(&self, job_id: &str, event_type: JobEvent, event_data: serde_json::Value) -> Result<()> {
        let conn = self.conn.lock();

        conn.execute(
            "INSERT INTO job_history (job_id, event_type, event_data, timestamp)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                job_id,
                format!("{:?}", event_type),
                serde_json::to_string(&event_data)?,
                Utc::now().to_rfc3339(),
            ],
        )?;

        Ok(())
    }

    /// Get job history
    pub fn get_history(&self, job_id: &str) -> Result<Vec<JobHistoryEntry>> {
        let conn = self.conn.lock();

        let mut stmt = conn.prepare(
            "SELECT id, job_id, event_type, event_data, timestamp
             FROM job_history WHERE job_id = ?1
             ORDER BY timestamp ASC",
        )?;

        let entries = stmt
            .query_map(params![job_id], |row| {
                Ok(JobHistoryEntry {
                    id: row.get(0)?,
                    job_id: row.get(1)?,
                    event_type: row.get(2)?,
                    event_data: row.get(3)?,
                    timestamp: row.get::<_, String>(4)?.parse().unwrap(),
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(entries)
    }

    /// Get all jobs with a specific state
    pub fn get_jobs_by_state(&self, state: JobState) -> Result<Vec<JobMetadata>> {
        let conn = self.conn.lock();

        let mut stmt = conn.prepare(
            "SELECT id, name, state, priority, created_at, started_at, completed_at,
                    attempt, max_retries, timeout_secs, worker_id, tags
             FROM job_metadata WHERE state = ?1",
        )?;

        let jobs = stmt
            .query_map(params![format!("{:?}", state)], |row| {
                let tags_json: String = row.get(11)?;
                let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();

                Ok(JobMetadata {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    state,
                    priority: row.get(3)?,
                    created_at: row.get::<_, String>(4)?.parse().unwrap(),
                    started_at: row.get::<_, Option<String>>(5)?.and_then(|s| s.parse().ok()),
                    completed_at: row.get::<_, Option<String>>(6)?.and_then(|s| s.parse().ok()),
                    attempt: row.get(7)?,
                    max_retries: row.get(8)?,
                    timeout_secs: row.get(9)?,
                    worker_id: row.get(10)?,
                    tags,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(jobs)
    }

    /// Delete old completed jobs
    pub fn cleanup_old_jobs(&self, older_than_days: i64) -> Result<usize> {
        let conn = self.conn.lock();

        let cutoff = (Utc::now() - chrono::Duration::days(older_than_days)).to_rfc3339();

        let deleted = conn.execute(
            "DELETE FROM job_metadata
             WHERE (state = 'Completed' OR state = 'Failed' OR state = 'Cancelled')
             AND completed_at < ?1",
            params![cutoff],
        )?;

        Ok(deleted)
    }
}

/// Job event types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobEvent {
    Created,
    Queued,
    Started,
    Completed,
    Failed,
    Retrying,
    Cancelled,
}

/// Job history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobHistoryEntry {
    pub id: i64,
    pub job_id: String,
    pub event_type: String,
    pub event_data: String,
    pub timestamp: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_persistence_basic() {
        let persistence = JobPersistence::in_memory().unwrap();

        let metadata = JobMetadata::new("job-1".to_string(), "test_job".to_string());
        persistence.save_metadata(&metadata).unwrap();

        let loaded = persistence.load_metadata("job-1").unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().id, "job-1");
    }

    #[test]
    fn test_persistence_result() {
        let persistence = JobPersistence::in_memory().unwrap();

        let result = JobResult::success("job-1".to_string(), serde_json::json!({"test": true}));
        persistence.save_result(&result).unwrap();

        let loaded = persistence.load_result("job-1").unwrap();
        assert!(loaded.is_some());
        assert!(loaded.unwrap().is_success());
    }

    #[test]
    fn test_job_history() {
        let persistence = JobPersistence::in_memory().unwrap();

        persistence
            .record_event("job-1", JobEvent::Created, serde_json::json!({}))
            .unwrap();

        persistence
            .record_event("job-1", JobEvent::Started, serde_json::json!({}))
            .unwrap();

        let history = persistence.get_history("job-1").unwrap();
        assert_eq!(history.len(), 2);
    }
}
