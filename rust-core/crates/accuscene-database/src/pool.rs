//! Database connection pool management with health checks
//!
//! Provides a high-performance connection pool using r2d2 with automatic
//! health checks, connection lifecycle management, and performance tuning.

use crate::config::{DatabaseConfig, PerformanceConfig};
use crate::error::{DatabaseError, DbResult};
use parking_lot::RwLock;
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{Connection, OpenFlags};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

/// Connection pool for SQLite database
pub struct DatabasePool {
    pool: Pool<SqliteConnectionManager>,
    config: Arc<DatabaseConfig>,
    stats: Arc<RwLock<PoolStats>>,
}

/// Connection pool statistics
#[derive(Debug, Clone, Default)]
pub struct PoolStats {
    /// Total connections created
    pub connections_created: u64,
    /// Total connections closed
    pub connections_closed: u64,
    /// Total queries executed
    pub queries_executed: u64,
    /// Total errors encountered
    pub errors: u64,
    /// Last health check time
    pub last_health_check: Option<Instant>,
    /// Health check failures
    pub health_check_failures: u64,
}

impl DatabasePool {
    /// Create a new database pool
    pub fn new(config: DatabaseConfig) -> DbResult<Self> {
        config.validate()?;

        info!(
            "Creating database pool for '{}' with max_size={}",
            config.url, config.pool.max_size
        );

        let manager = SqliteConnectionManager::file(&config.url)
            .with_flags(
                OpenFlags::SQLITE_OPEN_READ_WRITE
                    | OpenFlags::SQLITE_OPEN_CREATE
                    | OpenFlags::SQLITE_OPEN_NO_MUTEX,
            )
            .with_init(move |conn| configure_connection(conn, &config.performance));

        let pool = Pool::builder()
            .max_size(config.pool.max_size)
            .min_idle(config.pool.min_idle)
            .connection_timeout(config.connection_timeout())
            .max_lifetime(config.max_lifetime())
            .idle_timeout(config.idle_timeout())
            .connection_customizer(Box::new(ConnectionCustomizer {
                config: config.clone(),
            }))
            .build(manager)
            .map_err(|e| DatabaseError::PoolError(e.to_string()))?;

        let pool_instance = Self {
            pool,
            config: Arc::new(config),
            stats: Arc::new(RwLock::new(PoolStats::default())),
        };

        // Perform initial health check
        pool_instance.health_check()?;

        info!("Database pool created successfully");

        Ok(pool_instance)
    }

    /// Get a connection from the pool
    pub fn get(&self) -> DbResult<PooledConnection<SqliteConnectionManager>> {
        debug!("Acquiring connection from pool");

        let start = Instant::now();
        let conn = self.pool.get().map_err(|e| {
            error!("Failed to get connection from pool: {}", e);
            self.stats.write().errors += 1;
            DatabaseError::PoolError(e.to_string())
        })?;

        let elapsed = start.elapsed();
        debug!("Connection acquired in {:?}", elapsed);

        if elapsed > Duration::from_secs(1) {
            warn!(
                "Slow connection acquisition: {:?} (pool state: {})",
                elapsed,
                self.pool.state()
            );
        }

        Ok(conn)
    }

    /// Perform a health check on the pool
    pub fn health_check(&self) -> DbResult<()> {
        debug!("Performing pool health check");

        let start = Instant::now();
        let conn = self.get()?;

        // Execute a simple query to verify the connection works
        conn.execute_batch("SELECT 1")
            .map_err(|e| {
                error!("Health check failed: {}", e);
                self.stats.write().health_check_failures += 1;
                DatabaseError::ConnectionError(format!("Health check failed: {}", e))
            })?;

        drop(conn);

        let elapsed = start.elapsed();
        debug!("Health check passed in {:?}", elapsed);

        let mut stats = self.stats.write();
        stats.last_health_check = Some(Instant::now());

        Ok(())
    }

    /// Get pool state information
    pub fn state(&self) -> PoolState {
        let state = self.pool.state();
        PoolState {
            connections: state.connections,
            idle_connections: state.idle_connections,
            max_size: self.config.pool.max_size,
        }
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        self.stats.read().clone()
    }

    /// Get database configuration
    pub fn config(&self) -> &DatabaseConfig {
        &self.config
    }

    /// Record a query execution
    pub(crate) fn record_query(&self) {
        self.stats.write().queries_executed += 1;
    }

    /// Record an error
    pub(crate) fn record_error(&self) {
        self.stats.write().errors += 1;
    }

