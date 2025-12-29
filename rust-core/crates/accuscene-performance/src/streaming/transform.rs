//! Stream transformation operations

use bytes::Bytes;
use std::future::Future;
use std::pin::Pin;

/// Trait for stream transformations
pub trait Transform: Send + Sync {
    /// Transform input data to output data
    fn transform(
        &mut self,
        input: Bytes,
    ) -> Pin<Box<dyn Future<Output = Option<Bytes>> + Send + '_>>;

    /// Transform batch of items
    fn transform_batch(
        &mut self,
        inputs: Vec<Bytes>,
    ) -> Pin<Box<dyn Future<Output = Vec<Bytes>> + Send + '_>> {
        Box::pin(async move {
            let mut results = Vec::with_capacity(inputs.len());
            for input in inputs {
                if let Some(output) = self.transform(input).await {
                    results.push(output);
                }
            }
            results
        })
    }
}

/// Map transformation
pub struct MapTransform<F>
where
    F: Fn(Bytes) -> Bytes + Send + Sync,
{
    func: F,
}

impl<F> MapTransform<F>
where
    F: Fn(Bytes) -> Bytes + Send + Sync,
{
    /// Create a new map transform
    pub fn new(func: F) -> Self {
        Self { func }
    }
}

impl<F> Transform for MapTransform<F>
where
    F: Fn(Bytes) -> Bytes + Send + Sync,
{
    fn transform(
        &mut self,
        input: Bytes,
    ) -> Pin<Box<dyn Future<Output = Option<Bytes>> + Send + '_>> {
        Box::pin(async move { Some((self.func)(input)) })
    }
}

/// Filter transformation
pub struct FilterTransform<F>
where
    F: Fn(&Bytes) -> bool + Send + Sync,
{
    predicate: F,
}

impl<F> FilterTransform<F>
where
    F: Fn(&Bytes) -> bool + Send + Sync,
{
    /// Create a new filter transform
    pub fn new(predicate: F) -> Self {
        Self { predicate }
    }
}

impl<F> Transform for FilterTransform<F>
where
    F: Fn(&Bytes) -> bool + Send + Sync,
{
    fn transform(
        &mut self,
        input: Bytes,
    ) -> Pin<Box<dyn Future<Output = Option<Bytes>> + Send + '_>> {
        Box::pin(async move {
            if (self.predicate)(&input) {
                Some(input)
            } else {
                None
            }
        })
    }
}

/// FlatMap transformation
pub struct FlatMapTransform<F>
where
    F: Fn(Bytes) -> Vec<Bytes> + Send + Sync,
{
    func: F,
    buffer: Vec<Bytes>,
}

impl<F> FlatMapTransform<F>
where
    F: Fn(Bytes) -> Vec<Bytes> + Send + Sync,
{
    /// Create a new flatmap transform
    pub fn new(func: F) -> Self {
        Self {
            func,
            buffer: Vec::new(),
        }
    }
}

impl<F> Transform for FlatMapTransform<F>
where
    F: Fn(Bytes) -> Vec<Bytes> + Send + Sync,
{
    fn transform(
        &mut self,
        input: Bytes,
    ) -> Pin<Box<dyn Future<Output = Option<Bytes>> + Send + '_>> {
        Box::pin(async move {
            let outputs = (self.func)(input);
            self.buffer.extend(outputs);

            self.buffer.pop()
        })
    }
}

/// Compression transformation (simplified example)
pub struct CompressionTransform {
    level: u32,
}

impl CompressionTransform {
    /// Create a new compression transform
    pub fn new(level: u32) -> Self {
        Self { level }
    }
}

impl Transform for CompressionTransform {
    fn transform(
        &mut self,
        input: Bytes,
    ) -> Pin<Box<dyn Future<Output = Option<Bytes>> + Send + '_>> {
        Box::pin(async move {
            // Simplified: In production, use actual compression like zstd or lz4
            // For now, just return the input as-is
            Some(input)
        })
    }
}

/// Decompression transformation
pub struct DecompressionTransform;

impl DecompressionTransform {
    /// Create a new decompression transform
    pub fn new() -> Self {
        Self
    }
}

