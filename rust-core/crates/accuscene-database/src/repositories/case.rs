//! Case repository for managing investigation cases
//!
//! Provides CRUD operations and specialized queries for cases.

use crate::error::{DatabaseError, DbResult};
use crate::repositories::Repository;
use rusqlite::{params, Connection, Row};
use serde::{Deserialize, Serialize};

/// Case entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Case {
    pub id: String,
    pub case_number: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub assigned_to: Option<String>,
    pub created_by: String,
    pub organization: Option<String>,
    pub tags: Option<Vec<String>>,
    pub metadata: Option<serde_json::Value>,
    pub closed_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl Case {
    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        let tags_json: Option<String> = row.get(9)?;
        let tags = tags_json.and_then(|s| serde_json::from_str(&s).ok());

        let metadata_json: Option<String> = row.get(10)?;
        let metadata = metadata_json.and_then(|s| serde_json::from_str(&s).ok());

        Ok(Self {
            id: row.get(0)?,
            case_number: row.get(1)?,
            title: row.get(2)?,
            description: row.get(3)?,
            status: row.get(4)?,
            priority: row.get(5)?,
            assigned_to: row.get(6)?,
            created_by: row.get(7)?,
            organization: row.get(8)?,
            tags,
            metadata,
            closed_at: row.get(11)?,
            created_at: row.get(12)?,
            updated_at: row.get(13)?,
        })
    }
}

/// Case repository
pub struct CaseRepository;

impl CaseRepository {
    pub fn new() -> Self {
        Self
    }

    /// Find cases by status
    pub fn find_by_status(&self, conn: &Connection, status: &str) -> DbResult<Vec<Case>> {
        let mut stmt = conn.prepare(
            "SELECT id, case_number, title, description, status, priority, assigned_to,
                    created_by, organization, tags, metadata, closed_at, created_at, updated_at
             FROM cases WHERE status = ? ORDER BY created_at DESC",
        )?;

        let cases = stmt
            .query_map([status], Case::from_row)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(cases)
    }

    /// Find cases assigned to a user
    pub fn find_by_assigned_to(&self, conn: &Connection, user_id: &str) -> DbResult<Vec<Case>> {
        let mut stmt = conn.prepare(
            "SELECT id, case_number, title, description, status, priority, assigned_to,
                    created_by, organization, tags, metadata, closed_at, created_at, updated_at
             FROM cases WHERE assigned_to = ? ORDER BY created_at DESC",
        )?;

        let cases = stmt
            .query_map([user_id], Case::from_row)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(cases)
    }

    /// Find cases by organization
    pub fn find_by_organization(&self, conn: &Connection, organization: &str) -> DbResult<Vec<Case>> {
        let mut stmt = conn.prepare(
            "SELECT id, case_number, title, description, status, priority, assigned_to,
                    created_by, organization, tags, metadata, closed_at, created_at, updated_at
             FROM cases WHERE organization = ? ORDER BY created_at DESC",
        )?;

        let cases = stmt
            .query_map([organization], Case::from_row)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(cases)
    }

    /// Find case by case number
    pub fn find_by_case_number(&self, conn: &Connection, case_number: &str) -> DbResult<Option<Case>> {
        let mut stmt = conn.prepare(
            "SELECT id, case_number, title, description, status, priority, assigned_to,
                    created_by, organization, tags, metadata, closed_at, created_at, updated_at
             FROM cases WHERE case_number = ?",
        )?;

        let mut rows = stmt.query_map([case_number], Case::from_row)?;

        if let Some(row) = rows.next() {
            Ok(Some(row?))
        } else {
            Ok(None)
        }
    }

    /// Close a case
    pub fn close_case(&self, conn: &Connection, id: &str) -> DbResult<()> {
        let affected = conn.execute(
            "UPDATE cases SET status = 'closed', closed_at = datetime('now') WHERE id = ?",
            [id],
        )?;

        if affected == 0 {
            Err(DatabaseError::not_found("Case", "id", id))
        } else {
            Ok(())
        }
    }

    /// Reopen a case
    pub fn reopen_case(&self, conn: &Connection, id: &str) -> DbResult<()> {
        let affected = conn.execute(
            "UPDATE cases SET status = 'open', closed_at = NULL WHERE id = ?",
            [id],
        )?;

        if affected == 0 {
            Err(DatabaseError::not_found("Case", "id", id))
        } else {
            Ok(())
        }
    }

    /// Update case status
    pub fn update_status(&self, conn: &Connection, id: &str, status: &str) -> DbResult<()> {
        let affected = conn.execute(
            "UPDATE cases SET status = ? WHERE id = ?",
            params![status, id],
        )?;

        if affected == 0 {
            Err(DatabaseError::not_found("Case", "id", id))
        } else {
            Ok(())
        }
    }

