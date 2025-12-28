//! FlatMap operator for flattening nested streams.

use crate::error::Result;
use crate::stream::DataStream;
use async_trait::async_trait;

/// FlatMap operator that maps and flattens stream elements
pub struct FlatMapOperator<S, F, InnerS>
where
    S: DataStream,
    F: FnMut(S::Item) -> InnerS + Send + 'static,
    InnerS: DataStream + Send + 'static,
{
    stream: S,
    flat_map_fn: F,
    current_inner: Option<InnerS>,
}

impl<S, F, InnerS> FlatMapOperator<S, F, InnerS>
where
    S: DataStream,
    F: FnMut(S::Item) -> InnerS + Send + 'static,
    InnerS: DataStream + Send + 'static,
{
    /// Create a new flatmap operator
    pub fn new(stream: S, flat_map_fn: F) -> Self {
        Self {
            stream,
            flat_map_fn,
            current_inner: None,
        }
    }
}

#[async_trait]
impl<S, F, InnerS> DataStream for FlatMapOperator<S, F, InnerS>
where
    S: DataStream,
    F: FnMut(S::Item) -> InnerS + Send + 'static,
    InnerS: DataStream + Send + 'static,
{
    type Item = InnerS::Item;

    async fn next(&mut self) -> Result<Option<Self::Item>> {
        loop {
            // Try to get item from current inner stream
            if let Some(ref mut inner) = self.current_inner {
                match inner.next().await? {
                    Some(item) => return Ok(Some(item)),
                    None => {
                        // Current inner stream is exhausted
                        self.current_inner = None;
                    }
                }
            }

            // Get next item from outer stream and create new inner stream
            match self.stream.next().await? {
                Some(item) => {
                    self.current_inner = Some((self.flat_map_fn)(item));
                }
                None => return Ok(None),
            }
        }
    }

    fn is_complete(&self) -> bool {
        self.stream.is_complete() && self.current_inner.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::iterator::{RangeSource, RepeatSource};
    use crate::source::Source;
    use crate::stream::StreamExt;

    #[tokio::test]
    async fn test_flatmap_operator() {
        let mut source = RangeSource::new(1, 4);
        source.start().await.unwrap();

        let mut flatmapped = FlatMapOperator::new(source, |x| {
            let mut repeat = RepeatSource::new(x, x as usize);
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    repeat.start().await.unwrap();
                })
            });
            repeat
        });

        let results = flatmapped.collect().await.unwrap();
        assert_eq!(results, vec![1, 2, 2, 3, 3, 3]);
    }
}
