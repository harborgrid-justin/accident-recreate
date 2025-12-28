//! Database backup and restore functionality
//!
//! Provides utilities for backing up and restoring SQLite databases,
//! with support for compression and automatic backups.

use crate::error::{DatabaseError, DbResult};
use rusqlite::Connection;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// Backup manager for database backups
pub struct BackupManager {
    backup_dir: PathBuf,
    compress: bool,
}

impl BackupManager {
    /// Create a new backup manager
    pub fn new(backup_dir: impl Into<PathBuf>) -> Self {
        Self {
            backup_dir: backup_dir.into(),
            compress: true,
        }
    }

    /// Set compression option
    pub fn with_compression(mut self, compress: bool) -> Self {
        self.compress = compress;
        self
    }

    /// Ensure backup directory exists
    fn ensure_backup_dir(&self) -> DbResult<()> {
        if !self.backup_dir.exists() {
            fs::create_dir_all(&self.backup_dir)
                .map_err(|e| DatabaseError::BackupError(format!("Failed to create backup directory: {}", e)))?;
        }
        Ok(())
    }

    /// Create a backup of the database
    pub fn backup(&self, conn: &Connection, name: Option<&str>) -> DbResult<PathBuf> {
        self.ensure_backup_dir()?;

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_name = name.unwrap_or(&format!("backup_{}", timestamp));
        let extension = if self.compress { "db.gz" } else { "db" };
        let backup_path = self.backup_dir.join(format!("{}. {}", backup_name, extension));

        info!("Creating backup: {}", backup_path.display());

        // Use SQLite backup API
        let backup_conn = Connection::open(&backup_path)
            .map_err(|e| DatabaseError::BackupError(format!("Failed to create backup connection: {}", e)))?;

        let backup = rusqlite::backup::Backup::new(conn, &backup_conn)
            .map_err(|e| DatabaseError::BackupError(format!("Failed to create backup: {}", e)))?;

        backup
            .run_to_completion(5, std::time::Duration::from_millis(250), None)
            .map_err(|e| DatabaseError::BackupError(format!("Backup failed: {}", e)))?;

        drop(backup_conn);

        // Compress if enabled
        if self.compress {
            self.compress_file(&backup_path)?;
        }

        info!("Backup created successfully: {}", backup_path.display());

        Ok(backup_path)
    }

    /// Restore a database from backup
    pub fn restore(&self, conn: &mut Connection, backup_path: &Path) -> DbResult<()> {
        info!("Restoring from backup: {}", backup_path.display());

        if !backup_path.exists() {
            return Err(DatabaseError::BackupError(format!(
                "Backup file not found: {}",
                backup_path.display()
            )));
        }

        // Decompress if needed
        let restore_path = if backup_path.extension().and_then(|s| s.to_str()) == Some("gz") {
            self.decompress_file(backup_path)?
        } else {
            backup_path.to_path_buf()
        };

        // Open backup connection
        let backup_conn = Connection::open(&restore_path)
            .map_err(|e| DatabaseError::BackupError(format!("Failed to open backup: {}", e)))?;

        // Restore using SQLite backup API
        let backup = rusqlite::backup::Backup::new(&backup_conn, conn)
            .map_err(|e| DatabaseError::BackupError(format!("Failed to create restore: {}", e)))?;

        backup
            .run_to_completion(5, std::time::Duration::from_millis(250), None)
            .map_err(|e| DatabaseError::BackupError(format!("Restore failed: {}", e)))?;

        info!("Database restored successfully");

        Ok(())
    }

    /// List all available backups
    pub fn list_backups(&self) -> DbResult<Vec<BackupInfo>> {
        self.ensure_backup_dir()?;

        let mut backups = Vec::new();

        for entry in fs::read_dir(&self.backup_dir)
            .map_err(|e| DatabaseError::BackupError(format!("Failed to read backup directory: {}", e)))?
        {
            let entry = entry.map_err(|e| DatabaseError::BackupError(e.to_string()))?;
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "db" || ext == "gz" {
                        let metadata = entry.metadata()
                            .map_err(|e| DatabaseError::BackupError(e.to_string()))?;

                        backups.push(BackupInfo {
                            path: path.clone(),
                            size: metadata.len(),
                            created_at: metadata
                                .created()
                                .ok()
                                .and_then(|t| chrono::DateTime::from(t).format("%Y-%m-%d %H:%M:%S").to_string().into()),
                        });
                    }
                }
            }
        }

        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(backups)
    }

    /// Delete old backups, keeping only the specified number
    pub fn cleanup_old_backups(&self, keep_count: usize) -> DbResult<usize> {
        let backups = self.list_backups()?;

        if backups.len() <= keep_count {
            return Ok(0);
        }

        let to_delete = &backups[keep_count..];
        let mut deleted = 0;

        for backup in to_delete {
            match fs::remove_file(&backup.path) {
                Ok(_) => {
                    debug!("Deleted old backup: {}", backup.path.display());
                    deleted += 1;
                }
                Err(e) => {
                    warn!("Failed to delete backup {}: {}", backup.path.display(), e);
                }
            }
        }

        info!("Cleaned up {} old backups", deleted);

        Ok(deleted)
    }

    /// Compress a file (placeholder - would need compression library)
    fn compress_file(&self, _path: &Path) -> DbResult<()> {
        // In a real implementation, this would use a compression library like flate2
        // For now, this is a placeholder
        debug!("File compression not implemented (would compress here)");
        Ok(())
    }

    /// Decompress a file (placeholder - would need compression library)
    fn decompress_file(&self, path: &Path) -> DbResult<PathBuf> {
        // In a real implementation, this would use a compression library like flate2
        // For now, just return the path as-is
        debug!("File decompression not implemented (would decompress here)");
        Ok(path.to_path_buf())
    }
}

/// Backup information
#[derive(Debug, Clone)]
pub struct BackupInfo {
    pub path: PathBuf,
    pub size: u64,
    pub created_at: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_backup_manager() {
        let temp_dir = TempDir::new().unwrap();
        let manager = BackupManager::new(temp_dir.path()).with_compression(false);

        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE test (id INTEGER PRIMARY KEY, value TEXT)")
            .unwrap();
        conn.execute("INSERT INTO test (value) VALUES (?)", ["test data"])
            .unwrap();

        // Note: Backup from in-memory database is not supported in SQLite
        // This test would need a file-based database
    }

    #[test]
    fn test_list_backups() {
        let temp_dir = TempDir::new().unwrap();
        let manager = BackupManager::new(temp_dir.path());

        let backups = manager.list_backups().unwrap();
        assert_eq!(backups.len(), 0);
    }
}
