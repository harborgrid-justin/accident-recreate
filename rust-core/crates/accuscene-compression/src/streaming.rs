//! Streaming compression and decompression for large files

use crate::error::{CompressionError, Result};
use crate::traits::{Algorithm, CompressionLevel};
use bytes::{Bytes, BytesMut};
use std::io::{Read, Write};
use tracing::{debug, trace};

/// Chunk size for streaming operations (64KB)
const CHUNK_SIZE: usize = 64 * 1024;

/// Streaming compression context
pub struct StreamCompressor {
    algorithm: Algorithm,
    level: CompressionLevel,
    buffer: BytesMut,
}

impl StreamCompressor {
    /// Create a new streaming compressor
    pub fn new(algorithm: Algorithm, level: CompressionLevel) -> Self {
        Self {
            algorithm,
            level,
            buffer: BytesMut::with_capacity(CHUNK_SIZE * 2),
        }
    }

    /// Process a chunk of data
    pub fn process_chunk(&mut self, chunk: &[u8]) -> Result<Option<Bytes>> {
        trace!("Processing chunk of {} bytes", chunk.len());

        self.buffer.extend_from_slice(chunk);

        // If we have enough data, compress it
        if self.buffer.len() >= CHUNK_SIZE {
            let data = self.buffer.split().freeze();
            let compressed = self.compress_data(&data)?;
            Ok(Some(compressed))
        } else {
            Ok(None)
        }
    }

    /// Finalize compression and return any remaining data
    pub fn finalize(self) -> Result<Option<Bytes>> {
        if self.buffer.is_empty() {
            return Ok(None);
        }

        let data = self.buffer.freeze();
        let compressed = self.compress_data(&data)?;
        Ok(Some(compressed))
    }

    /// Compress data using the configured algorithm
    fn compress_data(&self, data: &[u8]) -> Result<Bytes> {
        use crate::algorithms;

        let compressed = algorithms::compress(data, self.algorithm, self.level)?;
        Ok(Bytes::from(compressed))
    }
}

/// Streaming decompression context
pub struct StreamDecompressor {
    algorithm: Algorithm,
    buffer: BytesMut,
}

impl StreamDecompressor {
    /// Create a new streaming decompressor
    pub fn new(algorithm: Algorithm) -> Self {
        Self {
            algorithm,
            buffer: BytesMut::with_capacity(CHUNK_SIZE * 2),
        }
    }

    /// Process a compressed chunk
    pub fn process_chunk(&mut self, chunk: &[u8]) -> Result<Option<Bytes>> {
        trace!("Processing compressed chunk of {} bytes", chunk.len());

        // For simplicity, decompress each chunk independently
        // In a production system, you'd want to handle chunk boundaries properly
        let decompressed = self.decompress_data(chunk)?;
        Ok(Some(Bytes::from(decompressed)))
    }

    /// Finalize decompression
    pub fn finalize(self) -> Result<Option<Bytes>> {
        if self.buffer.is_empty() {
            return Ok(None);
        }

        let decompressed = self.decompress_data(&self.buffer)?;
        Ok(Some(Bytes::from(decompressed)))
    }

    /// Decompress data using the configured algorithm
    fn decompress_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        use crate::algorithms;

        algorithms::decompress(data, self.algorithm)
    }
}

/// Compress a stream (Reader -> Writer)
pub fn compress_stream<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
    algorithm: Algorithm,
    level: CompressionLevel,
) -> Result<usize> {
    debug!("Starting stream compression with {:?}", algorithm);

    let mut buffer = vec![0u8; CHUNK_SIZE];
    let mut total_written = 0usize;

    loop {
        let bytes_read = reader
            .read(&mut buffer)
            .map_err(|e| CompressionError::Io(e))?;

        if bytes_read == 0 {
            break;
        }

        let chunk = &buffer[..bytes_read];
        let compressed = crate::algorithms::compress(chunk, algorithm, level)?;

        // Write compressed chunk size (4 bytes) followed by data
        let chunk_size = compressed.len() as u32;
        writer
            .write_all(&chunk_size.to_le_bytes())
            .map_err(|e| CompressionError::Io(e))?;
        writer
            .write_all(&compressed)
            .map_err(|e| CompressionError::Io(e))?;

        total_written += 4 + compressed.len();
    }

    debug!("Stream compression complete, wrote {} bytes", total_written);
    Ok(total_written)
}

