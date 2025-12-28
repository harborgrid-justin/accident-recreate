//! Audit logging for database operations
//!
//! Provides comprehensive audit trail for all database changes,
//! including who made the change, when, and what was changed.

use crate::error::{DatabaseError, DbResult};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, warn};
use uuid::Uuid;

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub action: AuditAction,
    pub user_id: Option<String>,
    pub user_email: Option<String>,
    pub changes: Option<HashMap<String, ChangeValue>>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub timestamp: String,
}

/// Audit action types
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AuditAction {
    Create,
    Update,
    Delete,
    Read,
    Login,
    Logout,
    Export,
    Import,
    Custom,
}

impl AuditAction {
    pub fn as_str(&self) -> &str {
        match self {
            AuditAction::Create => "CREATE",
            AuditAction::Update => "UPDATE",
            AuditAction::Delete => "DELETE",
            AuditAction::Read => "READ",
            AuditAction::Login => "LOGIN",
            AuditAction::Logout => "LOGOUT",
            AuditAction::Export => "EXPORT",
            AuditAction::Import => "IMPORT",
            AuditAction::Custom => "CUSTOM",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "CREATE" => AuditAction::Create,
            "UPDATE" => AuditAction::Update,
            "DELETE" => AuditAction::Delete,
            "READ" => AuditAction::Read,
            "LOGIN" => AuditAction::Login,
            "LOGOUT" => AuditAction::Logout,
            "EXPORT" => AuditAction::Export,
            "IMPORT" => AuditAction::Import,
            _ => AuditAction::Custom,
        }
    }
}

/// Change value for audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeValue {
    pub old_value: Option<serde_json::Value>,
    pub new_value: Option<serde_json::Value>,
}

/// Audit logger for database operations
pub struct AuditLogger {
    enabled: bool,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new() -> Self {
        Self { enabled: true }
    }

    /// Create a disabled audit logger
    pub fn disabled() -> Self {
        Self { enabled: false }
    }

    /// Enable or disable audit logging
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Log an audit entry
    pub fn log(
        &self,
        conn: &Connection,
        entity_type: &str,
        entity_id: &str,
        action: AuditAction,
        user_id: Option<&str>,
        user_email: Option<&str>,
        changes: Option<HashMap<String, ChangeValue>>,
    ) -> DbResult<()> {
        if !self.enabled {
            return Ok(());
        }

        debug!(
            "Logging audit entry: {} {} on {}:{}",
            action.as_str(),
            entity_type,
            entity_id,
            user_id.unwrap_or("unknown")
        );

        let entry = AuditEntry {
            id: Uuid::new_v4().to_string(),
            entity_type: entity_type.to_string(),
            entity_id: entity_id.to_string(),
            action,
            user_id: user_id.map(|s| s.to_string()),
            user_email: user_email.map(|s| s.to_string()),
            changes,
            ip_address: None,
            user_agent: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        self.write_entry(conn, &entry)?;

        Ok(())
    }

    /// Log a creation event
    pub fn log_create(
        &self,
        conn: &Connection,
        entity_type: &str,
        entity_id: &str,
        user_id: Option<&str>,
        user_email: Option<&str>,
    ) -> DbResult<()> {
        self.log(
            conn,
            entity_type,
            entity_id,
            AuditAction::Create,
            user_id,
            user_email,
            None,
        )
    }

    /// Log an update event
    pub fn log_update(
        &self,
        conn: &Connection,
        entity_type: &str,
        entity_id: &str,
        user_id: Option<&str>,
        user_email: Option<&str>,
        changes: HashMap<String, ChangeValue>,
    ) -> DbResult<()> {
        self.log(
            conn,
            entity_type,
            entity_id,
            AuditAction::Update,
            user_id,
            user_email,
            Some(changes),
        )
    }

    /// Log a deletion event
    pub fn log_delete(
        &self,
        conn: &Connection,
        entity_type: &str,
        entity_id: &str,
        user_id: Option<&str>,
        user_email: Option<&str>,
    ) -> DbResult<()> {
        self.log(
            conn,
            entity_type,
            entity_id,
            AuditAction::Delete,
            user_id,
            user_email,
            None,
        )
    }

    /// Write audit entry to database
    fn write_entry(&self, conn: &Connection, entry: &AuditEntry) -> DbResult<()> {
        let changes_json = entry
            .changes
            .as_ref()
            .map(|c| serde_json::to_string(c).ok())
            .flatten();

        conn.execute(
            "INSERT INTO audit_log (id, entity_type, entity_id, action, user_id, user_email,
                                   changes, ip_address, user_agent)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                entry.id,
                entry.entity_type,
                entry.entity_id,
                entry.action.as_str(),
                entry.user_id,
                entry.user_email,
                changes_json,
                entry.ip_address,
                entry.user_agent,
            ],
        )
        .map_err(|e| {
            warn!("Failed to write audit entry: {}", e);
            DatabaseError::AuditError(format!("Failed to write audit entry: {}", e))
        })?;

