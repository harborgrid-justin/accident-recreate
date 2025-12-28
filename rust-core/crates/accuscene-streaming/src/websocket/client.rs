//! WebSocket client implementation.

use crate::error::{Result, StreamingError};
use crate::websocket::protocol::{ConnectionState, WsMessage};
use futures_util::{SinkExt, StreamExt};
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

/// WebSocket client
pub struct WsClient {
    /// Server URL
    url: String,
    /// Connection state
    state: Arc<RwLock<ConnectionState>>,
    /// Message sender
    message_tx: Option<mpsc::UnboundedSender<WsMessage>>,
    /// Received message channel
    received_rx: Arc<RwLock<Option<mpsc::UnboundedReceiver<WsMessage>>>>,
    /// Reconnect configuration
    reconnect_config: ReconnectConfig,
}

impl WsClient {
    /// Create a new WebSocket client
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            message_tx: None,
            received_rx: Arc::new(RwLock::new(None)),
            reconnect_config: ReconnectConfig::default(),
        }
    }

    /// Create a client with reconnect configuration
    pub fn with_reconnect(mut self, config: ReconnectConfig) -> Self {
        self.reconnect_config = config;
        self
    }

    /// Connect to the WebSocket server
    pub async fn connect(&mut self) -> Result<()> {
        *self.state.write() = ConnectionState::Connecting;

        let (ws_stream, _) = connect_async(&self.url)
            .await
            .map_err(|e| StreamingError::WebSocketConnection(e.to_string()))?;

        let (message_tx, message_rx) = mpsc::unbounded_channel();
        let (received_tx, received_rx) = mpsc::unbounded_channel();

        self.message_tx = Some(message_tx);
        *self.received_rx.write() = Some(received_rx);

        *self.state.write() = ConnectionState::Connected;

        // Spawn task to handle WebSocket communication
        let state = self.state.clone();
        tokio::spawn(async move {
            Self::handle_connection(ws_stream, message_rx, received_tx, state).await;
        });

        Ok(())
    }

    /// Handle WebSocket connection
    async fn handle_connection(
        ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
        mut message_rx: mpsc::UnboundedReceiver<WsMessage>,
        received_tx: mpsc::UnboundedSender<WsMessage>,
        state: Arc<RwLock<ConnectionState>>,
    ) {
        let (mut write, mut read) = ws_stream.split();

        loop {
            tokio::select! {
                // Send messages from the queue
                msg = message_rx.recv() => {
                    if let Some(message) = msg {
                        match message.to_json() {
                            Ok(json) => {
                                if let Err(e) = write.send(tokio_tungstenite::tungstenite::Message::Text(json)).await {
                                    tracing::error!("Failed to send message: {}", e);
                                    break;
                                }
                            }
                            Err(e) => {
                                tracing::error!("Failed to serialize message: {}", e);
                            }
                        }
                    } else {
                        // Channel closed
                        break;
                    }
                }

                // Receive messages
                msg = read.next() => {
                    match msg {
                        Some(Ok(tokio_tungstenite::tungstenite::Message::Text(text))) => {
                            match WsMessage::from_json(&text) {
                                Ok(ws_msg) => {
                                    if let Err(e) = received_tx.send(ws_msg) {
                                        tracing::error!("Failed to forward received message: {}", e);
                                    }
                                }
                                Err(e) => {
                                    tracing::error!("Failed to parse message: {}", e);
                                }
                            }
                        }
                        Some(Ok(tokio_tungstenite::tungstenite::Message::Close(_))) => {
                            tracing::info!("WebSocket connection closed by server");
                            break;
                        }
                        Some(Err(e)) => {
                            tracing::error!("WebSocket error: {}", e);
                            break;
                        }
                        None => {
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }

        *state.write() = ConnectionState::Disconnected;
    }

    /// Send a message
    pub async fn send(&self, message: WsMessage) -> Result<()> {
        if !self.is_connected() {
            return Err(StreamingError::NotConnected(
                "Client is not connected".to_string(),
            ));
        }

        self.message_tx
            .as_ref()
            .ok_or_else(|| StreamingError::NotConnected("Message channel not initialized".to_string()))?
            .send(message)
            .map_err(|e| StreamingError::ChannelSend(e.to_string()))?;

        Ok(())
    }

    /// Receive the next message
    pub async fn recv(&self) -> Result<WsMessage> {
        let mut rx_guard = self.received_rx.write();
        let rx = rx_guard
            .as_mut()
            .ok_or_else(|| StreamingError::NotConnected("Client is not connected".to_string()))?;

        rx.recv()
            .await
            .ok_or_else(|| StreamingError::ConnectionClosed("Message channel closed".to_string()))
    }

    /// Try to receive a message without blocking
    pub fn try_recv(&self) -> Result<Option<WsMessage>> {
        let mut rx_guard = self.received_rx.write();
        let rx = rx_guard
            .as_mut()
            .ok_or_else(|| StreamingError::NotConnected("Client is not connected".to_string()))?;

        match rx.try_recv() {
            Ok(msg) => Ok(Some(msg)),
            Err(mpsc::error::TryRecvError::Empty) => Ok(None),
            Err(mpsc::error::TryRecvError::Disconnected) => {
                Err(StreamingError::ConnectionClosed("Message channel closed".to_string()))
            }
        }
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.state.read().is_active()
    }

    /// Get connection state
    pub fn state(&self) -> ConnectionState {
        *self.state.read()
    }

    /// Disconnect from the server
    pub async fn disconnect(&mut self) -> Result<()> {
        self.message_tx = None;
        *self.received_rx.write() = None;
        *self.state.write() = ConnectionState::Disconnected;
        Ok(())
    }

    /// Reconnect with exponential backoff
    pub async fn reconnect(&mut self) -> Result<()> {
        let mut attempts = 0;
        let max_attempts = self.reconnect_config.max_attempts;
        let mut delay = self.reconnect_config.initial_delay;

        *self.state.write() = ConnectionState::Reconnecting;

        while attempts < max_attempts {
            attempts += 1;

            tracing::info!(
                "Reconnect attempt {}/{} to {}",
                attempts,
                max_attempts,
                self.url
            );

            match self.connect().await {
                Ok(_) => {
                    tracing::info!("Reconnected successfully");
                    return Ok(());
                }
                Err(e) => {
                    tracing::warn!("Reconnect attempt {} failed: {}", attempts, e);

                    if attempts < max_attempts {
                        tokio::time::sleep(delay).await;
                        delay = std::cmp::min(
                            delay * 2,
                            self.reconnect_config.max_delay,
                        );
                    }
                }
            }
        }

        *self.state.write() = ConnectionState::Failed;

        Err(StreamingError::ReconnectionFailed {
            attempts,
            reason: format!("Failed to reconnect after {} attempts", attempts),
        })
    }
}

/// Reconnect configuration
#[derive(Debug, Clone)]
pub struct ReconnectConfig {
    /// Maximum reconnection attempts
    pub max_attempts: usize,
    /// Initial delay between attempts
    pub initial_delay: Duration,
    /// Maximum delay between attempts
    pub max_delay: Duration,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            max_attempts: 10,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
        }
    }
}

impl ReconnectConfig {
    /// Create a new reconnect configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum attempts
    pub fn with_max_attempts(mut self, max_attempts: usize) -> Self {
        self.max_attempts = max_attempts;
        self
    }

    /// Set initial delay
    pub fn with_initial_delay(mut self, delay: Duration) -> Self {
        self.initial_delay = delay;
        self
    }

    /// Set maximum delay
    pub fn with_max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = WsClient::new("ws://localhost:8080");
        assert_eq!(client.state(), ConnectionState::Disconnected);
        assert!(!client.is_connected());
    }

    #[test]
    fn test_reconnect_config() {
        let config = ReconnectConfig::new()
            .with_max_attempts(5)
            .with_initial_delay(Duration::from_millis(50))
            .with_max_delay(Duration::from_secs(10));

        assert_eq!(config.max_attempts, 5);
        assert_eq!(config.initial_delay, Duration::from_millis(50));
        assert_eq!(config.max_delay, Duration::from_secs(10));
    }
}
