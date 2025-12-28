//! User repository for managing user accounts

use crate::error::{DatabaseError, DbResult};
use crate::repositories::Repository;
use rusqlite::{params, Connection, Row};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub username: String,
    pub full_name: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: String,
    pub organization: Option<String>,
    pub phone: Option<String>,
    pub is_active: bool,
    pub last_login_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl User {
    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            email: row.get(1)?,
            username: row.get(2)?,
            full_name: row.get(3)?,
            password_hash: row.get(4)?,
            role: row.get(5)?,
            organization: row.get(6)?,
            phone: row.get(7)?,
            is_active: row.get::<_, i32>(8)? != 0,
            last_login_at: row.get(9)?,
            created_at: row.get(10)?,
            updated_at: row.get(11)?,
        })
    }
}

pub struct UserRepository;

impl UserRepository {
    pub fn new() -> Self {
        Self
    }

    pub fn find_by_email(&self, conn: &Connection, email: &str) -> DbResult<Option<User>> {
        let mut stmt = conn.prepare(
            "SELECT id, email, username, full_name, password_hash, role, organization,
                    phone, is_active, last_login_at, created_at, updated_at
             FROM users WHERE email = ?",
        )?;

        let mut rows = stmt.query_map([email], User::from_row)?;
        Ok(rows.next().transpose()?)
    }

    pub fn find_by_username(&self, conn: &Connection, username: &str) -> DbResult<Option<User>> {
        let mut stmt = conn.prepare(
            "SELECT id, email, username, full_name, password_hash, role, organization,
                    phone, is_active, last_login_at, created_at, updated_at
             FROM users WHERE username = ?",
        )?;

        let mut rows = stmt.query_map([username], User::from_row)?;
        Ok(rows.next().transpose()?)
    }

    pub fn find_by_organization(&self, conn: &Connection, organization: &str) -> DbResult<Vec<User>> {
        let mut stmt = conn.prepare(
            "SELECT id, email, username, full_name, password_hash, role, organization,
                    phone, is_active, last_login_at, created_at, updated_at
             FROM users WHERE organization = ? ORDER BY created_at DESC",
        )?;

        let users = stmt
            .query_map([organization], User::from_row)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(users)
    }

    pub fn find_by_role(&self, conn: &Connection, role: &str) -> DbResult<Vec<User>> {
        let mut stmt = conn.prepare(
            "SELECT id, email, username, full_name, password_hash, role, organization,
                    phone, is_active, last_login_at, created_at, updated_at
             FROM users WHERE role = ? ORDER BY created_at DESC",
        )?;

        let users = stmt
            .query_map([role], User::from_row)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(users)
    }

    pub fn update_last_login(&self, conn: &Connection, id: &str) -> DbResult<()> {
        let affected = conn.execute(
            "UPDATE users SET last_login_at = datetime('now') WHERE id = ?",
            [id],
        )?;

        if affected == 0 {
            Err(DatabaseError::not_found("User", "id", id))
        } else {
            Ok(())
        }
    }

    pub fn deactivate(&self, conn: &Connection, id: &str) -> DbResult<()> {
        let affected = conn.execute(
            "UPDATE users SET is_active = 0 WHERE id = ?",
            [id],
        )?;

        if affected == 0 {
            Err(DatabaseError::not_found("User", "id", id))
        } else {
            Ok(())
        }
    }

    pub fn activate(&self, conn: &Connection, id: &str) -> DbResult<()> {
        let affected = conn.execute(
            "UPDATE users SET is_active = 1 WHERE id = ?",
            [id],
        )?;

        if affected == 0 {
            Err(DatabaseError::not_found("User", "id", id))
        } else {
            Ok(())
        }
    }
}

impl Default for UserRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl Repository for UserRepository {
    type Entity = User;
    type Id = String;

    fn find_by_id(&self, conn: &Connection, id: &String) -> DbResult<Option<User>> {
        let mut stmt = conn.prepare(
            "SELECT id, email, username, full_name, password_hash, role, organization,
                    phone, is_active, last_login_at, created_at, updated_at
             FROM users WHERE id = ?",
        )?;

        let mut rows = stmt.query_map([id], User::from_row)?;
        Ok(rows.next().transpose()?)
    }

    fn find_all(&self, conn: &Connection) -> DbResult<Vec<User>> {
        let mut stmt = conn.prepare(
            "SELECT id, email, username, full_name, password_hash, role, organization,
                    phone, is_active, last_login_at, created_at, updated_at
             FROM users ORDER BY created_at DESC",
        )?;

        let users = stmt
            .query_map([], User::from_row)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(users)
    }

    fn create(&self, conn: &Connection, entity: &User) -> DbResult<()> {
        conn.execute(
            "INSERT INTO users (id, email, username, full_name, password_hash, role,
                               organization, phone, is_active)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                entity.id, entity.email, entity.username, entity.full_name,
                entity.password_hash, entity.role, entity.organization, entity.phone,
                entity.is_active as i32,
            ],
        )?;
        Ok(())
    }

    fn update(&self, conn: &Connection, entity: &User) -> DbResult<()> {
        let affected = conn.execute(
            "UPDATE users SET email = ?, username = ?, full_name = ?, role = ?,
                            organization = ?, phone = ?, is_active = ?
             WHERE id = ?",
            params![
                entity.email, entity.username, entity.full_name, entity.role,
                entity.organization, entity.phone, entity.is_active as i32, entity.id,
            ],
        )?;

        if affected == 0 {
            Err(DatabaseError::not_found("User", "id", &entity.id))
        } else {
            Ok(())
        }
    }

    fn delete(&self, conn: &Connection, id: &String) -> DbResult<()> {
        let affected = conn.execute("DELETE FROM users WHERE id = ?", [id])?;
        if affected == 0 {
            Err(DatabaseError::not_found("User", "id", id))
        } else {
            Ok(())
        }
    }

    fn count(&self, conn: &Connection) -> DbResult<i64> {
        let count = conn.query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))?;
        Ok(count)
    }
}
