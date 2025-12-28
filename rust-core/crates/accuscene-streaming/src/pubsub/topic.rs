//! Topic management for pub/sub system.

use crate::bus::{EventBus, EventBusBuilder, EventBusType};
use crate::error::{Result, StreamingError};
use crate::event::{Event, EventFilter, TopicName};
use crate::pubsub::subscriber::{DefaultSubscriber, Subscriber, SubscriberId};
use dashmap::DashMap;
use std::sync::Arc;

/// A topic for pub/sub messaging
pub struct Topic {
    /// Topic name
    pub name: TopicName,
    /// Event bus for this topic
    event_bus: Arc<dyn EventBus>,
    /// Active subscribers
    subscribers: DashMap<SubscriberId, ()>,
}

impl Topic {
    /// Create a new topic with default event bus
    pub fn new(name: impl Into<TopicName>) -> Self {
        let event_bus = EventBusBuilder::new(EventBusType::Broadcast)
            .with_capacity(1000)
            .build();

        Self {
            name: name.into(),
            event_bus: Arc::from(event_bus),
            subscribers: DashMap::new(),
        }
    }

    /// Create a new topic with custom event bus
    pub fn with_bus(name: impl Into<TopicName>, event_bus: Box<dyn EventBus>) -> Self {
        Self {
            name: name.into(),
            event_bus: Arc::from(event_bus),
            subscribers: DashMap::new(),
        }
    }

    /// Publish an event to this topic
    pub async fn publish(&self, event: Event) -> Result<()> {
        self.event_bus.publish(event).await
    }

    /// Subscribe to this topic
    pub async fn subscribe(&self) -> Result<Box<dyn Subscriber>> {
        let stream = self.event_bus.subscribe(EventFilter::default()).await?;
        let subscriber = DefaultSubscriber::new(stream);

        // Track subscriber
        self.subscribers.insert(subscriber.id().clone(), ());

        Ok(Box::new(subscriber))
    }

    /// Unsubscribe from this topic
    pub async fn unsubscribe(&self, subscriber_id: &SubscriberId) -> Result<()> {
        self.subscribers.remove(subscriber_id);
        Ok(())
    }

    /// Get the number of active subscribers
    pub async fn subscriber_count(&self) -> usize {
        self.subscribers.len()
    }
}

/// Topic manager for managing multiple topics
pub struct TopicManager {
    /// All topics indexed by name
    topics: DashMap<TopicName, Arc<Topic>>,
    /// Factory for creating event buses
    bus_factory: Option<Arc<dyn Fn() -> Box<dyn EventBus> + Send + Sync>>,
}

impl TopicManager {
    /// Create a new topic manager
    pub fn new() -> Self {
        Self {
            topics: DashMap::new(),
            bus_factory: None,
        }
    }

    /// Create a new topic manager with custom event bus factory
    pub fn with_bus_factory<F>(bus_factory: F) -> Self
    where
        F: Fn() -> Box<dyn EventBus> + Send + Sync + 'static,
    {
        Self {
            topics: DashMap::new(),
            bus_factory: Some(Arc::new(bus_factory)),
        }
    }

    /// Get or create a topic
    pub async fn get_or_create_topic(&self, name: &TopicName) -> Arc<Topic> {
        if let Some(topic) = self.topics.get(name) {
            return topic.clone();
        }

        // Create new topic
        let topic = if let Some(ref factory) = self.bus_factory {
            Arc::new(Topic::with_bus(name.clone(), factory()))
        } else {
            Arc::new(Topic::new(name.clone()))
        };

        self.topics.insert(name.clone(), topic.clone());
        topic
    }

    /// Create a new topic
    pub async fn create_topic(&self, name: &TopicName) -> Result<()> {
        if self.topics.contains_key(name) {
            return Err(StreamingError::Subscription(format!(
                "Topic '{}' already exists",
                name
            )));
        }

        let topic = if let Some(ref factory) = self.bus_factory {
            Arc::new(Topic::with_bus(name.clone(), factory()))
        } else {
            Arc::new(Topic::new(name.clone()))
        };

        self.topics.insert(name.clone(), topic);
        Ok(())
    }

    /// Delete a topic
    pub async fn delete_topic(&self, name: &TopicName) -> Result<()> {
        self.topics
            .remove(name)
            .ok_or_else(|| StreamingError::TopicNotFound(name.clone()))?;
        Ok(())
    }

    /// Get a topic
    pub async fn get_topic(&self, name: &TopicName) -> Result<Arc<Topic>> {
        self.topics
            .get(name)
            .map(|t| t.clone())
            .ok_or_else(|| StreamingError::TopicNotFound(name.clone()))
    }

    /// List all topics
    pub async fn list_topics(&self) -> Vec<TopicName> {
        self.topics.iter().map(|entry| entry.key().clone()).collect()
    }

    /// Publish to a topic
    pub async fn publish(&self, topic_name: &TopicName, event: Event) -> Result<()> {
        let topic = self.get_or_create_topic(topic_name).await;
        topic.publish(event).await
    }

    /// Subscribe to a topic
    pub async fn subscribe(&self, topic_name: &TopicName) -> Result<Box<dyn Subscriber>> {
        let topic = self.get_or_create_topic(topic_name).await;
        topic.subscribe().await
    }

    /// Unsubscribe from a topic
    pub async fn unsubscribe(&self, topic_name: &TopicName, subscriber_id: &SubscriberId) -> Result<()> {
        let topic = self.get_topic(topic_name).await?;
        topic.unsubscribe(subscriber_id).await
    }

    /// Get subscriber count for a topic
    pub async fn subscriber_count(&self, topic_name: &TopicName) -> Result<usize> {
        let topic = self.get_topic(topic_name).await?;
        Ok(topic.subscriber_count().await)
    }
}

impl Default for TopicManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{EventPayload, EventType};

    #[tokio::test]
    async fn test_topic_creation() {
        let manager = TopicManager::new();

        manager.create_topic(&"test-topic".to_string()).await.unwrap();

        let topics = manager.list_topics().await;
        assert_eq!(topics.len(), 1);
        assert_eq!(topics[0], "test-topic");
    }

    #[tokio::test]
    async fn test_topic_publish_subscribe() {
        let topic = Topic::new("test");

        let mut subscriber = topic.subscribe().await.unwrap();

        let event = Event::new(EventType::CaseCreated, EventPayload::Empty);
        topic.publish(event).await.unwrap();

        let received = subscriber.next().await;
        assert!(received.is_some());
    }

    #[tokio::test]
    async fn test_topic_manager_operations() {
        let manager = TopicManager::new();
        let topic_name = "test-topic".to_string();

        // Create topic
        manager.create_topic(&topic_name).await.unwrap();

        // Subscribe
        let _subscriber = manager.subscribe(&topic_name).await.unwrap();

        // Check subscriber count
        let count = manager.subscriber_count(&topic_name).await.unwrap();
        assert_eq!(count, 1);

        // Delete topic
        manager.delete_topic(&topic_name).await.unwrap();

        // Verify deletion
        assert!(manager.get_topic(&topic_name).await.is_err());
    }
}
