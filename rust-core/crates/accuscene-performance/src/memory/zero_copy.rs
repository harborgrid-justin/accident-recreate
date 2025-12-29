//! Zero-copy buffer management

use bytes::{Bytes, BytesMut};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

/// Zero-copy buffer with reference counting
#[derive(Clone)]
pub struct ZeroCopyBuffer {
    inner: Bytes,
}

impl ZeroCopyBuffer {
    /// Create a new zero-copy buffer
    pub fn new(data: Bytes) -> Self {
        Self { inner: data }
    }

    /// Create from a vector (will be converted to Bytes)
    pub fn from_vec(vec: Vec<u8>) -> Self {
        Self {
            inner: Bytes::from(vec),
        }
    }

    /// Create from a slice (will be copied)
    pub fn from_slice(slice: &[u8]) -> Self {
        Self {
            inner: Bytes::copy_from_slice(slice),
        }
    }

    /// Create from static data (zero-copy)
    pub fn from_static(data: &'static [u8]) -> Self {
        Self {
            inner: Bytes::from_static(data),
        }
    }

    /// Get a slice of the buffer (zero-copy)
    pub fn slice(&self, range: impl std::ops::RangeBounds<usize>) -> ZeroCopyBuffer {
        use std::ops::Bound;

        let len = self.inner.len();
        let start = match range.start_bound() {
            Bound::Included(&n) => n,
            Bound::Excluded(&n) => n + 1,
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(&n) => n + 1,
            Bound::Excluded(&n) => n,
            Bound::Unbounded => len,
        };

        Self {
            inner: self.inner.slice(start..end),
        }
    }

    /// Get the length
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Get as bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.inner
    }

    /// Convert to Bytes
    pub fn into_bytes(self) -> Bytes {
        self.inner
    }
}

impl Deref for ZeroCopyBuffer {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AsRef<[u8]> for ZeroCopyBuffer {
    fn as_ref(&self) -> &[u8] {
        &self.inner
    }
}

impl From<Bytes> for ZeroCopyBuffer {
    fn from(bytes: Bytes) -> Self {
        Self::new(bytes)
    }
}

impl From<Vec<u8>> for ZeroCopyBuffer {
    fn from(vec: Vec<u8>) -> Self {
        Self::from_vec(vec)
    }
}

impl From<&[u8]> for ZeroCopyBuffer {
    fn from(slice: &[u8]) -> Self {
        Self::from_slice(slice)
    }
}

/// Zero-copy slice with shared ownership
pub struct ZeroCopySlice {
    buffer: Arc<Vec<u8>>,
    offset: usize,
    length: usize,
}

impl ZeroCopySlice {
    /// Create a new zero-copy slice
    pub fn new(data: Vec<u8>) -> Self {
        let length = data.len();
        Self {
            buffer: Arc::new(data),
            offset: 0,
            length,
        }
    }

    /// Create a slice of this slice
    pub fn slice(&self, start: usize, end: usize) -> Self {
        let start = start.min(self.length);
        let end = end.min(self.length);

        Self {
            buffer: self.buffer.clone(),
            offset: self.offset + start,
            length: end - start,
        }
    }

    /// Get the length
    pub fn len(&self) -> usize {
        self.length
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    /// Get as a slice
    pub fn as_slice(&self) -> &[u8] {
        &self.buffer[self.offset..self.offset + self.length]
    }

    /// Copy to a new vector
    pub fn to_vec(&self) -> Vec<u8> {
        self.as_slice().to_vec()
    }
}

impl Clone for ZeroCopySlice {
    fn clone(&self) -> Self {
        Self {
            buffer: self.buffer.clone(),
            offset: self.offset,
            length: self.length,
        }
    }
}

impl Deref for ZeroCopySlice {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

/// Mutable zero-copy buffer
pub struct ZeroCopyBufferMut {
    inner: BytesMut,
}

impl ZeroCopyBufferMut {
    /// Create a new mutable buffer
    pub fn new() -> Self {
        Self {
            inner: BytesMut::new(),
        }
    }

    /// Create with capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: BytesMut::with_capacity(capacity),
        }
    }

    /// Get the length
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Get capacity
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    /// Reserve additional capacity
    pub fn reserve(&mut self, additional: usize) {
        self.inner.reserve(additional);
    }

    /// Extend from slice
    pub fn extend_from_slice(&mut self, slice: &[u8]) {
        self.inner.extend_from_slice(slice);
    }

    /// Clear the buffer
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Freeze into immutable buffer
    pub fn freeze(self) -> ZeroCopyBuffer {
        ZeroCopyBuffer {
            inner: self.inner.freeze(),
        }
    }

    /// Split off at index
    pub fn split_off(&mut self, at: usize) -> ZeroCopyBufferMut {
        Self {
            inner: self.inner.split_off(at),
        }
    }

