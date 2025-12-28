//! Database connection wrapper with transaction support
//!
//! Provides a high-level wrapper around database connections with
//! automatic transaction management, savepoints, and error handling.

use crate::error::{DatabaseError, DbResult};
use rusqlite::{Connection, Transaction};
use std::ops::{Deref, DerefMut};
use tracing::{debug, error, info, warn};

/// Wrapper around a database connection
pub struct DbConnection<'a> {
    conn: &'a mut Connection,
}

impl<'a> DbConnection<'a> {
    /// Create a new connection wrapper
    pub fn new(conn: &'a mut Connection) -> Self {
        Self { conn }
    }

    /// Begin a new transaction
    pub fn begin_transaction(&mut self) -> DbResult<DbTransaction<'_>> {
        debug!("Beginning transaction");
        let tx = self
            .conn
            .transaction()
            .map_err(|e| DatabaseError::TransactionError(e.to_string()))?;

        Ok(DbTransaction::new(tx))
    }

    /// Execute a function within a transaction
    pub fn transaction<F, T>(&mut self, f: F) -> DbResult<T>
    where
        F: FnOnce(&mut DbTransaction<'_>) -> DbResult<T>,
    {
        let mut tx = self.begin_transaction()?;
        let result = f(&mut tx)?;
        tx.commit()?;
        Ok(result)
    }

    /// Execute a function within a transaction with automatic rollback on error
    pub fn try_transaction<F, T>(&mut self, f: F) -> DbResult<T>
    where
        F: FnOnce(&mut DbTransaction<'_>) -> DbResult<T>,
    {
        match self.transaction(f) {
            Ok(result) => Ok(result),
            Err(e) => {
                warn!("Transaction failed, already rolled back: {}", e);
                Err(e)
            }
        }
    }
}

impl<'a> Deref for DbConnection<'a> {
    type Target = Connection;

    fn deref(&self) -> &Self::Target {
        self.conn
    }
}

impl<'a> DerefMut for DbConnection<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.conn
    }
}

/// Wrapper around a database transaction
pub struct DbTransaction<'a> {
    tx: Option<Transaction<'a>>,
    committed: bool,
    savepoint_counter: u32,
}

impl<'a> DbTransaction<'a> {
    /// Create a new transaction wrapper
    fn new(tx: Transaction<'a>) -> Self {
        Self {
            tx: Some(tx),
            committed: false,
            savepoint_counter: 0,
        }
    }

    /// Commit the transaction
    pub fn commit(mut self) -> DbResult<()> {
        debug!("Committing transaction");

        if self.committed {
            warn!("Transaction already committed");
            return Ok(());
        }

        if let Some(tx) = self.tx.take() {
            tx.commit()
                .map_err(|e| {
                    error!("Failed to commit transaction: {}", e);
                    DatabaseError::TransactionError(e.to_string())
                })?;
            self.committed = true;
            info!("Transaction committed successfully");
        }

        Ok(())
    }

    /// Rollback the transaction
    pub fn rollback(mut self) -> DbResult<()> {
        debug!("Rolling back transaction");

        if self.committed {
            warn!("Cannot rollback committed transaction");
            return Err(DatabaseError::TransactionError(
                "Cannot rollback committed transaction".to_string(),
            ));
        }

        if let Some(tx) = self.tx.take() {
            tx.rollback()
                .map_err(|e| {
                    error!("Failed to rollback transaction: {}", e);
                    DatabaseError::TransactionError(e.to_string())
                })?;
            info!("Transaction rolled back successfully");
        }

        Ok(())
    }

    /// Create a savepoint within the transaction
    pub fn savepoint(&mut self, name: &str) -> DbResult<Savepoint<'_>> {
        debug!("Creating savepoint: {}", name);

        let savepoint_name = format!("sp_{}_{}", name, self.savepoint_counter);
        self.savepoint_counter += 1;

        if let Some(ref mut tx) = self.tx {
            tx.execute_batch(&format!("SAVEPOINT {}", savepoint_name))
                .map_err(|e| DatabaseError::TransactionError(e.to_string()))?;

            Ok(Savepoint {
                tx,
                name: savepoint_name,
                released: false,
            })
        } else {
            Err(DatabaseError::TransactionError(
                "Transaction already consumed".to_string(),
            ))
        }
    }

    /// Execute a function within a savepoint
    pub fn with_savepoint<F, T>(&mut self, name: &str, f: F) -> DbResult<T>
    where
        F: FnOnce(&mut Transaction<'_>) -> DbResult<T>,
    {
        let mut savepoint = self.savepoint(name)?;
        let result = f(savepoint.tx)?;
        savepoint.release()?;
        Ok(result)
    }

    /// Check if transaction is committed
    pub fn is_committed(&self) -> bool {
        self.committed
    }
}

