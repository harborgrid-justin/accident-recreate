//! Event types and definitions for the AccuScene streaming system.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Event identifier
pub type EventId = Uuid;

/// Topic name
pub type TopicName = String;

/// Room identifier
pub type RoomId = String;

/// User identifier
pub type UserId = String;

/// Core event structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Unique event identifier
    pub id: EventId,

    /// Event type
    pub event_type: EventType,

    /// Event payload
    pub payload: EventPayload,

    /// Event metadata
    pub metadata: EventMetadata,

    /// Timestamp when the event was created
    pub timestamp: DateTime<Utc>,
}

impl Event {
    /// Create a new event
    pub fn new(event_type: EventType, payload: EventPayload) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type,
            payload,
            metadata: EventMetadata::default(),
            timestamp: Utc::now(),
        }
    }

    /// Create an event with metadata
    pub fn with_metadata(
        event_type: EventType,
        payload: EventPayload,
        metadata: EventMetadata,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type,
            payload,
            metadata,
            timestamp: Utc::now(),
        }
    }

    /// Get the event's room ID if it has one
    pub fn room_id(&self) -> Option<&str> {
        self.metadata.room_id.as_deref()
    }

    /// Get the event's user ID if it has one
    pub fn user_id(&self) -> Option<&str> {
        self.metadata.user_id.as_deref()
    }
}

/// Event types categorization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventType {
    /// Simulation-related events
    SimulationUpdate,
    SimulationStarted,
    SimulationPaused,
    SimulationStopped,
    SimulationReset,

    /// Case-related events
    CaseCreated,
    CaseUpdated,
    CaseDeleted,
    CaseShared,

    /// Collaborative editing events
    EditStarted,
    EditApplied,
    EditReverted,
    CursorMoved,
    SelectionChanged,

    /// User presence events
    UserJoined,
    UserLeft,
    UserActive,
    UserIdle,
    UserTyping,

    /// Room events
    RoomCreated,
    RoomJoined,
    RoomLeft,
    RoomClosed,

    /// Data synchronization events
    SyncRequested,
    SyncCompleted,
    SyncFailed,

    /// System events
    Connected,
    Disconnected,
    Reconnecting,
    Reconnected,
    Error,

    /// Custom application events
    Custom(String),
}

/// Event payload containing the actual event data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "payload_type", content = "data", rename_all = "snake_case")]
pub enum EventPayload {
    /// Simulation state update
    SimulationState {
        case_id: String,
        simulation_time: f64,
        vehicle_states: Vec<VehicleState>,
        analysis_data: Option<serde_json::Value>,
    },

    /// Case data update
    CaseData {
        case_id: String,
        changes: serde_json::Value,
    },

    /// Collaborative edit
    Edit {
        case_id: String,
        edit_id: String,
        operation: String,
        data: serde_json::Value,
    },

    /// Cursor position update
    Cursor {
        case_id: String,
        user_id: String,
        position: CursorPosition,
    },

    /// User presence update
    Presence {
        user_id: String,
        status: PresenceStatus,
        metadata: Option<serde_json::Value>,
    },

    /// Room activity
    Room {
        room_id: String,
        action: String,
        data: Option<serde_json::Value>,
    },

    /// Synchronization data
    Sync {
        sequence_number: u64,
        data: serde_json::Value,
    },

    /// Error information
    Error {
        code: String,
        message: String,
        details: Option<serde_json::Value>,
    },

    /// Generic JSON payload for custom events
    Json(serde_json::Value),

    /// Empty payload
    Empty,
}

/// Event metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EventMetadata {
    /// Room ID this event belongs to
    pub room_id: Option<RoomId>,

    /// User ID who triggered this event
    pub user_id: Option<UserId>,

    /// Correlation ID for tracking related events
    pub correlation_id: Option<String>,

    /// Event priority (0-10, higher is more important)
    pub priority: u8,

    /// Whether this event should be persisted for replay
    pub persistent: bool,

    /// Additional custom metadata
    pub custom: Option<serde_json::Value>,
}

/// Vehicle state in simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleState {
    pub vehicle_id: String,
    pub position: Position3D,
    pub velocity: Velocity3D,
    pub rotation: Rotation3D,
    pub timestamp: f64,
}

/// 3D position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// 3D velocity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Velocity3D {
    pub vx: f64,
    pub vy: f64,
    pub vz: f64,
}

/// 3D rotation (Euler angles)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rotation3D {
    pub pitch: f64,
    pub yaw: f64,
    pub roll: f64,
}

/// Cursor position in the UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPosition {
    pub x: f64,
    pub y: f64,
    pub view: Option<String>,
}

/// User presence status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PresenceStatus {
    Online,
    Away,
    Idle,
    Offline,
}

/// Event filter for subscribing to specific events
#[derive(Debug, Clone, Default)]
pub struct EventFilter {
    /// Filter by event types
    pub event_types: Option<Vec<EventType>>,

    /// Filter by room ID
    pub room_id: Option<RoomId>,

    /// Filter by user ID
    pub user_id: Option<UserId>,

    /// Custom filter predicate
    pub predicate: Option<fn(&Event) -> bool>,
}

impl EventFilter {
    /// Create a new empty filter
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by event types
    pub fn with_types(mut self, types: Vec<EventType>) -> Self {
        self.event_types = Some(types);
        self
    }

    /// Filter by room ID
    pub fn with_room(mut self, room_id: impl Into<RoomId>) -> Self {
        self.room_id = Some(room_id.into());
        self
    }

    /// Filter by user ID
    pub fn with_user(mut self, user_id: impl Into<UserId>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Check if an event passes this filter
    pub fn matches(&self, event: &Event) -> bool {
        // Check event type filter
        if let Some(ref types) = self.event_types {
            if !types.contains(&event.event_type) {
                return false;
            }
        }

        // Check room ID filter
        if let Some(ref room_id) = self.room_id {
            if event.metadata.room_id.as_ref() != Some(room_id) {
                return false;
            }
        }

        // Check user ID filter
        if let Some(ref user_id) = self.user_id {
            if event.metadata.user_id.as_ref() != Some(user_id) {
                return false;
            }
        }

        // Check custom predicate
        if let Some(predicate) = self.predicate {
            if !predicate(event) {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let event = Event::new(
            EventType::SimulationUpdate,
            EventPayload::Empty,
        );

        assert_eq!(event.event_type, EventType::SimulationUpdate);
    }

    #[test]
    fn test_event_filter() {
        let mut event = Event::new(
            EventType::UserJoined,
            EventPayload::Empty,
        );
        event.metadata.room_id = Some("room123".to_string());

        let filter = EventFilter::new()
            .with_types(vec![EventType::UserJoined, EventType::UserLeft])
            .with_room("room123");

        assert!(filter.matches(&event));

        let wrong_room_filter = EventFilter::new().with_room("room456");
        assert!(!wrong_room_filter.matches(&event));
    }
}
