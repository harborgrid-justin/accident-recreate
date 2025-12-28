//! Initial database schema migration for AccuScene Enterprise
//!
//! Creates the foundational tables for accident recreation platform:
//! - Users and authentication
//! - Cases and case management
//! - Accidents and accident data
//! - Vehicles involved in accidents
//! - Evidence storage and metadata
//! - Audit trail

use super::Migration;
use crate::error::DbResult;
use rusqlite::Connection;

pub struct InitialMigration;

impl Migration for InitialMigration {
    fn version(&self) -> u32 {
        1
    }

    fn name(&self) -> &str {
        "initial_schema"
    }

    fn description(&self) -> &str {
        "Create initial database schema for AccuScene Enterprise"
    }

    fn up(&self, conn: &mut Connection) -> DbResult<()> {
        conn.execute_batch(
            r#"
            -- Users table
            CREATE TABLE users (
                id TEXT PRIMARY KEY,
                email TEXT UNIQUE NOT NULL,
                username TEXT UNIQUE NOT NULL,
                full_name TEXT NOT NULL,
                password_hash TEXT NOT NULL,
                role TEXT NOT NULL DEFAULT 'investigator',
                organization TEXT,
                phone TEXT,
                is_active INTEGER NOT NULL DEFAULT 1,
                last_login_at TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE INDEX idx_users_email ON users(email);
            CREATE INDEX idx_users_username ON users(username);
            CREATE INDEX idx_users_organization ON users(organization);
            CREATE INDEX idx_users_created_at ON users(created_at);

            -- Cases table
            CREATE TABLE cases (
                id TEXT PRIMARY KEY,
                case_number TEXT UNIQUE NOT NULL,
                title TEXT NOT NULL,
                description TEXT,
                status TEXT NOT NULL DEFAULT 'open',
                priority TEXT NOT NULL DEFAULT 'medium',
                assigned_to TEXT,
                created_by TEXT NOT NULL,
                organization TEXT,
                tags TEXT, -- JSON array
                metadata TEXT, -- JSON object
                closed_at TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (assigned_to) REFERENCES users(id) ON DELETE SET NULL,
                FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE CASCADE
            );

            CREATE INDEX idx_cases_case_number ON cases(case_number);
            CREATE INDEX idx_cases_status ON cases(status);
            CREATE INDEX idx_cases_assigned_to ON cases(assigned_to);
            CREATE INDEX idx_cases_created_by ON cases(created_by);
            CREATE INDEX idx_cases_organization ON cases(organization);
            CREATE INDEX idx_cases_created_at ON cases(created_at);

            -- Accidents table
            CREATE TABLE accidents (
                id TEXT PRIMARY KEY,
                case_id TEXT NOT NULL,
                accident_date TEXT NOT NULL,
                location TEXT NOT NULL,
                location_lat REAL,
                location_lng REAL,
                weather_conditions TEXT,
                road_conditions TEXT,
                light_conditions TEXT,
                traffic_control TEXT,
                description TEXT,
                severity TEXT,
                fatalities INTEGER NOT NULL DEFAULT 0,
                injuries INTEGER NOT NULL DEFAULT 0,
                property_damage_estimate REAL,
                police_report_number TEXT,
                police_department TEXT,
                reconstruction_data TEXT, -- JSON object
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (case_id) REFERENCES cases(id) ON DELETE CASCADE
            );

            CREATE INDEX idx_accidents_case_id ON accidents(case_id);
            CREATE INDEX idx_accidents_accident_date ON accidents(accident_date);
            CREATE INDEX idx_accidents_location ON accidents(location);
            CREATE INDEX idx_accidents_severity ON accidents(severity);
            CREATE INDEX idx_accidents_created_at ON accidents(created_at);

            -- Vehicles table
            CREATE TABLE vehicles (
                id TEXT PRIMARY KEY,
                accident_id TEXT NOT NULL,
                vehicle_number INTEGER NOT NULL,
                make TEXT,
                model TEXT,
                year INTEGER,
                color TEXT,
                vin TEXT,
                license_plate TEXT,
                vehicle_type TEXT,
                damage_description TEXT,
                occupants INTEGER,
                speed_estimate REAL,
                direction_of_travel REAL, -- degrees
                final_position_lat REAL,
                final_position_lng REAL,
                airbag_deployment INTEGER,
                driver_info TEXT, -- JSON object
                insurance_info TEXT, -- JSON object
                metadata TEXT, -- JSON object
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (accident_id) REFERENCES accidents(id) ON DELETE CASCADE
            );

            CREATE INDEX idx_vehicles_accident_id ON vehicles(accident_id);
            CREATE INDEX idx_vehicles_vin ON vehicles(vin);
            CREATE INDEX idx_vehicles_license_plate ON vehicles(license_plate);
            CREATE INDEX idx_vehicles_created_at ON vehicles(created_at);

            -- Evidence table
            CREATE TABLE evidence (
                id TEXT PRIMARY KEY,
                case_id TEXT NOT NULL,
                accident_id TEXT,
                evidence_type TEXT NOT NULL,
                title TEXT NOT NULL,
                description TEXT,
                file_path TEXT,
                file_name TEXT,
                file_size INTEGER,
                file_mime_type TEXT,
                file_hash TEXT,
                collected_by TEXT,
                collected_at TEXT,
                location TEXT,
                chain_of_custody TEXT, -- JSON array
                tags TEXT, -- JSON array
                metadata TEXT, -- JSON object
                is_verified INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (case_id) REFERENCES cases(id) ON DELETE CASCADE,
                FOREIGN KEY (accident_id) REFERENCES accidents(id) ON DELETE CASCADE,
                FOREIGN KEY (collected_by) REFERENCES users(id) ON DELETE SET NULL
            );

            CREATE INDEX idx_evidence_case_id ON evidence(case_id);
            CREATE INDEX idx_evidence_accident_id ON evidence(accident_id);
            CREATE INDEX idx_evidence_evidence_type ON evidence(evidence_type);
            CREATE INDEX idx_evidence_collected_by ON evidence(collected_by);
            CREATE INDEX idx_evidence_collected_at ON evidence(collected_at);
            CREATE INDEX idx_evidence_created_at ON evidence(created_at);

            -- Audit log table
            CREATE TABLE audit_log (
                id TEXT PRIMARY KEY,
                entity_type TEXT NOT NULL,
                entity_id TEXT NOT NULL,
                action TEXT NOT NULL,
                user_id TEXT,
                user_email TEXT,
                changes TEXT, -- JSON object
                ip_address TEXT,
                user_agent TEXT,
                timestamp TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE INDEX idx_audit_log_entity ON audit_log(entity_type, entity_id);
            CREATE INDEX idx_audit_log_user_id ON audit_log(user_id);
            CREATE INDEX idx_audit_log_action ON audit_log(action);
            CREATE INDEX idx_audit_log_timestamp ON audit_log(timestamp);

            -- Full-text search virtual table for cases
            CREATE VIRTUAL TABLE cases_fts USING fts5(
                case_number,
                title,
                description,
                tags,
                content=cases,
                content_rowid=rowid
            );

            -- Triggers to keep FTS in sync with cases table
            CREATE TRIGGER cases_fts_insert AFTER INSERT ON cases BEGIN
                INSERT INTO cases_fts(rowid, case_number, title, description, tags)
                VALUES (new.rowid, new.case_number, new.title, new.description, new.tags);
            END;

            CREATE TRIGGER cases_fts_delete AFTER DELETE ON cases BEGIN
                DELETE FROM cases_fts WHERE rowid = old.rowid;
            END;

            CREATE TRIGGER cases_fts_update AFTER UPDATE ON cases BEGIN
                UPDATE cases_fts
                SET case_number = new.case_number,
                    title = new.title,
                    description = new.description,
                    tags = new.tags
                WHERE rowid = new.rowid;
            END;

            -- Full-text search virtual table for evidence
            CREATE VIRTUAL TABLE evidence_fts USING fts5(
                title,
                description,
                file_name,
                tags,
                content=evidence,
                content_rowid=rowid
            );

            -- Triggers to keep FTS in sync with evidence table
            CREATE TRIGGER evidence_fts_insert AFTER INSERT ON evidence BEGIN
                INSERT INTO evidence_fts(rowid, title, description, file_name, tags)
                VALUES (new.rowid, new.title, new.description, new.file_name, new.tags);
            END;

            CREATE TRIGGER evidence_fts_delete AFTER DELETE ON evidence BEGIN
                DELETE FROM evidence_fts WHERE rowid = old.rowid;
            END;

            CREATE TRIGGER evidence_fts_update AFTER UPDATE ON evidence BEGIN
                UPDATE evidence_fts
                SET title = new.title,
                    description = new.description,
                    file_name = new.file_name,
                    tags = new.tags
                WHERE rowid = new.rowid;
            END;

            -- Update timestamps trigger for users
            CREATE TRIGGER update_users_timestamp AFTER UPDATE ON users
            FOR EACH ROW BEGIN
                UPDATE users SET updated_at = datetime('now') WHERE id = NEW.id;
            END;

            -- Update timestamps trigger for cases
            CREATE TRIGGER update_cases_timestamp AFTER UPDATE ON cases
            FOR EACH ROW BEGIN
                UPDATE cases SET updated_at = datetime('now') WHERE id = NEW.id;
            END;

            -- Update timestamps trigger for accidents
            CREATE TRIGGER update_accidents_timestamp AFTER UPDATE ON accidents
            FOR EACH ROW BEGIN
                UPDATE accidents SET updated_at = datetime('now') WHERE id = NEW.id;
            END;

            -- Update timestamps trigger for vehicles
            CREATE TRIGGER update_vehicles_timestamp AFTER UPDATE ON vehicles
            FOR EACH ROW BEGIN
                UPDATE vehicles SET updated_at = datetime('now') WHERE id = NEW.id;
            END;

            -- Update timestamps trigger for evidence
            CREATE TRIGGER update_evidence_timestamp AFTER UPDATE ON evidence
            FOR EACH ROW BEGIN
                UPDATE evidence SET updated_at = datetime('now') WHERE id = NEW.id;
            END;
            "#,
        )?;

        Ok(())
    }

    fn down(&self, conn: &mut Connection) -> DbResult<()> {
        conn.execute_batch(
            r#"
            -- Drop triggers
            DROP TRIGGER IF EXISTS update_evidence_timestamp;
            DROP TRIGGER IF EXISTS update_vehicles_timestamp;
            DROP TRIGGER IF EXISTS update_accidents_timestamp;
            DROP TRIGGER IF EXISTS update_cases_timestamp;
            DROP TRIGGER IF EXISTS update_users_timestamp;

            DROP TRIGGER IF EXISTS evidence_fts_update;
            DROP TRIGGER IF EXISTS evidence_fts_delete;
            DROP TRIGGER IF EXISTS evidence_fts_insert;

            DROP TRIGGER IF EXISTS cases_fts_update;
            DROP TRIGGER IF EXISTS cases_fts_delete;
            DROP TRIGGER IF EXISTS cases_fts_insert;

            -- Drop FTS tables
            DROP TABLE IF EXISTS evidence_fts;
            DROP TABLE IF EXISTS cases_fts;

            -- Drop tables in reverse order of dependencies
            DROP TABLE IF EXISTS audit_log;
            DROP TABLE IF EXISTS evidence;
            DROP TABLE IF EXISTS vehicles;
            DROP TABLE IF EXISTS accidents;
            DROP TABLE IF EXISTS cases;
            DROP TABLE IF EXISTS users;
            "#,
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_migration_up() {
        let mut conn = Connection::open_in_memory().unwrap();
        let migration = InitialMigration;

        migration.up(&mut conn).unwrap();

        // Verify tables exist
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert!(tables.contains(&"users".to_string()));
        assert!(tables.contains(&"cases".to_string()));
        assert!(tables.contains(&"accidents".to_string()));
        assert!(tables.contains(&"vehicles".to_string()));
        assert!(tables.contains(&"evidence".to_string()));
        assert!(tables.contains(&"audit_log".to_string()));
    }

    #[test]
    fn test_initial_migration_down() {
        let mut conn = Connection::open_in_memory().unwrap();
        let migration = InitialMigration;

        migration.up(&mut conn).unwrap();
        migration.down(&mut conn).unwrap();

        // Verify tables are dropped
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('users', 'cases', 'accidents', 'vehicles', 'evidence', 'audit_log')",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(count, 0);
    }
}
