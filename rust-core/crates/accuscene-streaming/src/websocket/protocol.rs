//! WebSocket wire protocol definitions.

use crate::event::Event;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsMessage {
    /// Event message
    Event {
        event: Event,
    },

    /// Subscribe to a topic/room
    Subscribe {
        topic: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        filter: Option<String>,
    },

    /// Unsubscribe from a topic/room
    Unsubscribe {
        topic: String,
    },

    /// Join a room
    JoinRoom {
        room_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        user_id: Option<String>,
    },

    /// Leave a room
    LeaveRoom {
        room_id: String,
    },

    /// Authentication request
    Auth {
        token: String,
    },

    /// Authentication response
    AuthResponse {
        success: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    },

    /// Ping message for heartbeat
    Ping {
        timestamp: i64,
    },

    /// Pong response to ping
    Pong {
        timestamp: i64,
    },

    /// Request event replay
    ReplayRequest {
        #[serde(skip_serializing_if = "Option::is_none")]
        from_sequence: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        to_sequence: Option<u64>,
    },

    /// Replay response
    ReplayResponse {
        events: Vec<Event>,
        complete: bool,
    },

    /// Presence update
    PresenceUpdate {
        user_id: String,
        status: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<serde_json::Value>,
    },

    /// Error message
    Error {
        code: String,
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        request_id: Option<Uuid>,
    },

    /// Acknowledgment
    Ack {
        request_id: Uuid,
        success: bool,
    },
}

impl WsMessage {
    /// Create an event message
    pub fn event(event: Event) -> Self {
        Self::Event { event }
    }

    /// Create a subscribe message
    pub fn subscribe(topic: impl Into<String>) -> Self {
        Self::Subscribe {
            topic: topic.into(),
            filter: None,
        }
    }

    /// Create an unsubscribe message
    pub fn unsubscribe(topic: impl Into<String>) -> Self {
        Self::Unsubscribe {
            topic: topic.into(),
        }
    }

    /// Create a join room message
    pub fn join_room(room_id: impl Into<String>, user_id: Option<String>) -> Self {
        Self::JoinRoom {
            room_id: room_id.into(),
            user_id,
        }
    }

    /// Create a leave room message
    pub fn leave_room(room_id: impl Into<String>) -> Self {
        Self::LeaveRoom {
            room_id: room_id.into(),
        }
    }

    /// Create a ping message
    pub fn ping() -> Self {
        Self::Ping {
            timestamp: chrono::Utc::now().timestamp_millis(),
        }
    }

    /// Create a pong message
    pub fn pong(timestamp: i64) -> Self {
        Self::Pong { timestamp }
    }

    /// Create an error message
    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Error {
            code: code.into(),
            message: message.into(),
            request_id: None,
        }
    }

    /// Create an acknowledgment message
    pub fn ack(request_id: Uuid, success: bool) -> Self {
        Self::Ack {
            request_id,
            success,
        }
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> serde_json::Result<Self> {
        serde_json::from_str(json)
    }
}

/// WebSocket connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Connecting
    Connecting,
    /// Connected and authenticated
    Connected,
    /// Disconnected
    Disconnected,
    /// Reconnecting
    Reconnecting,
    /// Failed
    Failed,
}

impl ConnectionState {
    /// Check if the connection is active
    pub fn is_active(&self) -> bool {
        matches!(self, ConnectionState::Connected)
    }

    /// Check if the connection can send/receive
    pub fn can_communicate(&self) -> bool {
        matches!(self, ConnectionState::Connected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{EventPayload, EventType};

    #[test]
    fn test_message_serialization() {
        let msg = WsMessage::subscribe("test-topic");
        let json = msg.to_json().unwrap();
        let deserialized = WsMessage::from_json(&json).unwrap();

        match deserialized {
            WsMessage::Subscribe { topic, .. } => assert_eq!(topic, "test-topic"),
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_event_message() {
        let event = Event::new(EventType::UserJoined, EventPayload::Empty);
        let msg = WsMessage::event(event);

        let json = msg.to_json().unwrap();
        let deserialized = WsMessage::from_json(&json).unwrap();

        match deserialized {
            WsMessage::Event { event } => {
                assert_eq!(event.event_type, EventType::UserJoined);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_connection_state() {
        let state = ConnectionState::Connected;
        assert!(state.is_active());
        assert!(state.can_communicate());

        let state = ConnectionState::Disconnected;
        assert!(!state.is_active());
        assert!(!state.can_communicate());
    }
}
