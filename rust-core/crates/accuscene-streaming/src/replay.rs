//! Event replay functionality for synchronization.

use crate::error::{Result, StreamingError};
use crate::event::Event;
use parking_lot::RwLock;
use std::collections::VecDeque;
use std::sync::Arc;

/// Event replay buffer for storing and replaying events
pub struct ReplayBuffer {
    /// Events buffer
    events: Arc<RwLock<VecDeque<(u64, Event)>>>,
    /// Current sequence number
    sequence: Arc<RwLock<u64>>,
    /// Maximum buffer size
    max_size: usize,
}

impl ReplayBuffer {
    /// Create a new replay buffer with specified capacity
    pub fn new(max_size: usize) -> Self {
        Self {
            events: Arc::new(RwLock::new(VecDeque::with_capacity(max_size))),
            sequence: Arc::new(RwLock::new(0)),
            max_size,
        }
    }

    /// Add an event to the replay buffer
    pub fn add(&self, event: Event) -> u64 {
        let mut seq = self.sequence.write();
        *seq += 1;
        let sequence_number = *seq;

        let mut events = self.events.write();

        // Add event with sequence number
        events.push_back((sequence_number, event));

        // Trim buffer if exceeds max size
        while events.len() > self.max_size {
            events.pop_front();
        }

        sequence_number
    }

    /// Get events from a specific sequence number
    pub fn replay_from(&self, from_sequence: u64) -> Vec<Event> {
        let events = self.events.read();

        events
            .iter()
            .filter(|(seq, _)| *seq >= from_sequence)
            .map(|(_, event)| event.clone())
            .collect()
    }

    /// Get events in a range
    pub fn replay_range(&self, from_sequence: u64, to_sequence: u64) -> Vec<Event> {
        let events = self.events.read();

        events
            .iter()
            .filter(|(seq, _)| *seq >= from_sequence && *seq <= to_sequence)
            .map(|(_, event)| event.clone())
            .collect()
    }

    /// Get the last N events
    pub fn replay_last(&self, count: usize) -> Vec<Event> {
        let events = self.events.read();

        events
            .iter()
            .rev()
            .take(count)
            .map(|(_, event)| event.clone())
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    /// Get all events
    pub fn replay_all(&self) -> Vec<Event> {
        let events = self.events.read();
        events.iter().map(|(_, event)| event.clone()).collect()
    }

    /// Get current sequence number
    pub fn current_sequence(&self) -> u64 {
        *self.sequence.read()
    }

    /// Get buffer size
    pub fn size(&self) -> usize {
        self.events.read().len()
    }

    /// Clear the buffer
    pub fn clear(&self) {
        self.events.write().clear();
        *self.sequence.write() = 0;
    }
}

/// Replay manager for handling replay requests
pub struct ReplayManager {
    /// Replay buffer
    buffer: Arc<ReplayBuffer>,
}

impl ReplayManager {
    /// Create a new replay manager
    pub fn new(buffer_size: usize) -> Self {
        Self {
            buffer: Arc::new(ReplayBuffer::new(buffer_size)),
        }
    }

    /// Get the replay buffer
    pub fn buffer(&self) -> Arc<ReplayBuffer> {
        self.buffer.clone()
    }

    /// Record an event
    pub fn record(&self, event: Event) -> u64 {
        self.buffer.add(event)
    }

    /// Handle a replay request
    pub fn handle_replay(
        &self,
        from_sequence: Option<u64>,
        to_sequence: Option<u64>,
    ) -> Result<Vec<Event>> {
        match (from_sequence, to_sequence) {
            (Some(from), Some(to)) => {
                if from > to {
                    return Err(StreamingError::Replay(
                        "Invalid range: from_sequence > to_sequence".to_string(),
                    ));
                }
                Ok(self.buffer.replay_range(from, to))
            }
            (Some(from), None) => Ok(self.buffer.replay_from(from)),
            (None, Some(_)) => Err(StreamingError::Replay(
                "to_sequence specified without from_sequence".to_string(),
            )),
            (None, None) => Ok(self.buffer.replay_all()),
        }
    }

    /// Get current sequence number
    pub fn current_sequence(&self) -> u64 {
        self.buffer.current_sequence()
    }
}

/// Replay configuration
#[derive(Debug, Clone)]
pub struct ReplayConfig {
    /// Enable replay functionality
    pub enabled: bool,
    /// Maximum buffer size
    pub buffer_size: usize,
    /// Maximum events per replay request
    pub max_events_per_request: usize,
}

impl Default for ReplayConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            buffer_size: 10000,
            max_events_per_request: 1000,
        }
    }
}

impl ReplayConfig {
    /// Create a new replay configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable or disable replay
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set buffer size
    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// Set maximum events per request
    pub fn with_max_events(mut self, max: usize) -> Self {
        self.max_events_per_request = max;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{EventPayload, EventType};

    #[test]
    fn test_replay_buffer() {
        let buffer = ReplayBuffer::new(100);

        let event1 = Event::new(EventType::UserJoined, EventPayload::Empty);
        let event2 = Event::new(EventType::UserLeft, EventPayload::Empty);

        let seq1 = buffer.add(event1);
        let seq2 = buffer.add(event2);

        assert_eq!(seq1, 1);
        assert_eq!(seq2, 2);
        assert_eq!(buffer.current_sequence(), 2);
        assert_eq!(buffer.size(), 2);
    }

    #[test]
    fn test_replay_from() {
        let buffer = ReplayBuffer::new(100);

        buffer.add(Event::new(EventType::CaseCreated, EventPayload::Empty));
        buffer.add(Event::new(EventType::CaseUpdated, EventPayload::Empty));
        buffer.add(Event::new(EventType::CaseDeleted, EventPayload::Empty));

        let events = buffer.replay_from(2);
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn test_replay_range() {
        let buffer = ReplayBuffer::new(100);

        buffer.add(Event::new(EventType::SimulationStarted, EventPayload::Empty));
        buffer.add(Event::new(EventType::SimulationUpdate, EventPayload::Empty));
        buffer.add(Event::new(EventType::SimulationStopped, EventPayload::Empty));

        let events = buffer.replay_range(1, 2);
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn test_replay_last() {
        let buffer = ReplayBuffer::new(100);

        buffer.add(Event::new(EventType::UserJoined, EventPayload::Empty));
        buffer.add(Event::new(EventType::UserLeft, EventPayload::Empty));
        buffer.add(Event::new(EventType::UserActive, EventPayload::Empty));

        let events = buffer.replay_last(2);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event_type, EventType::UserLeft);
        assert_eq!(events[1].event_type, EventType::UserActive);
    }

    #[test]
    fn test_buffer_overflow() {
        let buffer = ReplayBuffer::new(2);

        buffer.add(Event::new(EventType::UserJoined, EventPayload::Empty));
        buffer.add(Event::new(EventType::UserLeft, EventPayload::Empty));
        buffer.add(Event::new(EventType::UserActive, EventPayload::Empty));

        // Buffer should contain only the last 2 events
        assert_eq!(buffer.size(), 2);

        let all_events = buffer.replay_all();
        assert_eq!(all_events.len(), 2);
    }

    #[test]
    fn test_replay_manager() {
        let manager = ReplayManager::new(100);

        manager.record(Event::new(EventType::CaseCreated, EventPayload::Empty));
        manager.record(Event::new(EventType::CaseUpdated, EventPayload::Empty));

        let events = manager.handle_replay(Some(1), None).unwrap();
        assert_eq!(events.len(), 2);

        let events = manager.handle_replay(Some(2), Some(2)).unwrap();
        assert_eq!(events.len(), 1);
    }
}