impl Default for DecompressionTransform {
    fn default() -> Self {
        Self::new()
    }
}

impl Transform for DecompressionTransform {
    fn transform(
        &mut self,
        input: Bytes,
    ) -> Pin<Box<dyn Future<Output = Option<Bytes>> + Send + '_>> {
        Box::pin(async move {
            // Simplified: In production, use actual decompression
            Some(input)
        })
    }
}

/// Encryption transformation (placeholder)
pub struct EncryptionTransform {
    key: Vec<u8>,
}

impl EncryptionTransform {
    /// Create a new encryption transform
    pub fn new(key: Vec<u8>) -> Self {
        Self { key }
    }
}

impl Transform for EncryptionTransform {
    fn transform(
        &mut self,
        input: Bytes,
    ) -> Pin<Box<dyn Future<Output = Option<Bytes>> + Send + '_>> {
        Box::pin(async move {
            // Simplified: In production, use actual encryption like AES
            Some(input)
        })
    }
}

/// Chain multiple transforms together
pub struct ChainTransform {
    transforms: Vec<Box<dyn Transform>>,
}

impl ChainTransform {
    /// Create a new chain transform
    pub fn new() -> Self {
        Self {
            transforms: Vec::new(),
        }
    }

    /// Add a transform to the chain
    pub fn add<T: Transform + 'static>(mut self, transform: T) -> Self {
        self.transforms.push(Box::new(transform));
        self
    }
}

impl Default for ChainTransform {
    fn default() -> Self {
        Self::new()
    }
}

impl Transform for ChainTransform {
    fn transform(
        &mut self,
        input: Bytes,
    ) -> Pin<Box<dyn Future<Output = Option<Bytes>> + Send + '_>> {
        Box::pin(async move {
            let mut current = Some(input);

            for transform in &mut self.transforms {
                if let Some(data) = current {
                    current = transform.transform(data).await;
                } else {
                    return None;
                }
            }

            current
        })
    }
}

/// Identity transform (no-op)
pub struct IdentityTransform;

impl Transform for IdentityTransform {
    fn transform(
        &mut self,
        input: Bytes,
    ) -> Pin<Box<dyn Future<Output = Option<Bytes>> + Send + '_>> {
        Box::pin(async move { Some(input) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_map_transform() {
        let mut transform = MapTransform::new(|b| {
            let mut v = b.to_vec();
            v.reverse();
            Bytes::from(v)
        });

        let input = Bytes::from(vec![1, 2, 3]);
        let output = transform.transform(input).await.unwrap();
        assert_eq!(output, Bytes::from(vec![3, 2, 1]));
    }

    #[tokio::test]
    async fn test_filter_transform() {
        let mut transform = FilterTransform::new(|b| b.len() > 2);

        let input1 = Bytes::from(vec![1, 2, 3]);
        let output1 = transform.transform(input1).await;
        assert!(output1.is_some());

        let input2 = Bytes::from(vec![1]);
        let output2 = transform.transform(input2).await;
        assert!(output2.is_none());
    }

    #[tokio::test]
    async fn test_flatmap_transform() {
        let mut transform = FlatMapTransform::new(|b| {
            vec![b.clone(), b.clone()]
        });

        let input = Bytes::from(vec![1, 2]);
        let output = transform.transform(input).await;
        assert!(output.is_some());
    }

    #[tokio::test]
    async fn test_chain_transform() {
        let mut chain = ChainTransform::new()
            .add(MapTransform::new(|b| {
                Bytes::from(b.to_vec().into_iter().map(|x| x + 1).collect::<Vec<_>>())
            }))
            .add(FilterTransform::new(|b| b.len() > 0));

        let input = Bytes::from(vec![1, 2, 3]);
        let output = chain.transform(input).await.unwrap();
        assert_eq!(output, Bytes::from(vec![2, 3, 4]));
    }

    #[tokio::test]
    async fn test_identity_transform() {
        let mut transform = IdentityTransform;
        let input = Bytes::from(vec![1, 2, 3]);
        let output = transform.transform(input.clone()).await.unwrap();
        assert_eq!(input, output);
    }
}
