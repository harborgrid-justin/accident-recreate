//! Stream aggregation utilities.

use crate::error::Result;
use crate::event::Event;
use async_trait::async_trait;
use std::collections::VecDeque;
use std::time::Duration;
use tokio::time::Instant;

/// Aggregator trait for combining multiple events
#[async_trait]
pub trait Aggregator: Send + Sync {
    /// Aggregate a batch of events
    async fn aggregate(&self, events: Vec<Event>) -> Result<Event>;
}

/// Window-based aggregator
pub struct WindowAggregator {
    /// Window size (number of events)
    size: usize,
    /// Time-based window duration
    duration: Option<Duration>,
    /// Buffered events
    buffer: VecDeque<(Event, Instant)>,
    /// Last flush time
    last_flush: Instant,
}

impl WindowAggregator {
    /// Create a new count-based window aggregator
    pub fn count(size: usize) -> Self {
        Self {
            size,
            duration: None,
            buffer: VecDeque::new(),
            last_flush: Instant::now(),
        }
    }

    /// Create a new time-based window aggregator
    pub fn time(duration: Duration) -> Self {
        Self {
            size: usize::MAX,
            duration: Some(duration),
            buffer: VecDeque::new(),
            last_flush: Instant::now(),
        }
    }

    /// Create a window aggregator with both count and time limits
    pub fn count_and_time(size: usize, duration: Duration) -> Self {
        Self {
            size,
            duration: Some(duration),
            buffer: VecDeque::new(),
            last_flush: Instant::now(),
        }
    }

    /// Add an event to the window
    pub fn add(&mut self, event: Event) {
        self.buffer.push_back((event, Instant::now()));
    }

    /// Check if the window should be flushed
    pub fn should_flush(&self) -> bool {
        // Check count-based flush
        if self.buffer.len() >= self.size {
            return true;
        }

        // Check time-based flush
        if let Some(duration) = self.duration {
            if self.last_flush.elapsed() >= duration && !self.buffer.is_empty() {
                return true;
            }
        }

        false
    }

    /// Flush the window and return buffered events
    pub fn flush(&mut self) -> Vec<Event> {
        let events = self
            .buffer
            .drain(..)
            .map(|(event, _)| event)
            .collect();

        self.last_flush = Instant::now();
        events
    }

    /// Get current buffer size
    pub fn buffer_size(&self) -> usize {
        self.buffer.len()
    }
}

/// Count aggregator - counts events by type
pub struct CountAggregator;

impl CountAggregator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CountAggregator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Aggregator for CountAggregator {
    async fn aggregate(&self, events: Vec<Event>) -> Result<Event> {
        use crate::event::{EventPayload, EventType};
        use std::collections::HashMap;

        let mut counts: HashMap<String, usize> = HashMap::new();

        for event in &events {
            let key = format!("{:?}", event.event_type);
            *counts.entry(key).or_insert(0) += 1;
        }

        let last_event = events.last().cloned().unwrap_or_else(|| {
            Event::new(EventType::Custom("empty".to_string()), EventPayload::Empty)
        });

        let mut aggregated = last_event;
        aggregated.event_type = EventType::Custom("aggregated".to_string());
        aggregated.payload = EventPayload::Json(serde_json::to_value(counts).unwrap());

        Ok(aggregated)
    }
}

/// Batch aggregator - combines events into a batch
pub struct BatchAggregator;

impl BatchAggregator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for BatchAggregator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Aggregator for BatchAggregator {
    async fn aggregate(&self, events: Vec<Event>) -> Result<Event> {
        use crate::event::{EventPayload, EventType};

        let event_data: Vec<_> = events
            .iter()
            .map(|e| serde_json::to_value(e).unwrap())
            .collect();

        let batch_event = Event::new(
            EventType::Custom("batch".to_string()),
            EventPayload::Json(serde_json::json!({
                "events": event_data,
                "count": events.len(),
            })),
        );

        Ok(batch_event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{EventPayload, EventType};

    #[test]
    fn test_window_aggregator_count() {
        let mut aggregator = WindowAggregator::count(3);

        assert!(!aggregator.should_flush());

        aggregator.add(Event::new(EventType::UserJoined, EventPayload::Empty));
        aggregator.add(Event::new(EventType::UserLeft, EventPayload::Empty));
        assert!(!aggregator.should_flush());

        aggregator.add(Event::new(EventType::CaseCreated, EventPayload::Empty));
        assert!(aggregator.should_flush());

        let events = aggregator.flush();
        assert_eq!(events.len(), 3);
        assert!(!aggregator.should_flush());
    }

    #[tokio::test]
    async fn test_window_aggregator_time() {
        let mut aggregator = WindowAggregator::time(Duration::from_millis(100));

        aggregator.add(Event::new(EventType::SimulationUpdate, EventPayload::Empty));

        // Should not flush immediately
        assert!(!aggregator.should_flush());

        // Wait for window duration
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Should flush now
        assert!(aggregator.should_flush());

        let events = aggregator.flush();
        assert_eq!(events.len(), 1);
    }

    #[tokio::test]
    async fn test_count_aggregator() {
        let aggregator = CountAggregator::new();

        let events = vec![
            Event::new(EventType::UserJoined, EventPayload::Empty),
            Event::new(EventType::UserJoined, EventPayload::Empty),
            Event::new(EventType::UserLeft, EventPayload::Empty),
        ];

        let result = aggregator.aggregate(events).await.unwrap();

        assert_eq!(result.event_type, EventType::Custom("aggregated".to_string()));
    }

    #[tokio::test]
    async fn test_batch_aggregator() {
        let aggregator = BatchAggregator::new();

        let events = vec![
            Event::new(EventType::CaseCreated, EventPayload::Empty),
            Event::new(EventType::CaseUpdated, EventPayload::Empty),
        ];

        let result = aggregator.aggregate(events).await.unwrap();

        match result.payload {
            EventPayload::Json(value) => {
                assert_eq!(value["count"], 2);
            }
            _ => panic!("Expected JSON payload"),
        }
    }
}
