//! # AccuScene Streaming
//!
//! Real-time event streaming and WebSocket system for AccuScene Enterprise.
//!
//! ## Features
//!
//! - **Event Bus**: Multiple event bus implementations (memory, channel, broadcast)
//! - **Pub/Sub**: Topic-based publish/subscribe messaging
//! - **WebSocket**: Server and client implementations with protocol support
//! - **Stream Processing**: Filter, transform, and aggregate event streams
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

// Event distribution
pub mod bus;
pub mod pubsub;

// WebSocket support
pub mod websocket;

// Stream processing
pub mod stream;

// Additional features
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
    pub use crate::auth::{AuthResult, AuthToken, Authenticator, Authorizer, Permission};
    pub use crate::bus::{EventBus, EventBusBuilder, EventBusType, EventStream};
    pub use crate::compression::{
        CompressionConfig, CompressionLevel, Compressor, Decompressor,
    };
    pub use crate::error::{Result, StreamingError};
    pub use crate::event::{
        CursorPosition, Event, EventFilter, EventMetadata, EventPayload, EventType,
        PresenceStatus, VehicleState,
    };
    pub use crate::heartbeat::{HeartbeatConfig, HeartbeatManager, HeartbeatMonitor};
    pub use crate::presence::{PresenceConfig, PresenceInfo, PresenceTracker};
    pub use crate::pubsub::{DefaultPubSub, PubSub, Publisher, Subscriber, SubscriberId};
    pub use crate::replay::{ReplayBuffer, ReplayConfig, ReplayManager};
    pub use crate::room::{Room, RoomInfo, RoomManager};
    pub use crate::stream::{
        Aggregator, EventFilterStream, EventTransformer, FilterStreamExt, TransformStreamExt,
        WindowAggregator,
    };
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
