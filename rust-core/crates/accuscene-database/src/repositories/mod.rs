//! Repository pattern implementations for data access
//!
//! Provides high-level CRUD operations and domain-specific queries
//! for all entities in the AccuScene database.

pub mod case;
pub mod accident;
pub mod vehicle;
pub mod evidence;
pub mod user;

pub use case::CaseRepository;
pub use accident::AccidentRepository;
pub use vehicle::VehicleRepository;
pub use evidence::EvidenceRepository;
pub use user::UserRepository;

use crate::error::DbResult;
use rusqlite::Connection;

/// Base repository trait
pub trait Repository {
    type Entity;
    type Id;

    /// Find an entity by ID
    fn find_by_id(&self, conn: &Connection, id: &Self::Id) -> DbResult<Option<Self::Entity>>;

    /// Find all entities
    fn find_all(&self, conn: &Connection) -> DbResult<Vec<Self::Entity>>;

    /// Create a new entity
    fn create(&self, conn: &Connection, entity: &Self::Entity) -> DbResult<()>;

    /// Update an existing entity
    fn update(&self, conn: &Connection, entity: &Self::Entity) -> DbResult<()>;

    /// Delete an entity by ID
    fn delete(&self, conn: &Connection, id: &Self::Id) -> DbResult<()>;

    /// Check if an entity exists by ID
    fn exists(&self, conn: &Connection, id: &Self::Id) -> DbResult<bool> {
        Ok(self.find_by_id(conn, id)?.is_some())
    }

    /// Count all entities
    fn count(&self, conn: &Connection) -> DbResult<i64>;
}
