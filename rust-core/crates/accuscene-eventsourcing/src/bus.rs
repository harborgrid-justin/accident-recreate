//! Event bus and command bus implementations.

use crate::command::{Command, CommandHandler, CommandResult};
use crate::error::{EventSourcingError, Result};
use crate::event::{Event, EventEnvelope};
use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use uuid::Uuid;

/// Trait for event subscribers.
#[async_trait]
pub trait EventSubscriber: Send + Sync {
    /// Handles an event.
    async fn handle_event(&self, event_data: &[u8], event_type: &str) -> Result<()>;

    /// Returns the event types this subscriber is interested in.
    fn interested_in(&self) -> Vec<&'static str>;
}

/// Event bus for publishing and subscribing to events.
#[derive(Clone)]
pub struct EventBus {
    /// Subscribers indexed by event type.
    subscribers: Arc<DashMap<String, Vec<Arc<dyn EventSubscriber>>>>,

    /// Broadcast channel for all events.
    broadcast_tx: broadcast::Sender<SerializedEventMessage>,
}

/// Serialized event message for the bus.
#[derive(Clone, Debug)]
struct SerializedEventMessage {
    event_type: String,
    event_data: Vec<u8>,
    event_id: Uuid,
    aggregate_id: String,
}

impl EventBus {
    /// Creates a new event bus.
    pub fn new() -> Self {
        let (broadcast_tx, _) = broadcast::channel(1000);

        Self {
            subscribers: Arc::new(DashMap::new()),
            broadcast_tx,
        }
    }

    /// Subscribes to specific event types.
    pub fn subscribe(&self, subscriber: Arc<dyn EventSubscriber>) {
        for event_type in subscriber.interested_in() {
            self.subscribers
                .entry(event_type.to_string())
                .or_insert_with(Vec::new)
                .push(Arc::clone(&subscriber));
        }
    }

    /// Publishes an event to all subscribers.
    pub async fn publish<E>(&self, envelope: &EventEnvelope<E>) -> Result<()>
    where
        E: Event + serde::Serialize,
    {
        let event_type = envelope.payload.event_type();
        let event_data = serde_json::to_vec(&envelope.payload)
            .map_err(EventSourcingError::serialization)?;

        let message = SerializedEventMessage {
            event_type: event_type.to_string(),
            event_data: event_data.clone(),
            event_id: envelope.event_id(),
            aggregate_id: envelope.aggregate_id().to_string(),
        };

        // Broadcast to channel
        let _ = self.broadcast_tx.send(message);

        // Notify specific subscribers
        if let Some(subscribers) = self.subscribers.get(event_type) {
            for subscriber in subscribers.value() {
                if let Err(e) = subscriber.handle_event(&event_data, event_type).await {
                    tracing::error!(
                        event_type = event_type,
                        error = ?e,
                        "Failed to handle event in subscriber"
                    );
                }
            }
        }

        Ok(())
    }

    /// Publishes multiple events.
    pub async fn publish_many<E>(&self, envelopes: &[EventEnvelope<E>]) -> Result<()>
    where
        E: Event + serde::Serialize,
    {
        for envelope in envelopes {
            self.publish(envelope).await?;
        }
        Ok(())
    }

    /// Creates a subscription to receive all events.
    pub fn subscribe_all(&self) -> broadcast::Receiver<SerializedEventMessage> {
        self.broadcast_tx.subscribe()
    }

    /// Returns the number of subscribers for an event type.
    pub fn subscriber_count(&self, event_type: &str) -> usize {
        self.subscribers
            .get(event_type)
            .map(|s| s.len())
            .unwrap_or(0)
    }

