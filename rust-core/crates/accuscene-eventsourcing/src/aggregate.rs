//! Aggregate root pattern for event sourcing.

use crate::error::{EventSourcingError, Result};
use crate::event::{Event, EventEnvelope};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display};

/// Trait for aggregate identifiers.
pub trait AggregateId: Debug + Display + Clone + Send + Sync + PartialEq + Eq + ToString {}

impl AggregateId for String {}

impl AggregateId for uuid::Uuid {}

/// Trait for aggregate roots in event sourcing.
#[async_trait]
pub trait Aggregate: Send + Sync + Debug + Clone {
    /// The aggregate's identifier type.
    type Id: AggregateId;

    /// The event type this aggregate handles.
    type Event: Event + Serialize + for<'de> Deserialize<'de>;

    /// Returns the aggregate type name.
    fn aggregate_type() -> &'static str;

    /// Returns the aggregate's unique identifier.
    fn aggregate_id(&self) -> &Self::Id;

    /// Returns the current version of the aggregate.
    fn version(&self) -> u64;

    /// Applies an event to the aggregate, updating its state.
    fn apply(&mut self, event: &Self::Event) -> Result<()>;

    /// Handles a command and returns the events to be persisted.
    async fn handle(&self, command: Box<dyn std::any::Any + Send>) -> Result<Vec<Self::Event>>;

    /// Creates a new aggregate with default state.
    fn default_state(id: Self::Id) -> Self;
}

/// Represents the state of an aggregate at a specific version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateState<A>
where
    A: Aggregate,
{
    /// The aggregate instance.
    pub aggregate: A,

    /// Current version of the aggregate.
    pub version: u64,

    /// Number of uncommitted events.
    pub uncommitted_events: Vec<A::Event>,
}

impl<A> AggregateState<A>
where
    A: Aggregate,
{
    /// Creates a new aggregate state.
    pub fn new(aggregate: A) -> Self {
        Self {
            aggregate,
            version: 0,
            uncommitted_events: Vec::new(),
        }
    }

    /// Creates an aggregate state with a specific version.
    pub fn with_version(aggregate: A, version: u64) -> Self {
        Self {
            aggregate,
            version,
            uncommitted_events: Vec::new(),
        }
    }

    /// Applies an event to the aggregate.
    pub fn apply_event(&mut self, event: A::Event) -> Result<()> {
        self.aggregate.apply(&event)?;
        self.version += 1;
        self.uncommitted_events.push(event);
        Ok(())
    }

    /// Applies multiple events to the aggregate.
    pub fn apply_events(&mut self, events: Vec<A::Event>) -> Result<()> {
        for event in events {
            self.apply_event(event)?;
        }
        Ok(())
    }

    /// Returns uncommitted events and clears the list.
    pub fn take_uncommitted_events(&mut self) -> Vec<A::Event> {
        std::mem::take(&mut self.uncommitted_events)
    }

    /// Returns the current version.
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Returns whether there are uncommitted events.
    pub fn has_uncommitted_events(&self) -> bool {
        !self.uncommitted_events.is_empty()
    }

    /// Loads events into the aggregate from history.
    pub fn load_from_history(&mut self, events: Vec<EventEnvelope<A::Event>>) -> Result<()> {
        for envelope in events {
            // Verify sequence
            if envelope.sequence() != self.version + 1 {
                return Err(EventSourcingError::InvalidSequence {
                    aggregate_id: self.aggregate.aggregate_id().to_string(),
                    expected: self.version + 1,
                    actual: envelope.sequence(),
                });
            }

            self.aggregate.apply(&envelope.payload)?;
            self.version = envelope.sequence();
        }
        Ok(())
    }
}

/// Repository trait for loading and saving aggregates.
#[async_trait]
pub trait AggregateRepository<A>: Send + Sync
where
    A: Aggregate,
{
    /// Loads an aggregate by ID.
    async fn load(&self, id: &A::Id) -> Result<AggregateState<A>>;

    /// Saves an aggregate's uncommitted events.
    async fn save(&self, state: &mut AggregateState<A>) -> Result<()>;

    /// Checks if an aggregate exists.
    async fn exists(&self, id: &A::Id) -> Result<bool>;

    /// Loads an aggregate at a specific version.
    async fn load_at_version(&self, id: &A::Id, version: u64) -> Result<AggregateState<A>>;
}

/// Version information for optimistic concurrency control.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Version(pub u64);

impl Version {
    /// Creates a new version.
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the next version.
    pub fn next(&self) -> Self {
        Self(self.0 + 1)
    }

    /// Returns the version number.
    pub fn value(&self) -> u64 {
        self.0
    }

    /// Initial version.
    pub const INITIAL: Version = Version(0);
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u64> for Version {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<Version> for u64 {
    fn from(version: Version) -> Self {
        version.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestEvent {
        aggregate_id: String,
        value: i32,
    }

    impl Event for TestEvent {
        fn event_type(&self) -> &'static str {
            "TestEvent"
        }

        fn aggregate_id(&self) -> &str {
            &self.aggregate_id
        }

        fn aggregate_type(&self) -> &'static str {
            "TestAggregate"
        }
    }

    #[derive(Debug, Clone)]
    struct TestAggregate {
        id: String,
        version: u64,
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
            self.version
        }

        fn apply(&mut self, event: &Self::Event) -> Result<()> {
            self.value += event.value;
            Ok(())
        }

        async fn handle(&self, _command: Box<dyn std::any::Any + Send>) -> Result<Vec<Self::Event>> {
            Ok(vec![])
        }

        fn default_state(id: Self::Id) -> Self {
            Self {
                id,
                version: 0,
                value: 0,
            }
        }
    }

    #[test]
    fn test_aggregate_state() {
        let aggregate = TestAggregate::default_state("test-1".to_string());
        let mut state = AggregateState::new(aggregate);

        let event = TestEvent {
            aggregate_id: "test-1".to_string(),
            value: 10,
        };

        state.apply_event(event).unwrap();

        assert_eq!(state.version(), 1);
        assert_eq!(state.aggregate.value, 10);
        assert!(state.has_uncommitted_events());
    }

    #[test]
    fn test_version() {
        let v1 = Version::new(1);
        let v2 = v1.next();

        assert_eq!(v1.value(), 1);
        assert_eq!(v2.value(), 2);
    }
}
