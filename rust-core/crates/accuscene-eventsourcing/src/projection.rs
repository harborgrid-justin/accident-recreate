//! Projection system for building read models from events.

use crate::error::{EventSourcingError, Result};
use crate::event::{Event, EventEnvelope};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Trait for event projections.
#[async_trait]
pub trait Projection: Send + Sync {
    /// The event type this projection handles.
    type Event: Event;

    /// The projection state type.
    type State: Clone + Send + Sync;

    /// Returns the projection name.
    fn projection_name(&self) -> &str;

    /// Projects an event into the read model.
    async fn project(&self, event: &EventEnvelope<Self::Event>) -> Result<()>;

    /// Rebuilds the projection from event history.
    async fn rebuild(&self, events: Vec<EventEnvelope<Self::Event>>) -> Result<()> {
        for event in events {
            self.project(&event).await?;
        }
        Ok(())
    }

    /// Returns the current state of the projection.
    async fn get_state(&self) -> Result<Self::State>;

    /// Resets the projection state.
    async fn reset(&self) -> Result<()>;
}

/// State tracking for projections.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectionState {
    /// Name of the projection.
    pub name: String,

    /// Last processed event sequence.
    pub last_sequence: u64,

    /// Last processed event timestamp.
    pub last_timestamp: DateTime<Utc>,

    /// Number of events processed.
    pub events_processed: u64,

    /// Projection status.
    pub status: ProjectionStatus,

    /// Last error if any.
    pub last_error: Option<String>,

    /// Timestamp when the projection was last updated.
    pub updated_at: DateTime<Utc>,
}

impl ProjectionState {
    /// Creates a new projection state.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            last_sequence: 0,
            last_timestamp: Utc::now(),
            events_processed: 0,
            status: ProjectionStatus::Running,
            last_error: None,
            updated_at: Utc::now(),
        }
    }

    /// Updates the state after processing an event.
    pub fn update(&mut self, sequence: u64, timestamp: DateTime<Utc>) {
        self.last_sequence = sequence;
        self.last_timestamp = timestamp;
        self.events_processed += 1;
        self.updated_at = Utc::now();
    }

    /// Marks the projection as failed.
    pub fn mark_failed(&mut self, error: impl Into<String>) {
        self.status = ProjectionStatus::Failed;
        self.last_error = Some(error.into());
        self.updated_at = Utc::now();
    }

    /// Marks the projection as stopped.
    pub fn mark_stopped(&mut self) {
        self.status = ProjectionStatus::Stopped;
        self.updated_at = Utc::now();
    }

    /// Marks the projection as running.
    pub fn mark_running(&mut self) {
        self.status = ProjectionStatus::Running;
        self.last_error = None;
        self.updated_at = Utc::now();
    }
}

/// Status of a projection.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProjectionStatus {
    /// Projection is running.
    Running,

    /// Projection is stopped.
    Stopped,

    /// Projection has failed.
    Failed,

    /// Projection is rebuilding.
    Rebuilding,
}

/// Manager for multiple projections.
#[derive(Clone)]
pub struct ProjectionManager {
    /// Registered projections.
    projections: Arc<DashMap<String, Arc<dyn ProjectionHandler>>>,

    /// Projection states.
    states: Arc<DashMap<String, ProjectionState>>,
}

impl ProjectionManager {
    /// Creates a new projection manager.
    pub fn new() -> Self {
        Self {
            projections: Arc::new(DashMap::new()),
            states: Arc::new(DashMap::new()),
        }
    }

    /// Registers a projection.
    pub fn register(&self, name: String, handler: Arc<dyn ProjectionHandler>) {
        self.projections.insert(name.clone(), handler);
        self.states.insert(name.clone(), ProjectionState::new(name));
    }

    /// Projects an event to all registered projections.
    pub async fn project_event(&self, event_type: &str, event_data: &[u8]) -> Result<()> {
        for entry in self.projections.iter() {
            let handler = entry.value();
            if handler.handles_event_type(event_type) {
                if let Err(e) = handler.handle_event(event_data).await {
                    if let Some(mut state) = self.states.get_mut(entry.key()) {
                        state.mark_failed(e.to_string());
                    }
                    return Err(e);
                }
            }
        }
        Ok(())
    }

