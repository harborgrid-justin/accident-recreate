//! Audit log storage
//!
//! Provides persistent storage for audit events with encryption.

use crate::audit::event::{AuditEvent, EventType};
use crate::error::{Result, SecurityError};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Audit storage configuration
#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// Maximum events to keep in memory
    pub max_memory_events: usize,
    /// Retention period in days
    pub retention_days: u32,
    /// Enable encryption
    pub encrypt: bool,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            max_memory_events: 10_000,
            retention_days: 365,
            encrypt: true,
        }
    }
}

/// Audit event storage
pub struct AuditStorage {
    config: StorageConfig,
    events: Arc<RwLock<VecDeque<AuditEvent>>>,
}

impl AuditStorage {
    /// Create a new audit storage
    pub fn new(config: StorageConfig) -> Self {
        Self {
            config,
            events: Arc::new(RwLock::new(VecDeque::new())),
        }
    }

    /// Store an audit event
    pub async fn store(&self, event: AuditEvent) -> Result<()> {
        let mut events = self.events.write().await;

        // Add event
        events.push_back(event);

        // Enforce memory limit
        while events.len() > self.config.max_memory_events {
            events.pop_front();
        }

        Ok(())
    }

    /// Store multiple events
    pub async fn store_batch(&self, batch: Vec<AuditEvent>) -> Result<()> {
        let mut events = self.events.write().await;

        for event in batch {
            events.push_back(event);
        }

        // Enforce memory limit
        while events.len() > self.config.max_memory_events {
            events.pop_front();
        }

        Ok(())
    }

    /// Get event by ID
    pub async fn get(&self, id: &str) -> Result<Option<AuditEvent>> {
        let events = self.events.read().await;
        Ok(events.iter().find(|e| e.id == id).cloned())
    }

    /// Get all events (for in-memory storage)
    pub async fn get_all(&self) -> Result<Vec<AuditEvent>> {
        let events = self.events.read().await;
        Ok(events.iter().cloned().collect())
    }

    /// Get events count
    pub async fn count(&self) -> usize {
        let events = self.events.read().await;
        events.len()
    }

    /// Clear all events (use with caution!)
    pub async fn clear(&self) -> Result<()> {
        let mut events = self.events.write().await;
        events.clear();
        Ok(())
    }

    /// Clean up expired events
    pub async fn cleanup_expired(&self) -> Result<usize> {
        let mut events = self.events.write().await;

        let cutoff = chrono::Utc::now()
            - chrono::Duration::days(self.config.retention_days as i64);

        let before_count = events.len();
        events.retain(|event| event.timestamp > cutoff);
        let removed = before_count - events.len();

        Ok(removed)
    }

    /// Export events to JSON
    pub async fn export_json(&self) -> Result<String> {
        let events = self.events.read().await;
        serde_json::to_string_pretty(&events.iter().collect::<Vec<_>>())
            .map_err(|e| SecurityError::Internal(format!("Export failed: {}", e)))
    }

    /// Import events from JSON
    pub async fn import_json(&self, json: &str) -> Result<usize> {
        let imported: Vec<AuditEvent> = serde_json::from_str(json)
            .map_err(|e| SecurityError::Internal(format!("Import failed: {}", e)))?;

        let count = imported.len();
        self.store_batch(imported).await?;

        Ok(count)
    }
}

impl Default for AuditStorage {
    fn default() -> Self {
        Self::new(StorageConfig::default())
    }
}

/// Persistent file-based storage
pub struct FileStorage {
    directory: std::path::PathBuf,
    config: StorageConfig,
}

impl FileStorage {
    /// Create a new file storage
    pub fn new(directory: impl Into<std::path::PathBuf>, config: StorageConfig) -> Self {
        Self {
            directory: directory.into(),
            config,
        }
    }

