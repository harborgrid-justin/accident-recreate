//! In-memory event bus implementation.

use crate::bus::{EventBus, EventStream};
use crate::error::{Result, StreamingError};
use crate::event::{Event, EventFilter};
use async_trait::async_trait;
use futures::stream::{self, StreamExt};
use parking_lot::RwLock;
use std::sync::Arc;
use tokio::sync::broadcast;

/// In-memory event bus that stores events and broadcasts to multiple subscribers
pub struct MemoryEventBus {
    /// Broadcast channel for events
    sender: broadcast::Sender<Event>,
    /// Receiver template (kept to create new receivers)
    _receiver: broadcast::Receiver<Event>,
    /// Subscriber count
    subscriber_count: Arc<RwLock<usize>>,
}

impl MemoryEventBus {
    /// Create a new in-memory event bus with default capacity (1000)
    pub fn new() -> Self {
        Self::with_capacity(1000)
    }

    /// Create a new in-memory event bus with specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        let (sender, receiver) = broadcast::channel(capacity);
        Self {
            sender,
            _receiver: receiver,
            subscriber_count: Arc::new(RwLock::new(0)),
        }
    }
}

impl Default for MemoryEventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventBus for MemoryEventBus {
    async fn publish(&self, event: Event) -> Result<()> {
        self.sender
            .send(event)
            .map_err(|e| StreamingError::EventBus(format!("Failed to publish event: {}", e)))?;
        Ok(())
    }

    async fn subscribe(&self, filter: EventFilter) -> Result<EventStream> {
        let mut receiver = self.sender.subscribe();

        // Increment subscriber count
        {
            let mut count = self.subscriber_count.write();
            *count += 1;
        }

        let subscriber_count = self.subscriber_count.clone();

        let stream = stream::unfold(
            (receiver, filter, subscriber_count, false),
            |(mut rx, filter, count, mut dropped)| async move {
                loop {
                    match rx.recv().await {
                        Ok(event) => {
                            if filter.matches(&event) {
                                return Some((event, (rx, filter, count, dropped)));
                            }
                            // Event doesn't match filter, continue
                        }
                        Err(broadcast::error::RecvError::Lagged(_)) => {
                            // Subscriber lagged behind, continue receiving
                            if !dropped {
                                dropped = true;
                                tracing::warn!("Subscriber lagged behind, some events may have been dropped");
                            }
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            // Channel closed, end the stream
                            if !dropped {
                                let mut c = count.write();
                                if *c > 0 {
                                    *c -= 1;
                                }
                            }
                            return None;
                        }
                    }
                }
            },
        );

        Ok(Box::pin(stream))
    }

    async fn subscriber_count(&self) -> usize {
        *self.subscriber_count.read()
    }

    async fn clear(&self) -> Result<()> {
        // Broadcast channels don't support clearing
        // We can only track and report subscriber count
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{EventPayload, EventType};
    

    #[tokio::test]
    async fn test_publish_subscribe() {
        let bus = MemoryEventBus::new();

        let mut stream = bus.subscribe(EventFilter::default()).await.unwrap();

        let event = Event::new(EventType::SimulationUpdate, EventPayload::Empty);
        bus.publish(event.clone()).await.unwrap();

        let received = stream.next().await;
        assert!(received.is_some());
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let bus = MemoryEventBus::new();

        let mut stream1 = bus.subscribe(EventFilter::default()).await.unwrap();
        let mut stream2 = bus.subscribe(EventFilter::default()).await.unwrap();

        assert_eq!(bus.subscriber_count().await, 2);

        let event = Event::new(EventType::UserJoined, EventPayload::Empty);
        bus.publish(event.clone()).await.unwrap();

        assert!(stream1.next().await.is_some());
        assert!(stream2.next().await.is_some());
    }

    #[tokio::test]
    async fn test_filtered_subscription() {
        let bus = MemoryEventBus::new();

        let filter = EventFilter::new()
            .with_types(vec![EventType::UserJoined]);
        let mut stream = bus.subscribe(filter).await.unwrap();

        // Publish a non-matching event
        bus.publish(Event::new(EventType::SimulationUpdate, EventPayload::Empty))
            .await
            .unwrap();

        // Publish a matching event
        bus.publish(Event::new(EventType::UserJoined, EventPayload::Empty))
            .await
            .unwrap();

        // Should only receive the matching event
        let received = tokio::time::timeout(
            std::time::Duration::from_millis(100),
            stream.next()
        ).await;

        assert!(received.is_ok());
        if let Ok(Some(event)) = received {
            assert_eq!(event.event_type, EventType::UserJoined);
        }
    }
}