/// Decompress a stream (Reader -> Writer)
pub fn decompress_stream<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
    algorithm: Algorithm,
) -> Result<usize> {
    debug!("Starting stream decompression with {:?}", algorithm);

    let mut total_written = 0usize;

    loop {
        // Read chunk size
        let mut size_buf = [0u8; 4];
        match reader.read_exact(&mut size_buf) {
            Ok(_) => {}
            Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(CompressionError::Io(e)),
        }

        let chunk_size = u32::from_le_bytes(size_buf) as usize;

        // Read compressed chunk
        let mut compressed = vec![0u8; chunk_size];
        reader
            .read_exact(&mut compressed)
            .map_err(|e| CompressionError::Io(e))?;

        // Decompress and write
        let decompressed = crate::algorithms::decompress(&compressed, algorithm)?;
        writer
            .write_all(&decompressed)
            .map_err(|e| CompressionError::Io(e))?;

        total_written += decompressed.len();
    }

    debug!("Stream decompression complete, wrote {} bytes", total_written);
    Ok(total_written)
}

/// Compress a file to another file
#[cfg(feature = "async")]
pub async fn compress_file(
    input_path: &std::path::Path,
    output_path: &std::path::Path,
    algorithm: Algorithm,
    level: CompressionLevel,
) -> Result<usize> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    debug!("Compressing file {:?} -> {:?}", input_path, output_path);

    let mut input = tokio::fs::File::open(input_path).await?;
    let mut output = tokio::fs::File::create(output_path).await?;

    let mut buffer = vec![0u8; CHUNK_SIZE];
    let mut total_written = 0usize;

    loop {
        let bytes_read = input.read(&mut buffer).await?;
        if bytes_read == 0 {
            break;
        }

        let chunk = &buffer[..bytes_read];
        let compressed = crate::algorithms::compress(chunk, algorithm, level)?;

        // Write chunk size + data
        let chunk_size = compressed.len() as u32;
        output.write_all(&chunk_size.to_le_bytes()).await?;
        output.write_all(&compressed).await?;

        total_written += 4 + compressed.len();
    }

    output.flush().await?;
    debug!("File compression complete, wrote {} bytes", total_written);
    Ok(total_written)
}

/// Decompress a file to another file
#[cfg(feature = "async")]
pub async fn decompress_file(
    input_path: &std::path::Path,
    output_path: &std::path::Path,
    algorithm: Algorithm,
) -> Result<usize> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    debug!("Decompressing file {:?} -> {:?}", input_path, output_path);

    let mut input = tokio::fs::File::open(input_path).await?;
    let mut output = tokio::fs::File::create(output_path).await?;

    let mut total_written = 0usize;

    loop {
        // Read chunk size
        let mut size_buf = [0u8; 4];
        match input.read_exact(&mut size_buf).await {
            Ok(_) => {}
            Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(CompressionError::Io(e)),
        }

        let chunk_size = u32::from_le_bytes(size_buf) as usize;

        // Read and decompress chunk
        let mut compressed = vec![0u8; chunk_size];
        input.read_exact(&mut compressed).await?;

        let decompressed = crate::algorithms::decompress(&compressed, algorithm)?;
        output.write_all(&decompressed).await?;

        total_written += decompressed.len();
    }

    output.flush().await?;
    debug!("File decompression complete, wrote {} bytes", total_written);
    Ok(total_written)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_compression() {
        let data = b"Hello, AccuScene! This is streaming compression test.".repeat(1000);
        let mut input = &data[..];
        let mut output = Vec::new();

        let bytes_written = compress_stream(
            &mut input,
            &mut output,
            Algorithm::Lz4,
            CompressionLevel::Default,
        )
        .unwrap();

        assert!(bytes_written > 0);
        assert!(output.len() < data.len());
    }

    #[test]
    fn test_stream_round_trip() {
        let data = b"Test data for streaming round trip.".repeat(500);
        let mut input = &data[..];
        let mut compressed = Vec::new();

        compress_stream(
            &mut input,
            &mut compressed,
            Algorithm::Zstd,
            CompressionLevel::Fast,
        )
        .unwrap();

        let mut comp_input = &compressed[..];
        let mut decompressed = Vec::new();

        decompress_stream(&mut comp_input, &mut decompressed, Algorithm::Zstd).unwrap();

        assert_eq!(data.to_vec(), decompressed);
    }

    #[test]
    fn test_stream_compressor() {
        let mut compressor = StreamCompressor::new(Algorithm::Lz4, CompressionLevel::Default);

        let chunk1 = b"First chunk of data for streaming compression.".repeat(100);
        let chunk2 = b"Second chunk of data.".repeat(100);

        let result1 = compressor.process_chunk(&chunk1).unwrap();
        assert!(result1.is_some());

        let result2 = compressor.process_chunk(&chunk2).unwrap();
        let final_chunk = compressor.finalize().unwrap();

        assert!(result2.is_some() || final_chunk.is_some());
    }
}
