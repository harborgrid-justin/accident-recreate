//! In-memory event store implementation.

use crate::error::{EventSourcingError, Result};
use crate::event::{Event, EventEnvelope, SerializedEvent};
use crate::store::EventStore;
use async_trait::async_trait;
use dashmap::DashMap;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// In-memory event store for testing and development.
#[derive(Debug, Clone)]
pub struct InMemoryEventStore {
    /// Events indexed by aggregate ID.
    events: Arc<DashMap<String, Arc<RwLock<Vec<SerializedEvent>>>>>,

    /// Global event log for projection rebuilding.
    global_log: Arc<RwLock<Vec<SerializedEvent>>>,
}

impl InMemoryEventStore {
    /// Creates a new in-memory event store.
    pub fn new() -> Self {
        Self {
            events: Arc::new(DashMap::new()),
            global_log: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Returns the total number of events stored.
    pub fn total_events(&self) -> usize {
        self.global_log.read().len()
    }

    /// Returns the number of aggregates stored.
    pub fn total_aggregates(&self) -> usize {
        self.events.len()
    }

    /// Clears all events from the store.
    pub fn clear(&self) {
        self.events.clear();
        self.global_log.write().clear();
    }

    /// Returns all events in the global log.
    pub fn get_global_log(&self) -> Vec<SerializedEvent> {
        self.global_log.read().clone()
    }

    /// Returns events for a specific aggregate.
    pub fn get_aggregate_events(&self, aggregate_id: &str) -> Option<Vec<SerializedEvent>> {
        self.events
            .get(aggregate_id)
            .map(|events| events.read().clone())
    }
}

impl Default for InMemoryEventStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventStore for InMemoryEventStore {
    async fn append_events<E>(&self, events: Vec<EventEnvelope<E>>) -> Result<()>
    where
        E: Event + Serialize + Send + Sync,
    {
        if events.is_empty() {
            return Ok(());
        }

        let aggregate_id = events[0].aggregate_id().to_string();

        // Get or create the event list for this aggregate
        let event_list = self
            .events
            .entry(aggregate_id.clone())
            .or_insert_with(|| Arc::new(RwLock::new(Vec::new())))
            .clone();

        let mut list = event_list.write();

        // Check version consistency
        let current_version = list.len() as u64;
        for (i, envelope) in events.iter().enumerate() {
            let expected_sequence = current_version + i as u64 + 1;
            if envelope.sequence() != expected_sequence {
                return Err(EventSourcingError::InvalidSequence {
                    aggregate_id: aggregate_id.clone(),
                    expected: expected_sequence,
                    actual: envelope.sequence(),
                });
            }
        }

        // Serialize and store events
        let mut global_log = self.global_log.write();
        for envelope in events {
            let serialized = SerializedEvent::from_envelope(&envelope)
                .map_err(EventSourcingError::serialization)?;

            list.push(serialized.clone());
            global_log.push(serialized);
        }

        Ok(())
    }

    async fn load_events<E>(
        &self,
        aggregate_id: &str,
        from_sequence: u64,
    ) -> Result<Vec<EventEnvelope<E>>>
    where
        E: Event + for<'de> Deserialize<'de>,
    {
        let events = self
            .events
            .get(aggregate_id)
            .ok_or_else(|| EventSourcingError::AggregateNotFound(aggregate_id.to_string()))?;

        let list = events.read();
        let mut result = Vec::new();

        for serialized in list.iter() {
            if serialized.metadata.sequence >= from_sequence {
                let envelope = serialized
                    .to_envelope()
                    .map_err(EventSourcingError::deserialization)?;
                result.push(envelope);
            }
        }

        Ok(result)
    }

    async fn get_version(&self, aggregate_id: &str) -> Result<u64> {
        let events = self
            .events
            .get(aggregate_id)
            .ok_or_else(|| EventSourcingError::AggregateNotFound(aggregate_id.to_string()))?;

        let version = events.read().len() as u64;
        Ok(version)
    }

    async fn load_events_range<E>(
        &self,
        aggregate_id: &str,
        from_sequence: u64,
        to_sequence: u64,
    ) -> Result<Vec<EventEnvelope<E>>>
    where
        E: Event + for<'de> Deserialize<'de>,
    {
        let events = self
            .events
            .get(aggregate_id)
            .ok_or_else(|| EventSourcingError::AggregateNotFound(aggregate_id.to_string()))?;

        let list = events.read();
        let mut result = Vec::new();

        for serialized in list.iter() {
            let seq = serialized.metadata.sequence;
            if seq >= from_sequence && seq <= to_sequence {
                let envelope = serialized
                    .to_envelope()
                    .map_err(EventSourcingError::deserialization)?;
                result.push(envelope);
            }
        }

        Ok(result)
    }

    async fn stream_events_by_type<E>(&self, event_type: &str) -> Result<Vec<EventEnvelope<E>>>
    where
        E: Event + for<'de> Deserialize<'de>,
    {
        let global_log = self.global_log.read();
        let mut result = Vec::new();

        for serialized in global_log.iter() {
            if serialized.metadata.event_type == event_type {
                let envelope = serialized
                    .to_envelope()
                    .map_err(EventSourcingError::deserialization)?;
                result.push(envelope);
            }
        }

        Ok(result)
    }

    async fn delete_events(&self, aggregate_id: &str) -> Result<()> {
        self.events.remove(aggregate_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::EventMetadata;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestEvent {
        id: String,
        value: i32,
    }

    impl Event for TestEvent {
        fn event_type(&self) -> &'static str {
            "TestEvent"
        }

        fn aggregate_id(&self) -> &str {
            &self.id
        }

        fn aggregate_type(&self) -> &'static str {
            "Test"
        }
    }

    #[tokio::test]
    async fn test_append_and_load_events() {
        let store = InMemoryEventStore::new();

        let event = TestEvent {
            id: "test-1".to_string(),
            value: 42,
        };

        let envelope = EventEnvelope::new(event.clone(), 1);
        store.append_events(vec![envelope]).await.unwrap();

        let loaded: Vec<EventEnvelope<TestEvent>> =
            store.load_all_events("test-1").await.unwrap();

        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].payload, event);
    }

