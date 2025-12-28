//! Event bus implementations for the AccuScene streaming system.

pub mod broadcast;
pub mod channel;
pub mod memory;

use crate::error::Result;
use crate::event::{Event, EventFilter};
use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;

/// Type alias for event stream
pub type EventStream = Pin<Box<dyn Stream<Item = Event> + Send>>;

/// Event bus trait for publishing and subscribing to events
#[async_trait]
pub trait EventBus: Send + Sync {
    /// Publish an event to the bus
    async fn publish(&self, event: Event) -> Result<()>;

    /// Publish multiple events at once
    async fn publish_batch(&self, events: Vec<Event>) -> Result<()> {
        for event in events {
            self.publish(event).await?;
        }
        Ok(())
    }

    /// Subscribe to events matching the filter
    async fn subscribe(&self, filter: EventFilter) -> Result<EventStream>;

    /// Subscribe to all events
    async fn subscribe_all(&self) -> Result<EventStream> {
        self.subscribe(EventFilter::default()).await
    }

    /// Get the number of active subscribers
    async fn subscriber_count(&self) -> usize;

    /// Clear all subscribers
    async fn clear(&self) -> Result<()>;
}

/// Builder for creating event buses
pub struct EventBusBuilder {
    bus_type: EventBusType,
    capacity: Option<usize>,
}

/// Available event bus implementations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventBusType {
    /// In-memory event bus with multiple subscribers
    Memory,
    /// Channel-based event bus
    Channel,
    /// Broadcast event bus
    Broadcast,
}

impl EventBusBuilder {
    /// Create a new event bus builder
    pub fn new(bus_type: EventBusType) -> Self {
        Self {
            bus_type,
            capacity: None,
        }
    }

    /// Set the channel capacity
    pub fn with_capacity(mut self, capacity: usize) -> Self {
        self.capacity = Some(capacity);
        self
    }

    /// Build the event bus
    pub fn build(self) -> Box<dyn EventBus> {
        match self.bus_type {
            EventBusType::Memory => {
                Box::new(memory::MemoryEventBus::new())
            }
            EventBusType::Channel => {
                let capacity = self.capacity.unwrap_or(1000);
                Box::new(channel::ChannelEventBus::new(capacity))
            }
            EventBusType::Broadcast => {
                let capacity = self.capacity.unwrap_or(1000);
                Box::new(broadcast::BroadcastEventBus::new(capacity))
            }
        }
    }
}

impl Default for EventBusBuilder {
    fn default() -> Self {
        Self::new(EventBusType::Broadcast)
    }
}
