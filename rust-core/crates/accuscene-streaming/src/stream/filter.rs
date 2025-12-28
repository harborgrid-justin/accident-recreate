//! Stream filtering utilities.

use crate::event::{Event, EventFilter, EventType};
use futures::{Stream, StreamExt};
use std::pin::Pin;
use std::task::{Context, Poll};

/// Filter predicate function type
pub type FilterPredicate = fn(&Event) -> bool;

/// Filtered event stream
pub struct EventFilterStream<S>
where
    S: Stream<Item = Event>,
{
    inner: S,
    filter: EventFilter,
}

impl<S> EventFilterStream<S>
where
    S: Stream<Item = Event>,
{
    /// Create a new filtered stream
    pub fn new(stream: S, filter: EventFilter) -> Self {
        Self {
            inner: stream,
            filter,
        }
    }
}

impl<S> Stream for EventFilterStream<S>
where
    S: Stream<Item = Event> + Unpin,
{
    type Item = Event;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            match Pin::new(&mut self.inner).poll_next(cx) {
                Poll::Ready(Some(event)) => {
                    if self.filter.matches(&event) {
                        return Poll::Ready(Some(event));
                    }
                    // Event doesn't match, continue polling
                }
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

/// Extension trait for filtering event streams
pub trait FilterStreamExt: Stream<Item = Event> + Sized {
    /// Filter events by type
    fn filter_by_type(self, event_types: Vec<EventType>) -> EventFilterStream<Self> {
        let filter = EventFilter::new().with_types(event_types);
        EventFilterStream::new(self, filter)
    }

    /// Filter events by room
    fn filter_by_room(self, room_id: impl Into<String>) -> EventFilterStream<Self> {
        let filter = EventFilter::new().with_room(room_id);
        EventFilterStream::new(self, filter)
    }

    /// Filter events by user
    fn filter_by_user(self, user_id: impl Into<String>) -> EventFilterStream<Self> {
        let filter = EventFilter::new().with_user(user_id);
        EventFilterStream::new(self, filter)
    }

    /// Filter events with custom predicate
    fn filter_events(self, predicate: FilterPredicate) -> impl Stream<Item = Event> {
        self.filter(move |event| predicate(event))
    }

    /// Filter events with a complete EventFilter
    fn filter_with(self, filter: EventFilter) -> EventFilterStream<Self> {
        EventFilterStream::new(self, filter)
    }
}

impl<S> FilterStreamExt for S where S: Stream<Item = Event> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{EventPayload, EventType};
    use futures::stream;

    #[tokio::test]
    async fn test_filter_by_type() {
        let events = vec![
            Event::new(EventType::UserJoined, EventPayload::Empty),
            Event::new(EventType::SimulationUpdate, EventPayload::Empty),
            Event::new(EventType::UserLeft, EventPayload::Empty),
        ];

        let stream = stream::iter(events);
        let filtered: Vec<_> = stream
            .filter_by_type(vec![EventType::UserJoined, EventType::UserLeft])
            .collect()
            .await;

        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].event_type, EventType::UserJoined);
        assert_eq!(filtered[1].event_type, EventType::UserLeft);
    }

    #[tokio::test]
    async fn test_filter_by_room() {
        let mut event1 = Event::new(EventType::CaseCreated, EventPayload::Empty);
        event1.metadata.room_id = Some("room1".to_string());

        let mut event2 = Event::new(EventType::CaseUpdated, EventPayload::Empty);
        event2.metadata.room_id = Some("room2".to_string());

        let events = vec![event1, event2];
        let stream = stream::iter(events);

        let filtered: Vec<_> = stream.filter_by_room("room1").collect().await;

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].metadata.room_id.as_ref().unwrap(), "room1");
    }

    #[tokio::test]
    async fn test_custom_filter() {
        let events = vec![
            Event::new(EventType::SimulationUpdate, EventPayload::Empty),
            Event::new(EventType::UserJoined, EventPayload::Empty),
        ];

        let stream = stream::iter(events);
        let predicate: FilterPredicate = |event| {
            matches!(event.event_type, EventType::SimulationUpdate)
        };

        let filtered: Vec<_> = stream.filter_events(predicate).collect().await;

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].event_type, EventType::SimulationUpdate);
    }
}
