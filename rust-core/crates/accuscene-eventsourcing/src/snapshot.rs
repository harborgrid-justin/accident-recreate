//! Snapshotting system for aggregates to improve performance.

use crate::aggregate::Aggregate;
use crate::error::{EventSourcingError, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Snapshot of an aggregate at a specific version.
#[derive(Serialize, Deserialize)]
#[serde(bound(deserialize = "A: serde::de::DeserializeOwned"))]
pub struct Snapshot<A>
where
    A: Aggregate,
{
    /// Aggregate identifier.
    pub aggregate_id: String,

    /// Aggregate type.
    pub aggregate_type: String,

    /// Version of the aggregate.
    pub version: u64,

    /// Serialized aggregate state.
    pub state: A,

    /// Timestamp when the snapshot was created.
    pub created_at: DateTime<Utc>,

    /// Metadata associated with the snapshot.
    pub metadata: SnapshotMetadata,
}

impl<A> Clone for Snapshot<A>
where
    A: Aggregate + Clone,
{
    fn clone(&self) -> Self {
        Self {
            aggregate_id: self.aggregate_id.clone(),
            aggregate_type: self.aggregate_type.clone(),
            version: self.version,
            state: self.state.clone(),
            created_at: self.created_at,
            metadata: self.metadata.clone(),
        }
    }
}

impl<A> std::fmt::Debug for Snapshot<A>
where
    A: Aggregate + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Snapshot")
            .field("aggregate_id", &self.aggregate_id)
            .field("aggregate_type", &self.aggregate_type)
            .field("version", &self.version)
            .field("state", &self.state)
            .field("created_at", &self.created_at)
            .field("metadata", &self.metadata)
            .finish()
    }
}

impl<A> Snapshot<A>
where
    A: Aggregate + Serialize + for<'de> Deserialize<'de>,
{
    /// Creates a new snapshot.
    pub fn new(aggregate: A, version: u64) -> Self {
        Self {
            aggregate_id: aggregate.aggregate_id().to_string(),
            aggregate_type: A::aggregate_type().to_string(),
            version,
            state: aggregate,
            created_at: Utc::now(),
            metadata: SnapshotMetadata::default(),
        }
    }

    /// Creates a snapshot with metadata.
    pub fn with_metadata(aggregate: A, version: u64, metadata: SnapshotMetadata) -> Self {
        Self {
            aggregate_id: aggregate.aggregate_id().to_string(),
            aggregate_type: A::aggregate_type().to_string(),
            version,
            state: aggregate,
            created_at: Utc::now(),
            metadata,
        }
    }

    /// Returns the aggregate state.
    pub fn aggregate(&self) -> &A {
        &self.state
    }

    /// Consumes the snapshot and returns the aggregate.
    pub fn into_aggregate(self) -> A {
        self.state
    }
}

/// Metadata associated with a snapshot.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SnapshotMetadata {
    /// Size of the snapshot in bytes.
    pub size_bytes: Option<usize>,

    /// Compression algorithm used.
    pub compression: Option<String>,

    /// Additional custom metadata.
    pub custom: std::collections::HashMap<String, String>,
}

impl Default for SnapshotMetadata {
    fn default() -> Self {
        Self {
            size_bytes: None,
            compression: None,
            custom: std::collections::HashMap::new(),
        }
    }
}

/// Trait for snapshot stores.
#[async_trait]
pub trait SnapshotStore<A>: Send + Sync
where
    A: Aggregate + Serialize + for<'de> Deserialize<'de>,
{
    /// Saves a snapshot.
    async fn save(&self, snapshot: &Snapshot<A>) -> Result<()>;

    /// Loads the latest snapshot for an aggregate.
    async fn load(&self, aggregate_id: &str) -> Result<Option<Snapshot<A>>>;

    /// Loads a snapshot at a specific version.
    async fn load_at_version(&self, aggregate_id: &str, version: u64) -> Result<Option<Snapshot<A>>>;

    /// Deletes snapshots for an aggregate.
    async fn delete(&self, aggregate_id: &str) -> Result<()>;

    /// Lists all snapshots for an aggregate.
    async fn list(&self, aggregate_id: &str) -> Result<Vec<SnapshotInfo>>;
}

