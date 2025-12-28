//! Channel-based event bus implementation.

use crate::bus::{EventBus, EventStream};
use crate::error::{Result, StreamingError};
use crate::event::{Event, EventFilter};
use async_channel::{Receiver, Sender};
use async_trait::async_trait;
use futures::stream::{self, StreamExt};
use parking_lot::RwLock;
use std::sync::Arc;

/// Channel-based event bus using async-channel
pub struct ChannelEventBus {
    /// Subscribers receiving events
    subscribers: Arc<RwLock<Vec<Sender<Event>>>>,
    /// Channel capacity
    capacity: usize,
}

impl ChannelEventBus {
    /// Create a new channel-based event bus with specified capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            subscribers: Arc::new(RwLock::new(Vec::new())),
            capacity,
        }
    }
}

#[async_trait]
impl EventBus for ChannelEventBus {
    async fn publish(&self, event: Event) -> Result<()> {
        let subscribers = self.subscribers.read();

        // Send to all subscribers
        for sender in subscribers.iter() {
            // Try to send, but don't block if the channel is full
            if let Err(e) = sender.try_send(event.clone()) {
                tracing::warn!("Failed to send event to subscriber: {}", e);
            }
        }

        Ok(())
    }

    async fn publish_batch(&self, events: Vec<Event>) -> Result<()> {
        let subscribers = self.subscribers.read();

        for event in events {
            for sender in subscribers.iter() {
                if let Err(e) = sender.try_send(event.clone()) {
                    tracing::warn!("Failed to send event to subscriber: {}", e);
                }
            }
        }

        Ok(())
    }

    async fn subscribe(&self, filter: EventFilter) -> Result<EventStream> {
        let (sender, receiver) = async_channel::bounded::<Event>(self.capacity);

        // Add sender to subscribers list
        {
            let mut subscribers = self.subscribers.write();
            subscribers.push(sender);
        }

        let stream = create_filtered_stream(receiver, filter);

        Ok(Box::pin(stream))
    }

    async fn subscriber_count(&self) -> usize {
        self.subscribers.read().len()
    }

    async fn clear(&self) -> Result<()> {
        let mut subscribers = self.subscribers.write();
        subscribers.clear();
        Ok(())
    }
}

/// Create a filtered stream from a receiver
fn create_filtered_stream(
    receiver: Receiver<Event>,
    filter: EventFilter,
) -> impl futures::Stream<Item = Event> {
    stream::unfold((receiver, filter), |(rx, filter)| async move {
        loop {
            match rx.recv().await {
                Ok(event) => {
                    if filter.matches(&event) {
                        return Some((event, (rx, filter)));
                    }
                    // Event doesn't match filter, continue
                }
                Err(_) => {
                    // Channel closed
                    return None;
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{EventPayload, EventType};
    

    #[tokio::test]
    async fn test_channel_publish_subscribe() {
        let bus = ChannelEventBus::new(100);

        let mut stream = bus.subscribe(EventFilter::default()).await.unwrap();

        let event = Event::new(EventType::CaseCreated, EventPayload::Empty);
        bus.publish(event.clone()).await.unwrap();

        let received = stream.next().await;
        assert!(received.is_some());
    }

    #[tokio::test]
    async fn test_channel_multiple_subscribers() {
        let bus = ChannelEventBus::new(100);

        let mut stream1 = bus.subscribe(EventFilter::default()).await.unwrap();
        let mut stream2 = bus.subscribe(EventFilter::default()).await.unwrap();

        assert_eq!(bus.subscriber_count().await, 2);

        let event = Event::new(EventType::UserJoined, EventPayload::Empty);
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
    async fn test_channel_clear() {
        let bus = ChannelEventBus::new(100);

        let _stream = bus.subscribe(EventFilter::default()).await.unwrap();
        assert_eq!(bus.subscriber_count().await, 1);

        bus.clear().await.unwrap();
        assert_eq!(bus.subscriber_count().await, 0);
    }
}
