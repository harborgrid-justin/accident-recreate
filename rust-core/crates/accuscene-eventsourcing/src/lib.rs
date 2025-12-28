//! Event Sourcing and CQRS system for AccuScene Enterprise.
//!
//! This crate provides a complete implementation of Event Sourcing and CQRS patterns
//! for building scalable, auditable, and event-driven applications.
//!
//! # Features
//!
//! - **Event Sourcing**: Complete event store with memory and PostgreSQL implementations
//! - **CQRS**: Separate command and query models for optimal performance
//! - **Aggregates**: Aggregate root pattern with version control
//! - **Projections**: Build read models from event streams
//! - **Snapshots**: Optimize aggregate loading with periodic snapshots
//! - **Event Bus**: Publish-subscribe pattern for event handling
//! - **Command Bus**: Route commands to appropriate handlers
//! - **Sagas**: Orchestrate long-running business processes
//! - **Domain Events**: Pre-built events for accident reconstruction domain
//!
//! # Example
//!
//! ```rust,no_run
//! use accuscene_eventsourcing::{
//!     store::memory::InMemoryEventStore,
//!     event::{Event, EventEnvelope},
//! };
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create an in-memory event store
//! let store = InMemoryEventStore::new();
//!
//! // Store events
//! // let envelope = EventEnvelope::new(my_event, 1);
//! // store.append_events(vec![envelope]).await?;
//!
//! // Load events
//! // let events = store.load_all_events("aggregate-id").await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Architecture
//!
//! The crate is organized into the following modules:
//!
//! - `event`: Core event types and envelopes
//! - `aggregate`: Aggregate root pattern and repository
//! - `store`: Event store trait and implementations
//! - `projection`: Projection system for read models
//! - `snapshot`: Snapshot system for performance optimization
//! - `command`: Command handling (write side of CQRS)
//! - `query`: Query handling (read side of CQRS)
//! - `bus`: Event bus and command bus implementations
//! - `saga`: Saga pattern for distributed transactions
//! - `config`: Configuration structures
//! - `domain`: Domain-specific events for accident reconstruction

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod aggregate;
pub mod bus;
pub mod command;
pub mod config;
pub mod domain;
pub mod error;
pub mod event;
pub mod projection;
pub mod query;
pub mod saga;
pub mod snapshot;
pub mod store;

#[cfg(test)]
mod tests;

// Re-export commonly used types
pub use error::{EventSourcingError, Result};
pub use event::{Event, EventEnvelope, EventMetadata};
pub use aggregate::{Aggregate, AggregateId, AggregateState};
pub use store::EventStore;

/// Prelude module for convenient imports.
pub mod prelude {
    pub use crate::aggregate::{Aggregate, AggregateId, AggregateRepository, AggregateState, Version};
    pub use crate::bus::{CommandBus, EventBus, EventSubscriber};
    pub use crate::command::{Command, CommandEnvelope, CommandHandler, CommandResult};
    pub use crate::config::EventSourcingConfig;
    pub use crate::error::{EventSourcingError, Result};
    pub use crate::event::{Event, EventEnvelope, EventMetadata, SerializedEvent};
    pub use crate::projection::{Projection, ProjectionManager, ProjectionState, ProjectionStatus};
    pub use crate::query::{Query, QueryEnvelope, QueryHandler, QueryResult};
    pub use crate::saga::{Saga, SagaAction, SagaInstance, SagaManager, SagaState};
    pub use crate::snapshot::{Snapshot, SnapshotStore, SnapshotStrategy};
    pub use crate::store::EventStore;

    #[cfg(feature = "memory")]
    pub use crate::store::memory::InMemoryEventStore;

    #[cfg(feature = "postgres")]
    pub use crate::store::postgres::PostgresEventStore;
}

/// Version information for the crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Returns the version string.
pub fn version() -> &'static str {
    VERSION
}

#[cfg(test)]
mod lib_tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!version().is_empty());
    }
}