    /// Close the pool gracefully
    pub fn close(self) {
        info!("Closing database pool");
        drop(self.pool);
        info!("Database pool closed");
    }

    /// Execute a function with a connection from the pool
    pub fn with_connection<F, T>(&self, f: F) -> DbResult<T>
    where
        F: FnOnce(&Connection) -> DbResult<T>,
    {
        let conn = self.get()?;
        let result = f(&*conn)?;
        Ok(result)
    }

    /// Execute a function with a mutable connection from the pool
    pub fn with_connection_mut<F, T>(&self, f: F) -> DbResult<T>
    where
        F: FnOnce(&mut Connection) -> DbResult<T>,
    {
        let mut conn = self.get()?;
        let result = f(&mut *conn)?;
        Ok(result)
    }
}

/// Pool state snapshot
#[derive(Debug, Clone)]
pub struct PoolState {
    /// Total number of connections
    pub connections: u32,
    /// Number of idle connections
    pub idle_connections: u32,
    /// Maximum pool size
    pub max_size: u32,
}

impl PoolState {
    /// Get the number of active connections
    pub fn active_connections(&self) -> u32 {
        self.connections.saturating_sub(self.idle_connections)
    }

    /// Check if the pool is at capacity
    pub fn is_at_capacity(&self) -> bool {
        self.connections >= self.max_size
    }

    /// Get pool utilization percentage
    pub fn utilization(&self) -> f64 {
        if self.max_size == 0 {
            0.0
        } else {
            (self.connections as f64 / self.max_size as f64) * 100.0
        }
    }
}

/// Connection customizer for r2d2
struct ConnectionCustomizer {
    config: DatabaseConfig,
}

impl r2d2::CustomizeConnection<Connection, rusqlite::Error> for ConnectionCustomizer {
    fn on_acquire(&self, conn: &mut Connection) -> Result<(), rusqlite::Error> {
        debug!("Connection acquired from pool");
        configure_connection(conn, &self.config.performance)
    }

    fn on_release(&self, _conn: Connection) {
        debug!("Connection released to pool");
    }
}

/// Configure a SQLite connection with performance settings
fn configure_connection(conn: &Connection, config: &PerformanceConfig) -> Result<(), rusqlite::Error> {
    // Set journal mode
    conn.pragma_update(None, "journal_mode", config.journal_mode.as_str())?;

    // Set synchronous mode
    conn.pragma_update(None, "synchronous", config.synchronous.as_str())?;

    // Set cache size
    conn.pragma_update(None, "cache_size", config.cache_size)?;

    // Set mmap size
    if config.mmap_size > 0 {
        conn.pragma_update(None, "mmap_size", config.mmap_size)?;
    }

    // Set page size (only takes effect on new databases)
    conn.pragma_update(None, "page_size", config.page_size)?;

    // Set auto vacuum
    conn.pragma_update(None, "auto_vacuum", config.auto_vacuum.as_str())?;

    // Set busy timeout
    conn.busy_timeout(Duration::from_millis(config.busy_timeout))?;

    // Enable foreign keys
    conn.pragma_update(None, "foreign_keys", "ON")?;

    // Optimize for write-ahead log if enabled
    if config.wal_mode {
        conn.pragma_update(None, "wal_autocheckpoint", 1000)?;
    }

    debug!("Connection configured with performance settings");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_pool() {
        let config = DatabaseConfig::test();
        let pool = DatabasePool::new(config).unwrap();
        assert_eq!(pool.state().max_size, 4);
    }

    #[test]
    fn test_get_connection() {
        let config = DatabaseConfig::test();
        let pool = DatabasePool::new(config).unwrap();
        let conn = pool.get().unwrap();

        // Verify connection works
        conn.execute_batch("SELECT 1").unwrap();
    }

    #[test]
    fn test_health_check() {
        let config = DatabaseConfig::test();
        let pool = DatabasePool::new(config).unwrap();
        pool.health_check().unwrap();

        let stats = pool.stats();
        assert!(stats.last_health_check.is_some());
    }

    #[test]
    fn test_pool_state() {
        let config = DatabaseConfig::test();
        let pool = DatabasePool::new(config).unwrap();

        let state = pool.state();
        assert_eq!(state.max_size, 4);
        assert!(state.utilization() <= 100.0);
    }

    #[test]
    fn test_with_connection() {
        let config = DatabaseConfig::test();
        let pool = DatabasePool::new(config).unwrap();

        let result = pool.with_connection(|conn| {
            conn.execute_batch("SELECT 1")?;
            Ok(42)
        }).unwrap();

        assert_eq!(result, 42);
    }
}