    /// Split to at index
    pub fn split_to(&mut self, at: usize) -> ZeroCopyBufferMut {
        Self {
            inner: self.inner.split_to(at),
        }
    }
}

impl Default for ZeroCopyBufferMut {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for ZeroCopyBufferMut {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for ZeroCopyBufferMut {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// Chain multiple buffers together without copying
pub struct BufferChain {
    buffers: Vec<ZeroCopyBuffer>,
    total_len: usize,
}

impl BufferChain {
    /// Create a new buffer chain
    pub fn new() -> Self {
        Self {
            buffers: Vec::new(),
            total_len: 0,
        }
    }

    /// Add a buffer to the chain
    pub fn push(&mut self, buffer: ZeroCopyBuffer) {
        self.total_len += buffer.len();
        self.buffers.push(buffer);
    }

    /// Get total length
    pub fn len(&self) -> usize {
        self.total_len
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.total_len == 0
    }

    /// Get number of buffers
    pub fn buffer_count(&self) -> usize {
        self.buffers.len()
    }

    /// Copy all data to a single vector
    pub fn to_vec(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(self.total_len);
        for buffer in &self.buffers {
            result.extend_from_slice(buffer.as_bytes());
        }
        result
    }

    /// Iterate over buffers
    pub fn iter(&self) -> impl Iterator<Item = &ZeroCopyBuffer> {
        self.buffers.iter()
    }

    /// Clear the chain
    pub fn clear(&mut self) {
        self.buffers.clear();
        self.total_len = 0;
    }

    /// Flatten into a single buffer (may copy)
    pub fn flatten(self) -> ZeroCopyBuffer {
        if self.buffers.len() == 1 {
            self.buffers.into_iter().next().unwrap()
        } else {
            ZeroCopyBuffer::from_vec(self.to_vec())
        }
    }
}

impl Default for BufferChain {
    fn default() -> Self {
        Self::new()
    }
}

impl FromIterator<ZeroCopyBuffer> for BufferChain {
    fn from_iter<T: IntoIterator<Item = ZeroCopyBuffer>>(iter: T) -> Self {
        let mut chain = BufferChain::new();
        for buffer in iter {
            chain.push(buffer);
        }
        chain
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_copy_buffer() {
        let data = vec![1, 2, 3, 4, 5];
        let buffer = ZeroCopyBuffer::from_vec(data);

        assert_eq!(buffer.len(), 5);
        assert_eq!(buffer.as_bytes(), &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_zero_copy_slice() {
        let buffer = ZeroCopyBuffer::from_vec(vec![1, 2, 3, 4, 5]);
        let slice = buffer.slice(1..4);

        assert_eq!(slice.len(), 3);
        assert_eq!(slice.as_bytes(), &[2, 3, 4]);
    }

    #[test]
    fn test_zero_copy_slice_nested() {
        let slice1 = ZeroCopySlice::new(vec![1, 2, 3, 4, 5, 6, 7, 8]);
        let slice2 = slice1.slice(2, 6);

        assert_eq!(slice2.len(), 4);
        assert_eq!(slice2.as_slice(), &[3, 4, 5, 6]);

        let slice3 = slice2.slice(1, 3);
        assert_eq!(slice3.as_slice(), &[4, 5]);
    }

    #[test]
    fn test_zero_copy_buffer_mut() {
        let mut buffer = ZeroCopyBufferMut::with_capacity(10);

        buffer.extend_from_slice(&[1, 2, 3]);
        assert_eq!(buffer.len(), 3);

        buffer.extend_from_slice(&[4, 5]);
        assert_eq!(buffer.len(), 5);

        let frozen = buffer.freeze();
        assert_eq!(frozen.as_bytes(), &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_buffer_chain() {
        let mut chain = BufferChain::new();

        chain.push(ZeroCopyBuffer::from_vec(vec![1, 2]));
        chain.push(ZeroCopyBuffer::from_vec(vec![3, 4, 5]));
        chain.push(ZeroCopyBuffer::from_vec(vec![6]));

        assert_eq!(chain.len(), 6);
        assert_eq!(chain.buffer_count(), 3);

        let vec = chain.to_vec();
        assert_eq!(vec, vec![1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn test_buffer_chain_flatten() {
        let mut chain = BufferChain::new();
        chain.push(ZeroCopyBuffer::from_vec(vec![1, 2, 3]));

        let flattened = chain.flatten();
        assert_eq!(flattened.as_bytes(), &[1, 2, 3]);
    }

    #[test]
    fn test_split_buffer() {
        let mut buffer = ZeroCopyBufferMut::from_iter(vec![1, 2, 3, 4, 5, 6]);

        let right = buffer.split_off(3);
        assert_eq!(buffer.len(), 3);
        assert_eq!(right.len(), 3);
    }
}
