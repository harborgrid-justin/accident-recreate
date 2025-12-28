//! # AccuScene Streaming v0.2.0
//!
//! Real-time distributed streaming pipeline for AccuScene Enterprise.
//!
//! ## Features
//!
//! ### Real-time Streaming Infrastructure
//! - **Stream Processing**: High-performance streaming traits and operators
//! - **Source Connectors**: Channel, file, WebSocket, and iterator sources
//! - **Sink Connectors**: Channel, file, WebSocket, and Parquet sinks
//! - **Operators**: Map, filter, flatmap, window, join, aggregate, and keyby
//! - **Backpressure**: Adaptive flow control and backpressure handling
//! - **Watermarks**: Event-time processing with watermark support
//! - **Checkpointing**: Fault tolerance through state checkpointing
//! - **State Management**: Stateful processing with multiple state backends
//! - **Partitioning**: Distributed processing with flexible partitioning strategies
//! - **Apache Arrow**: Columnar data processing with Parquet support
//!
//! ### Legacy Event System
//! - **Event Bus**: Multiple event bus implementations (memory, channel, broadcast)
//! - **Pub/Sub**: Topic-based publish/subscribe messaging
//! - **WebSocket**: Server and client implementations with protocol support
//! - **Event Replay**: Synchronization through event replay
//! - **Presence Tracking**: User presence and activity monitoring
//! - **Room Management**: Per-case collaboration rooms
//! - **Heartbeat**: Connection health monitoring
//! - **Compression**: Message compression for bandwidth optimization
//! - **Authentication**: Token-based authentication and authorization
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use accuscene_streaming::{
//!     bus::{EventBus, EventBusBuilder, EventBusType},
//!     event::{Event, EventType, EventPayload},
//! };
//!
//! #[tokio::main]
//! async fn main() {
//!     // Create an event bus
//!     let bus = EventBusBuilder::new(EventBusType::Broadcast)
//!         .with_capacity(1000)
//!         .build();
//!
//!     // Subscribe to events
//!     let mut stream = bus.subscribe_all().await.unwrap();
//!
//!     // Publish an event
//!     let event = Event::new(EventType::SimulationUpdate, EventPayload::Empty);
//!     bus.publish(event).await.unwrap();
//! }
//! ```

// Core modules
pub mod error;
pub mod event;
pub mod config;

// Streaming infrastructure (v0.2.0)
pub mod stream;
pub mod source;
pub mod sink;
pub mod operators;
pub mod buffer;
pub mod backpressure;
pub mod checkpoint;
pub mod watermark;
pub mod state;
pub mod partition;
pub mod pipeline;
pub mod runtime;
pub mod domain;

// Legacy event distribution
pub mod bus;
pub mod pubsub;

// WebSocket support
pub mod websocket;

// Legacy additional features
pub mod auth;
pub mod compression;
pub mod heartbeat;
pub mod presence;
pub mod replay;
pub mod room;

// Re-exports for convenience
pub use error::{Result, StreamingError};
pub use event::{Event, EventFilter, EventMetadata, EventPayload, EventType};

/// Prelude module with commonly used imports
pub mod prelude {
    // Core types
    pub use crate::error::{Result, StreamingError};
    pub use crate::config::StreamingConfig;

    // Streaming infrastructure (v0.2.0)
    pub use crate::stream::{DataStream, StreamExt};
    pub use crate::source::{Source, ChannelSource, FileSource, IteratorSource};
    pub use crate::sink::{Sink, ChannelSink, FileSink};
    pub use crate::operators::{
        AggregateOperator, Aggregator, FilterOperator, FlatMapOperator,
        JoinOperator, JoinType, KeyByOperator, KeyExtractor, MapOperator,
        WindowOperator, WindowAssigner, WindowType,
    };
    pub use crate::backpressure::{BackpressureController, AdaptiveBackpressure};
    pub use crate::checkpoint::{CheckpointCoordinator, Checkpoint};
    pub use crate::watermark::{Watermark, Timestamp, WatermarkTracker};
    pub use crate::state::{StateContext, ValueState, ListState, MapState};
    pub use crate::partition::{Partitioner, PartitionAssignment};
    pub use crate::pipeline::{Pipeline, PipelineBuilder};
    pub use crate::runtime::{StreamingRuntime, RuntimeBuilder};

    // Domain-specific streams
    pub use crate::domain::{
        SimulationStream, SimulationData, SimulationState,
        SensorStream, SensorData, SensorType,
        EventStream, SystemEvent, SystemEventType,
        TelemetryStream, TelemetryData, TelemetryType,
    };

    // Legacy event system
    pub use crate::auth::{AuthResult, AuthToken, Authenticator, Authorizer, Permission};
    pub use crate::bus::{EventBus, EventBusBuilder, EventBusType, EventStream as LegacyEventStream};
    pub use crate::compression::{
        CompressionConfig, CompressionLevel, Compressor, Decompressor,
    };
    pub use crate::event::{
        CursorPosition, Event, EventFilter, EventMetadata, EventPayload, EventType,
        PresenceStatus, VehicleState,
    };
    pub use crate::heartbeat::{HeartbeatConfig, HeartbeatManager, HeartbeatMonitor};
    pub use crate::presence::{PresenceConfig, PresenceInfo, PresenceTracker};
    pub use crate::pubsub::{DefaultPubSub, PubSub, Publisher, Subscriber, SubscriberId};
    pub use crate::replay::{ReplayBuffer, ReplayConfig, ReplayManager};
    pub use crate::room::{Room, RoomInfo, RoomManager};
    pub use crate::websocket::{
        ConnectionState, MessageHandler, ReconnectConfig, WsClient, WsMessage, WsServer,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        // Ensure the crate compiles
        assert!(true);
    }
}
