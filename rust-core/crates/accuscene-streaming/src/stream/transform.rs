//! Stream transformation utilities.

use crate::error::Result;
use crate::event::Event;
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use std::pin::Pin;
use std::task::{Context, Poll};

/// Event transformer trait
#[async_trait]
pub trait EventTransformer: Send + Sync {
    /// Transform an event
    async fn transform(&self, event: Event) -> Result<Event>;
}

/// Transform stream that applies a transformer to each event
pub struct TransformStream<S, T>
where
    S: Stream<Item = Event>,
    T: EventTransformer,
{
    inner: S,
    transformer: T,
    pending: Option<Pin<Box<dyn futures::Future<Output = Result<Event>> + Send>>>,
}

impl<S, T> TransformStream<S, T>
where
    S: Stream<Item = Event>,
    T: EventTransformer,
{
    /// Create a new transform stream
    pub fn new(stream: S, transformer: T) -> Self {
        Self {
            inner: stream,
            transformer,
            pending: None,
        }
    }
}

impl<S, T> Stream for TransformStream<S, T>
where
    S: Stream<Item = Event> + Unpin,
    T: EventTransformer + Unpin,
{
    type Item = Result<Event>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Check if we have a pending transformation
        if let Some(mut future) = self.pending.take() {
            match future.as_mut().poll(cx) {
                Poll::Ready(result) => return Poll::Ready(Some(result)),
                Poll::Pending => {
                    self.pending = Some(future);
                    return Poll::Pending;
                }
            }
        }

        // Poll for next event
        match Pin::new(&mut self.inner).poll_next(cx) {
            Poll::Ready(Some(event)) => {
                // Start transformation
                let transformer = &self.transformer;
                let future = Box::pin(transformer.transform(event));
                self.pending = Some(future);

                // Poll the future immediately
                if let Some(mut future) = self.pending.take() {
                    match future.as_mut().poll(cx) {
                        Poll::Ready(result) => Poll::Ready(Some(result)),
                        Poll::Pending => {
                            self.pending = Some(future);
                            Poll::Pending
                        }
                    }
                } else {
                    Poll::Pending
                }
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Simple function-based transformer
pub struct FnTransformer<F>
where
    F: Fn(Event) -> Event + Send + Sync,
{
    func: F,
}

impl<F> FnTransformer<F>
where
    F: Fn(Event) -> Event + Send + Sync,
{
    pub fn new(func: F) -> Self {
        Self { func }
    }
}

#[async_trait]
impl<F> EventTransformer for FnTransformer<F>
where
    F: Fn(Event) -> Event + Send + Sync,
{
    async fn transform(&self, event: Event) -> Result<Event> {
        Ok((self.func)(event))
    }
}

/// Metadata enrichment transformer
pub struct MetadataEnricher {
    room_id: Option<String>,
    user_id: Option<String>,
}

impl MetadataEnricher {
    pub fn new() -> Self {
        Self {
            room_id: None,
            user_id: None,
        }
    }

    pub fn with_room(mut self, room_id: impl Into<String>) -> Self {
        self.room_id = Some(room_id.into());
        self
    }

    pub fn with_user(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }
}

impl Default for MetadataEnricher {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventTransformer for MetadataEnricher {
    async fn transform(&self, mut event: Event) -> Result<Event> {
        if let Some(ref room_id) = self.room_id {
            event.metadata.room_id = Some(room_id.clone());
        }

        if let Some(ref user_id) = self.user_id {
            event.metadata.user_id = Some(user_id.clone());
        }

        Ok(event)
    }
}

/// Extension trait for transforming event streams
pub trait TransformStreamExt: Stream<Item = Event> + Sized {
    /// Transform events with a transformer
    fn transform_with<T: EventTransformer + Unpin>(
        self,
        transformer: T,
    ) -> TransformStream<Self, T> {
        TransformStream::new(self, transformer)
    }

    /// Map events with a function
    fn map_events<F>(self, func: F) -> TransformStream<Self, FnTransformer<F>>
    where
        F: Fn(Event) -> Event + Send + Sync,
    {
        TransformStream::new(self, FnTransformer::new(func))
    }

    /// Enrich events with metadata
    fn enrich_metadata(
        self,
        enricher: MetadataEnricher,
    ) -> TransformStream<Self, MetadataEnricher> {
        TransformStream::new(self, enricher)
    }
}

impl<S> TransformStreamExt for S where S: Stream<Item = Event> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{EventPayload, EventType};
    use futures::stream;

    #[tokio::test]
    async fn test_metadata_enricher() {
        let enricher = MetadataEnricher::new()
            .with_room("room123")
            .with_user("user456");

        let event = Event::new(EventType::CaseCreated, EventPayload::Empty);
        let result = enricher.transform(event).await.unwrap();

        assert_eq!(result.metadata.room_id.as_ref().unwrap(), "room123");
        assert_eq!(result.metadata.user_id.as_ref().unwrap(), "user456");
    }

    #[tokio::test]
    async fn test_transform_stream() {
        let events = vec![
            Event::new(EventType::UserJoined, EventPayload::Empty),
            Event::new(EventType::UserLeft, EventPayload::Empty),
        ];

        let enricher = MetadataEnricher::new().with_room("test-room");

        let stream = stream::iter(events);
        let transformed: Vec<_> = stream
            .transform_with(enricher)
            .filter_map(|r| async { r.ok() })
            .collect()
            .await;

        assert_eq!(transformed.len(), 2);
        assert_eq!(
            transformed[0].metadata.room_id.as_ref().unwrap(),
            "test-room"
        );
    }

    #[tokio::test]
    async fn test_fn_transformer() {
        let events = vec![
            Event::new(EventType::SimulationUpdate, EventPayload::Empty),
        ];

        let transformer = |mut event: Event| {
            event.metadata.priority = 10;
            event
        };

        let stream = stream::iter(events);
        let transformed: Vec<_> = stream
            .map_events(transformer)
            .filter_map(|r| async { r.ok() })
            .collect()
            .await;

        assert_eq!(transformed[0].metadata.priority, 10);
    }
}