    /// Assign case to user
    pub fn assign_to(&self, conn: &Connection, id: &str, user_id: &str) -> DbResult<()> {
        let affected = conn.execute(
            "UPDATE cases SET assigned_to = ? WHERE id = ?",
            params![user_id, id],
        )?;

        if affected == 0 {
            Err(DatabaseError::not_found("Case", "id", id))
        } else {
            Ok(())
        }
    }
}

impl Default for CaseRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl Repository for CaseRepository {
    type Entity = Case;
    type Id = String;

    fn find_by_id(&self, conn: &Connection, id: &String) -> DbResult<Option<Case>> {
        let mut stmt = conn.prepare(
            "SELECT id, case_number, title, description, status, priority, assigned_to,
                    created_by, organization, tags, metadata, closed_at, created_at, updated_at
             FROM cases WHERE id = ?",
        )?;

        let mut rows = stmt.query_map([id], Case::from_row)?;

        if let Some(row) = rows.next() {
            Ok(Some(row?))
        } else {
            Ok(None)
        }
    }

    fn find_all(&self, conn: &Connection) -> DbResult<Vec<Case>> {
        let mut stmt = conn.prepare(
            "SELECT id, case_number, title, description, status, priority, assigned_to,
                    created_by, organization, tags, metadata, closed_at, created_at, updated_at
             FROM cases ORDER BY created_at DESC",
        )?;

        let cases = stmt
            .query_map([], Case::from_row)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(cases)
    }

    fn create(&self, conn: &Connection, entity: &Case) -> DbResult<()> {
        let tags_json = entity.tags.as_ref().map(|t| serde_json::to_string(t).ok()).flatten();
        let metadata_json = entity.metadata.as_ref().map(|m| serde_json::to_string(m).ok()).flatten();

        conn.execute(
            "INSERT INTO cases (id, case_number, title, description, status, priority,
                               assigned_to, created_by, organization, tags, metadata)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                entity.id,
                entity.case_number,
                entity.title,
                entity.description,
                entity.status,
                entity.priority,
                entity.assigned_to,
                entity.created_by,
                entity.organization,
                tags_json,
                metadata_json,
            ],
        )?;

        Ok(())
    }

    fn update(&self, conn: &Connection, entity: &Case) -> DbResult<()> {
        let tags_json = entity.tags.as_ref().map(|t| serde_json::to_string(t).ok()).flatten();
        let metadata_json = entity.metadata.as_ref().map(|m| serde_json::to_string(m).ok()).flatten();

        let affected = conn.execute(
            "UPDATE cases SET case_number = ?, title = ?, description = ?, status = ?,
                             priority = ?, assigned_to = ?, organization = ?, tags = ?, metadata = ?
             WHERE id = ?",
            params![
                entity.case_number,
                entity.title,
                entity.description,
                entity.status,
                entity.priority,
                entity.assigned_to,
                entity.organization,
                tags_json,
                metadata_json,
                entity.id,
            ],
        )?;

        if affected == 0 {
            Err(DatabaseError::not_found("Case", "id", &entity.id))
        } else {
            Ok(())
        }
    }

    fn delete(&self, conn: &Connection, id: &String) -> DbResult<()> {
        let affected = conn.execute("DELETE FROM cases WHERE id = ?", [id])?;

        if affected == 0 {
            Err(DatabaseError::not_found("Case", "id", id))
        } else {
            Ok(())
        }
    }

    fn count(&self, conn: &Connection) -> DbResult<i64> {
        let count = conn.query_row("SELECT COUNT(*) FROM cases", [], |row| row.get(0))?;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn create_test_case() -> Case {
        Case {
            id: Uuid::new_v4().to_string(),
            case_number: "CASE-2024-001".to_string(),
            title: "Test Case".to_string(),
            description: Some("Test description".to_string()),
            status: "open".to_string(),
            priority: "high".to_string(),
            assigned_to: None,
            created_by: Uuid::new_v4().to_string(),
            organization: Some("Test Org".to_string()),
            tags: Some(vec!["test".to_string()]),
            metadata: None,
            closed_at: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    #[test]
    fn test_case_crud() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(include_str!("../migrations/v001_initial.sql")).ok();

        let repo = CaseRepository::new();
        let case = create_test_case();

        // Create
        repo.create(&conn, &case).unwrap();

        // Read
        let found = repo.find_by_id(&conn, &case.id).unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().case_number, "CASE-2024-001");

        // Count
        assert_eq!(repo.count(&conn).unwrap(), 1);
    }
}