        Ok(())
    }

    /// Query audit logs by entity
    pub fn query_by_entity(
        &self,
        conn: &Connection,
        entity_type: &str,
        entity_id: &str,
    ) -> DbResult<Vec<AuditEntry>> {
        let mut stmt = conn.prepare(
            "SELECT id, entity_type, entity_id, action, user_id, user_email,
                    changes, ip_address, user_agent, timestamp
             FROM audit_log
             WHERE entity_type = ? AND entity_id = ?
             ORDER BY timestamp DESC",
        )?;

        let entries = stmt
            .query_map(params![entity_type, entity_id], |row| {
                let changes_json: Option<String> = row.get(6)?;
                let changes = changes_json.and_then(|s| serde_json::from_str(&s).ok());

                Ok(AuditEntry {
                    id: row.get(0)?,
                    entity_type: row.get(1)?,
                    entity_id: row.get(2)?,
                    action: AuditAction::from_str(&row.get::<_, String>(3)?),
                    user_id: row.get(4)?,
                    user_email: row.get(5)?,
                    changes,
                    ip_address: row.get(7)?,
                    user_agent: row.get(8)?,
                    timestamp: row.get(9)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(entries)
    }

    /// Query audit logs by user
    pub fn query_by_user(&self, conn: &Connection, user_id: &str) -> DbResult<Vec<AuditEntry>> {
        let mut stmt = conn.prepare(
            "SELECT id, entity_type, entity_id, action, user_id, user_email,
                    changes, ip_address, user_agent, timestamp
             FROM audit_log
             WHERE user_id = ?
             ORDER BY timestamp DESC",
        )?;

        let entries = stmt
            .query_map([user_id], |row| {
                let changes_json: Option<String> = row.get(6)?;
                let changes = changes_json.and_then(|s| serde_json::from_str(&s).ok());

                Ok(AuditEntry {
                    id: row.get(0)?,
                    entity_type: row.get(1)?,
                    entity_id: row.get(2)?,
                    action: AuditAction::from_str(&row.get::<_, String>(3)?),
                    user_id: row.get(4)?,
                    user_email: row.get(5)?,
                    changes,
                    ip_address: row.get(7)?,
                    user_agent: row.get(8)?,
                    timestamp: row.get(9)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(entries)
    }

    /// Query audit logs by action
    pub fn query_by_action(
        &self,
        conn: &Connection,
        action: AuditAction,
    ) -> DbResult<Vec<AuditEntry>> {
        let mut stmt = conn.prepare(
            "SELECT id, entity_type, entity_id, action, user_id, user_email,
                    changes, ip_address, user_agent, timestamp
             FROM audit_log
             WHERE action = ?
             ORDER BY timestamp DESC",
        )?;

        let entries = stmt
            .query_map([action.as_str()], |row| {
                let changes_json: Option<String> = row.get(6)?;
                let changes = changes_json.and_then(|s| serde_json::from_str(&s).ok());

                Ok(AuditEntry {
                    id: row.get(0)?,
                    entity_type: row.get(1)?,
                    entity_id: row.get(2)?,
                    action: AuditAction::from_str(&row.get::<_, String>(3)?),
                    user_id: row.get(4)?,
                    user_email: row.get(5)?,
                    changes,
                    ip_address: row.get(7)?,
                    user_agent: row.get(8)?,
                    timestamp: row.get(9)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(entries)
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE audit_log (
                id TEXT PRIMARY KEY,
                entity_type TEXT NOT NULL,
                entity_id TEXT NOT NULL,
                action TEXT NOT NULL,
                user_id TEXT,
                user_email TEXT,
                changes TEXT,
                ip_address TEXT,
                user_agent TEXT,
                timestamp TEXT NOT NULL DEFAULT (datetime('now'))
            )",
        )
        .unwrap();
        conn
    }

    #[test]
    fn test_audit_logger() {
        let conn = setup_db();
        let logger = AuditLogger::new();

        logger
            .log_create(
                &conn,
                "Case",
                "case-123",
                Some("user-1"),
                Some("user@example.com"),
            )
            .unwrap();

        let entries = logger.query_by_entity(&conn, "Case", "case-123").unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].entity_type, "Case");
        assert_eq!(entries[0].entity_id, "case-123");
    }

    #[test]
    fn test_audit_logger_disabled() {
        let conn = setup_db();
        let logger = AuditLogger::disabled();

        logger
            .log_create(&conn, "Case", "case-123", Some("user-1"), None)
            .unwrap();

        let entries = logger.query_by_entity(&conn, "Case", "case-123").unwrap();
        assert_eq!(entries.len(), 0);
    }
}