/// Information about a snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotInfo {
    /// Aggregate identifier.
    pub aggregate_id: String,

    /// Version of the snapshot.
    pub version: u64,

    /// Timestamp when created.
    pub created_at: DateTime<Utc>,

    /// Size in bytes.
    pub size_bytes: Option<usize>,
}

/// In-memory snapshot store for testing and development.
pub struct InMemorySnapshotStore<A>
where
    A: Aggregate + Serialize + for<'de> Deserialize<'de>,
{
    snapshots: Arc<DashMap<String, Vec<Snapshot<A>>>>,
}

impl<A> InMemorySnapshotStore<A>
where
    A: Aggregate + Serialize + for<'de> Deserialize<'de>,
{
    /// Creates a new in-memory snapshot store.
    pub fn new() -> Self {
        Self {
            snapshots: Arc::new(DashMap::new()),
        }
    }

    /// Clears all snapshots.
    pub fn clear(&self) {
        self.snapshots.clear();
    }

    /// Returns the total number of snapshots.
    pub fn total_snapshots(&self) -> usize {
        self.snapshots
            .iter()
            .map(|entry| entry.value().len())
            .sum()
    }
}

impl<A> Default for InMemorySnapshotStore<A>
where
    A: Aggregate + Serialize + for<'de> Deserialize<'de>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<A> Clone for InMemorySnapshotStore<A>
where
    A: Aggregate + Serialize + for<'de> Deserialize<'de>,
{
    fn clone(&self) -> Self {
        Self {
            snapshots: Arc::clone(&self.snapshots),
        }
    }
}

#[async_trait]
impl<A> SnapshotStore<A> for InMemorySnapshotStore<A>
where
    A: Aggregate + Serialize + for<'de> Deserialize<'de> + Clone,
{
    async fn save(&self, snapshot: &Snapshot<A>) -> Result<()> {
        let aggregate_id = snapshot.aggregate_id.clone();

        self.snapshots
            .entry(aggregate_id)
            .or_insert_with(Vec::new)
            .push(snapshot.clone());

        Ok(())
    }

    async fn load(&self, aggregate_id: &str) -> Result<Option<Snapshot<A>>> {
        let snapshots = match self.snapshots.get(aggregate_id) {
            Some(s) => s,
            None => return Ok(None),
        };

        Ok(snapshots
            .iter()
            .max_by_key(|s| s.version)
            .cloned())
    }

    async fn load_at_version(&self, aggregate_id: &str, version: u64) -> Result<Option<Snapshot<A>>> {
        let snapshots = match self.snapshots.get(aggregate_id) {
            Some(s) => s,
            None => return Ok(None),
        };

        Ok(snapshots
            .iter()
            .filter(|s| s.version <= version)
            .max_by_key(|s| s.version)
            .cloned())
    }

    async fn delete(&self, aggregate_id: &str) -> Result<()> {
        self.snapshots.remove(aggregate_id);
        Ok(())
    }

    async fn list(&self, aggregate_id: &str) -> Result<Vec<SnapshotInfo>> {
        let snapshots = match self.snapshots.get(aggregate_id) {
            Some(s) => s,
            None => return Ok(Vec::new()),
        };

        Ok(snapshots
            .iter()
            .map(|s| SnapshotInfo {
                aggregate_id: s.aggregate_id.clone(),
                version: s.version,
                created_at: s.created_at,
                size_bytes: s.metadata.size_bytes,
            })
            .collect())
    }
}

/// Snapshot strategy for determining when to take snapshots.
#[derive(Debug, Clone, Copy)]
pub enum SnapshotStrategy {
    /// Take a snapshot every N events.
    EveryNEvents(u64),

    /// Never take snapshots.
    Never,

    /// Always take snapshots.
    Always,

    /// Take snapshots based on a custom threshold.
    Custom(u64),
}

