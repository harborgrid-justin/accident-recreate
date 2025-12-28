//! Evidence repository for managing evidence records

use crate::error::{DatabaseError, DbResult};
use crate::repositories::Repository;
use rusqlite::{params, Connection, Row};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub id: String,
    pub case_id: String,
    pub accident_id: Option<String>,
    pub evidence_type: String,
    pub title: String,
    pub description: Option<String>,
    pub file_path: Option<String>,
    pub file_name: Option<String>,
    pub file_size: Option<i64>,
    pub file_mime_type: Option<String>,
    pub file_hash: Option<String>,
    pub collected_by: Option<String>,
    pub collected_at: Option<String>,
    pub location: Option<String>,
    pub chain_of_custody: Option<Vec<serde_json::Value>>,
    pub tags: Option<Vec<String>>,
    pub metadata: Option<serde_json::Value>,
    pub is_verified: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl Evidence {
    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        let custody_json: Option<String> = row.get(14)?;
        let chain_of_custody = custody_json.and_then(|s| serde_json::from_str(&s).ok());

        let tags_json: Option<String> = row.get(15)?;
        let tags = tags_json.and_then(|s| serde_json::from_str(&s).ok());

        let metadata_json: Option<String> = row.get(16)?;
        let metadata = metadata_json.and_then(|s| serde_json::from_str(&s).ok());

        Ok(Self {
            id: row.get(0)?,
            case_id: row.get(1)?,
            accident_id: row.get(2)?,
            evidence_type: row.get(3)?,
            title: row.get(4)?,
            description: row.get(5)?,
            file_path: row.get(6)?,
            file_name: row.get(7)?,
            file_size: row.get(8)?,
            file_mime_type: row.get(9)?,
            file_hash: row.get(10)?,
            collected_by: row.get(11)?,
            collected_at: row.get(12)?,
            location: row.get(13)?,
            chain_of_custody,
            tags,
            metadata,
            is_verified: row.get::<_, i32>(17)? != 0,
            created_at: row.get(18)?,
            updated_at: row.get(19)?,
        })
    }
}

pub struct EvidenceRepository;

impl EvidenceRepository {
    pub fn new() -> Self {
        Self
    }

    pub fn find_by_case_id(&self, conn: &Connection, case_id: &str) -> DbResult<Vec<Evidence>> {
        let mut stmt = conn.prepare(
            "SELECT id, case_id, accident_id, evidence_type, title, description,
                    file_path, file_name, file_size, file_mime_type, file_hash,
                    collected_by, collected_at, location, chain_of_custody, tags, metadata,
                    is_verified, created_at, updated_at
             FROM evidence WHERE case_id = ? ORDER BY collected_at DESC",
        )?;

        let evidence = stmt
            .query_map([case_id], Evidence::from_row)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(evidence)
    }

    pub fn find_by_accident_id(&self, conn: &Connection, accident_id: &str) -> DbResult<Vec<Evidence>> {
        let mut stmt = conn.prepare(
            "SELECT id, case_id, accident_id, evidence_type, title, description,
                    file_path, file_name, file_size, file_mime_type, file_hash,
                    collected_by, collected_at, location, chain_of_custody, tags, metadata,
                    is_verified, created_at, updated_at
             FROM evidence WHERE accident_id = ? ORDER BY collected_at DESC",
        )?;

        let evidence = stmt
            .query_map([accident_id], Evidence::from_row)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(evidence)
    }

    pub fn find_by_type(&self, conn: &Connection, evidence_type: &str) -> DbResult<Vec<Evidence>> {
        let mut stmt = conn.prepare(
            "SELECT id, case_id, accident_id, evidence_type, title, description,
                    file_path, file_name, file_size, file_mime_type, file_hash,
                    collected_by, collected_at, location, chain_of_custody, tags, metadata,
                    is_verified, created_at, updated_at
             FROM evidence WHERE evidence_type = ? ORDER BY collected_at DESC",
        )?;

        let evidence = stmt
            .query_map([evidence_type], Evidence::from_row)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(evidence)
    }

    pub fn verify_evidence(&self, conn: &Connection, id: &str) -> DbResult<()> {
        let affected = conn.execute(
            "UPDATE evidence SET is_verified = 1 WHERE id = ?",
            [id],
        )?;

        if affected == 0 {
            Err(DatabaseError::not_found("Evidence", "id", id))
        } else {
            Ok(())
        }
    }
}