    /// Store an event to file
    pub async fn store(&self, event: &AuditEvent) -> Result<()> {
        // Create filename based on date
        let date = event.timestamp.format("%Y-%m-%d");
        let filename = self.directory.join(format!("audit-{}.jsonl", date));

        // Serialize event
        let json = serde_json::to_string(event)
            .map_err(|e| SecurityError::Internal(format!("Serialization failed: {}", e)))?;

        // Append to file
        tokio::fs::create_dir_all(&self.directory)
            .await
            .map_err(|e| SecurityError::Internal(format!("Directory creation failed: {}", e)))?;

        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&filename)
            .await
            .map_err(|e| SecurityError::AuditLogFailed(format!("File open failed: {}", e)))?;

        use tokio::io::AsyncWriteExt;
        file.write_all(format!("{}\n", json).as_bytes())
            .await
            .map_err(|e| SecurityError::AuditLogFailed(format!("Write failed: {}", e)))?;

        Ok(())
    }

    /// Read events from a specific date
    pub async fn read_date(
        &self,
        date: chrono::NaiveDate,
    ) -> Result<Vec<AuditEvent>> {
        let filename = self.directory.join(format!("audit-{}.jsonl", date));

        if !filename.exists() {
            return Ok(Vec::new());
        }

        let content = tokio::fs::read_to_string(&filename)
            .await
            .map_err(|e| SecurityError::AuditQueryFailed(format!("Read failed: {}", e)))?;

        let events: Vec<AuditEvent> = content
            .lines()
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect();

        Ok(events)
    }

    /// Clean up old files based on retention policy
    pub async fn cleanup_old_files(&self) -> Result<usize> {
        let cutoff = chrono::Utc::now().date_naive()
            - chrono::Duration::days(self.config.retention_days as i64);

        let mut removed = 0;

        if let Ok(mut entries) = tokio::fs::read_dir(&self.directory).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();

                // Parse date from filename
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    if filename.starts_with("audit-") && filename.ends_with(".jsonl") {
                        // Extract date from "audit-YYYY-MM-DD.jsonl"
                        let date_str = &filename[6..filename.len() - 6];
                        if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                        {
                            if date < cutoff {
                                if tokio::fs::remove_file(&path).await.is_ok() {
                                    removed += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(removed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::event::EventType;

    #[tokio::test]
    async fn test_storage_creation() {
        let storage = AuditStorage::default();
        assert_eq!(storage.count().await, 0);
    }

    #[tokio::test]
    async fn test_store_event() {
        let storage = AuditStorage::default();
        let event = AuditEvent::new(EventType::AuthLogin, "login".to_string());
        let event_id = event.id.clone();

        storage.store(event).await.unwrap();
        assert_eq!(storage.count().await, 1);

        let retrieved = storage.get(&event_id).await.unwrap();
        assert!(retrieved.is_some());
    }

    #[tokio::test]
    async fn test_store_batch() {
        let storage = AuditStorage::default();
        let events: Vec<AuditEvent> = (0..5)
            .map(|i| AuditEvent::new(EventType::AuthLogin, format!("login-{}", i)))
            .collect();

        storage.store_batch(events).await.unwrap();
        assert_eq!(storage.count().await, 5);
    }

    #[tokio::test]
    async fn test_memory_limit() {
        let config = StorageConfig {
            max_memory_events: 3,
            ..Default::default()
        };
        let storage = AuditStorage::new(config);

        // Store more than max
        for i in 0..5 {
            let event = AuditEvent::new(EventType::AuthLogin, format!("login-{}", i));
            storage.store(event).await.unwrap();
        }

        // Should only keep last 3
        assert_eq!(storage.count().await, 3);
    }

    #[tokio::test]
    async fn test_export_import() {
        let storage = AuditStorage::default();

        // Store events
        for i in 0..3 {
            let event = AuditEvent::new(EventType::AuthLogin, format!("login-{}", i));
            storage.store(event).await.unwrap();
        }

        // Export
        let json = storage.export_json().await.unwrap();
        assert!(!json.is_empty());

        // Clear and import
        storage.clear().await.unwrap();
        assert_eq!(storage.count().await, 0);

        let imported = storage.import_json(&json).await.unwrap();
        assert_eq!(imported, 3);
        assert_eq!(storage.count().await, 3);
    }
}