    #[tokio::test]
    async fn test_version_tracking() {
        let store = InMemoryEventStore::new();

        let event1 = TestEvent {
            id: "test-1".to_string(),
            value: 1,
        };
        let event2 = TestEvent {
            id: "test-1".to_string(),
            value: 2,
        };

        store
            .append_events(vec![EventEnvelope::new(event1, 1)])
            .await
            .unwrap();
        store
            .append_events(vec![EventEnvelope::new(event2, 2)])
            .await
            .unwrap();

        let version = store.get_version("test-1").await.unwrap();
        assert_eq!(version, 2);
    }

    #[tokio::test]
    async fn test_sequence_validation() {
        let store = InMemoryEventStore::new();

        let event = TestEvent {
            id: "test-1".to_string(),
            value: 1,
        };

        // Try to append with wrong sequence
        let result = store
            .append_events(vec![EventEnvelope::new(event, 5)])
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_load_events_range() {
        let store = InMemoryEventStore::new();

        for i in 1..=5 {
            let event = TestEvent {
                id: "test-1".to_string(),
                value: i,
            };
            store
                .append_events(vec![EventEnvelope::new(event, i as u64)])
                .await
                .unwrap();
        }

        let events: Vec<EventEnvelope<TestEvent>> =
            store.load_events_range("test-1", 2, 4).await.unwrap();

        assert_eq!(events.len(), 3);
        assert_eq!(events[0].payload.value, 2);
        assert_eq!(events[2].payload.value, 4);
    }

    #[tokio::test]
    async fn test_clear_store() {
        let store = InMemoryEventStore::new();

        let event = TestEvent {
            id: "test-1".to_string(),
            value: 42,
        };

        store
            .append_events(vec![EventEnvelope::new(event, 1)])
            .await
            .unwrap();

        assert_eq!(store.total_events(), 1);

        store.clear();

        assert_eq!(store.total_events(), 0);
        assert_eq!(store.total_aggregates(), 0);
    }
}
