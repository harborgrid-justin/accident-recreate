//! Filter operator for filtering stream elements.

use crate::error::Result;
use crate::stream::DataStream;
use async_trait::async_trait;

/// Filter operator that filters items in a stream
pub struct FilterOperator<S, F>
where
    S: DataStream,
    F: FnMut(&S::Item) -> bool + Send + 'static,
{
    stream: S,
    predicate: F,
}

impl<S, F> FilterOperator<S, F>
where
    S: DataStream,
    F: FnMut(&S::Item) -> bool + Send + 'static,
{
    /// Create a new filter operator
    pub fn new(stream: S, predicate: F) -> Self {
        Self { stream, predicate }
    }
}

#[async_trait]
impl<S, F> DataStream for FilterOperator<S, F>
where
    S: DataStream,
    F: FnMut(&S::Item) -> bool + Send + 'static,
{
    type Item = S::Item;

    async fn next(&mut self) -> Result<Option<Self::Item>> {
        loop {
            match self.stream.next().await? {
                Some(item) => {
                    if (self.predicate)(&item) {
                        return Ok(Some(item));
                    }
                }
                None => return Ok(None),
            }
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
    async fn test_filter_operator() {
        let mut source = RangeSource::new(0, 10);
        source.start().await.unwrap();

        let mut filtered = FilterOperator::new(source, |x| x % 2 == 0);

        let results = filtered.collect().await.unwrap();
        assert_eq!(results, vec![0, 2, 4, 6, 8]);
    }
}