impl<'a> Deref for DbTransaction<'a> {
    type Target = Transaction<'a>;

    fn deref(&self) -> &Self::Target {
        self.tx.as_ref().expect("Transaction already consumed")
    }
}

impl<'a> DerefMut for DbTransaction<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.tx.as_mut().expect("Transaction already consumed")
    }
}

impl<'a> Drop for DbTransaction<'a> {
    fn drop(&mut self) {
        if !self.committed && self.tx.is_some() {
            warn!("Transaction dropped without commit, rolling back");
            // Transaction will be automatically rolled back when dropped
        }
    }
}

/// Savepoint within a transaction
pub struct Savepoint<'a> {
    tx: &'a mut Transaction<'a>,
    name: String,
    released: bool,
}

impl<'a> Savepoint<'a> {
    /// Release the savepoint (commit it)
    pub fn release(mut self) -> DbResult<()> {
        debug!("Releasing savepoint: {}", self.name);

        if self.released {
            warn!("Savepoint already released");
            return Ok(());
        }

        self.tx
            .execute_batch(&format!("RELEASE SAVEPOINT {}", self.name))
            .map_err(|e| DatabaseError::TransactionError(e.to_string()))?;

        self.released = true;
        info!("Savepoint released: {}", self.name);

        Ok(())
    }

    /// Rollback to the savepoint
    pub fn rollback(mut self) -> DbResult<()> {
        debug!("Rolling back to savepoint: {}", self.name);

        if self.released {
            return Err(DatabaseError::TransactionError(
                "Cannot rollback released savepoint".to_string(),
            ));
        }

        self.tx
            .execute_batch(&format!("ROLLBACK TO SAVEPOINT {}", self.name))
            .map_err(|e| DatabaseError::TransactionError(e.to_string()))?;

        // Also release the savepoint after rollback
        self.tx
            .execute_batch(&format!("RELEASE SAVEPOINT {}", self.name))
            .map_err(|e| DatabaseError::TransactionError(e.to_string()))?;

        self.released = true;
        info!("Rolled back to savepoint: {}", self.name);

        Ok(())
    }
}

impl<'a> Drop for Savepoint<'a> {
    fn drop(&mut self) {
        if !self.released {
            warn!("Savepoint dropped without release: {}", self.name);
            // Savepoint will be automatically rolled back
        }
    }
}

/// Transaction isolation level
#[derive(Debug, Clone, Copy)]
pub enum IsolationLevel {
    /// Read uncommitted
    ReadUncommitted,
    /// Read committed
    ReadCommitted,
    /// Repeatable read
    RepeatableRead,
    /// Serializable
    Serializable,
}

impl IsolationLevel {
    pub fn as_str(&self) -> &str {
        match self {
            IsolationLevel::ReadUncommitted => "READ UNCOMMITTED",
            IsolationLevel::ReadCommitted => "READ COMMITTED",
            IsolationLevel::RepeatableRead => "REPEATABLE READ",
            IsolationLevel::Serializable => "SERIALIZABLE",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE test (id INTEGER PRIMARY KEY, value TEXT);",
        )
        .unwrap();
        conn
    }

    #[test]
    fn test_transaction_commit() {
        let mut conn = setup_test_db();
        let mut db_conn = DbConnection::new(&mut conn);

        db_conn
            .transaction(|tx| {
                tx.execute("INSERT INTO test (value) VALUES (?)", ["test"])?;
                Ok(())
            })
            .unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM test", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_transaction_rollback() {
        let mut conn = setup_test_db();
        let mut db_conn = DbConnection::new(&mut conn);

        let mut tx = db_conn.begin_transaction().unwrap();
        tx.execute("INSERT INTO test (value) VALUES (?)", ["test"])
            .unwrap();
        tx.rollback().unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM test", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_savepoint() {
        let mut conn = setup_test_db();
        let mut db_conn = DbConnection::new(&mut conn);

        db_conn
            .transaction(|tx| {
                tx.execute("INSERT INTO test (value) VALUES (?)", ["first"])?;

                {
                    let mut sp = tx.savepoint("test_sp")?;
                    sp.tx.execute("INSERT INTO test (value) VALUES (?)", ["second"])?;
                    sp.rollback()?;
                }

                Ok(())
            })
            .unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM test", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_transaction_auto_rollback() {
        let mut conn = setup_test_db();
        let mut db_conn = DbConnection::new(&mut conn);

        let result = db_conn.transaction(|tx| {
            tx.execute("INSERT INTO test (value) VALUES (?)", ["test"])?;
            Err(DatabaseError::Other("Test error".to_string()))
        });

        assert!(result.is_err());

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM test", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }
}
