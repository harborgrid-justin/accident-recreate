//! Windowing strategies for streaming data analysis

use chrono::{DateTime, Duration, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;

/// Sliding window that moves continuously
pub struct SlidingWindow<T> {
    window_size: usize,
    data: Arc<RwLock<VecDeque<T>>>,
}

impl<T: Clone> SlidingWindow<T> {
    pub fn new(window_size: usize) -> Self {
        Self {
            window_size,
            data: Arc::new(RwLock::new(VecDeque::with_capacity(window_size))),
        }
    }

    /// Add an element to the window
    pub fn add(&self, item: T) {
        let mut data = self.data.write();
        data.push_back(item);

        if data.len() > self.window_size {
            data.pop_front();
        }
    }

    /// Get all elements in the window
    pub fn elements(&self) -> Vec<T> {
        self.data.read().iter().cloned().collect()
    }

    /// Get the number of elements in the window
    pub fn len(&self) -> usize {
        self.data.read().len()
    }

    /// Check if the window is empty
    pub fn is_empty(&self) -> bool {
        self.data.read().is_empty()
    }

    /// Check if the window is full
    pub fn is_full(&self) -> bool {
        self.data.read().len() == self.window_size
    }

    /// Clear the window
    pub fn clear(&self) {
        self.data.write().clear();
    }
}

/// Tumbling window that non-overlapping fixed-size windows
pub struct TumblingWindow<T> {
    window_size: usize,
    current_window: Arc<RwLock<Vec<T>>>,
    completed_windows: Arc<RwLock<Vec<Vec<T>>>>,
}

impl<T: Clone> TumblingWindow<T> {
    pub fn new(window_size: usize) -> Self {
        Self {
            window_size,
            current_window: Arc::new(RwLock::new(Vec::with_capacity(window_size))),
            completed_windows: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Add an element to the current window
    pub fn add(&self, item: T) -> Option<Vec<T>> {
        let mut current = self.current_window.write();
        current.push(item);

        if current.len() >= self.window_size {
            let completed = current.clone();
            current.clear();

            self.completed_windows.write().push(completed.clone());

            Some(completed)
        } else {
            None
        }
    }

    /// Get the current (incomplete) window
    pub fn current(&self) -> Vec<T> {
        self.current_window.read().clone()
    }

    /// Get all completed windows
    pub fn completed(&self) -> Vec<Vec<T>> {
        self.completed_windows.read().clone()
    }

    /// Clear all windows
    pub fn clear(&self) {
        self.current_window.write().clear();
        self.completed_windows.write().clear();
    }
}

/// Session window that groups events based on inactivity gaps
pub struct SessionWindow<T> {
    gap_duration: Duration,
    sessions: Arc<RwLock<Vec<Session<T>>>>,
    current_session: Arc<RwLock<Option<Session<T>>>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Session<T> {
    pub items: Vec<(DateTime<Utc>, T)>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
}

impl<T: Clone> SessionWindow<T> {
    pub fn new(gap_duration: Duration) -> Self {
        Self {
            gap_duration,
            sessions: Arc::new(RwLock::new(Vec::new())),
            current_session: Arc::new(RwLock::new(None)),
        }
    }

    /// Add an element with a timestamp
    pub fn add(&self, timestamp: DateTime<Utc>, item: T) -> Option<Session<T>> {
        let mut current = self.current_session.write();

        let mut completed_session = None;

        match &mut *current {
            Some(session) => {
                // Check if this event is within the gap duration
                if timestamp - session.end_time > self.gap_duration {
                    // Close current session and start new one
                    completed_session = Some(session.clone());
                    self.sessions.write().push(session.clone());

                    *session = Session {
                        items: vec![(timestamp, item)],
                        start_time: timestamp,
                        end_time: timestamp,
                    };
                } else {
                    // Add to current session
                    session.items.push((timestamp, item));
                    session.end_time = timestamp;
                }
            }
            None => {
                // Start new session
                *current = Some(Session {
                    items: vec![(timestamp, item)],
                    start_time: timestamp,
                    end_time: timestamp,
                });
            }
        }

        completed_session
    }

    /// Force close the current session
    pub fn close_current(&self) -> Option<Session<T>> {
        let mut current = self.current_session.write();

        if let Some(session) = current.take() {
            self.sessions.write().push(session.clone());
            Some(session)
        } else {
            None
        }
    }

    /// Get all completed sessions
    pub fn sessions(&self) -> Vec<Session<T>> {
        self.sessions.read().clone()
    }

    /// Get the current (incomplete) session
    pub fn current(&self) -> Option<Session<T>> {
        self.current_session.read().clone()
    }

    /// Clear all sessions
    pub fn clear(&self) {
        self.current_session.write().take();
        self.sessions.write().clear();
    }
}

/// Hopping window (sliding window with fixed slide interval)
pub struct HoppingWindow<T> {
    window_size: usize,
    hop_size: usize,
    data: Arc<RwLock<VecDeque<T>>>,
    windows: Arc<RwLock<Vec<Vec<T>>>>,
    count: Arc<RwLock<usize>>,
}

impl<T: Clone> HoppingWindow<T> {
    pub fn new(window_size: usize, hop_size: usize) -> Self {
        Self {
            window_size,
            hop_size,
            data: Arc::new(RwLock::new(VecDeque::new())),
            windows: Arc::new(RwLock::new(Vec::new())),
            count: Arc::new(RwLock::new(0)),
        }
    }

    /// Add an element
    pub fn add(&self, item: T) -> Option<Vec<T>> {
        let mut data = self.data.write();
        data.push_back(item);

        // Maintain only what we need
        if data.len() > self.window_size {
            data.pop_front();
        }

        drop(data);

        let mut count = self.count.write();
        *count += 1;

        // Emit window every hop_size elements
        if *count >= self.hop_size && *count % self.hop_size == 0 {
            let data = self.data.read();
            if data.len() >= self.window_size {
                let window: Vec<T> = data.iter().take(self.window_size).cloned().collect();
                self.windows.write().push(window.clone());
                Some(window)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Get all emitted windows
    pub fn windows(&self) -> Vec<Vec<T>> {
        self.windows.read().clone()
    }

    /// Clear the window
    pub fn clear(&self) {
        self.data.write().clear();
        self.windows.write().clear();
        *self.count.write() = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sliding_window() {
        let window = SlidingWindow::new(3);

        window.add(1);
        window.add(2);
        window.add(3);

        assert_eq!(window.len(), 3);
        assert!(window.is_full());

        window.add(4);
        assert_eq!(window.len(), 3);
        assert_eq!(window.elements(), vec![2, 3, 4]);
    }

    #[test]
    fn test_tumbling_window() {
        let window = TumblingWindow::new(3);

        assert!(window.add(1).is_none());
        assert!(window.add(2).is_none());

        let completed = window.add(3);
        assert!(completed.is_some());
        assert_eq!(completed.unwrap(), vec![1, 2, 3]);
    }

    #[test]
    fn test_session_window() {
        let window = SessionWindow::new(Duration::seconds(5));

        let now = Utc::now();
        window.add(now, 1);
        window.add(now + Duration::seconds(2), 2);

        let current = window.current().unwrap();
        assert_eq!(current.items.len(), 2);
    }
}
