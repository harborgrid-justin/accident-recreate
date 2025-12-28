//! Saga pattern implementation for distributed transactions.

use crate::command::Command;
use crate::error::{EventSourcingError, Result};
use crate::event::{Event, EventEnvelope};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Trait for saga definitions.
#[async_trait]
pub trait Saga: Send + Sync {
    /// Returns the saga type identifier.
    fn saga_type(&self) -> &'static str;

    /// Handles an event and potentially triggers compensation or next steps.
    async fn handle_event(&mut self, event_data: &[u8], event_type: &str) -> Result<SagaAction>;

    /// Returns the current state of the saga.
    fn state(&self) -> &SagaState;

    /// Sets the state of the saga.
    fn set_state(&mut self, state: SagaState);
}

/// Actions a saga can take in response to an event.
#[derive(Debug)]
pub enum SagaAction {
    /// Continue with the next command.
    Continue(Vec<Box<dyn std::any::Any + Send>>),

    /// Compensate by executing rollback commands.
    Compensate(Vec<Box<dyn std::any::Any + Send>>),

    /// Saga completed successfully.
    Complete,

    /// Saga failed and cannot be compensated.
    Fail(String),

    /// No action needed.
    None,
}

/// State of a saga instance.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SagaState {
    /// Saga is starting.
    Starting,

    /// Saga is running.
    Running,

    /// Saga is compensating (rolling back).
    Compensating,

    /// Saga completed successfully.
    Completed,

    /// Saga failed.
    Failed,

    /// Saga was aborted.
    Aborted,
}

/// Saga instance tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaInstance {
    /// Unique saga instance identifier.
    pub saga_id: Uuid,

    /// Saga type.
    pub saga_type: String,

    /// Current state.
    pub state: SagaState,

    /// Correlation ID linking related events.
    pub correlation_id: Uuid,

    /// Steps completed so far.
    pub steps_completed: Vec<String>,

    /// Commands executed.
    pub commands_executed: Vec<String>,

    /// Compensation commands executed.
    pub compensations_executed: Vec<String>,

    /// Timestamp when started.
    pub started_at: DateTime<Utc>,

    /// Timestamp when completed/failed.
    pub ended_at: Option<DateTime<Utc>>,

    /// Error message if failed.
    pub error: Option<String>,

    /// Custom metadata.
    pub metadata: std::collections::HashMap<String, String>,
}

impl SagaInstance {
    /// Creates a new saga instance.
    pub fn new(saga_type: impl Into<String>, correlation_id: Uuid) -> Self {
        Self {
            saga_id: Uuid::new_v4(),
            saga_type: saga_type.into(),
            state: SagaState::Starting,
            correlation_id,
            steps_completed: Vec::new(),
            commands_executed: Vec::new(),
            compensations_executed: Vec::new(),
            started_at: Utc::now(),
            ended_at: None,
            error: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Records a completed step.
    pub fn record_step(&mut self, step: impl Into<String>) {
        self.steps_completed.push(step.into());
    }

    /// Records an executed command.
    pub fn record_command(&mut self, command: impl Into<String>) {
        self.commands_executed.push(command.into());
    }

    /// Records an executed compensation.
    pub fn record_compensation(&mut self, compensation: impl Into<String>) {
        self.compensations_executed.push(compensation.into());
    }

    /// Marks the saga as completed.
    pub fn complete(&mut self) {
        self.state = SagaState::Completed;
        self.ended_at = Some(Utc::now());
    }

    /// Marks the saga as failed.
    pub fn fail(&mut self, error: impl Into<String>) {
        self.state = SagaState::Failed;
        self.error = Some(error.into());
        self.ended_at = Some(Utc::now());
    }

    /// Starts compensation.
    pub fn start_compensation(&mut self) {
        self.state = SagaState::Compensating;
    }

    /// Returns whether the saga is in a terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.state,
            SagaState::Completed | SagaState::Failed | SagaState::Aborted
        )
    }

    /// Returns the duration of the saga.
    pub fn duration(&self) -> Option<chrono::Duration> {
        self.ended_at.map(|end| end - self.started_at)
    }
}

/// Manager for coordinating sagas.
pub struct SagaManager {
    /// Active saga instances.
    instances: Arc<DashMap<Uuid, SagaInstance>>,

    /// Saga definitions indexed by type.
    definitions: Arc<DashMap<String, Arc<dyn SagaFactory>>>,
}

impl SagaManager {
    /// Creates a new saga manager.
    pub fn new() -> Self {
        Self {
            instances: Arc::new(DashMap::new()),
            definitions: Arc::new(DashMap::new()),
        }
    }

    /// Registers a saga definition.
    pub fn register(&self, saga_type: &str, factory: Arc<dyn SagaFactory>) {
        self.definitions.insert(saga_type.to_string(), factory);
    }

    /// Starts a new saga instance.
    pub async fn start_saga(
        &self,
        saga_type: &str,
        correlation_id: Uuid,
    ) -> Result<Uuid> {
        let factory = self
            .definitions
            .get(saga_type)
            .ok_or_else(|| {
                EventSourcingError::Saga(format!("Unknown saga type: {}", saga_type))
            })?;

        let mut instance = SagaInstance::new(saga_type, correlation_id);
        instance.state = SagaState::Running;

        let saga_id = instance.saga_id;
        self.instances.insert(saga_id, instance);

        Ok(saga_id)
    }

