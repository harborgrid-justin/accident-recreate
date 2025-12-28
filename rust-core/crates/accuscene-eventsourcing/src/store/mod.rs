//! Event store implementations.

use crate::error::Result;
use crate::event::{Event, EventEnvelope, SerializedEvent};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[cfg(feature = "memory")]
pub mod memory;

#[cfg(feature = "postgres")]
pub mod postgres;

/// Trait for event stores.
#[async_trait]
pub trait EventStore: Send + Sync {
    /// Appends events to the store.
    async fn append_events<E>(&self, events: Vec<EventEnvelope<E>>) -> Result<()>
    where
        E: Event + Serialize + Send + Sync;

    /// Loads events for an aggregate.
    async fn load_events<E>(
        &self,
        aggregate_id: &str,
        from_sequence: u64,
    ) -> Result<Vec<EventEnvelope<E>>>
    where
        E: Event + for<'de> Deserialize<'de>;

    /// Loads all events for an aggregate.
    async fn load_all_events<E>(&self, aggregate_id: &str) -> Result<Vec<EventEnvelope<E>>>
    where
        E: Event + for<'de> Deserialize<'de>,
    {
        self.load_events(aggregate_id, 0).await
    }

    /// Gets the current version of an aggregate.
    async fn get_version(&self, aggregate_id: &str) -> Result<u64>;

    /// Checks if an aggregate exists.
    async fn exists(&self, aggregate_id: &str) -> Result<bool> {
        Ok(self.get_version(aggregate_id).await.is_ok())
    }

    /// Loads events in a specific range.
    async fn load_events_range<E>(
        &self,
        aggregate_id: &str,
        from_sequence: u64,
        to_sequence: u64,
    ) -> Result<Vec<EventEnvelope<E>>>
    where
        E: Event + for<'de> Deserialize<'de>;

    /// Streams all events of a specific type.
    async fn stream_events_by_type<E>(&self, event_type: &str) -> Result<Vec<EventEnvelope<E>>>
    where
        E: Event + for<'de> Deserialize<'de>;

    /// Deletes events for an aggregate (use with caution).
    async fn delete_events(&self, aggregate_id: &str) -> Result<()>;
}

/// Options for appending events with optimistic concurrency control.
#[derive(Debug, Clone)]
pub struct AppendOptions {
    /// Expected version of the aggregate.
    pub expected_version: Option<u64>,

    /// Whether to check for version conflicts.
    pub check_version: bool,

    /// Metadata to attach to all events.
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

impl Default for AppendOptions {
    fn default() -> Self {
        Self {
            expected_version: None,
            check_version: true,
            metadata: None,
        }
    }
}

impl AppendOptions {
    /// Creates new append options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the expected version.
    pub fn with_expected_version(mut self, version: u64) -> Self {
        self.expected_version = Some(version);
        self.check_version = true;
        self
    }

    /// Disables version checking.
    pub fn without_version_check(mut self) -> Self {
        self.check_version = false;
        self
    }

    /// Adds metadata.
    pub fn with_metadata(
        mut self,
        metadata: std::collections::HashMap<String, String>,
    ) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// Event stream for reading events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventStream {
    /// Stream identifier.
    pub stream_id: String,

    /// Current version of the stream.
    pub version: u64,

    /// Events in the stream.
    pub events: Vec<SerializedEvent>,
}

impl EventStream {
    /// Creates a new event stream.
    pub fn new(stream_id: impl Into<String>) -> Self {
        Self {
            stream_id: stream_id.into(),
            version: 0,
            events: Vec::new(),
        }
    }

    /// Appends a serialized event to the stream.
    pub fn append(&mut self, event: SerializedEvent) {
        self.version += 1;
        self.events.push(event);
    }

    /// Returns the number of events in the stream.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Returns whether the stream is empty.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_append_options() {
        let options = AppendOptions::new()
            .with_expected_version(5)
            .with_metadata([("key".to_string(), "value".to_string())].into());

        assert_eq!(options.expected_version, Some(5));
        assert!(options.check_version);
        assert!(options.metadata.is_some());
    }

    #[test]
    fn test_event_stream() {
        let mut stream = EventStream::new("test-stream");
        assert_eq!(stream.version, 0);
        assert!(stream.is_empty());
    }
}
