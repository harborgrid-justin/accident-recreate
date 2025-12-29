//! Storage engine components for durable data management.
//!
//! This module provides building blocks for a storage engine:
//! - Write-Ahead Log (WAL) for durability
//! - Multi-Version Concurrency Control (MVCC)
//! - Page management for disk-based storage
//! - LRU buffer pool for caching

pub mod buffer_pool;
pub mod mvcc;
pub mod page;
pub mod wal;

pub use buffer_pool::BufferPool;
pub use mvcc::{MvccEngine, Transaction, TransactionId};
pub use page::{Page, PageId};
pub use wal::{LogEntry, WriteAheadLog};

/// Common storage trait.
pub trait Storage {
    type Key;
    type Value;

    /// Read a value by key.
    fn read(&self, key: &Self::Key) -> crate::error::Result<Option<Self::Value>>;

    /// Write a value with key.
    fn write(&mut self, key: Self::Key, value: Self::Value) -> crate::error::Result<()>;

    /// Delete a value by key.
    fn delete(&mut self, key: &Self::Key) -> crate::error::Result<()>;

    /// Flush pending writes to durable storage.
    fn flush(&mut self) -> crate::error::Result<()>;
}
