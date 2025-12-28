//! Iterator-based streaming source.

use crate::error::Result;
use crate::source::Source;
use crate::stream::DataStream;
use async_trait::async_trait;

/// Iterator source that wraps any iterator
pub struct IteratorSource<I>
where
    I: Iterator + Send + 'static,
{
    iterator: I,
    running: bool,
}

impl<I> IteratorSource<I>
where
    I: Iterator + Send + 'static,
{
    /// Create a new iterator source
    pub fn new(iterator: I) -> Self {
        Self {
            iterator,
            running: false,
        }
    }
}

#[async_trait]
impl<I> DataStream for IteratorSource<I>
where
    I: Iterator + Send + 'static,
    I::Item: Send + 'static,
{
    type Item = I::Item;

    async fn next(&mut self) -> Result<Option<Self::Item>> {
        if !self.running {
            return Ok(None);
        }

        Ok(self.iterator.next())
    }

    fn is_complete(&self) -> bool {
        !self.running
    }
}

#[async_trait]
impl<I> Source for IteratorSource<I>
where
    I: Iterator + Send + 'static,
    I::Item: Send + 'static,
{
    async fn start(&mut self) -> Result<()> {
        self.running = true;
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        self.running = false;
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running
    }
}

/// Range source that generates a range of values
pub struct RangeSource {
    start: i64,
    end: i64,
    current: i64,
    running: bool,
}

impl RangeSource {
    /// Create a new range source
    pub fn new(start: i64, end: i64) -> Self {
        Self {
            start,
            end,
            current: start,
            running: false,
        }
    }
}

#[async_trait]
impl DataStream for RangeSource {
    type Item = i64;

    async fn next(&mut self) -> Result<Option<Self::Item>> {
        if !self.running {
            return Ok(None);
        }

        if self.current >= self.end {
            return Ok(None);
        }

        let value = self.current;
        self.current += 1;
        Ok(Some(value))
    }

    fn is_complete(&self) -> bool {
        !self.running || self.current >= self.end
    }
}

#[async_trait]
impl Source for RangeSource {
    async fn start(&mut self) -> Result<()> {
        self.running = true;
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        self.running = false;
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running
    }
}

/// Repeat source that repeats a value n times
pub struct RepeatSource<T: Clone> {
    value: T,
    count: usize,
    current: usize,
    running: bool,
}

impl<T: Clone> RepeatSource<T> {
    /// Create a new repeat source
    pub fn new(value: T, count: usize) -> Self {
        Self {
            value,
            count,
            current: 0,
            running: false,
        }
    }

    /// Create a repeat source that repeats infinitely
    pub fn infinite(value: T) -> RepeatInfiniteSource<T> {
        RepeatInfiniteSource {
            value,
            running: false,
        }
    }
}

#[async_trait]
impl<T: Clone + Send + 'static> DataStream for RepeatSource<T> {
    type Item = T;

    async fn next(&mut self) -> Result<Option<Self::Item>> {
        if !self.running {
            return Ok(None);
        }

        if self.current >= self.count {
            return Ok(None);
        }

        self.current += 1;
        Ok(Some(self.value.clone()))
    }

    fn is_complete(&self) -> bool {
        !self.running || self.current >= self.count
    }
}

#[async_trait]
impl<T: Clone + Send + 'static> Source for RepeatSource<T> {
    async fn start(&mut self) -> Result<()> {
        self.running = true;
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        self.running = false;
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running
    }
}

/// Repeat source that repeats infinitely
pub struct RepeatInfiniteSource<T: Clone> {
    value: T,
    running: bool,
}

#[async_trait]
impl<T: Clone + Send + 'static> DataStream for RepeatInfiniteSource<T> {
    type Item = T;

    async fn next(&mut self) -> Result<Option<Self::Item>> {
        if !self.running {
            return Ok(None);
        }

        Ok(Some(self.value.clone()))
    }

    fn is_complete(&self) -> bool {
        !self.running
    }
}

#[async_trait]
impl<T: Clone + Send + 'static> Source for RepeatInfiniteSource<T> {
    async fn start(&mut self) -> Result<()> {
        self.running = true;
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        self.running = false;
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stream::StreamExt;

    #[tokio::test]
    async fn test_iterator_source() {
        let mut source = IteratorSource::new(vec![1, 2, 3].into_iter());
        source.start().await.unwrap();

        assert_eq!(source.next().await.unwrap(), Some(1));
        assert_eq!(source.next().await.unwrap(), Some(2));
        assert_eq!(source.next().await.unwrap(), Some(3));
        assert_eq!(source.next().await.unwrap(), None);
    }

    #[tokio::test]
    async fn test_range_source() {
        let mut source = RangeSource::new(0, 5);
        source.start().await.unwrap();

        let items = source.collect().await.unwrap();
        assert_eq!(items, vec![0, 1, 2, 3, 4]);
    }

    #[tokio::test]
    async fn test_repeat_source() {
        let mut source = RepeatSource::new(42, 3);
        source.start().await.unwrap();

        assert_eq!(source.next().await.unwrap(), Some(42));
        assert_eq!(source.next().await.unwrap(), Some(42));
        assert_eq!(source.next().await.unwrap(), Some(42));
        assert_eq!(source.next().await.unwrap(), None);
    }
}
