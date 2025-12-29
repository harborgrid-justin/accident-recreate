//! Multi-Version Concurrency Control (MVCC).
//!
//! Enables concurrent reads and writes without locks by maintaining
//! multiple versions of each data item.
//!
//! # Complexity
//! - Read: O(log v) where v is versions per key
//! - Write: O(log v) where v is versions per key
//! - Garbage collection: O(n * v) where n is keys, v is versions

use crate::config::MvccConfig;
use crate::error::{AlgorithmError, Result};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Transaction ID.
pub type TransactionId = u64;

/// Version number.
pub type Version = u64;

/// Versioned value.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct VersionedValue<V> {
    version: Version,
    value: Option<V>, // None represents deletion
    created_by: TransactionId,
    deleted_by: Option<TransactionId>,
}

/// Transaction state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TxnState {
    Active,
    Committed,
    Aborted,
}

/// Transaction metadata.
#[derive(Debug, Clone)]
pub struct Transaction {
    pub id: TransactionId,
    pub start_version: Version,
    pub state: TxnState,
}

impl Transaction {
    fn new(id: TransactionId, start_version: Version) -> Self {
        Self {
            id,
            start_version,
            state: TxnState::Active,
        }
    }
}

/// MVCC storage engine.
pub struct MvccEngine<K: Ord + Clone, V: Clone> {
    // Data: key -> sorted versions
    data: Arc<RwLock<HashMap<K, BTreeMap<Version, VersionedValue<V>>>>>,
    // Active transactions
    transactions: Arc<RwLock<HashMap<TransactionId, Transaction>>>,
    // Next transaction ID
    next_txn_id: Arc<AtomicU64>,
    // Current version number
    current_version: Arc<AtomicU64>,
    // Configuration
    config: MvccConfig,
}

impl<K: Ord + Clone, V: Clone> MvccEngine<K, V> {
    /// Create a new MVCC engine.
    pub fn new(config: MvccConfig) -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            transactions: Arc::new(RwLock::new(HashMap::new())),
            next_txn_id: Arc::new(AtomicU64::new(1)),
            current_version: Arc::new(AtomicU64::new(1)),
            config,
        }
    }

    /// Create with default configuration.
    pub fn default() -> Self {
        Self::new(MvccConfig::default())
    }

    /// Begin a new transaction.
    pub fn begin(&self) -> TransactionId {
        let txn_id = self.next_txn_id.fetch_add(1, Ordering::SeqCst);
        let start_version = self.current_version.load(Ordering::SeqCst);

        let txn = Transaction::new(txn_id, start_version);
        self.transactions.write().insert(txn_id, txn);

        txn_id
    }

    /// Read a value in a transaction (snapshot isolation).
    ///
    /// Returns the most recent committed version visible to this transaction.
    ///
    /// # Complexity
    /// O(log v) where v is versions per key
    pub fn read(&self, txn_id: TransactionId, key: &K) -> Result<Option<V>> {
        let transactions = self.transactions.read();
        let txn = transactions
            .get(&txn_id)
            .ok_or_else(|| AlgorithmError::MvccError("Transaction not found".to_string()))?;

        if txn.state != TxnState::Active {
            return Err(AlgorithmError::MvccError(format!(
                "Transaction {:?} not active",
                txn.state
            )));
        }

        let data = self.data.read();
        if let Some(versions) = data.get(key) {
            // Find the most recent version <= txn start version
            for (version, versioned_value) in versions.range(..=txn.start_version).rev() {
                // Check if this version is visible to our transaction
                if versioned_value.created_by < txn_id {
                    if let Some(deleted_by) = versioned_value.deleted_by {
                        if deleted_by > txn_id {
                            return Ok(versioned_value.value.clone());
                        }
                    } else {
                        return Ok(versioned_value.value.clone());
                    }
                }
            }
        }

        Ok(None)
    }

    /// Write a value in a transaction.
    ///
    /// # Complexity
    /// O(log v) where v is versions per key
    pub fn write(&self, txn_id: TransactionId, key: K, value: V) -> Result<()> {
        let transactions = self.transactions.read();
        let txn = transactions
            .get(&txn_id)
            .ok_or_else(|| AlgorithmError::MvccError("Transaction not found".to_string()))?;

        if txn.state != TxnState::Active {
            return Err(AlgorithmError::MvccError(
                "Transaction not active".to_string(),
            ));
        }

        let version = self.current_version.fetch_add(1, Ordering::SeqCst);

        let versioned_value = VersionedValue {
            version,
            value: Some(value),
            created_by: txn_id,
            deleted_by: None,
        };

        let mut data = self.data.write();
        data.entry(key)
            .or_insert_with(BTreeMap::new)
            .insert(version, versioned_value);

        Ok(())
    }

    /// Delete a value in a transaction.
    pub fn delete(&self, txn_id: TransactionId, key: K) -> Result<()> {
        let transactions = self.transactions.read();
        let txn = transactions
            .get(&txn_id)
            .ok_or_else(|| AlgorithmError::MvccError("Transaction not found".to_string()))?;

        if txn.state != TxnState::Active {
            return Err(AlgorithmError::MvccError(
                "Transaction not active".to_string(),
            ));
        }

        let version = self.current_version.fetch_add(1, Ordering::SeqCst);

        let versioned_value = VersionedValue {
            version,
            value: None,
            created_by: txn_id,
            deleted_by: Some(txn_id),
        };

        let mut data = self.data.write();
        data.entry(key)
            .or_insert_with(BTreeMap::new)
            .insert(version, versioned_value);

        Ok(())
    }

    /// Commit a transaction.
    pub fn commit(&self, txn_id: TransactionId) -> Result<()> {
        let mut transactions = self.transactions.write();
        if let Some(txn) = transactions.get_mut(&txn_id) {
            if txn.state != TxnState::Active {
                return Err(AlgorithmError::MvccError(
                    "Transaction not active".to_string(),
                ));
            }
            txn.state = TxnState::Committed;
            Ok(())
        } else {
            Err(AlgorithmError::MvccError(
                "Transaction not found".to_string(),
            ))
        }
    }

    /// Abort a transaction.
    pub fn abort(&self, txn_id: TransactionId) -> Result<()> {
        let mut transactions = self.transactions.write();
        if let Some(txn) = transactions.get_mut(&txn_id) {
            if txn.state != TxnState::Active {
                return Err(AlgorithmError::MvccError(
                    "Transaction not active".to_string(),
                ));
            }
            txn.state = TxnState::Aborted;

            // Remove versions created by this transaction
            let mut data = self.data.write();
            for versions in data.values_mut() {
                versions.retain(|_, v| v.created_by != txn_id);
            }

            Ok(())
        } else {
            Err(AlgorithmError::MvccError(
                "Transaction not found".to_string(),
            ))
        }
    }

    /// Garbage collect old versions.
    ///
    /// Removes versions that are no longer visible to any active transaction.
    ///
    /// # Complexity
    /// O(n * v) where n is keys, v is versions per key
    pub fn garbage_collect(&self) -> usize {
        let transactions = self.transactions.read();

        // Find minimum start version of active transactions
        let min_active_version = transactions
            .values()
            .filter(|txn| txn.state == TxnState::Active)
            .map(|txn| txn.start_version)
            .min()
            .unwrap_or(self.current_version.load(Ordering::SeqCst));

        let mut data = self.data.write();
        let mut removed_count = 0;

        for versions in data.values_mut() {
            let original_len = versions.len();

            // Keep only versions that might be visible
            if versions.len() > self.config.max_versions {
                let keep_from = versions
                    .range(..min_active_version)
                    .next_back()
                    .map(|(&v, _)| v)
                    .unwrap_or(0);

                versions.retain(|&v, _| v >= keep_from);
            }

            removed_count += original_len - versions.len();
        }

        // Clean up empty entries
        data.retain(|_, versions| !versions.is_empty());

        removed_count
    }

    /// Get transaction state.
    pub fn transaction_state(&self, txn_id: TransactionId) -> Option<TxnState> {
        self.transactions.read().get(&txn_id).map(|txn| txn.state)
    }

    /// Get number of active transactions.
    pub fn active_transaction_count(&self) -> usize {
        self.transactions
            .read()
            .values()
            .filter(|txn| txn.state == TxnState::Active)
            .count()
    }

    /// Get total version count across all keys.
    pub fn version_count(&self) -> usize {
        self.data.read().values().map(|v| v.len()).sum()
    }
}

