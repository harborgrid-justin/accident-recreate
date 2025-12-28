//! WebSocket streaming sink.

use crate::error::{Result, StreamingError};
use crate::sink::Sink;
use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

/// WebSocket sink that sends messages to a WebSocket connection
pub struct WebSocketSink<T> {
    url: String,
    writer: Option<futures_util::stream::SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>,
    connected: bool,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> WebSocketSink<T>
where
    T: Serialize + Send + 'static,
{
    /// Create a new WebSocket sink
    pub fn new(url: String) -> Self {
        Self {
            url,
            writer: None,
            connected: false,
            _phantom: std::marker::PhantomData,
        }
    }

    async fn ensure_connected(&mut self) -> Result<()> {
        if self.connected {
            return Ok(());
        }

        let (ws_stream, _) = connect_async(&self.url)
            .await
            .map_err(|e| StreamingError::WebSocketConnection(e.to_string()))?;

        let (write, _read) = ws_stream.split();
        self.writer = Some(write);
        self.connected = true;

        Ok(())
    }
}

#[async_trait]
impl<T> Sink<T> for WebSocketSink<T>
where
    T: Serialize + Send + 'static,
{
    async fn write(&mut self, item: T) -> Result<()> {
        self.ensure_connected().await?;

        let json = serde_json::to_string(&item)
            .map_err(|e| StreamingError::Sink(format!("JSON serialization error: {}", e)))?;

        let writer = self.writer.as_mut().unwrap();
        writer
            .send(Message::Text(json))
            .await
            .map_err(|e| StreamingError::Sink(format!("WebSocket send error: {}", e)))?;

        Ok(())
    }

    async fn flush(&mut self) -> Result<()> {
        if let Some(writer) = &mut self.writer {
            writer
                .flush()
                .await
                .map_err(|e| StreamingError::Sink(format!("WebSocket flush error: {}", e)))?;
        }
        Ok(())
    }

    async fn close(&mut self) -> Result<()> {
        if let Some(mut writer) = self.writer.take() {
            writer
                .send(Message::Close(None))
                .await
                .map_err(|e| StreamingError::Sink(format!("WebSocket close error: {}", e)))?;
            writer
                .close()
                .await
                .map_err(|e| StreamingError::Sink(format!("WebSocket close error: {}", e)))?;
        }
        self.connected = false;
        Ok(())
    }
}

/// WebSocket text sink for simple string messages
pub struct WebSocketTextSink {
    url: String,
    writer: Option<futures_util::stream::SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>,
    connected: bool,
}

impl WebSocketTextSink {
    /// Create a new WebSocket text sink
    pub fn new(url: String) -> Self {
        Self {
            url,
            writer: None,
            connected: false,
        }
    }

    async fn ensure_connected(&mut self) -> Result<()> {
        if self.connected {
            return Ok(());
        }

        let (ws_stream, _) = connect_async(&self.url)
            .await
            .map_err(|e| StreamingError::WebSocketConnection(e.to_string()))?;

        let (write, _read) = ws_stream.split();
        self.writer = Some(write);
        self.connected = true;

        Ok(())
    }
}

#[async_trait]
impl Sink<String> for WebSocketTextSink {
    async fn write(&mut self, item: String) -> Result<()> {
        self.ensure_connected().await?;

        let writer = self.writer.as_mut().unwrap();
        writer
            .send(Message::Text(item))
            .await
            .map_err(|e| StreamingError::Sink(format!("WebSocket send error: {}", e)))?;

        Ok(())
    }

    async fn flush(&mut self) -> Result<()> {
        if let Some(writer) = &mut self.writer {
            writer
                .flush()
                .await
                .map_err(|e| StreamingError::Sink(format!("WebSocket flush error: {}", e)))?;
        }
        Ok(())
    }

    async fn close(&mut self) -> Result<()> {
        if let Some(mut writer) = self.writer.take() {
            writer
                .send(Message::Close(None))
                .await
                .map_err(|e| StreamingError::Sink(format!("WebSocket close error: {}", e)))?;
            writer
                .close()
                .await
                .map_err(|e| StreamingError::Sink(format!("WebSocket close error: {}", e)))?;
        }
        self.connected = false;
        Ok(())
    }
}

/// Broadcast WebSocket sink that sends to multiple connections
pub struct BroadcastWebSocketSink<T> {
    sinks: Vec<WebSocketSink<T>>,
}

impl<T> BroadcastWebSocketSink<T>
where
    T: Serialize + Send + Clone + 'static,
{
    /// Create a new broadcast WebSocket sink
    pub fn new(urls: Vec<String>) -> Self {
        let sinks = urls.into_iter().map(WebSocketSink::new).collect();
        Self { sinks }
    }
}

#[async_trait]
impl<T> Sink<T> for BroadcastWebSocketSink<T>
where
    T: Serialize + Send + Clone + 'static,
{
    async fn write(&mut self, item: T) -> Result<()> {
        // Send to all sinks, collecting errors
        let mut errors = Vec::new();

        for sink in &mut self.sinks {
            if let Err(e) = sink.write(item.clone()).await {
                errors.push(e);
            }
        }

        if !errors.is_empty() {
            return Err(StreamingError::Sink(format!(
                "Failed to write to {} sinks",
                errors.len()
            )));
        }

        Ok(())
    }

    async fn flush(&mut self) -> Result<()> {
        for sink in &mut self.sinks {
            sink.flush().await?;
        }
        Ok(())
    }

    async fn close(&mut self) -> Result<()> {
        for sink in &mut self.sinks {
            sink.close().await?;
        }
        Ok(())
    }
}