    /// Gets the state of a projection.
    pub fn get_state(&self, name: &str) -> Option<ProjectionState> {
        self.states.get(name).map(|s| s.clone())
    }

    /// Gets all projection states.
    pub fn get_all_states(&self) -> Vec<ProjectionState> {
        self.states.iter().map(|entry| entry.value().clone()).collect()
    }

    /// Rebuilds a specific projection.
    pub async fn rebuild(&self, name: &str) -> Result<()> {
        let handler = self
            .projections
            .get(name)
            .ok_or_else(|| EventSourcingError::Projection(format!("Projection not found: {}", name)))?;

        if let Some(mut state) = self.states.get_mut(name) {
            state.status = ProjectionStatus::Rebuilding;
        }

        handler.rebuild().await?;

        if let Some(mut state) = self.states.get_mut(name) {
            state.mark_running();
        }

        Ok(())
    }

    /// Stops a projection.
    pub fn stop(&self, name: &str) -> Result<()> {
        if let Some(mut state) = self.states.get_mut(name) {
            state.mark_stopped();
            Ok(())
        } else {
            Err(EventSourcingError::Projection(format!("Projection not found: {}", name)))
        }
    }

    /// Starts a projection.
    pub fn start(&self, name: &str) -> Result<()> {
        if let Some(mut state) = self.states.get_mut(name) {
            state.mark_running();
            Ok(())
        } else {
            Err(EventSourcingError::Projection(format!("Projection not found: {}", name)))
        }
    }
}

impl Default for ProjectionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Handler trait for dynamic projection handling.
#[async_trait]
pub trait ProjectionHandler: Send + Sync {
    /// Checks if this handler processes the given event type.
    fn handles_event_type(&self, event_type: &str) -> bool;

    /// Handles an event.
    async fn handle_event(&self, event_data: &[u8]) -> Result<()>;

    /// Rebuilds the projection.
    async fn rebuild(&self) -> Result<()>;
}

/// Simple in-memory projection implementation.
pub struct InMemoryProjection<K, V>
where
    K: Clone + Send + Sync + std::hash::Hash + Eq,
    V: Clone + Send + Sync,
{
    name: String,
    data: Arc<DashMap<K, V>>,
}

impl<K, V> InMemoryProjection<K, V>
where
    K: Clone + Send + Sync + std::hash::Hash + Eq,
    V: Clone + Send + Sync,
{
    /// Creates a new in-memory projection.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            data: Arc::new(DashMap::new()),
        }
    }

    /// Inserts a value.
    pub fn insert(&self, key: K, value: V) {
        self.data.insert(key, value);
    }

    /// Gets a value.
    pub fn get(&self, key: &K) -> Option<V> {
        self.data.get(key).map(|v| v.clone())
    }

    /// Removes a value.
    pub fn remove(&self, key: &K) -> Option<V> {
        self.data.remove(key).map(|(_, v)| v)
    }

    /// Clears all data.
    pub fn clear(&self) {
        self.data.clear();
    }

    /// Returns the number of entries.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns whether the projection is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns all entries.
    pub fn entries(&self) -> Vec<(K, V)> {
        self.data
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_projection_state() {
        let mut state = ProjectionState::new("test");
        assert_eq!(state.status, ProjectionStatus::Running);

        state.update(1, Utc::now());
        assert_eq!(state.events_processed, 1);
        assert_eq!(state.last_sequence, 1);

        state.mark_failed("test error");
        assert_eq!(state.status, ProjectionStatus::Failed);
        assert!(state.last_error.is_some());
    }

    #[test]
    fn test_in_memory_projection() {
        let projection = InMemoryProjection::<String, i32>::new("test");

        projection.insert("key1".to_string(), 42);
        assert_eq!(projection.get(&"key1".to_string()), Some(42));
        assert_eq!(projection.len(), 1);

        projection.clear();
        assert!(projection.is_empty());
    }

    #[tokio::test]
    async fn test_projection_manager() {
        let manager = ProjectionManager::new();

        let state = manager.get_state("test");
        assert!(state.is_none());

        let states = manager.get_all_states();
        assert_eq!(states.len(), 0);
    }
}
