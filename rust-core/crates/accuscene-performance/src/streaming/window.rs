//! Time and count-based windowing for stream processing

use bytes::Bytes;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Trait for windowing operations
pub trait Window: Send + Sync {
    /// Add an item to the window
    fn add(&mut self, item: Bytes, timestamp: Instant);

    /// Get items in the current window
    fn get_window(&self) -> Vec<Bytes>;

    /// Advance the window (remove expired items)
    fn advance(&mut self);

    /// Check if window is full
    fn is_full(&self) -> bool;

    /// Get window size
    fn size(&self) -> usize;

    /// Clear the window
    fn clear(&mut self);
}

/// Time-based window
pub struct TimeWindow {
    duration: Duration,
    items: VecDeque<(Bytes, Instant)>,
    max_items: Option<usize>,
}

impl TimeWindow {
    /// Create a new time window
    pub fn new(duration: Duration) -> Self {
        Self {
            duration,
            items: VecDeque::new(),
            max_items: None,
        }
    }

    /// Create with maximum item limit
    pub fn with_max_items(duration: Duration, max_items: usize) -> Self {
        Self {
            duration,
            items: VecDeque::with_capacity(max_items),
            max_items: Some(max_items),
        }
    }

    /// Remove expired items
    fn remove_expired(&mut self) {
        let now = Instant::now();
        while let Some((_, timestamp)) = self.items.front() {
            if now.duration_since(*timestamp) > self.duration {
                self.items.pop_front();
            } else {
                break;
            }
        }
    }
}

impl Window for TimeWindow {
    fn add(&mut self, item: Bytes, timestamp: Instant) {
        self.remove_expired();

        if let Some(max) = self.max_items {
            if self.items.len() >= max {
                self.items.pop_front();
            }
        }

        self.items.push_back((item, timestamp));
    }

    fn get_window(&self) -> Vec<Bytes> {
        self.items.iter().map(|(item, _)| item.clone()).collect()
    }

    fn advance(&mut self) {
        self.remove_expired();
    }

    fn is_full(&self) -> bool {
        if let Some(max) = self.max_items {
            self.items.len() >= max
        } else {
            false
        }
    }

    fn size(&self) -> usize {
        self.items.len()
    }

    fn clear(&mut self) {
        self.items.clear();
    }
}

/// Count-based window
pub struct CountWindow {
    count: usize,
    items: VecDeque<Bytes>,
}

impl CountWindow {
    /// Create a new count window
    pub fn new(count: usize) -> Self {
        Self {
            count,
            items: VecDeque::with_capacity(count),
        }
    }

    /// Get the window count
    pub fn count(&self) -> usize {
        self.count
    }
}

impl Window for CountWindow {
    fn add(&mut self, item: Bytes, _timestamp: Instant) {
        if self.items.len() >= self.count {
            self.items.pop_front();
        }
        self.items.push_back(item);
    }

    fn get_window(&self) -> Vec<Bytes> {
        self.items.iter().cloned().collect()
    }

    fn advance(&mut self) {
        // Count window doesn't need advancing based on time
    }

    fn is_full(&self) -> bool {
        self.items.len() >= self.count
    }

    fn size(&self) -> usize {
        self.items.len()
    }

    fn clear(&mut self) {
        self.items.clear();
    }
}

/// Sliding window that emits windows at regular intervals
pub struct SlidingWindow {
    window_size: usize,
    slide_size: usize,
    items: VecDeque<Bytes>,
    items_since_slide: usize,
}

impl SlidingWindow {
    /// Create a new sliding window
    pub fn new(window_size: usize, slide_size: usize) -> Self {
        assert!(slide_size > 0, "Slide size must be greater than 0");
        assert!(
            slide_size <= window_size,
            "Slide size must be less than or equal to window size"
        );

        Self {
            window_size,
            slide_size,
            items: VecDeque::with_capacity(window_size),
            items_since_slide: 0,
        }
    }

    /// Check if should emit window
    pub fn should_emit(&self) -> bool {
        self.items_since_slide >= self.slide_size && self.items.len() >= self.window_size
    }

    /// Emit window and slide
    pub fn emit_and_slide(&mut self) -> Option<Vec<Bytes>> {
        if !self.should_emit() {
            return None;
        }

        let window = self.get_window();

        // Slide the window
        for _ in 0..self.slide_size.min(self.items.len()) {
            self.items.pop_front();
        }
        self.items_since_slide = 0;

        Some(window)
    }
}