    /// Returns the total number of registered subscribers.
    pub fn total_subscribers(&self) -> usize {
        self.subscribers.iter().map(|entry| entry.len()).sum()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// Command bus for routing commands to handlers.
pub struct CommandBus {
    /// Command handlers indexed by command type.
    handlers: DashMap<String, Arc<dyn CommandHandlerWrapper>>,
}

impl CommandBus {
    /// Creates a new command bus.
    pub fn new() -> Self {
        Self {
            handlers: DashMap::new(),
        }
    }

    /// Registers a command handler.
    pub fn register<C, H>(&self, command_type: &str, handler: Arc<H>)
    where
        C: Command + for<'de> serde::Deserialize<'de> + 'static,
        H: CommandHandler<C> + 'static,
    {
        let wrapper = Arc::new(TypedCommandHandler {
            handler,
            _phantom: std::marker::PhantomData,
        });

        self.handlers.insert(command_type.to_string(), wrapper);
    }

    /// Dispatches a command to its handler.
    pub async fn dispatch<C>(&self, command: C) -> Result<CommandResult>
    where
        C: Command + serde::Serialize + 'static,
    {
        let command_type = command.command_type();

        let handler = self
            .handlers
            .get(command_type)
            .ok_or_else(|| {
                EventSourcingError::CommandBus(format!("No handler for command: {}", command_type))
            })?;

        let command_id = Uuid::new_v4();
        let command_data = serde_json::to_vec(&command)
            .map_err(EventSourcingError::serialization)?;

        handler.handle(command_id, &command_data).await
    }

    /// Checks if a handler is registered for a command type.
    pub fn has_handler(&self, command_type: &str) -> bool {
        self.handlers.contains_key(command_type)
    }

    /// Returns the number of registered handlers.
    pub fn handler_count(&self) -> usize {
        self.handlers.len()
    }
}

impl Default for CommandBus {
    fn default() -> Self {
        Self::new()
    }
}

/// Wrapper trait for type-erased command handlers.
#[async_trait]
trait CommandHandlerWrapper: Send + Sync {
    async fn handle(&self, command_id: Uuid, command_data: &[u8]) -> Result<CommandResult>;
}

/// Typed command handler wrapper.
struct TypedCommandHandler<C, H>
where
    C: Command,
    H: CommandHandler<C>,
{
    handler: Arc<H>,
    _phantom: std::marker::PhantomData<C>,
}

#[async_trait]
impl<C, H> CommandHandlerWrapper for TypedCommandHandler<C, H>
where
    C: Command + for<'de> serde::Deserialize<'de> + 'static,
    H: CommandHandler<C> + 'static,
{
    async fn handle(&self, command_id: Uuid, command_data: &[u8]) -> Result<CommandResult> {
        let command: C = serde_json::from_slice(command_data)
            .map_err(EventSourcingError::deserialization)?;

        match self.handler.handle(command).await {
            Ok(events) => Ok(CommandResult::success(command_id, events.len(), 1)),
            Err(e) => Ok(CommandResult::failure(command_id, e.to_string())),
        }
    }
}

/// Async event handler trait for simpler event handling.
#[async_trait]
pub trait AsyncEventHandler<E>: Send + Sync
where
    E: Event,
{
    /// Handles an event.
    async fn handle(&self, event: &EventEnvelope<E>) -> Result<()>;
}

/// Event handler registry for managing typed handlers.
pub struct EventHandlerRegistry {
    handlers: DashMap<String, Vec<Arc<dyn EventSubscriber>>>,
}

impl EventHandlerRegistry {
    /// Creates a new event handler registry.
    pub fn new() -> Self {
        Self {
            handlers: DashMap::new(),
        }
    }

    /// Registers an event handler.
    pub fn register(&self, event_type: &str, handler: Arc<dyn EventSubscriber>) {
        self.handlers
            .entry(event_type.to_string())
            .or_insert_with(Vec::new)
            .push(handler);
    }

    /// Gets handlers for an event type.
    pub fn get_handlers(&self, event_type: &str) -> Vec<Arc<dyn EventSubscriber>> {
        self.handlers
            .get(event_type)
            .map(|h| h.value().clone())
            .unwrap_or_default()
    }

    /// Returns the number of handlers for an event type.
    pub fn handler_count(&self, event_type: &str) -> usize {
        self.handlers
            .get(event_type)
            .map(|h| h.len())
            .unwrap_or(0)
    }
}

impl Default for EventHandlerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::EventMetadata;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestEvent {
        id: String,
        value: i32,
    }

    impl Event for TestEvent {
        fn event_type(&self) -> &'static str {
            "TestEvent"
        }

        fn aggregate_id(&self) -> &str {
            &self.id
        }

        fn aggregate_type(&self) -> &'static str {
            "Test"
        }
    }

    struct TestSubscriber {
        received_count: Arc<std::sync::atomic::AtomicUsize>,
    }

    #[async_trait]
    impl EventSubscriber for TestSubscriber {
        async fn handle_event(&self, _event_data: &[u8], _event_type: &str) -> Result<()> {
            self.received_count
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }

        fn interested_in(&self) -> Vec<&'static str> {
            vec!["TestEvent"]
        }
    }

    #[tokio::test]
    async fn test_event_bus() {
        let bus = EventBus::new();
        let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));

        let subscriber = Arc::new(TestSubscriber {
            received_count: Arc::clone(&counter),
        });

        bus.subscribe(subscriber);

        let event = TestEvent {
            id: "test-1".to_string(),
            value: 42,
        };

        let envelope = EventEnvelope::new(event, 1);
        bus.publish(&envelope).await.unwrap();

        // Small delay to let async processing happen
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1);
        assert_eq!(bus.subscriber_count("TestEvent"), 1);
    }

    #[test]
    fn test_command_bus() {
        let bus = CommandBus::new();
        assert_eq!(bus.handler_count(), 0);
        assert!(!bus.has_handler("TestCommand"));
    }

    #[test]
    fn test_event_handler_registry() {
        let registry = EventHandlerRegistry::new();
        assert_eq!(registry.handler_count("TestEvent"), 0);
    }
}
