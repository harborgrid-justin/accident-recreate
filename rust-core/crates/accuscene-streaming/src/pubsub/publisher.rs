//! Publisher interface for pub/sub system.

use crate::error::Result;
use crate::event::{Event, TopicName};
use async_trait::async_trait;
use std::sync::Arc;

/// Publisher trait for publishing events to topics
#[async_trait]
pub trait Publisher: Send + Sync {
    /// Publish an event to a topic
    async fn publish(&self, topic: &TopicName, event: Event) -> Result<()>;

    /// Publish multiple events to a topic
    async fn publish_batch(&self, topic: &TopicName, events: Vec<Event>) -> Result<()>;

    /// Publish an event to multiple topics
    async fn publish_to_many(&self, topics: Vec<TopicName>, event: Event) -> Result<()>;
}

/// Default publisher implementation using topic manager
pub struct DefaultPublisher<T>
where
    T: super::PubSub,
{
    pubsub: Arc<T>,
}

impl<T> DefaultPublisher<T>
where
    T: super::PubSub,
{
    /// Create a new publisher
    pub fn new(pubsub: Arc<T>) -> Self {
        Self { pubsub }
    }
}

#[async_trait]
impl<T> Publisher for DefaultPublisher<T>
where
    T: super::PubSub + 'static,
{
    async fn publish(&self, topic: &TopicName, event: Event) -> Result<()> {
        self.pubsub.publish(topic, event).await
    }

    async fn publish_batch(&self, topic: &TopicName, events: Vec<Event>) -> Result<()> {
        for event in events {
            self.pubsub.publish(topic, event).await?;
        }
        Ok(())
    }

    async fn publish_to_many(&self, topics: Vec<TopicName>, event: Event) -> Result<()> {
        for topic in topics {
            self.pubsub.publish(&topic, event.clone()).await?;
        }
        Ok(())
    }
}

/// Publisher builder for creating publishers with configuration
pub struct PublisherBuilder<T>
where
    T: super::PubSub,
{
    pubsub: Arc<T>,
}

impl<T> PublisherBuilder<T>
where
    T: super::PubSub,
{
    /// Create a new publisher builder
    pub fn new(pubsub: Arc<T>) -> Self {
        Self { pubsub }
    }

    /// Build the publisher
    pub fn build(self) -> DefaultPublisher<T> {
        DefaultPublisher::new(self.pubsub)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{EventPayload, EventType};
    use crate::pubsub::DefaultPubSub;

    #[tokio::test]
    async fn test_publisher_publish() {
        let pubsub = Arc::new(DefaultPubSub::new());
        let publisher = DefaultPublisher::new(pubsub.clone());

        let topic = "test-topic".to_string();
        let event = Event::new(EventType::CaseCreated, EventPayload::Empty);

        let result = publisher.publish(&topic, event).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_publisher_batch() {
        let pubsub = Arc::new(DefaultPubSub::new());
        let publisher = DefaultPublisher::new(pubsub);

        let topic = "test-topic".to_string();
        let events = vec![
            Event::new(EventType::SimulationUpdate, EventPayload::Empty),
            Event::new(EventType::SimulationStarted, EventPayload::Empty),
        ];

        let result = publisher.publish_batch(&topic, events).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_publisher_to_many() {
        let pubsub = Arc::new(DefaultPubSub::new());
        let publisher = DefaultPublisher::new(pubsub);

        let topics = vec!["topic1".to_string(), "topic2".to_string()];
        let event = Event::new(EventType::UserJoined, EventPayload::Empty);

        let result = publisher.publish_to_many(topics, event).await;
        assert!(result.is_ok());
    }
}
