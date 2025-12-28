//! Cross-crate event system
//!
//! This module provides a unified event bus that allows different services
//! to communicate through events without tight coupling.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, trace};

/// Event type identifier
pub type EventType = String;

/// Event handler ID
pub type HandlerId = uuid::Uuid;

/// Base event trait
#[async_trait]
pub trait Event: Send + Sync {
    /// Get the event type
    fn event_type(&self) -> EventType;

    /// Get the event timestamp
    fn timestamp(&self) -> chrono::DateTime<chrono::Utc>;

    /// Get event metadata
    fn metadata(&self) -> EventMetadata;
}

/// Event metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    /// Event ID
    pub id: uuid::Uuid,

    /// Event type
    pub event_type: String,

    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Source service
    pub source: Option<String>,

    /// Correlation ID for event chains
    pub correlation_id: Option<uuid::Uuid>,

    /// Custom metadata
    pub custom: HashMap<String, serde_json::Value>,
}

impl EventMetadata {
    /// Create new event metadata
    pub fn new(event_type: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            event_type,
            timestamp: chrono::Utc::now(),
            source: None,
            correlation_id: None,
            custom: HashMap::new(),
        }
    }

    /// Set source service
    pub fn with_source(mut self, source: String) -> Self {
        self.source = Some(source);
        self
    }

    /// Set correlation ID
    pub fn with_correlation_id(mut self, correlation_id: uuid::Uuid) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }

    /// Add custom metadata
    pub fn with_custom(mut self, key: String, value: serde_json::Value) -> Self {
        self.custom.insert(key, value);
        self
    }
}

/// Event handler trait
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// Handle an event
    async fn handle(&self, event: Arc<dyn Event>) -> Result<(), EventError>;

    /// Get handler ID
    fn handler_id(&self) -> HandlerId;

    /// Get event types this handler subscribes to
    fn subscribes_to(&self) -> Vec<EventType>;
}

