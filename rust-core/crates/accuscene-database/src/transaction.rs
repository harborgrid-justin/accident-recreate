//! Transaction management utilities
//!
//! Provides high-level transaction management with retry logic,
//! deadlock detection, and automatic rollback on errors.

use crate::error::{DatabaseError, DbResult};
use rusqlite::Connection;
use std::time::Duration;
use tracing::{debug, warn};

/// Transaction options
#[derive(Debug, Clone)]
pub struct TransactionOptions {
    /// Maximum number of retry attempts on transient errors
    pub max_retries: u32,
    /// Delay between retry attempts
    pub retry_delay: Duration,
    /// Enable automatic retry on deadlock
    pub retry_on_deadlock: bool,
    /// Enable automatic retry on lock timeout
    pub retry_on_lock_timeout: bool,
}

impl Default for TransactionOptions {
    fn default() -> Self {
        Self {
            max_retries: 3,
            retry_delay: Duration::from_millis(100),
            retry_on_deadlock: true,
            retry_on_lock_timeout: true,
        }
    }
}

/// Execute a function within a transaction with retry logic
pub fn with_transaction<F, T>(
    conn: &mut Connection,
    options: &TransactionOptions,
    f: F,
) -> DbResult<T>
where
    F: Fn(&rusqlite::Transaction) -> DbResult<T>,
{
    let mut attempts = 0;

    loop {
        attempts += 1;

        debug!("Starting transaction attempt {}/{}", attempts, options.max_retries + 1);

        let tx = conn
            .transaction()
            .map_err(|e| DatabaseError::TransactionError(e.to_string()))?;

        match f(&tx) {
            Ok(result) => {
                // Commit the transaction
                tx.commit()
                    .map_err(|e| DatabaseError::TransactionError(e.to_string()))?;

                debug!("Transaction committed successfully on attempt {}", attempts);
                return Ok(result);
            }
            Err(e) => {
                // Rollback is automatic when tx is dropped
                warn!("Transaction failed on attempt {}: {}", attempts, e);

                // Check if we should retry
                if attempts > options.max_retries {
                    return Err(e);
                }

                let should_retry = match &e {
                    DatabaseError::Deadlock(_) => options.retry_on_deadlock,
                    DatabaseError::LockTimeout(_) => options.retry_on_lock_timeout,
                    _ if e.is_transient() => true,
                    _ => false,
                };

                if !should_retry {
                    return Err(e);
                }

                // Wait before retrying
                if attempts < options.max_retries {
                    debug!("Retrying transaction in {:?}", options.retry_delay);
                    std::thread::sleep(options.retry_delay);
                }
            }
        }
    }
}

/// Execute a function within a transaction (simple version)
pub fn execute_transaction<F, T>(conn: &mut Connection, f: F) -> DbResult<T>
where
    F: Fn(&rusqlite::Transaction) -> DbResult<T>,
{
    with_transaction(conn, &TransactionOptions::default(), f)
}

/// Transaction manager for managing complex transactions
pub struct TransactionManager {
    options: TransactionOptions,
}

impl TransactionManager {
    /// Create a new transaction manager
    pub fn new() -> Self {
        Self {
            options: TransactionOptions::default(),
        }
    }

    /// Create a transaction manager with custom options
    pub fn with_options(options: TransactionOptions) -> Self {
        Self { options }
    }

    /// Execute a function within a managed transaction
    pub fn execute<F, T>(&self, conn: &mut Connection, f: F) -> DbResult<T>
    where
        F: Fn(&rusqlite::Transaction) -> DbResult<T>,
    {
        with_transaction(conn, &self.options, f)
    }

    /// Set max retries
    pub fn set_max_retries(&mut self, max_retries: u32) {
        self.options.max_retries = max_retries;
    }

    /// Set retry delay
    pub fn set_retry_delay(&mut self, delay: Duration) {
        self.options.retry_delay = delay;
    }
}

impl Default for TransactionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_success() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE test (id INTEGER PRIMARY KEY, value TEXT)")
            .unwrap();

        let result = execute_transaction(&mut conn, |tx| {
            tx.execute("INSERT INTO test (value) VALUES (?)", ["test"])?;
            Ok(42)
        });

        assert_eq!(result.unwrap(), 42);

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM test", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_transaction_rollback() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE test (id INTEGER PRIMARY KEY, value TEXT)")
            .unwrap();

        let result: DbResult<i32> = execute_transaction(&mut conn, |tx| {
            tx.execute("INSERT INTO test (value) VALUES (?)", ["test"])?;
            Err(DatabaseError::Other("Test error".to_string()))
        });

        assert!(result.is_err());

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM test", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_transaction_manager() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE test (id INTEGER PRIMARY KEY, value TEXT)")
            .unwrap();

        let manager = TransactionManager::new();
        let result = manager.execute(&mut conn, |tx| {
            tx.execute("INSERT INTO test (value) VALUES (?)", ["test"])?;
            Ok(())
        });

        assert!(result.is_ok());
    }
}
