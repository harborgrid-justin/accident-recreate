//! Span management for distributed tracing

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Trace ID (128-bit unique identifier)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TraceId(Uuid);

impl TraceId {
    /// Generate a new trace ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create from a UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Get the underlying UUID
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }

    /// Convert to string
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl Default for TraceId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for TraceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Span ID (64-bit unique identifier within a trace)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpanId(Uuid);

impl SpanId {
    /// Generate a new span ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create from a UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Get the underlying UUID
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }

    /// Convert to string
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl Default for SpanId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for SpanId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Span context for distributed tracing
#[derive(Debug, Clone)]
pub struct SpanContext {
    trace_id: TraceId,
    span_id: SpanId,
    parent_span_id: Option<SpanId>,
    name: String,
    service_name: String,
    start_time: DateTime<Utc>,
    end_time: Arc<RwLock<Option<DateTime<Utc>>>>,
    attributes: Arc<RwLock<HashMap<String, String>>>,
    events: Arc<RwLock<Vec<SpanEvent>>>,
}

impl SpanContext {
    /// Create a new root span
    pub fn new(name: impl Into<String>, service_name: impl Into<String>) -> Self {
        Self {
            trace_id: TraceId::new(),
            span_id: SpanId::new(),
            parent_span_id: None,
            name: name.into(),
            service_name: service_name.into(),
            start_time: Utc::now(),
            end_time: Arc::new(RwLock::new(None)),
            attributes: Arc::new(RwLock::new(HashMap::new())),
            events: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Create a child span
    pub fn child(&self, name: impl Into<String>) -> Self {
        Self {
            trace_id: self.trace_id,
            span_id: SpanId::new(),
            parent_span_id: Some(self.span_id),
            name: name.into(),
            service_name: self.service_name.clone(),
            start_time: Utc::now(),
            end_time: Arc::new(RwLock::new(None)),
            attributes: Arc::new(RwLock::new(HashMap::new())),
            events: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Get the trace ID
    pub fn trace_id(&self) -> TraceId {
        self.trace_id
    }

    /// Get the span ID
    pub fn span_id(&self) -> SpanId {
        self.span_id
    }

    /// Get the parent span ID
    pub fn parent_span_id(&self) -> Option<SpanId> {
        self.parent_span_id
    }

    /// Get the span name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the service name
    pub fn service_name(&self) -> &str {
        &self.service_name
    }

    /// Get the start time
    pub fn start_time(&self) -> DateTime<Utc> {
        self.start_time
    }

    /// Get the end time
    pub fn end_time(&self) -> Option<DateTime<Utc>> {
        *self.end_time.read()
    }

    /// Get the duration in milliseconds
    pub fn duration_ms(&self) -> Option<i64> {
        self.end_time().map(|end| {
            (end - self.start_time).num_milliseconds()
        })
    }

    /// Check if the span is active
    pub fn is_active(&self) -> bool {
        self.end_time.read().is_none()
    }

    /// Set an attribute
    pub fn set_attribute(&self, key: impl Into<String>, value: impl Into<String>) {
        self.attributes.write().insert(key.into(), value.into());
    }

    /// Get an attribute
    pub fn get_attribute(&self, key: &str) -> Option<String> {
        self.attributes.read().get(key).cloned()
    }

    /// Get all attributes
    pub fn attributes(&self) -> HashMap<String, String> {
        self.attributes.read().clone()
    }

    /// Add an event to the span
    pub fn add_event(&self, name: impl Into<String>, message: impl Into<String>) {
        let event = SpanEvent {
            timestamp: Utc::now(),
            name: name.into(),
            message: message.into(),
        };
        self.events.write().push(event);
    }

    /// Get all events
    pub fn events(&self) -> Vec<SpanEvent> {
        self.events.read().clone()
    }

    /// End the span
    pub fn end(&self) {
        *self.end_time.write() = Some(Utc::now());
    }

    /// Create a serializable snapshot
    pub fn snapshot(&self) -> SpanSnapshot {
        SpanSnapshot {
            trace_id: self.trace_id.to_string(),
            span_id: self.span_id.to_string(),
            parent_span_id: self.parent_span_id.map(|id| id.to_string()),
            name: self.name.clone(),
            service_name: self.service_name.clone(),
            start_time: self.start_time,
            end_time: *self.end_time.read(),
            duration_ms: self.duration_ms(),
            attributes: self.attributes.read().clone(),
            events: self.events.read().clone(),
        }
    }
}

/// Span event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanEvent {
    pub timestamp: DateTime<Utc>,
    pub name: String,
    pub message: String,
}

/// Serializable span snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanSnapshot {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub name: String,
    pub service_name: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_ms: Option<i64>,
    pub attributes: HashMap<String, String>,
    pub events: Vec<SpanEvent>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_creation() {
        let span = SpanContext::new("test_span", "test_service");
        assert_eq!(span.name(), "test_span");
        assert_eq!(span.service_name(), "test_service");
        assert!(span.is_active());
    }

    #[test]
    fn test_child_span() {
        let parent = SpanContext::new("parent", "service");
        let child = parent.child("child");

        assert_eq!(child.trace_id(), parent.trace_id());
        assert_eq!(child.parent_span_id(), Some(parent.span_id()));
    }

    #[test]
    fn test_span_attributes() {
        let span = SpanContext::new("test", "service");
        span.set_attribute("user_id", "123");
        span.set_attribute("method", "GET");

        assert_eq!(span.get_attribute("user_id"), Some("123".to_string()));
        assert_eq!(span.get_attribute("method"), Some("GET".to_string()));
    }

    #[test]
    fn test_span_end() {
        let span = SpanContext::new("test", "service");
        assert!(span.is_active());

        span.end();
        assert!(!span.is_active());
        assert!(span.duration_ms().is_some());
    }
}