    /// Handles an event for a saga.
    pub async fn handle_event<E>(
        &self,
        saga_id: &Uuid,
        event: &EventEnvelope<E>,
    ) -> Result<SagaAction>
    where
        E: Event + serde::Serialize,
    {
        let mut instance = self
            .instances
            .get_mut(saga_id)
            .ok_or_else(|| EventSourcingError::Saga(format!("Saga not found: {}", saga_id)))?;

        if instance.is_terminal() {
            return Ok(SagaAction::None);
        }

        let saga_type = instance.saga_type.clone();
        drop(instance);

        let factory = self
            .definitions
            .get(&saga_type)
            .ok_or_else(|| {
                EventSourcingError::Saga(format!("Unknown saga type: {}", saga_type))
            })?;

        let mut saga = factory.create();

        let event_data = serde_json::to_vec(&event.payload)
            .map_err(EventSourcingError::serialization)?;
        let event_type = event.payload.event_type();

        let action = saga.handle_event(&event_data, event_type).await?;

        // Update instance based on action
        let mut instance = self.instances.get_mut(saga_id).unwrap();
        match &action {
            SagaAction::Complete => instance.complete(),
            SagaAction::Fail(error) => instance.fail(error),
            SagaAction::Compensate(_) => instance.start_compensation(),
            _ => {}
        }

        Ok(action)
    }

    /// Gets a saga instance.
    pub fn get_instance(&self, saga_id: &Uuid) -> Option<SagaInstance> {
        self.instances.get(saga_id).map(|i| i.clone())
    }

    /// Lists all active sagas.
    pub fn list_active(&self) -> Vec<SagaInstance> {
        self.instances
            .iter()
            .filter(|entry| !entry.value().is_terminal())
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Lists all completed sagas.
    pub fn list_completed(&self) -> Vec<SagaInstance> {
        self.instances
            .iter()
            .filter(|entry| entry.value().state == SagaState::Completed)
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Removes completed sagas older than the specified duration.
    pub fn cleanup_old_sagas(&self, older_than: chrono::Duration) {
        let cutoff = Utc::now() - older_than;

        self.instances.retain(|_, instance| {
            if let Some(ended_at) = instance.ended_at {
                ended_at > cutoff
            } else {
                true
            }
        });
    }

    /// Returns the total number of saga instances.
    pub fn total_instances(&self) -> usize {
        self.instances.len()
    }
}

impl Default for SagaManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for SagaManager {
    fn clone(&self) -> Self {
        Self {
            instances: Arc::clone(&self.instances),
            definitions: Arc::clone(&self.definitions),
        }
    }
}

/// Factory trait for creating saga instances.
pub trait SagaFactory: Send + Sync {
    /// Creates a new saga instance.
    fn create(&self) -> Box<dyn Saga>;
}

/// Saga step definition for building sagas declaratively.
#[derive(Clone)]
pub struct SagaStep {
    /// Step name.
    pub name: String,

    /// Command to execute.
    pub command: String,

    /// Compensation command to execute on rollback.
    pub compensation: Option<String>,

    /// Event types this step waits for.
    pub wait_for_events: Vec<String>,
}

impl SagaStep {
    /// Creates a new saga step.
    pub fn new(name: impl Into<String>, command: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            command: command.into(),
            compensation: None,
            wait_for_events: Vec::new(),
        }
    }

    /// Sets the compensation command.
    pub fn with_compensation(mut self, compensation: impl Into<String>) -> Self {
        self.compensation = Some(compensation.into());
        self
    }

    /// Adds an event type to wait for.
    pub fn wait_for(mut self, event_type: impl Into<String>) -> Self {
        self.wait_for_events.push(event_type.into());
        self
    }
}

/// Builder for creating sagas declaratively.
pub struct SagaBuilder {
    saga_type: String,
    steps: Vec<SagaStep>,
}

impl SagaBuilder {
    /// Creates a new saga builder.
    pub fn new(saga_type: impl Into<String>) -> Self {
        Self {
            saga_type: saga_type.into(),
            steps: Vec::new(),
        }
    }

    /// Adds a step to the saga.
    pub fn add_step(mut self, step: SagaStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Returns the saga type.
    pub fn saga_type(&self) -> &str {
        &self.saga_type
    }

    /// Returns the steps.
    pub fn steps(&self) -> &[SagaStep] {
        &self.steps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_saga_instance() {
        let correlation_id = Uuid::new_v4();
        let mut instance = SagaInstance::new("TestSaga", correlation_id);

        assert_eq!(instance.state, SagaState::Starting);
        assert!(!instance.is_terminal());

        instance.record_step("step1");
        instance.record_command("command1");

        assert_eq!(instance.steps_completed.len(), 1);
        assert_eq!(instance.commands_executed.len(), 1);

        instance.complete();
        assert_eq!(instance.state, SagaState::Completed);
        assert!(instance.is_terminal());
        assert!(instance.duration().is_some());
    }

    #[test]
    fn test_saga_manager() {
        let manager = SagaManager::new();
        assert_eq!(manager.total_instances(), 0);

        let active = manager.list_active();
        assert_eq!(active.len(), 0);
    }

    #[test]
    fn test_saga_builder() {
        let step = SagaStep::new("step1", "command1")
            .with_compensation("compensation1")
            .wait_for("Event1");

        assert_eq!(step.name, "step1");
        assert_eq!(step.command, "command1");
        assert!(step.compensation.is_some());
        assert_eq!(step.wait_for_events.len(), 1);

        let saga = SagaBuilder::new("TestSaga").add_step(step);

        assert_eq!(saga.saga_type(), "TestSaga");
        assert_eq!(saga.steps().len(), 1);
    }
}
