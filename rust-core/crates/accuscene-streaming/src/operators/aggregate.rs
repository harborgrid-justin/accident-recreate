//! Aggregation operators for stream processing.

use crate::error::Result;
use crate::stream::DataStream;
use async_trait::async_trait;
use std::collections::HashMap;
use std::hash::Hash;

/// Trait for aggregating values
pub trait Aggregator<T, R>: Send + Sync {
    /// Create initial accumulator
    fn create_accumulator(&self) -> R;

    /// Add a value to the accumulator
    fn add(&self, accumulator: &mut R, value: T);

    /// Get the result from the accumulator
    fn get_result(&self, accumulator: &R) -> R
    where
        R: Clone,
    {
        accumulator.clone()
    }

    /// Merge two accumulators
    fn merge(&self, acc1: &mut R, acc2: &R);
}

/// Sum aggregator
pub struct SumAggregator;

impl<T> Aggregator<T, T> for SumAggregator
where
    T: std::ops::AddAssign + Default + Clone + Send + Sync,
{
    fn create_accumulator(&self) -> T {
        T::default()
    }

    fn add(&self, accumulator: &mut T, value: T) {
        *accumulator += value;
    }

    fn merge(&self, acc1: &mut T, acc2: &T) {
        *acc1 += acc2.clone();
    }
}

/// Count aggregator
pub struct CountAggregator;

impl<T> Aggregator<T, usize> for CountAggregator
where
    T: Send + Sync,
{
    fn create_accumulator(&self) -> usize {
        0
    }

    fn add(&self, accumulator: &mut usize, _value: T) {
        *accumulator += 1;
    }

    fn merge(&self, acc1: &mut usize, acc2: &usize) {
        *acc1 += acc2;
    }
}

/// Average aggregator
pub struct AverageAggregator;

#[derive(Clone)]
pub struct AverageAccumulator {
    sum: f64,
    count: usize,
}

impl<T> Aggregator<T, AverageAccumulator> for AverageAggregator
where
    T: Into<f64> + Send + Sync,
{
    fn create_accumulator(&self) -> AverageAccumulator {
        AverageAccumulator { sum: 0.0, count: 0 }
    }

    fn add(&self, accumulator: &mut AverageAccumulator, value: T) {
        accumulator.sum += value.into();
        accumulator.count += 1;
    }

    fn merge(&self, acc1: &mut AverageAccumulator, acc2: &AverageAccumulator) {
        acc1.sum += acc2.sum;
        acc1.count += acc2.count;
    }
}

impl AverageAccumulator {
    pub fn average(&self) -> Option<f64> {
        if self.count > 0 {
            Some(self.sum / self.count as f64)
        } else {
            None
        }
    }
}

/// Min/Max aggregator
pub struct MinAggregator;

impl<T> Aggregator<T, Option<T>> for MinAggregator
where
    T: Ord + Clone + Send + Sync,
{
    fn create_accumulator(&self) -> Option<T> {
        None
    }

    fn add(&self, accumulator: &mut Option<T>, value: T) {
        *accumulator = Some(match accumulator.take() {
            Some(current) => current.min(value),
            None => value,
        });
    }

    fn merge(&self, acc1: &mut Option<T>, acc2: &Option<T>) {
        if let Some(val2) = acc2 {
            self.add(acc1, val2.clone());
        }
    }
}

pub struct MaxAggregator;

impl<T> Aggregator<T, Option<T>> for MaxAggregator
where
    T: Ord + Clone + Send + Sync,
{
    fn create_accumulator(&self) -> Option<T> {
        None
    }

    fn add(&self, accumulator: &mut Option<T>, value: T) {
        *accumulator = Some(match accumulator.take() {
            Some(current) => current.max(value),
            None => value,
        });
    }

    fn merge(&self, acc1: &mut Option<T>, acc2: &Option<T>) {
        if let Some(val2) = acc2 {
            self.add(acc1, val2.clone());
        }
    }
}

/// Aggregate operator
pub struct AggregateOperator<S, A, R>
where
    S: DataStream,
    A: Aggregator<S::Item, R> + 'static,
    R: Send + 'static,
{
    stream: S,
    aggregator: A,
    accumulator: R,
    emitted: bool,
}

impl<S, A, R> AggregateOperator<S, A, R>
where
    S: DataStream,
    A: Aggregator<S::Item, R> + 'static,
    R: Send + 'static,
{
    /// Create a new aggregate operator
    pub fn new(stream: S, aggregator: A) -> Self {
        let accumulator = aggregator.create_accumulator();
        Self {
            stream,
            aggregator,
            accumulator,
            emitted: false,
        }
    }
}

