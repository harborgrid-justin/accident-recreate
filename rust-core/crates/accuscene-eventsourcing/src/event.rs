//! Event trait and envelope for event sourcing.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use uuid::Uuid;

/// Trait that all events must implement.
pub trait Event: Send + Sync + Debug + Clone {
    /// Returns the event type identifier.
    fn event_type(&self) -> &'static str;

    /// Returns the aggregate ID this event belongs to.
    fn aggregate_id(&self) -> &str;

    /// Returns the aggregate type.
    fn aggregate_type(&self) -> &'static str;

    /// Serializes the event to JSON.
    fn to_json(&self) -> serde_json::Result<String>
    where
        Self: Serialize,
    {
        serde_json::to_string(self)
    }

    /// Serializes the event to binary format.
    fn to_binary(&self) -> bincode::Result<Vec<u8>>
    where
        Self: Serialize,
    {
        bincode::serialize(self)
    }
}

/// Metadata associated with an event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventMetadata {
    /// Unique event identifier.
    pub event_id: Uuid,

    /// Event type identifier.
    pub event_type: String,

    /// Aggregate identifier.
    pub aggregate_id: String,

    /// Aggregate type.
    pub aggregate_type: String,

    /// Event sequence number for the aggregate.
    pub sequence: u64,

    /// Timestamp when the event occurred.
    pub timestamp: DateTime<Utc>,

    /// Correlation ID for tracking related events.
    pub correlation_id: Option<Uuid>,

    /// Causation ID for tracking event causality.
    pub causation_id: Option<Uuid>,

    /// User or service that caused the event.
    pub actor: Option<String>,

    /// Additional custom metadata.
    pub custom: HashMap<String, String>,
}

impl EventMetadata {
    /// Creates new event metadata.
    pub fn new(
        event_type: impl Into<String>,
        aggregate_id: impl Into<String>,
        aggregate_type: impl Into<String>,
        sequence: u64,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            event_type: event_type.into(),
            aggregate_id: aggregate_id.into(),
            aggregate_type: aggregate_type.into(),
            sequence,
            timestamp: Utc::now(),
            correlation_id: None,
            causation_id: None,
            actor: None,
            custom: HashMap::new(),
        }
    }

    /// Sets the correlation ID.
    pub fn with_correlation_id(mut self, id: Uuid) -> Self {
        self.correlation_id = Some(id);
        self
    }

    /// Sets the causation ID.
    pub fn with_causation_id(mut self, id: Uuid) -> Self {
        self.causation_id = Some(id);
        self
    }

    /// Sets the actor.
    pub fn with_actor(mut self, actor: impl Into<String>) -> Self {
        self.actor = Some(actor.into());
        self
    }

    /// Adds custom metadata.
    pub fn with_custom(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.custom.insert(key.into(), value.into());
        self
    }
}

/// Envelope containing an event and its metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope<E> {
    /// Event metadata.
    pub metadata: EventMetadata,

    /// The actual event payload.
    pub payload: E,
}

impl<E> EventEnvelope<E>
where
    E: Event,
{
    /// Creates a new event envelope.
    pub fn new(payload: E, sequence: u64) -> Self {
        let metadata = EventMetadata::new(
            payload.event_type(),
            payload.aggregate_id(),
            payload.aggregate_type(),
            sequence,
        );

        Self { metadata, payload }
    }

    /// Creates a new event envelope with metadata.
    pub fn with_metadata(payload: E, metadata: EventMetadata) -> Self {
        Self { metadata, payload }
    }

    /// Returns the event ID.
    pub fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    /// Returns the aggregate ID.
    pub fn aggregate_id(&self) -> &str {
        &self.metadata.aggregate_id
    }

    /// Returns the sequence number.
    pub fn sequence(&self) -> u64 {
        self.metadata.sequence
    }

    /// Returns the timestamp.
    pub fn timestamp(&self) -> DateTime<Utc> {
        self.metadata.timestamp
    }

    /// Sets the correlation ID.
    pub fn with_correlation_id(mut self, id: Uuid) -> Self {
        self.metadata.correlation_id = Some(id);
        self
    }

    /// Sets the causation ID.
    pub fn with_causation_id(mut self, id: Uuid) -> Self {
        self.metadata.causation_id = Some(id);
        self
    }

    /// Sets the actor.
    pub fn with_actor(mut self, actor: impl Into<String>) -> Self {
        self.metadata.actor = Some(actor.into());
        self
    }
}

/// Serialized event envelope for storage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedEvent {
    /// Event metadata.
    pub metadata: EventMetadata,

    /// Serialized event payload (JSON).
    pub payload_json: String,

    /// Serialized event payload (binary).
    pub payload_binary: Option<Vec<u8>>,
}

impl SerializedEvent {
    /// Creates a new serialized event from an envelope.
    pub fn from_envelope<E>(envelope: &EventEnvelope<E>) -> Result<Self, serde_json::Error>
    where
        E: Event + Serialize,
    {
        let payload_json = serde_json::to_string(&envelope.payload)?;
        let payload_binary = bincode::serialize(&envelope.payload).ok();

        Ok(Self {
            metadata: envelope.metadata.clone(),
            payload_json,
            payload_binary,
        })
    }

    /// Deserializes the event payload.
    pub fn deserialize_payload<E>(&self) -> Result<E, serde_json::Error>
    where
        E: for<'de> Deserialize<'de>,
    {
        serde_json::from_str(&self.payload_json)
    }

    /// Deserializes the event payload from binary if available.
    pub fn deserialize_payload_binary<E>(&self) -> Option<Result<E, bincode::Error>>
    where
        E: for<'de> Deserialize<'de>,
    {
        self.payload_binary
            .as_ref()
            .map(|data| bincode::deserialize(data))
    }

    /// Converts back to an event envelope.
    pub fn to_envelope<E>(&self) -> Result<EventEnvelope<E>, serde_json::Error>
    where
        E: for<'de> Deserialize<'de>,
    {
        let payload = self.deserialize_payload()?;
        Ok(EventEnvelope {
            metadata: self.metadata.clone(),
            payload,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestEvent {
        id: String,
        data: String,
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

    #[test]
    fn test_event_envelope_creation() {
        let event = TestEvent {
            id: "test-123".to_string(),
            data: "test data".to_string(),
        };

        let envelope = EventEnvelope::new(event.clone(), 1);

        assert_eq!(envelope.aggregate_id(), "test-123");
        assert_eq!(envelope.sequence(), 1);
        assert_eq!(envelope.payload, event);
    }

    #[test]
    fn test_event_metadata() {
        let metadata = EventMetadata::new("TestEvent", "test-123", "Test", 1)
            .with_actor("user-456")
            .with_custom("key", "value");

        assert_eq!(metadata.event_type, "TestEvent");
        assert_eq!(metadata.aggregate_id, "test-123");
        assert_eq!(metadata.sequence, 1);
        assert_eq!(metadata.actor, Some("user-456".to_string()));
        assert_eq!(metadata.custom.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_serialized_event() {
        let event = TestEvent {
            id: "test-123".to_string(),
            data: "test data".to_string(),
        };

        let envelope = EventEnvelope::new(event.clone(), 1);
        let serialized = SerializedEvent::from_envelope(&envelope).unwrap();

        let deserialized: TestEvent = serialized.deserialize_payload().unwrap();
        assert_eq!(deserialized, event);
    }
}
