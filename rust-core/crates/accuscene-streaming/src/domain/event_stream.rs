//! System event streaming.

use crate::error::Result;
use crate::stream::DataStream;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// System event type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SystemEventType {
    SimulationStarted,
    SimulationStopped,
    SimulationPaused,
    SimulationResumed,
    CollisionDetected,
    UserAction,
    SystemError,
    Warning,
    Info,
    Debug,
    Custom(String),
}

impl SystemEventType {
    /// Get severity level
    pub fn severity(&self) -> u8 {
        match self {
            SystemEventType::Debug => 0,
            SystemEventType::Info => 1,
            SystemEventType::Warning => 2,
            SystemEventType::SystemError => 3,
            SystemEventType::CollisionDetected => 3,
            _ => 1,
        }
    }
}

/// System event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemEvent {
    pub event_id: String,
    pub event_type: SystemEventType,
    pub timestamp: i64,
    pub source: String,
    pub message: String,
    pub data: serde_json::Value,
    pub tags: Vec<String>,
    pub metadata: std::collections::HashMap<String, String>,
}

impl SystemEvent {
    /// Create a new system event
    pub fn new(event_type: SystemEventType, source: String, message: String) -> Self {
        Self {
            event_id: uuid::Uuid::new_v4().to_string(),
            event_type,
            timestamp: chrono::Utc::now().timestamp_millis(),
            source,
            message,
            data: serde_json::Value::Null,
            tags: Vec::new(),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Set event data
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = data;
        self
    }

    /// Add tag
    pub fn with_tag(mut self, tag: String) -> Self {
        self.tags.push(tag);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Get severity level
    pub fn severity(&self) -> u8 {
        self.event_type.severity()
    }

    /// Check if event is critical
    pub fn is_critical(&self) -> bool {
        self.severity() >= 3
    }
}

/// Event stream for system events
pub struct EventStream<S>
where
    S: DataStream<Item = SystemEvent>,
{
    inner: S,
    min_severity: Option<u8>,
    filter_types: Option<Vec<SystemEventType>>,
    filter_tags: Option<Vec<String>>,
}

impl<S> EventStream<S>
where
    S: DataStream<Item = SystemEvent>,
{
    /// Create a new event stream
    pub fn new(inner: S) -> Self {
        Self {
            inner,
            min_severity: None,
            filter_types: None,
            filter_tags: None,
        }
    }

    /// Filter by minimum severity
    pub fn min_severity(mut self, severity: u8) -> Self {
        self.min_severity = Some(severity);
        self
    }

    /// Filter by event types
    pub fn filter_types(mut self, types: Vec<SystemEventType>) -> Self {
        self.filter_types = Some(types);
        self
    }

    /// Filter by tags
    pub fn filter_tags(mut self, tags: Vec<String>) -> Self {
        self.filter_tags = Some(tags);
        self
    }

    /// Filter critical events only
    pub fn critical_only(mut self) -> Self {
        self.min_severity = Some(3);
        self
    }

    fn should_include(&self, event: &SystemEvent) -> bool {
        // Check severity
        if let Some(min_severity) = self.min_severity {
            if event.severity() < min_severity {
                return false;
            }
        }

        // Check event type
        if let Some(ref types) = self.filter_types {
            if !types.contains(&event.event_type) {
                return false;
            }
        }

        // Check tags
        if let Some(ref tags) = self.filter_tags {
            if !tags.iter().any(|tag| event.tags.contains(tag)) {
                return false;
            }
        }

        true
    }
}

#[async_trait]
impl<S> DataStream for EventStream<S>
where
    S: DataStream<Item = SystemEvent>,
{
    type Item = SystemEvent;

    async fn next(&mut self) -> Result<Option<Self::Item>> {
        loop {
            match self.inner.next().await? {
                Some(event) => {
                    if self.should_include(&event) {
                        return Ok(Some(event));
                    }
                }
                None => return Ok(None),
            }
        }
    }

    fn is_complete(&self) -> bool {
        self.inner.is_complete()
    }
}

/// Event statistics
#[derive(Debug, Clone, Default)]
pub struct EventStats {
    pub total_events: u64,
    pub events_by_type: std::collections::HashMap<String, u64>,
    pub events_by_severity: std::collections::HashMap<u8, u64>,
    pub critical_events: u64,
}

impl EventStats {
    pub fn update(&mut self, event: &SystemEvent) {
        self.total_events += 1;

        let type_key = format!("{:?}", event.event_type);
        *self.events_by_type.entry(type_key).or_insert(0) += 1;

        let severity = event.severity();
        *self.events_by_severity.entry(severity).or_insert(0) += 1;

        if event.is_critical() {
            self.critical_events += 1;
        }
    }
}

/// Event aggregator for grouping related events
pub struct EventAggregator {
    window_duration: std::time::Duration,
    events: Vec<SystemEvent>,
}

impl EventAggregator {
    pub fn new(window_duration: std::time::Duration) -> Self {
        Self {
            window_duration,
            events: Vec::new(),
        }
    }

    pub fn add(&mut self, event: SystemEvent) {
        self.events.push(event);
    }

    pub fn get_events(&self) -> &[SystemEvent] {
        &self.events
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::iterator::IteratorSource;

    #[tokio::test]
    async fn test_system_event() {
        let event = SystemEvent::new(
            SystemEventType::CollisionDetected,
            "physics_engine".to_string(),
            "Collision detected between vehicle A and B".to_string(),
        )
        .with_tag("collision".to_string())
        .with_metadata("severity".to_string(), "high".to_string());

        assert!(event.is_critical());
        assert_eq!(event.severity(), 3);
    }

    #[tokio::test]
    async fn test_event_stream_filtering() {
        let events = vec![
            SystemEvent::new(
                SystemEventType::Info,
                "system".to_string(),
                "Info message".to_string(),
            ),
            SystemEvent::new(
                SystemEventType::Warning,
                "system".to_string(),
                "Warning message".to_string(),
            ),
            SystemEvent::new(
                SystemEventType::SystemError,
                "system".to_string(),
                "Error message".to_string(),
            ),
        ];

        let source = IteratorSource::new(events.into_iter());
        let stream = EventStream::new(source).min_severity(2);

        // Would filter to only Warning and Error events
    }
}