#[async_trait]
impl<S, A, R> DataStream for AggregateOperator<S, A, R>
where
    S: DataStream,
    A: Aggregator<S::Item, R> + 'static,
    R: Clone + Send + 'static,
{
    type Item = R;

    async fn next(&mut self) -> Result<Option<Self::Item>> {
        // Consume all items from the stream
        while let Some(item) = self.stream.next().await? {
            self.aggregator.add(&mut self.accumulator, item);
        }

        // Emit result once
        if !self.emitted {
            self.emitted = true;
            Ok(Some(self.aggregator.get_result(&self.accumulator)))
        } else {
            Ok(None)
        }
    }

    fn is_complete(&self) -> bool {
        self.stream.is_complete() && self.emitted
    }
}

/// Keyed aggregate operator
pub struct KeyedAggregateOperator<S, K, A, R>
where
    S: DataStream,
    K: Hash + Eq + Clone + Send + 'static,
    A: Aggregator<S::Item, R> + 'static,
    R: Send + 'static,
{
    stream: S,
    key_fn: Box<dyn Fn(&S::Item) -> K + Send + 'static>,
    aggregator: A,
    accumulators: HashMap<K, R>,
}

impl<S, K, A, R> KeyedAggregateOperator<S, K, A, R>
where
    S: DataStream,
    K: Hash + Eq + Clone + Send + 'static,
    A: Aggregator<S::Item, R> + 'static,
    R: Send + 'static,
{
    /// Create a new keyed aggregate operator
    pub fn new<F>(stream: S, key_fn: F, aggregator: A) -> Self
    where
        F: Fn(&S::Item) -> K + Send + 'static,
    {
        Self {
            stream,
            key_fn: Box::new(key_fn),
            aggregator,
            accumulators: HashMap::new(),
        }
    }
}

/// Keyed aggregate result
#[derive(Debug, Clone)]
pub struct KeyedAggregate<K, R> {
    pub key: K,
    pub result: R,
}

#[async_trait]
impl<S, K, A, R> DataStream for KeyedAggregateOperator<S, K, A, R>
where
    S: DataStream,
    S::Item: Clone,
    K: Hash + Eq + Clone + Send + 'static,
    A: Aggregator<S::Item, R> + 'static,
    R: Clone + Send + 'static,
{
    type Item = KeyedAggregate<K, R>;

    async fn next(&mut self) -> Result<Option<Self::Item>> {
        // Process one item at a time and emit results
        match self.stream.next().await? {
            Some(item) => {
                let key = (self.key_fn)(&item);

                let accumulator = self
                    .accumulators
                    .entry(key.clone())
                    .or_insert_with(|| self.aggregator.create_accumulator());

                self.aggregator.add(accumulator, item);

                let result = self.aggregator.get_result(accumulator);
                Ok(Some(KeyedAggregate { key, result }))
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
    use crate::source::iterator::RangeSource;
    use crate::source::Source;
    use crate::stream::StreamExt;

    #[tokio::test]
    async fn test_sum_aggregator() {
        let mut source = RangeSource::new(1, 6);
        source.start().await.unwrap();

        let mut agg = AggregateOperator::new(source, SumAggregator);

        let result = agg.next().await.unwrap().unwrap();
        assert_eq!(result, 15); // 1+2+3+4+5 = 15
    }

    #[tokio::test]
    async fn test_count_aggregator() {
        let mut source = RangeSource::new(0, 5);
        source.start().await.unwrap();

        let mut agg = AggregateOperator::new(source, CountAggregator);

        let result = agg.next().await.unwrap().unwrap();
        assert_eq!(result, 5);
    }

    #[tokio::test]
    async fn test_min_aggregator() {
        let mut source = RangeSource::new(3, 8);
        source.start().await.unwrap();

        let mut agg = AggregateOperator::new(source, MinAggregator);

        let result = agg.next().await.unwrap().unwrap();
        assert_eq!(result, Some(3));
    }

    #[tokio::test]
    async fn test_max_aggregator() {
        let mut source = RangeSource::new(3, 8);
        source.start().await.unwrap();

        let mut agg = AggregateOperator::new(source, MaxAggregator);

        let result = agg.next().await.unwrap().unwrap();
        assert_eq!(result, Some(7));
    }
}