impl Window for SlidingWindow {
    fn add(&mut self, item: Bytes, _timestamp: Instant) {
        if self.items.len() >= self.window_size {
            self.items.pop_front();
        }
        self.items.push_back(item);
        self.items_since_slide += 1;
    }

    fn get_window(&self) -> Vec<Bytes> {
        self.items.iter().cloned().collect()
    }

    fn advance(&mut self) {
        if self.should_emit() {
            self.emit_and_slide();
        }
    }

    fn is_full(&self) -> bool {
        self.items.len() >= self.window_size
    }

    fn size(&self) -> usize {
        self.items.len()
    }

    fn clear(&mut self) {
        self.items.clear();
        self.items_since_slide = 0;
    }
}

/// Session window based on inactivity gaps
pub struct SessionWindow {
    gap: Duration,
    items: VecDeque<(Bytes, Instant)>,
    last_activity: Option<Instant>,
}

impl SessionWindow {
    /// Create a new session window
    pub fn new(gap: Duration) -> Self {
        Self {
            gap,
            items: VecDeque::new(),
            last_activity: None,
        }
    }

    /// Check if session has expired
    pub fn is_expired(&self) -> bool {
        if let Some(last) = self.last_activity {
            Instant::now().duration_since(last) > self.gap
        } else {
            false
        }
    }

    /// Get and clear the session if expired
    pub fn get_and_clear_if_expired(&mut self) -> Option<Vec<Bytes>> {
        if self.is_expired() && !self.items.is_empty() {
            let window = self.get_window();
            self.clear();
            Some(window)
        } else {
            None
        }
    }
}

impl Window for SessionWindow {
    fn add(&mut self, item: Bytes, timestamp: Instant) {
        // If session expired, clear previous session
        if self.is_expired() {
            self.items.clear();
        }

        self.items.push_back((item, timestamp));
        self.last_activity = Some(timestamp);
    }

    fn get_window(&self) -> Vec<Bytes> {
        self.items.iter().map(|(item, _)| item.clone()).collect()
    }

    fn advance(&mut self) {
        // Check and clear expired session
        if self.is_expired() {
            self.items.clear();
        }
    }

    fn is_full(&self) -> bool {
        false // Session windows don't have a fixed size
    }

    fn size(&self) -> usize {
        self.items.len()
    }

    fn clear(&mut self) {
        self.items.clear();
        self.last_activity = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_count_window() {
        let mut window = CountWindow::new(3);

        window.add(Bytes::from("a"), Instant::now());
        window.add(Bytes::from("b"), Instant::now());
        assert_eq!(window.size(), 2);
        assert!(!window.is_full());

        window.add(Bytes::from("c"), Instant::now());
        assert_eq!(window.size(), 3);
        assert!(window.is_full());

        window.add(Bytes::from("d"), Instant::now());
        assert_eq!(window.size(), 3); // Should still be 3 (oldest removed)
    }

    #[test]
    fn test_time_window() {
        let mut window = TimeWindow::new(Duration::from_millis(100));

        window.add(Bytes::from("a"), Instant::now());
        window.add(Bytes::from("b"), Instant::now());
        assert_eq!(window.size(), 2);

        thread::sleep(Duration::from_millis(150));
        window.advance();
        assert_eq!(window.size(), 0); // All items expired
    }

    #[test]
    fn test_sliding_window() {
        let mut window = SlidingWindow::new(4, 2);

        for i in 0..4 {
            window.add(Bytes::from(vec![i]), Instant::now());
        }

        assert!(window.should_emit());
        let emitted = window.emit_and_slide().unwrap();
        assert_eq!(emitted.len(), 4);
        assert_eq!(window.size(), 2); // 2 items remain after sliding
    }

    #[test]
    fn test_session_window() {
        let mut window = SessionWindow::new(Duration::from_millis(50));

        window.add(Bytes::from("a"), Instant::now());
        window.add(Bytes::from("b"), Instant::now());
        assert_eq!(window.size(), 2);

        thread::sleep(Duration::from_millis(100));
        assert!(window.is_expired());

        window.add(Bytes::from("c"), Instant::now());
        assert_eq!(window.size(), 1); // New session started
    }

    #[test]
    fn test_window_clear() {
        let mut window = CountWindow::new(5);
        window.add(Bytes::from("test"), Instant::now());
        assert_eq!(window.size(), 1);

        window.clear();
        assert_eq!(window.size(), 0);
    }
}
