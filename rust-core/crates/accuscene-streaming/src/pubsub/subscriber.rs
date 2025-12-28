//! Subscriber handling for pub/sub system.

use crate::bus::EventStream;
use crate::event::Event;
use async_trait::async_trait;
use futures::StreamExt;
use uuid::Uuid;

/// Unique identifier for a subscriber
pub type SubscriberId = Uuid;

/// Subscriber trait for receiving events
#[async_trait]
pub trait Subscriber: Send + Sync {
    /// Get the subscriber's unique ID
    fn id(&self) -> &SubscriberId;

    /// Receive the next event (blocking)
    async fn next(&mut self) -> Option<Event>;

    /// Try to receive an event without blocking
    async fn try_next(&mut self) -> Option<Event>;

    /// Close the subscriber
    async fn close(&mut self);
}

/// Default subscriber implementation
pub struct DefaultSubscriber {
    id: SubscriberId,
    stream: EventStream,
    closed: bool,
}

impl DefaultSubscriber {
    /// Create a new subscriber
    pub fn new(stream: EventStream) -> Self {
        Self {
            id: Uuid::new_v4(),
            stream,
            closed: false,
        }
    }

    /// Create a subscriber with a specific ID
    pub fn with_id(id: SubscriberId, stream: EventStream) -> Self {
        Self {
            id,
            stream,
            closed: false,
        }
    }
}

#[async_trait]
impl Subscriber for DefaultSubscriber {
    fn id(&self) -> &SubscriberId {
        &self.id
    }

    async fn next(&mut self) -> Option<Event> {
        if self.closed {
            return None;
        }
        self.stream.next().await
    }

    async fn try_next(&mut self) -> Option<Event> {
        if self.closed {
            return None;
        }

        // Use a small timeout to make it non-blocking
        match tokio::time::timeout(
            std::time::Duration::from_millis(1),
            self.stream.next()
        ).await {
            Ok(event) => event,
            Err(_) => None,
        }
    }

    async fn close(&mut self) {
        self.closed = true;
    }
}

/// Subscriber builder for creating subscribers with configuration
pub struct SubscriberBuilder {
    id: Option<SubscriberId>,
}

impl SubscriberBuilder {
    /// Create a new subscriber builder
    pub fn new() -> Self {
        Self { id: None }
    }

    /// Set a specific subscriber ID
    pub fn with_id(mut self, id: SubscriberId) -> Self {
        self.id = Some(id);
        self
    }

    /// Build the subscriber
    pub fn build(self, stream: EventStream) -> DefaultSubscriber {
        match self.id {
            Some(id) => DefaultSubscriber::with_id(id, stream),
            None => DefaultSubscriber::new(stream),
        }
    }
}

impl Default for SubscriberBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{EventPayload, EventType};
    use futures::stream;

    #[tokio::test]
    async fn test_subscriber_creation() {
        let events = vec![
            Event::new(EventType::UserJoined, EventPayload::Empty),
            Event::new(EventType::UserLeft, EventPayload::Empty),
        ];

        let stream = Box::pin(stream::iter(events));
        let mut subscriber = DefaultSubscriber::new(stream);

        assert!(subscriber.next().await.is_some());
        assert!(subscriber.next().await.is_some());
        assert!(subscriber.next().await.is_none());
    }

    #[tokio::test]
    async fn test_subscriber_close() {
        let events = vec![
            Event::new(EventType::Connected, EventPayload::Empty),
        ];

        let stream = Box::pin(stream::iter(events));
        let mut subscriber = DefaultSubscriber::new(stream);

        subscriber.close().await;
        assert!(subscriber.next().await.is_none());
    }

    #[tokio::test]
    async fn test_subscriber_builder() {
        let events = vec![
            Event::new(EventType::SimulationStarted, EventPayload::Empty),
        ];

        let stream = Box::pin(stream::iter(events));
        let custom_id = Uuid::new_v4();

        let subscriber = SubscriberBuilder::new()
            .with_id(custom_id)
            .build(stream);

        assert_eq!(subscriber.id(), &custom_id);
    }
}
