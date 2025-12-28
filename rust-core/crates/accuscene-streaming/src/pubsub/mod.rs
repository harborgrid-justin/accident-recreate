//! Pub/sub system for topic-based event distribution.

pub mod publisher;
pub mod subscriber;
pub mod topic;

use crate::bus::EventBus;
use crate::error::Result;
use crate::event::{Event, TopicName};
use async_trait::async_trait;
use std::sync::Arc;

pub use publisher::Publisher;
pub use subscriber::{Subscriber, SubscriberId};
pub use topic::{Topic, TopicManager};

/// Pub/sub system trait
#[async_trait]
pub trait PubSub: Send + Sync {
    /// Publish an event to a topic
    async fn publish(&self, topic: &TopicName, event: Event) -> Result<()>;

    /// Subscribe to a topic
    async fn subscribe(&self, topic: &TopicName) -> Result<Box<dyn Subscriber>>;

    /// Unsubscribe from a topic
    async fn unsubscribe(&self, topic: &TopicName, subscriber_id: &SubscriberId) -> Result<()>;

    /// Create a new topic
    async fn create_topic(&self, topic: &TopicName) -> Result<()>;

    /// Delete a topic
    async fn delete_topic(&self, topic: &TopicName) -> Result<()>;

    /// List all topics
    async fn list_topics(&self) -> Result<Vec<TopicName>>;

    /// Get subscriber count for a topic
    async fn subscriber_count(&self, topic: &TopicName) -> Result<usize>;
}

/// Default pub/sub implementation
pub struct DefaultPubSub {
    topic_manager: Arc<TopicManager>,
}

impl DefaultPubSub {
    /// Create a new pub/sub system with default event bus
    pub fn new() -> Self {
        Self {
            topic_manager: Arc::new(TopicManager::new()),
        }
    }

    /// Create a new pub/sub system with custom event bus factory
    pub fn with_bus_factory<F>(bus_factory: F) -> Self
    where
        F: Fn() -> Box<dyn EventBus> + Send + Sync + 'static,
    {
        Self {
            topic_manager: Arc::new(TopicManager::with_bus_factory(bus_factory)),
        }
    }
}

impl Default for DefaultPubSub {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PubSub for DefaultPubSub {
    async fn publish(&self, topic_name: &TopicName, event: Event) -> Result<()> {
        self.topic_manager.publish(topic_name, event).await
    }

    async fn subscribe(&self, topic_name: &TopicName) -> Result<Box<dyn Subscriber>> {
        self.topic_manager.subscribe(topic_name).await
    }

    async fn unsubscribe(&self, topic_name: &TopicName, subscriber_id: &SubscriberId) -> Result<()> {
        self.topic_manager.unsubscribe(topic_name, subscriber_id).await
    }

    async fn create_topic(&self, topic_name: &TopicName) -> Result<()> {
        self.topic_manager.create_topic(topic_name).await
    }

    async fn delete_topic(&self, topic_name: &TopicName) -> Result<()> {
        self.topic_manager.delete_topic(topic_name).await
    }

    async fn list_topics(&self) -> Result<Vec<TopicName>> {
        Ok(self.topic_manager.list_topics().await)
    }

    async fn subscriber_count(&self, topic_name: &TopicName) -> Result<usize> {
        self.topic_manager.subscriber_count(topic_name).await
    }
}
