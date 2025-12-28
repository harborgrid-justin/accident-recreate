//! Window operators for time-based and count-based windowing.

use crate::error::Result;
use crate::stream::DataStream;
use crate::watermark::{Timestamp, Watermark};
use async_trait::async_trait;
use std::collections::VecDeque;
use std::time::Duration;

/// Window type
#[derive(Debug, Clone)]
pub enum WindowType {
    /// Tumbling window (non-overlapping)
    Tumbling { size: Duration },
    /// Sliding window (overlapping)
    Sliding { size: Duration, slide: Duration },
    /// Session window (activity-based)
    Session { gap: Duration },
    /// Count-based window
    Count { size: usize },
}

/// Trait for window assignment
pub trait WindowAssigner: Send + Sync {
    /// Assign an event to windows
    fn assign_windows(&self, timestamp: Timestamp) -> Vec<Window>;
}

/// Window definition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Window {
    pub start: Timestamp,
    pub end: Timestamp,
}

impl Window {
    pub fn new(start: Timestamp, end: Timestamp) -> Self {
        Self { start, end }
    }

    pub fn contains(&self, timestamp: Timestamp) -> bool {
        timestamp >= self.start && timestamp < self.end
    }

    pub fn duration(&self) -> Duration {
        Duration::from_millis((self.end.as_millis() - self.start.as_millis()) as u64)
    }
}

/// Tumbling window assigner
pub struct TumblingWindowAssigner {
    size: Duration,
}

impl TumblingWindowAssigner {
    pub fn new(size: Duration) -> Self {
        Self { size }
    }
}

impl WindowAssigner for TumblingWindowAssigner {
    fn assign_windows(&self, timestamp: Timestamp) -> Vec<Window> {
        let size_millis = self.size.as_millis() as i64;
        let start = (timestamp.as_millis() / size_millis) * size_millis;
        let end = start + size_millis;

        vec![Window::new(
            Timestamp::from_millis(start),
            Timestamp::from_millis(end),
        )]
    }
}

/// Sliding window assigner
pub struct SlidingWindowAssigner {
    size: Duration,
    slide: Duration,
}

impl SlidingWindowAssigner {
    pub fn new(size: Duration, slide: Duration) -> Self {
        Self { size, slide }
    }
}

impl WindowAssigner for SlidingWindowAssigner {
    fn assign_windows(&self, timestamp: Timestamp) -> Vec<Window> {
        let size_millis = self.size.as_millis() as i64;
        let slide_millis = self.slide.as_millis() as i64;

        let mut windows = Vec::new();
        let last_start = (timestamp.as_millis() / slide_millis) * slide_millis;

        // Generate all windows that contain this timestamp
        let mut start = last_start;
        while start > timestamp.as_millis() - size_millis {
            let end = start + size_millis;
            windows.push(Window::new(
                Timestamp::from_millis(start),
                Timestamp::from_millis(end),
            ));
            start -= slide_millis;
        }

        windows
    }
}

/// Window operator
pub struct WindowOperator<S, A>
where
    S: DataStream,
    A: WindowAssigner + 'static,
{
    stream: S,
    assigner: A,
    buffer: VecDeque<(S::Item, Timestamp, Vec<Window>)>,
    current_watermark: Watermark,
}

impl<S, A> WindowOperator<S, A>
where
    S: DataStream,
    A: WindowAssigner + 'static,
{
    /// Create a new window operator
    pub fn new(stream: S, assigner: A) -> Self {
        Self {
            stream,
            assigner,
            buffer: VecDeque::new(),
            current_watermark: Watermark::min(),
        }
    }
}

/// Windowed item
#[derive(Debug, Clone)]
pub struct WindowedItem<T> {
    pub window: Window,
    pub items: Vec<T>,
}

impl<T> WindowedItem<T> {
    pub fn new(window: Window, items: Vec<T>) -> Self {
        Self { window, items }
    }
}

#[async_trait]
impl<S, A> DataStream for WindowOperator<S, A>
where
    S: DataStream,
    S::Item: Clone + Send + 'static,
    A: WindowAssigner + 'static,
{
    type Item = WindowedItem<S::Item>;

    async fn next(&mut self) -> Result<Option<Self::Item>> {
        // For simplicity, using processing time
        // In a real implementation, this would use event time
        match self.stream.next().await? {
            Some(item) => {
                let timestamp = Timestamp::now();
                let windows = self.assigner.assign_windows(timestamp);

                self.buffer.push_back((item.clone(), timestamp, windows.clone()));

                // Emit first complete window
                if let Some(window) = windows.first() {
                    // Collect all items in this window
                    let items: Vec<_> = self
                        .buffer
                        .iter()
                        .filter(|(_, _, wins)| wins.contains(window))
                        .map(|(item, _, _)| item.clone())
                        .collect();

                    if !items.is_empty() {
                        return Ok(Some(WindowedItem::new(*window, items)));
                    }
                }

                // Continue to next item
                self.next().await
            }
            None => Ok(None),
        }
    }

    fn is_complete(&self) -> bool {
        self.stream.is_complete()
    }
}

/// Count-based window operator
pub struct CountWindowOperator<S>
where
    S: DataStream,
{
    stream: S,
    window_size: usize,
    buffer: Vec<S::Item>,
}

impl<S> CountWindowOperator<S>
where
    S: DataStream,
{
    /// Create a new count window operator
    pub fn new(stream: S, window_size: usize) -> Self {
        Self {
            stream,
            window_size,
            buffer: Vec::new(),
        }
    }
}

#[async_trait]
impl<S> DataStream for CountWindowOperator<S>
where
    S: DataStream,
    S::Item: Clone + Send + 'static,
{
    type Item = Vec<S::Item>;

    async fn next(&mut self) -> Result<Option<Self::Item>> {
        loop {
            match self.stream.next().await? {
                Some(item) => {
                    self.buffer.push(item);

                    if self.buffer.len() >= self.window_size {
                        let window: Vec<_> = self.buffer.drain(..).collect();
                        return Ok(Some(window));
                    }
                }
                None => {
                    if !self.buffer.is_empty() {
                        let window = self.buffer.drain(..).collect();
                        return Ok(Some(window));
                    }
                    return Ok(None);
                }
            }
        }
    }

    fn is_complete(&self) -> bool {
        self.stream.is_complete() && self.buffer.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::iterator::RangeSource;
    use crate::source::Source;

    #[test]
    fn test_tumbling_window_assigner() {
        let assigner = TumblingWindowAssigner::new(Duration::from_secs(10));

        let windows = assigner.assign_windows(Timestamp::from_millis(5000));
        assert_eq!(windows.len(), 1);
        assert_eq!(windows[0].start.as_millis(), 0);
        assert_eq!(windows[0].end.as_millis(), 10000);

        let windows = assigner.assign_windows(Timestamp::from_millis(15000));
        assert_eq!(windows.len(), 1);
        assert_eq!(windows[0].start.as_millis(), 10000);
        assert_eq!(windows[0].end.as_millis(), 20000);
    }

    #[tokio::test]
    async fn test_count_window_operator() {
        let mut source = RangeSource::new(0, 10);
        source.start().await.unwrap();

        let mut windowed = CountWindowOperator::new(source, 3);

        let window1 = windowed.next().await.unwrap().unwrap();
        assert_eq!(window1, vec![0, 1, 2]);

        let window2 = windowed.next().await.unwrap().unwrap();
        assert_eq!(window2, vec![3, 4, 5]);
    }
}