impl Default for EvidenceRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl Repository for EvidenceRepository {
    type Entity = Evidence;
    type Id = String;

    fn find_by_id(&self, conn: &Connection, id: &String) -> DbResult<Option<Evidence>> {
        let mut stmt = conn.prepare(
            "SELECT id, case_id, accident_id, evidence_type, title, description,
                    file_path, file_name, file_size, file_mime_type, file_hash,
                    collected_by, collected_at, location, chain_of_custody, tags, metadata,
                    is_verified, created_at, updated_at
             FROM evidence WHERE id = ?",
        )?;

        let mut rows = stmt.query_map([id], Evidence::from_row)?;
        Ok(rows.next().transpose()?)
    }

    fn find_all(&self, conn: &Connection) -> DbResult<Vec<Evidence>> {
        let mut stmt = conn.prepare(
            "SELECT id, case_id, accident_id, evidence_type, title, description,
                    file_path, file_name, file_size, file_mime_type, file_hash,
                    collected_by, collected_at, location, chain_of_custody, tags, metadata,
                    is_verified, created_at, updated_at
             FROM evidence ORDER BY collected_at DESC",
        )?;

        let evidence = stmt
            .query_map([], Evidence::from_row)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(evidence)
    }

    fn create(&self, conn: &Connection, entity: &Evidence) -> DbResult<()> {
        let custody_json = entity.chain_of_custody.as_ref().map(|c| serde_json::to_string(c).ok()).flatten();
        let tags_json = entity.tags.as_ref().map(|t| serde_json::to_string(t).ok()).flatten();
        let metadata_json = entity.metadata.as_ref().map(|m| serde_json::to_string(m).ok()).flatten();

        conn.execute(
            "INSERT INTO evidence (id, case_id, accident_id, evidence_type, title, description,
                                  file_path, file_name, file_size, file_mime_type, file_hash,
                                  collected_by, collected_at, location, chain_of_custody, tags,
                                  metadata, is_verified)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                entity.id, entity.case_id, entity.accident_id, entity.evidence_type,
                entity.title, entity.description, entity.file_path, entity.file_name,
                entity.file_size, entity.file_mime_type, entity.file_hash, entity.collected_by,
                entity.collected_at, entity.location, custody_json, tags_json, metadata_json,
                entity.is_verified as i32,
            ],
        )?;
        Ok(())
    }

    fn update(&self, conn: &Connection, entity: &Evidence) -> DbResult<()> {
        let custody_json = entity.chain_of_custody.as_ref().map(|c| serde_json::to_string(c).ok()).flatten();
        let tags_json = entity.tags.as_ref().map(|t| serde_json::to_string(t).ok()).flatten();
        let metadata_json = entity.metadata.as_ref().map(|m| serde_json::to_string(m).ok()).flatten();

        let affected = conn.execute(
            "UPDATE evidence SET case_id = ?, accident_id = ?, evidence_type = ?, title = ?,
                               description = ?, file_path = ?, file_name = ?, file_size = ?,
                               file_mime_type = ?, file_hash = ?, collected_by = ?, collected_at = ?,
                               location = ?, chain_of_custody = ?, tags = ?, metadata = ?,
                               is_verified = ?
             WHERE id = ?",
            params![
                entity.case_id, entity.accident_id, entity.evidence_type, entity.title,
                entity.description, entity.file_path, entity.file_name, entity.file_size,
                entity.file_mime_type, entity.file_hash, entity.collected_by, entity.collected_at,
                entity.location, custody_json, tags_json, metadata_json, entity.is_verified as i32,
                entity.id,
            ],
        )?;

        if affected == 0 {
            Err(DatabaseError::not_found("Evidence", "id", &entity.id))
        } else {
            Ok(())
        }
    }

    fn delete(&self, conn: &Connection, id: &String) -> DbResult<()> {
        let affected = conn.execute("DELETE FROM evidence WHERE id = ?", [id])?;
        if affected == 0 {
            Err(DatabaseError::not_found("Evidence", "id", id))
        } else {
            Ok(())
        }
    }

    fn count(&self, conn: &Connection) -> DbResult<i64> {
        let count = conn.query_row("SELECT COUNT(*) FROM evidence", [], |row| row.get(0))?;
        Ok(count)
    }
}