/// Event handler error
#[derive(Debug, thiserror::Error)]
pub enum EventError {
    /// Handler error
    #[error("Handler error: {0}")]
    HandlerError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Unknown error
    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Event bus for publishing and subscribing to events
pub struct EventBus {
    handlers: Arc<RwLock<HashMap<EventType, Vec<Arc<dyn EventHandler>>>>>,
    event_log: Arc<RwLock<Vec<EventMetadata>>>,
    max_log_size: usize,
}

impl EventBus {
    /// Create a new event bus
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
            event_log: Arc::new(RwLock::new(Vec::new())),
            max_log_size: 10000,
        }
    }

    /// Create a new event bus with custom log size
    pub fn with_log_size(max_log_size: usize) -> Self {
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
            event_log: Arc::new(RwLock::new(Vec::new())),
            max_log_size,
        }
    }

    /// Subscribe a handler to events
    pub async fn subscribe(&self, handler: Arc<dyn EventHandler>) {
        let mut handlers = self.handlers.write().await;

        for event_type in handler.subscribes_to() {
            handlers
                .entry(event_type.clone())
                .or_insert_with(Vec::new)
                .push(Arc::clone(&handler));

            debug!("Handler {:?} subscribed to {}", handler.handler_id(), event_type);
        }
    }

    /// Unsubscribe a handler
    pub async fn unsubscribe(&self, handler_id: HandlerId) {
        let mut handlers = self.handlers.write().await;

        for (event_type, handler_list) in handlers.iter_mut() {
            handler_list.retain(|h| h.handler_id() != handler_id);
            debug!("Handler {:?} unsubscribed from {}", handler_id, event_type);
        }
    }

    /// Publish an event
    pub async fn publish(&self, event: Arc<dyn Event>) -> Result<(), EventError> {
        let event_type = event.event_type();
        let metadata = event.metadata();

        trace!("Publishing event: {} ({})", event_type, metadata.id);

        // Log the event
        self.log_event(metadata.clone()).await;

        // Get handlers for this event type
        let handlers = self.handlers.read().await;
        let event_handlers = handlers.get(&event_type);

        if let Some(handler_list) = event_handlers {
            // Execute all handlers concurrently
            let futures: Vec<_> = handler_list
                .iter()
                .map(|handler| {
                    let event = Arc::clone(&event);
                    let handler = Arc::clone(handler);
                    async move {
                        match handler.handle(event).await {
                            Ok(()) => Ok(()),
                            Err(e) => {
                                tracing::error!(
                                    "Handler {:?} failed: {}",
                                    handler.handler_id(),
                                    e
                                );
                                Err(e)
                            }
                        }
                    }
                })
                .collect();

            // Wait for all handlers to complete
            let results = futures::future::join_all(futures).await;

            // Check for any errors
            let errors: Vec<_> = results.into_iter().filter_map(|r| r.err()).collect();

            if !errors.is_empty() {
                return Err(EventError::HandlerError(format!(
                    "{} handler(s) failed",
                    errors.len()
                )));
            }

            debug!("Event {} handled by {} handler(s)", event_type, handler_list.len());
        } else {
            trace!("No handlers registered for event: {}", event_type);
        }

        Ok(())
    }

    /// Log an event to the event log
    async fn log_event(&self, metadata: EventMetadata) {
        let mut log = self.event_log.write().await;

        // Add the event
        log.push(metadata);

        // Trim log if it exceeds max size
        if log.len() > self.max_log_size {
            let remove_count = log.len() - self.max_log_size;
            log.drain(0..remove_count);
        }
    }

    /// Get recent events from the log
    pub async fn recent_events(&self, count: usize) -> Vec<EventMetadata> {
        let log = self.event_log.read().await;
        let start = log.len().saturating_sub(count);
        log[start..].to_vec()
    }

    /// Get event count
    pub async fn event_count(&self) -> usize {
        self.event_log.read().await.len()
    }

    /// Get handler count
    pub async fn handler_count(&self) -> usize {
        let handlers = self.handlers.read().await;
        handlers.values().map(|v| v.len()).sum()
    }

    /// Get subscribed event types
    pub async fn subscribed_types(&self) -> Vec<EventType> {
        let handlers = self.handlers.read().await;
        handlers.keys().cloned().collect()
    }

    /// Clear event log
    pub async fn clear_log(&self) {
        let mut log = self.event_log.write().await;
        log.clear();
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Common Event Types
// ============================================================================

/// System event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemEvent {
    /// Event metadata
    pub metadata: EventMetadata,

    /// Event name
    pub name: String,

    /// Event payload
    pub payload: serde_json::Value,
}

impl SystemEvent {
    /// Create a new system event
    pub fn new(name: String, payload: serde_json::Value) -> Self {
        Self {
            metadata: EventMetadata::new("system".to_string()),
            name,
            payload,
        }
    }
}

#[async_trait]
impl Event for SystemEvent {
    fn event_type(&self) -> EventType {
        "system".to_string()
    }

    fn timestamp(&self) -> chrono::DateTime<chrono::Utc> {
        self.metadata.timestamp
    }

    fn metadata(&self) -> EventMetadata {
        self.metadata.clone()
    }
}

/// Service event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEvent {
    /// Event metadata
    pub metadata: EventMetadata,

    /// Service name
    pub service: String,

    /// Event action
    pub action: ServiceAction,
}

/// Service action
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceAction {
    /// Service started
    Started,
    /// Service stopped
    Stopped,
    /// Service error
    Error(String),
    /// Service healthy
    Healthy,
    /// Service unhealthy
    Unhealthy,
}

impl ServiceEvent {
    /// Create a new service event
    pub fn new(service: String, action: ServiceAction) -> Self {
        Self {
            metadata: EventMetadata::new("service".to_string()).with_source(service.clone()),
            service,
            action,
        }
    }
}

#[async_trait]
impl Event for ServiceEvent {
    fn event_type(&self) -> EventType {
        "service".to_string()
    }

    fn timestamp(&self) -> chrono::DateTime<chrono::Utc> {
        self.metadata.timestamp
    }

    fn metadata(&self) -> EventMetadata {
        self.metadata.clone()
    }
}
