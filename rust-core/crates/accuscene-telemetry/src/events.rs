//! Structured events for telemetry

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Event severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventSeverity {
    /// Debug information
    Debug,
    /// Informational events
    Info,
    /// Warning events
    Warning,
    /// Error events
    Error,
    /// Critical events
    Critical,
}

impl EventSeverity {
    /// Convert to string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
            Self::Critical => "critical",
        }
    }
}

impl std::fmt::Display for EventSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Event category
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventCategory {
    /// System events
    System,
    /// Application events
    Application,
    /// User action events
    UserAction,
    /// Simulation events
    Simulation,
    /// Database events
    Database,
    /// Network events
    Network,
    /// Security events
    Security,
    /// Performance events
    Performance,
    /// Custom category
    Custom(String),
}

impl EventCategory {
    /// Convert to string
    pub fn as_str(&self) -> &str {
        match self {
            Self::System => "system",
            Self::Application => "application",
            Self::UserAction => "user_action",
            Self::Simulation => "simulation",
            Self::Database => "database",
            Self::Network => "network",
            Self::Security => "security",
            Self::Performance => "performance",
            Self::Custom(s) => s,
        }
    }
}

impl std::fmt::Display for EventCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Structured event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Unique event ID
    pub id: String,

    /// Event timestamp
    pub timestamp: DateTime<Utc>,

    /// Event severity
    pub severity: EventSeverity,

    /// Event category
    pub category: EventCategory,

    /// Event name/type
    pub name: String,

    /// Event message
    pub message: String,

    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,

    /// Source of the event (service, component, etc.)
    pub source: Option<String>,

    /// User ID associated with the event
    pub user_id: Option<String>,

    /// Session ID associated with the event
    pub session_id: Option<String>,

    /// Trace ID for distributed tracing correlation
    pub trace_id: Option<String>,
}

impl Event {
    /// Create a new event
    pub fn new(
        severity: EventSeverity,
        category: EventCategory,
        name: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            severity,
            category,
            name: name.into(),
            message: message.into(),
            metadata: HashMap::new(),
            source: None,
            user_id: None,
            session_id: None,
            trace_id: None,
        }
    }

    /// Create a debug event
    pub fn debug(category: EventCategory, name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(EventSeverity::Debug, category, name, message)
    }

    /// Create an info event
    pub fn info(category: EventCategory, name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(EventSeverity::Info, category, name, message)
    }

    /// Create a warning event
    pub fn warning(category: EventCategory, name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(EventSeverity::Warning, category, name, message)
    }

    /// Create an error event
    pub fn error(category: EventCategory, name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(EventSeverity::Error, category, name, message)
    }

    /// Create a critical event
    pub fn critical(category: EventCategory, name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(EventSeverity::Critical, category, name, message)
    }

    /// Add metadata
    pub fn with_metadata<V: Serialize>(mut self, key: impl Into<String>, value: V) -> Self {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.metadata.insert(key.into(), json_value);
        }
        self
    }

    /// Set the source
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Set the user ID
    pub fn with_user(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Set the session ID
    pub fn with_session(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    /// Set the trace ID
    pub fn with_trace(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = Some(trace_id.into());
        self
    }

    /// Convert to JSON
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }

    /// Convert to pretty JSON
    pub fn to_json_pretty(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }
}

/// Event logger for recording structured events
pub struct EventLogger {
    events: Arc<RwLock<Vec<Event>>>,
    max_events: usize,
}

impl EventLogger {
    /// Create a new event logger
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
            max_events: 10000,
        }
    }

    /// Create with custom max events
    pub fn with_max_events(max_events: usize) -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
            max_events,
        }
    }

    /// Record an event
    pub fn record(&self, event: Event) {
        tracing::info!(
            event_id = %event.id,
            severity = %event.severity,
            category = %event.category,
            name = %event.name,
            "{}",
            event.message
        );

        let mut events = self.events.write();
        events.push(event);

        // Trim old events if exceeding max
        if events.len() > self.max_events {
            events.drain(0..events.len() - self.max_events);
        }
    }

    /// Get all events
    pub fn events(&self) -> Vec<Event> {
        self.events.read().clone()
    }

    /// Get events by severity
    pub fn events_by_severity(&self, severity: EventSeverity) -> Vec<Event> {
        self.events
            .read()
            .iter()
            .filter(|e| e.severity == severity)
            .cloned()
            .collect()
    }

    /// Get events by category
    pub fn events_by_category(&self, category: &EventCategory) -> Vec<Event> {
        self.events
            .read()
            .iter()
            .filter(|e| &e.category == category)
            .cloned()
            .collect()
    }

    /// Get events in time range
    pub fn events_in_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<Event> {
        self.events
            .read()
            .iter()
            .filter(|e| e.timestamp >= start && e.timestamp <= end)
            .cloned()
            .collect()
    }

    /// Get recent events
    pub fn recent_events(&self, count: usize) -> Vec<Event> {
        let events = self.events.read();
        let start = events.len().saturating_sub(count);
        events[start..].to_vec()
    }

    /// Clear all events
    pub fn clear(&self) {
        self.events.write().clear();
    }

    /// Get event count
    pub fn count(&self) -> usize {
        self.events.read().len()
    }

    /// Get event count by severity
    pub fn count_by_severity(&self, severity: EventSeverity) -> usize {
        self.events
            .read()
            .iter()
            .filter(|e| e.severity == severity)
            .count()
    }
}

impl Default for EventLogger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let event = Event::info(
            EventCategory::Application,
            "test_event",
            "Test message",
        )
        .with_metadata("key", "value")
        .with_source("test_service")
        .with_user("user123");

        assert_eq!(event.severity, EventSeverity::Info);
        assert_eq!(event.category, EventCategory::Application);
        assert_eq!(event.name, "test_event");
        assert!(event.metadata.contains_key("key"));
        assert_eq!(event.source, Some("test_service".to_string()));
    }

    #[test]
    fn test_event_logger() {
        let logger = EventLogger::new();

        let event1 = Event::info(EventCategory::System, "event1", "Message 1");
        let event2 = Event::error(EventCategory::Application, "event2", "Message 2");

        logger.record(event1);
        logger.record(event2);

        assert_eq!(logger.count(), 2);
        assert_eq!(logger.count_by_severity(EventSeverity::Info), 1);
        assert_eq!(logger.count_by_severity(EventSeverity::Error), 1);
    }

    #[test]
    fn test_event_filtering() {
        let logger = EventLogger::new();

        logger.record(Event::info(EventCategory::System, "e1", "M1"));
        logger.record(Event::error(EventCategory::Application, "e2", "M2"));
        logger.record(Event::info(EventCategory::System, "e3", "M3"));

        let system_events = logger.events_by_category(&EventCategory::System);
        assert_eq!(system_events.len(), 2);

        let error_events = logger.events_by_severity(EventSeverity::Error);
        assert_eq!(error_events.len(), 1);
    }
}
