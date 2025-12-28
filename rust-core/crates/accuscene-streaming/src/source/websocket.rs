//! WebSocket streaming source.

use crate::error::{Result, StreamingError};
use crate::source::Source;
use crate::stream::DataStream;
use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

/// WebSocket source that receives messages from a WebSocket connection
pub struct WebSocketSource<T> {
    url: String,
    receiver: mpsc::UnboundedReceiver<T>,
    running: bool,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> WebSocketSource<T>
where
    T: for<'de> Deserialize<'de> + Send + 'static,
{
    /// Create a new WebSocket source
    pub fn new(url: String) -> (mpsc::UnboundedSender<T>, Self) {
        let (tx, rx) = mpsc::unbounded_channel();

        (
            tx,
            Self {
                url,
                receiver: rx,
                running: false,
                _phantom: std::marker::PhantomData,
            },
        )
    }

    /// Create and start a WebSocket connection
    pub async fn connect(url: String) -> Result<Self> {
        let (tx, mut source) = Self::new(url.clone());

        // Spawn connection task
        tokio::spawn(async move {
            if let Err(e) = Self::connection_task(url, tx).await {
                eprintln!("WebSocket connection error: {}", e);
            }
        });

        source.start().await?;
        Ok(source)
    }

    async fn connection_task(
        url: String,
        sender: mpsc::UnboundedSender<T>,
    ) -> Result<()> {
        let (ws_stream, _) = connect_async(&url)
            .await
            .map_err(|e| StreamingError::WebSocketConnection(e.to_string()))?;

        let (mut _write, mut read) = ws_stream.split();

        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    match serde_json::from_str::<T>(&text) {
                        Ok(item) => {
                            if sender.send(item).is_err() {
                                break; // Receiver dropped
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to deserialize message: {}", e);
                        }
                    }
                }
                Ok(Message::Binary(data)) => {
                    match serde_json::from_slice::<T>(&data) {
                        Ok(item) => {
                            if sender.send(item).is_err() {
                                break;
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to deserialize binary message: {}", e);
                        }
                    }
                }
                Ok(Message::Close(_)) => break,
                Err(e) => {
                    eprintln!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }

        Ok(())
    }
}

#[async_trait]
impl<T: Send + 'static> DataStream for WebSocketSource<T> {
    type Item = T;

    async fn next(&mut self) -> Result<Option<Self::Item>> {
        if !self.running {
            return Ok(None);
        }

        Ok(self.receiver.recv().await)
    }

    fn is_complete(&self) -> bool {
        !self.running
    }
}

#[async_trait]
impl<T: Send + 'static> Source for WebSocketSource<T> {
    async fn start(&mut self) -> Result<()> {
        self.running = true;
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        self.running = false;
        self.receiver.close();
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running
    }
}

/// WebSocket message wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessage<T> {
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<MessageMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageMetadata {
    pub timestamp: i64,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sequence: Option<u64>,
}

impl<T> WebSocketMessage<T> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            metadata: None,
        }
    }

    pub fn with_metadata(data: T, source: String) -> Self {
        Self {
            data,
            metadata: Some(MessageMetadata {
                timestamp: chrono::Utc::now().timestamp_millis(),
                source,
                sequence: None,
            }),
        }
    }
}

/// Simple WebSocket text source
pub struct WebSocketTextSource {
    url: String,
    receiver: mpsc::UnboundedReceiver<String>,
    running: bool,
}

impl WebSocketTextSource {
    /// Create a new WebSocket text source
    pub fn new(url: String) -> (mpsc::UnboundedSender<String>, Self) {
        let (tx, rx) = mpsc::unbounded_channel();

        (
            tx,
            Self {
                url,
                receiver: rx,
                running: false,
            },
        )
    }

    /// Create and start a WebSocket connection
    pub async fn connect(url: String) -> Result<Self> {
        let (tx, mut source) = Self::new(url.clone());

        // Spawn connection task
        tokio::spawn(async move {
            if let Err(e) = Self::connection_task(url, tx).await {
                eprintln!("WebSocket connection error: {}", e);
            }
        });

        source.start().await?;
        Ok(source)
    }

    async fn connection_task(
        url: String,
        sender: mpsc::UnboundedSender<String>,
    ) -> Result<()> {
        let (ws_stream, _) = connect_async(&url)
            .await
            .map_err(|e| StreamingError::WebSocketConnection(e.to_string()))?;

        let (mut _write, mut read) = ws_stream.split();

        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if sender.send(text).is_err() {
                        break;
                    }
                }
                Ok(Message::Close(_)) => break,
                Err(e) => {
                    eprintln!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }

        Ok(())
    }
}

#[async_trait]
impl DataStream for WebSocketTextSource {
    type Item = String;

    async fn next(&mut self) -> Result<Option<Self::Item>> {
        if !self.running {
            return Ok(None);
        }

        Ok(self.receiver.recv().await)
    }

    fn is_complete(&self) -> bool {
        !self.running
    }
}

#[async_trait]
impl Source for WebSocketTextSource {
    async fn start(&mut self) -> Result<()> {
        self.running = true;
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        self.running = false;
        self.receiver.close();
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running
    }
}
