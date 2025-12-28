//! Broadcast event bus implementation using async-broadcast.

use crate::bus::{EventBus, EventStream};
use crate::error::Result;
use crate::event::{Event, EventFilter};
use async_broadcast::{broadcast, Receiver, Sender};
use async_trait::async_trait;
use futures::stream::{self, StreamExt};
use parking_lot::RwLock;
use std::sync::Arc;

/// Broadcast event bus using async-broadcast
pub struct BroadcastEventBus {
    /// Broadcast sender
    sender: Sender<Event>,
    /// Receiver template (kept to create new receivers)
    receiver: Receiver<Event>,
    /// Subscriber count
    subscriber_count: Arc<RwLock<usize>>,
}

impl BroadcastEventBus {
    /// Create a new broadcast event bus with specified capacity
    pub fn new(capacity: usize) -> Self {
        let (sender, receiver) = broadcast(capacity);
        Self {
            sender,
            receiver,
            subscriber_count: Arc::new(RwLock::new(0)),
        }
    }
}

#[async_trait]
impl EventBus for BroadcastEventBus {
    async fn publish(&self, event: Event) -> Result<()> {
        // async-broadcast doesn't fail on send, it just overwrites old messages if full
        self.sender.broadcast(event).await.ok();
        Ok(())
    }

    async fn publish_batch(&self, events: Vec<Event>) -> Result<()> {
        for event in events {
            self.sender.broadcast(event).await.ok();
        }
        Ok(())
    }

    async fn subscribe(&self, filter: EventFilter) -> Result<EventStream> {
        let receiver = self.receiver.clone();

        // Increment subscriber count
        {
            let mut count = self.subscriber_count.write();
            *count += 1;
        }

        let subscriber_count = self.subscriber_count.clone();

        let stream = stream::unfold(
            (receiver, filter, subscriber_count, false),
            |(mut rx, filter, count, mut decremented)| async move {
                loop {
                    match rx.recv().await {
                        Ok(event) => {
                            if filter.matches(&event) {
                                return Some((event, (rx, filter, count, decremented)));
                            }
                            // Event doesn't match filter, continue
                        }
                        Err(_) => {
                            // Channel closed or error, end the stream
                            if !decremented {
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
        // Reset subscriber count
        {
            let mut count = self.subscriber_count.write();
            *count = 0;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{EventPayload, EventType};
    

    #[tokio::test]
    async fn test_broadcast_publish_subscribe() {
        let bus = BroadcastEventBus::new(100);

        let mut stream = bus.subscribe(EventFilter::default()).await.unwrap();

        let event = Event::new(EventType::SimulationStarted, EventPayload::Empty);
        bus.publish(event.clone()).await.unwrap();

        let received = tokio::time::timeout(
            std::time::Duration::from_millis(100),
            stream.next()
        ).await;

        assert!(received.is_ok());
    }

    #[tokio::test]
    async fn test_broadcast_multiple_subscribers() {
        let bus = BroadcastEventBus::new(100);

        let mut stream1 = bus.subscribe(EventFilter::default()).await.unwrap();
        let mut stream2 = bus.subscribe(EventFilter::default()).await.unwrap();

        assert_eq!(bus.subscriber_count().await, 2);

        let event = Event::new(EventType::Connected, EventPayload::Empty);
        bus.publish(event.clone()).await.unwrap();

        let received1 = tokio::time::timeout(
            std::time::Duration::from_millis(100),
            stream1.next()
        ).await;
        let received2 = tokio::time::timeout(
            std::time::Duration::from_millis(100),
            stream2.next()
        ).await;

        assert!(received1.is_ok());
        assert!(received2.is_ok());
    }

    #[tokio::test]
    async fn test_broadcast_filter() {
        let bus = BroadcastEventBus::new(100);

        let filter = EventFilter::new()
            .with_types(vec![EventType::SimulationUpdate]);
        let mut stream = bus.subscribe(filter).await.unwrap();

        // Publish non-matching event
        bus.publish(Event::new(EventType::UserJoined, EventPayload::Empty))
            .await
            .unwrap();

        // Publish matching event
        bus.publish(Event::new(EventType::SimulationUpdate, EventPayload::Empty))
            .await
            .unwrap();

        // Should receive only the matching event
        let received = tokio::time::timeout(
            std::time::Duration::from_millis(100),
            stream.next()
        ).await;

        assert!(received.is_ok());
        if let Ok(Some(event)) = received {
            assert_eq!(event.event_type, EventType::SimulationUpdate);
        }
    }
}
