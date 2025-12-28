//! Map operator for transforming stream elements.

use crate::error::Result;
use crate::stream::DataStream;
use async_trait::async_trait;

/// Map operator that transforms each item in a stream
pub struct MapOperator<S, F, U>
where
    S: DataStream,
    F: FnMut(S::Item) -> U + Send + 'static,
    U: Send + 'static,
{
    stream: S,
    map_fn: F,
    _phantom: std::marker::PhantomData<U>,
}

impl<S, F, U> MapOperator<S, F, U>
where
    S: DataStream,
    F: FnMut(S::Item) -> U + Send + 'static,
    U: Send + 'static,
{
    /// Create a new map operator
    pub fn new(stream: S, map_fn: F) -> Self {
        Self {
            stream,
            map_fn,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<S, F, U> DataStream for MapOperator<S, F, U>
where
    S: DataStream,
    F: FnMut(S::Item) -> U + Send + 'static,
    U: Send + 'static,
{
    type Item = U;

    async fn next(&mut self) -> Result<Option<Self::Item>> {
        match self.stream.next().await? {
            Some(item) => Ok(Some((self.map_fn)(item))),
            None => Ok(None),
        }
    }

    fn is_complete(&self) -> bool {
        self.stream.is_complete()
    }
}

/// Async map operator
pub struct AsyncMapOperator<S, F, Fut, U>
where
    S: DataStream,
    F: FnMut(S::Item) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = U> + Send + 'static,
    U: Send + 'static,
{
    stream: S,
    map_fn: F,
    _phantom: std::marker::PhantomData<(Fut, U)>,
}

impl<S, F, Fut, U> AsyncMapOperator<S, F, Fut, U>
where
    S: DataStream,
    F: FnMut(S::Item) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = U> + Send + 'static,
    U: Send + 'static,
{
    /// Create a new async map operator
    pub fn new(stream: S, map_fn: F) -> Self {
        Self {
            stream,
            map_fn,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<S, F, Fut, U> DataStream for AsyncMapOperator<S, F, Fut, U>
where
    S: DataStream,
    F: FnMut(S::Item) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = U> + Send + 'static,
    U: Send + 'static,
{
    type Item = U;

    async fn next(&mut self) -> Result<Option<Self::Item>> {
        match self.stream.next().await? {
            Some(item) => {
                let result = (self.map_fn)(item).await;
                Ok(Some(result))
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
    async fn test_map_operator() {
        let mut source = RangeSource::new(0, 5);
        source.start().await.unwrap();

        let mut mapped = MapOperator::new(source, |x| x * 2);

        let results = mapped.collect().await.unwrap();
        assert_eq!(results, vec![0, 2, 4, 6, 8]);
    }

    #[tokio::test]
    async fn test_async_map_operator() {
        let mut source = RangeSource::new(0, 3);
        source.start().await.unwrap();

        let mut mapped = AsyncMapOperator::new(source, |x| async move { x * 2 });

        let results = mapped.collect().await.unwrap();
        assert_eq!(results, vec![0, 2, 4]);
    }
}
