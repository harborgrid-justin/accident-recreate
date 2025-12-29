//! Data source abstractions for streaming

use crate::error::Result;
use bytes::Bytes;
use std::future::Future;
use std::pin::Pin;

/// Trait for data sources
pub trait Source: Send + Sync {
    /// Get the next item from the source
    fn next(&mut self) -> Pin<Box<dyn Future<Output = Result<Option<Bytes>>> + Send + '_>>;

    /// Get size hint for the source
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }

    /// Reset the source to the beginning
    fn reset(&mut self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async { Ok(()) })
    }
}

/// Vector-based data source
pub struct VectorSource {
    data: Vec<Bytes>,
    index: usize,
}

impl VectorSource {
    /// Create a new vector source
    pub fn new(data: Vec<Bytes>) -> Self {
        Self { data, index: 0 }
    }

    /// Create from a vector of byte slices
    pub fn from_slices(slices: Vec<&[u8]>) -> Self {
        let data = slices.into_iter().map(Bytes::copy_from_slice).collect();
        Self::new(data)
    }

    /// Create from a vector of strings
    pub fn from_strings(strings: Vec<String>) -> Self {
        let data = strings.into_iter().map(|s| Bytes::from(s)).collect();
        Self::new(data)
    }

    /// Get remaining items
    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.index)
    }
}

impl Source for VectorSource {
    fn next(&mut self) -> Pin<Box<dyn Future<Output = Result<Option<Bytes>>> + Send + '_>> {
        Box::pin(async move {
            if self.index < self.data.len() {
                let item = self.data[self.index].clone();
                self.index += 1;
                Ok(Some(item))
            } else {
                Ok(None)
            }
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.remaining();
        (remaining, Some(remaining))
    }

    fn reset(&mut self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move {
            self.index = 0;
            Ok(())
        })
    }
}

/// Iterator-based data source
pub struct IteratorSource<I>
where
    I: Iterator<Item = Bytes> + Send + Sync,
{
    iter: I,
    size_hint: (usize, Option<usize>),
}

impl<I> IteratorSource<I>
where
    I: Iterator<Item = Bytes> + Send + Sync,
{
    /// Create a new iterator source
    pub fn new(iter: I) -> Self {
        let size_hint = iter.size_hint();
        Self { iter, size_hint }
    }
}

impl<I> Source for IteratorSource<I>
where
    I: Iterator<Item = Bytes> + Send + Sync,
{
    fn next(&mut self) -> Pin<Box<dyn Future<Output = Result<Option<Bytes>>> + Send + '_>> {
        Box::pin(async move { Ok(self.iter.next()) })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.size_hint
    }
}

/// Range source that generates sequential bytes
pub struct RangeSource {
    start: u64,
    end: u64,
    current: u64,
    chunk_size: usize,
}

impl RangeSource {
    /// Create a new range source
    pub fn new(start: u64, end: u64, chunk_size: usize) -> Self {
        Self {
            start,
            end,
            current: start,
            chunk_size,
        }
    }

    /// Create with default chunk size (1024 bytes)
    pub fn with_range(start: u64, end: u64) -> Self {
        Self::new(start, end, 1024)
    }
}

impl Source for RangeSource {
    fn next(&mut self) -> Pin<Box<dyn Future<Output = Result<Option<Bytes>>> + Send + '_>> {
        Box::pin(async move {
            if self.current >= self.end {
                return Ok(None);
            }

            let remaining = (self.end - self.current) as usize;
            let size = remaining.min(self.chunk_size);
            let mut data = Vec::with_capacity(size);

            for _ in 0..size {
                data.push((self.current % 256) as u8);
                self.current += 1;
            }

            Ok(Some(Bytes::from(data)))
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.end.saturating_sub(self.current) as usize;
        let chunks = (remaining + self.chunk_size - 1) / self.chunk_size;
        (chunks, Some(chunks))
    }

    fn reset(&mut self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move {
            self.current = self.start;
            Ok(())
        })
    }
}

/// Repeating source that cycles through data
pub struct RepeatingSource {
    data: Vec<Bytes>,
    index: usize,
    max_repeats: Option<usize>,
    current_repeat: usize,
}

impl RepeatingSource {
    /// Create a new repeating source
    pub fn new(data: Vec<Bytes>, max_repeats: Option<usize>) -> Self {
        Self {
            data,
            index: 0,
            max_repeats,
            current_repeat: 0,
        }
    }

    /// Create infinite repeating source
    pub fn infinite(data: Vec<Bytes>) -> Self {
        Self::new(data, None)
    }

    /// Create finite repeating source
    pub fn finite(data: Vec<Bytes>, repeats: usize) -> Self {
        Self::new(data, Some(repeats))
    }
}

impl Source for RepeatingSource {
    fn next(&mut self) -> Pin<Box<dyn Future<Output = Result<Option<Bytes>>> + Send + '_>> {
        Box::pin(async move {
            if self.data.is_empty() {
                return Ok(None);
            }

            if let Some(max) = self.max_repeats {
                if self.current_repeat >= max {
                    return Ok(None);
                }
            }

            let item = self.data[self.index].clone();
            self.index += 1;

            if self.index >= self.data.len() {
                self.index = 0;
                self.current_repeat += 1;
            }

            Ok(Some(item))
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if let Some(max) = self.max_repeats {
            let remaining_repeats = max.saturating_sub(self.current_repeat);
            let remaining_items = remaining_repeats * self.data.len()
                + self.data.len().saturating_sub(self.index);
            (remaining_items, Some(remaining_items))
        } else {
            (usize::MAX, None)
        }
    }

    fn reset(&mut self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move {
            self.index = 0;
            self.current_repeat = 0;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_vector_source() {
        let data = vec![Bytes::from("a"), Bytes::from("b"), Bytes::from("c")];
        let mut source = VectorSource::new(data);

        assert_eq!(source.remaining(), 3);

        let item1 = source.next().await.unwrap();
        assert_eq!(item1, Some(Bytes::from("a")));

        let item2 = source.next().await.unwrap();
        assert_eq!(item2, Some(Bytes::from("b")));

        assert_eq!(source.remaining(), 1);
    }

    #[tokio::test]
    async fn test_range_source() {
        let mut source = RangeSource::new(0, 100, 10);

        let item = source.next().await.unwrap();
        assert!(item.is_some());
        assert_eq!(item.unwrap().len(), 10);
    }

    #[tokio::test]
    async fn test_repeating_source() {
        let data = vec![Bytes::from("x")];
        let mut source = RepeatingSource::finite(data, 3);

        for _ in 0..3 {
            let item = source.next().await.unwrap();
            assert_eq!(item, Some(Bytes::from("x")));
        }

        let item = source.next().await.unwrap();
        assert_eq!(item, None);
    }

    #[tokio::test]
    async fn test_source_reset() {
        let data = vec![Bytes::from("a"), Bytes::from("b")];
        let mut source = VectorSource::new(data);

        source.next().await.unwrap();
        source.next().await.unwrap();
        assert_eq!(source.remaining(), 0);

        source.reset().await.unwrap();
        assert_eq!(source.remaining(), 2);
    }
}
