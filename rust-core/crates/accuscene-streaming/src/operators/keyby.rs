//! KeyBy operator for partitioning streams by key.

use crate::error::Result;
use crate::partition::{KeyPartitioner, Partitioner};
use crate::stream::DataStream;
use async_trait::async_trait;
use std::collections::HashMap;
use std::hash::Hash;

/// Trait for extracting keys from items
pub trait KeyExtractor<T, K>: Send + Sync {
    fn extract_key(&self, item: &T) -> K;
}

impl<T, K, F> KeyExtractor<T, K> for F
where
    F: Fn(&T) -> K + Send + Sync,
{
    fn extract_key(&self, item: &T) -> K {
        self(item)
    }
}

/// KeyBy operator that partitions stream by key
pub struct KeyByOperator<S, K, E>
where
    S: DataStream,
    K: Hash + Eq + Clone + Send + 'static,
    E: KeyExtractor<S::Item, K> + 'static,
{
    stream: S,
    key_extractor: E,
    num_partitions: usize,
    _phantom: std::marker::PhantomData<K>,
}

impl<S, K, E> KeyByOperator<S, K, E>
where
    S: DataStream,
    K: Hash + Eq + Clone + Send + 'static,
    E: KeyExtractor<S::Item, K> + 'static,
{
    /// Create a new keyby operator
    pub fn new(stream: S, key_extractor: E, num_partitions: usize) -> Self {
        Self {
            stream,
            key_extractor,
            num_partitions,
            _phantom: std::marker::PhantomData,
        }
    }
}

/// Keyed stream item
#[derive(Debug, Clone)]
pub struct KeyedItem<K, V> {
    pub key: K,
    pub value: V,
}

impl<K, V> KeyedItem<K, V> {
    pub fn new(key: K, value: V) -> Self {
        Self { key, value }
    }
}

#[async_trait]
impl<S, K, E> DataStream for KeyByOperator<S, K, E>
where
    S: DataStream,
    S::Item: Clone,
    K: Hash + Eq + Clone + Send + 'static,
    E: KeyExtractor<S::Item, K> + 'static,
{
    type Item = KeyedItem<K, S::Item>;

    async fn next(&mut self) -> Result<Option<Self::Item>> {
        match self.stream.next().await? {
            Some(item) => {
                let key = self.key_extractor.extract_key(&item);
                Ok(Some(KeyedItem::new(key, item)))
            }
            None => Ok(None),
        }
    }

    fn is_complete(&self) -> bool {
        self.stream.is_complete()
    }
}

/// Keyed stream that maintains separate state per key
pub struct KeyedStream<S, K>
where
    S: DataStream,
    K: Hash + Eq + Clone + Send + 'static,
{
    stream: S,
    buffers: HashMap<K, Vec<S::Item>>,
}

impl<S, K> KeyedStream<S, K>
where
    S: DataStream,
    K: Hash + Eq + Clone + Send + 'static,
{
    /// Create a new keyed stream
    pub fn new(stream: S) -> Self {
        Self {
            stream,
            buffers: HashMap::new(),
        }
    }
}

#[async_trait]
impl<S, K> DataStream for KeyedStream<S, K>
where
    S: DataStream<Item = KeyedItem<K, <S as DataStream>::Item>>,
    K: Hash + Eq + Clone + Send + 'static,
    <S as DataStream>::Item: Send + 'static,
{
    type Item = (K, Vec<<S as DataStream>::Item>);

    async fn next(&mut self) -> Result<Option<Self::Item>> {
        match self.stream.next().await? {
            Some(keyed_item) => {
                self.buffers
                    .entry(keyed_item.key.clone())
                    .or_insert_with(Vec::new)
                    .push(keyed_item.value);

                // For simplicity, emit when we have items
                // In a real implementation, this would be based on triggers
                let key = keyed_item.key;
                let values = self.buffers.remove(&key).unwrap();
                Ok(Some((key, values)))
            }
            None => Ok(None),
        }
    }

    fn is_complete(&self) -> bool {
        self.stream.is_complete()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::iterator::IteratorSource;
    use crate::source::Source;
    use crate::stream::StreamExt;

    #[derive(Debug, Clone, PartialEq)]
    struct Event {
        key: String,
        value: i32,
    }

    #[tokio::test]
    async fn test_keyby_operator() {
        let events = vec![
            Event {
                key: "a".to_string(),
                value: 1,
            },
            Event {
                key: "b".to_string(),
                value: 2,
            },
            Event {
                key: "a".to_string(),
                value: 3,
            },
        ];

        let mut source = IteratorSource::new(events.into_iter());
        source.start().await.unwrap();

        let mut keyed = KeyByOperator::new(source, |e: &Event| e.key.clone(), 2);

        let mut results = Vec::new();
        while let Some(item) = keyed.next().await.unwrap() {
            results.push((item.key, item.value.value));
        }

        assert_eq!(results.len(), 3);
    }
}
