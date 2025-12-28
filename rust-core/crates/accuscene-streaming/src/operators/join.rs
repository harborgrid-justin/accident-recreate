//! Join operators for combining multiple streams.

use crate::error::Result;
use crate::operators::window::Window;
use crate::stream::DataStream;
use crate::watermark::Timestamp;
use async_trait::async_trait;
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;

/// Join type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinType {
    Inner,
    LeftOuter,
    RightOuter,
    FullOuter,
}

/// Join operator that combines two streams based on a key
pub struct JoinOperator<S1, S2, K, F1, F2>
where
    S1: DataStream,
    S2: DataStream,
    K: Hash + Eq + Clone + Send + 'static,
    F1: Fn(&S1::Item) -> K + Send + 'static,
    F2: Fn(&S2::Item) -> K + Send + 'static,
{
    stream1: S1,
    stream2: S2,
    key_fn1: F1,
    key_fn2: F2,
    join_type: JoinType,
    buffer1: HashMap<K, Vec<S1::Item>>,
    buffer2: HashMap<K, Vec<S2::Item>>,
    output_queue: VecDeque<(S1::Item, Option<S2::Item>)>,
}

impl<S1, S2, K, F1, F2> JoinOperator<S1, S2, K, F1, F2>
where
    S1: DataStream,
    S2: DataStream,
    S1::Item: Clone,
    S2::Item: Clone,
    K: Hash + Eq + Clone + Send + 'static,
    F1: Fn(&S1::Item) -> K + Send + 'static,
    F2: Fn(&S2::Item) -> K + Send + 'static,
{
    /// Create a new join operator
    pub fn new(stream1: S1, stream2: S2, key_fn1: F1, key_fn2: F2, join_type: JoinType) -> Self {
        Self {
            stream1,
            stream2,
            key_fn1,
            key_fn2,
            join_type,
            buffer1: HashMap::new(),
            buffer2: HashMap::new(),
            output_queue: VecDeque::new(),
        }
    }

    async fn process_item1(&mut self, item: S1::Item) {
        let key = (self.key_fn1)(&item);

        // Add to buffer
        self.buffer1.entry(key.clone()).or_insert_with(Vec::new).push(item.clone());

        // Try to join with items from stream2
        if let Some(items2) = self.buffer2.get(&key) {
            for item2 in items2 {
                self.output_queue.push_back((item.clone(), Some(item2.clone())));
            }
        } else if matches!(self.join_type, JoinType::LeftOuter | JoinType::FullOuter) {
            self.output_queue.push_back((item.clone(), None));
        }
    }

    async fn process_item2(&mut self, item: S2::Item) {
        let key = (self.key_fn2)(&item);

        // Add to buffer
        self.buffer2.entry(key.clone()).or_insert_with(Vec::new).push(item.clone());

        // Try to join with items from stream1
        if let Some(items1) = self.buffer1.get(&key) {
            for item1 in items1 {
                self.output_queue.push_back((item1.clone(), Some(item.clone())));
            }
        } else if matches!(self.join_type, JoinType::RightOuter | JoinType::FullOuter) {
            // For right/full outer, we need a default left item
            // This is simplified; real implementation would need proper handling
        }
    }
}

/// Joined item result
#[derive(Debug, Clone)]
pub struct JoinedItem<L, R> {
    pub left: L,
    pub right: Option<R>,
}

impl<L, R> JoinedItem<L, R> {
    pub fn new(left: L, right: Option<R>) -> Self {
        Self { left, right }
    }

    pub fn inner(left: L, right: R) -> Self {
        Self {
            left,
            right: Some(right),
        }
    }
}

#[async_trait]
impl<S1, S2, K, F1, F2> DataStream for JoinOperator<S1, S2, K, F1, F2>
where
    S1: DataStream,
    S2: DataStream,
    S1::Item: Clone + Send + 'static,
    S2::Item: Clone + Send + 'static,
    K: Hash + Eq + Clone + Send + 'static,
    F1: Fn(&S1::Item) -> K + Send + 'static,
    F2: Fn(&S2::Item) -> K + Send + 'static,
{
    type Item = JoinedItem<S1::Item, S2::Item>;

    async fn next(&mut self) -> Result<Option<Self::Item>> {
        loop {
            // First, check if we have any queued output
            if let Some((left, right)) = self.output_queue.pop_front() {
                return Ok(Some(JoinedItem::new(left, right)));
            }

            // Try to get items from both streams
            // Simplified: in reality, this should be more sophisticated
            match (self.stream1.next().await?, self.stream2.next().await?) {
                (Some(item1), Some(item2)) => {
                    self.process_item1(item1).await;
                    self.process_item2(item2).await;
                }
                (Some(item1), None) => {
                    self.process_item1(item1).await;
                }
                (None, Some(item2)) => {
                    self.process_item2(item2).await;
                }
                (None, None) => {
                    return Ok(None);
                }
            }
        }
    }

    fn is_complete(&self) -> bool {
        self.stream1.is_complete() && self.stream2.is_complete() && self.output_queue.is_empty()
    }
}

/// Window join operator for time-based joins
pub struct WindowJoinOperator<S1, S2>
where
    S1: DataStream,
    S2: DataStream,
{
    stream1: S1,
    stream2: S2,
    window: Window,
}

impl<S1, S2> WindowJoinOperator<S1, S2>
where
    S1: DataStream,
    S2: DataStream,
{
    /// Create a new window join operator
    pub fn new(stream1: S1, stream2: S2, window: Window) -> Self {
        Self {
            stream1,
            stream2,
            window,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::iterator::IteratorSource;
    use crate::source::Source;

    #[derive(Debug, Clone, PartialEq)]
    struct Event {
        id: i32,
        value: String,
    }

    #[tokio::test]
    async fn test_join_operator() {
        let events1 = vec![
            Event {
                id: 1,
                value: "a".to_string(),
            },
            Event {
                id: 2,
                value: "b".to_string(),
            },
        ];

        let events2 = vec![
            Event {
                id: 1,
                value: "x".to_string(),
            },
            Event {
                id: 3,
                value: "y".to_string(),
            },
        ];

        let mut source1 = IteratorSource::new(events1.into_iter());
        let mut source2 = IteratorSource::new(events2.into_iter());

        source1.start().await.unwrap();
        source2.start().await.unwrap();

        let mut joined = JoinOperator::new(
            source1,
            source2,
            |e: &Event| e.id,
            |e: &Event| e.id,
            JoinType::Inner,
        );

        // Should find one match (id=1)
        let mut results = Vec::new();
        while let Some(item) = joined.next().await.unwrap() {
            results.push(item);
            if results.len() > 10 {
                break; // Safety limit
            }
        }

        // Verify we got at least one join result
        assert!(!results.is_empty());
    }
}