impl<K: Ord + Clone, V: Clone> Clone for MvccEngine<K, V> {
    fn clone(&self) -> Self {
        Self {
            data: Arc::new(RwLock::new(self.data.read().clone())),
            transactions: Arc::new(RwLock::new(self.transactions.read().clone())),
            next_txn_id: Arc::new(AtomicU64::new(self.next_txn_id.load(Ordering::SeqCst))),
            current_version: Arc::new(AtomicU64::new(self.current_version.load(Ordering::SeqCst))),
            config: self.config.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_transaction() {
        let mvcc = MvccEngine::default();

        let txn = mvcc.begin();
        mvcc.write(txn, "key".to_string(), "value".to_string())
            .unwrap();
        mvcc.commit(txn).unwrap();

        let txn2 = mvcc.begin();
        let value = mvcc.read(txn2, &"key".to_string()).unwrap();
        assert_eq!(value, Some("value".to_string()));
    }

    #[test]
    fn test_snapshot_isolation() {
        let mvcc = MvccEngine::default();

        // Transaction 1 writes
        let txn1 = mvcc.begin();
        mvcc.write(txn1, "key".to_string(), "v1".to_string())
            .unwrap();
        mvcc.commit(txn1).unwrap();

        // Transaction 2 starts (should see v1)
        let txn2 = mvcc.begin();

        // Transaction 3 writes (after txn2 started)
        let txn3 = mvcc.begin();
        mvcc.write(txn3, "key".to_string(), "v2".to_string())
            .unwrap();
        mvcc.commit(txn3).unwrap();

        // Transaction 2 should still see v1 (snapshot isolation)
        let value = mvcc.read(txn2, &"key".to_string()).unwrap();
        assert_eq!(value, Some("v1".to_string()));
    }

    #[test]
    fn test_abort() {
        let mvcc = MvccEngine::default();

        let txn = mvcc.begin();
        mvcc.write(txn, "key".to_string(), "value".to_string())
            .unwrap();
        mvcc.abort(txn).unwrap();

        let txn2 = mvcc.begin();
        let value = mvcc.read(txn2, &"key".to_string()).unwrap();
        assert_eq!(value, None);
    }
}