impl SnapshotStrategy {
    /// Determines if a snapshot should be taken.
    pub fn should_snapshot(&self, version: u64, last_snapshot_version: u64) -> bool {
        match self {
            SnapshotStrategy::EveryNEvents(n) => {
                let events_since_last = version - last_snapshot_version;
                events_since_last >= *n
            }
            SnapshotStrategy::Never => false,
            SnapshotStrategy::Always => true,
            SnapshotStrategy::Custom(threshold) => {
                let events_since_last = version - last_snapshot_version;
                events_since_last >= *threshold
            }
        }
    }
}

impl Default for SnapshotStrategy {
    fn default() -> Self {
        SnapshotStrategy::EveryNEvents(100)
    }
}

/// Snapshot manager for coordinating snapshot operations.
pub struct SnapshotManager<A>
where
    A: Aggregate + Serialize + for<'de> Deserialize<'de>,
{
    store: Arc<dyn SnapshotStore<A>>,
    strategy: SnapshotStrategy,
}

impl<A> SnapshotManager<A>
where
    A: Aggregate + Serialize + for<'de> Deserialize<'de> + Clone,
{
    /// Creates a new snapshot manager.
    pub fn new(store: Arc<dyn SnapshotStore<A>>, strategy: SnapshotStrategy) -> Self {
        Self { store, strategy }
    }

    /// Checks if a snapshot should be taken and saves it if needed.
    pub async fn maybe_snapshot(
        &self,
        aggregate: &A,
        version: u64,
        last_snapshot_version: u64,
    ) -> Result<bool> {
        if self.strategy.should_snapshot(version, last_snapshot_version) {
            let snapshot = Snapshot::new(aggregate.clone(), version);
            self.store.save(&snapshot).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Loads the latest snapshot for an aggregate.
    pub async fn load_latest(&self, aggregate_id: &str) -> Result<Option<Snapshot<A>>> {
        self.store.load(aggregate_id).await
    }

    /// Loads a snapshot at a specific version.
    pub async fn load_at_version(&self, aggregate_id: &str, version: u64) -> Result<Option<Snapshot<A>>> {
        self.store.load_at_version(aggregate_id, version).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aggregate::AggregateId;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestAggregate {
        id: String,
        value: i32,
    }

    #[async_trait]
    impl Aggregate for TestAggregate {
        type Id = String;
        type Event = TestEvent;

        fn aggregate_type() -> &'static str {
            "TestAggregate"
        }

        fn aggregate_id(&self) -> &Self::Id {
            &self.id
        }

        fn version(&self) -> u64 {
            0
        }

        fn apply(&mut self, _event: &Self::Event) -> Result<()> {
            Ok(())
        }

        async fn handle(&self, _command: Box<dyn std::any::Any + Send>) -> Result<Vec<Self::Event>> {
            Ok(vec![])
        }

        fn default_state(id: Self::Id) -> Self {
            Self { id, value: 0 }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestEvent;

    impl crate::event::Event for TestEvent {
        fn event_type(&self) -> &'static str {
            "TestEvent"
        }

        fn aggregate_id(&self) -> &str {
            "test"
        }

        fn aggregate_type(&self) -> &'static str {
            "TestAggregate"
        }
    }

    #[tokio::test]
    async fn test_snapshot_creation() {
        let aggregate = TestAggregate {
            id: "test-1".to_string(),
            value: 42,
        };

        let snapshot = Snapshot::new(aggregate.clone(), 5);

        assert_eq!(snapshot.aggregate_id, "test-1");
        assert_eq!(snapshot.version, 5);
        assert_eq!(snapshot.state.value, 42);
    }

    #[tokio::test]
    async fn test_in_memory_snapshot_store() {
        let store = InMemorySnapshotStore::<TestAggregate>::new();

        let aggregate = TestAggregate {
            id: "test-1".to_string(),
            value: 42,
        };

        let snapshot = Snapshot::new(aggregate, 5);
        store.save(&snapshot).await.unwrap();

        let loaded = store.load("test-1").await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().version, 5);
    }

    #[test]
    fn test_snapshot_strategy() {
        let strategy = SnapshotStrategy::EveryNEvents(10);
        assert!(!strategy.should_snapshot(5, 0));
        assert!(strategy.should_snapshot(10, 0));
        assert!(strategy.should_snapshot(20, 10));
    }
}
