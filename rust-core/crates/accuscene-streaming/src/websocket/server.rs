//! WebSocket server implementation.

use crate::error::{Result, StreamingError};
use crate::websocket::handler::MessageHandler;
use crate::websocket::protocol::{ConnectionState, WsMessage};
use futures_util::{SinkExt, StreamExt};
use parking_lot::RwLock;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Message, WebSocketStream};

/// WebSocket connection
pub struct WsConnection {
    /// Connection ID
    pub id: String,
    /// Remote address
    pub addr: SocketAddr,
    /// WebSocket stream
    stream: WebSocketStream<TcpStream>,
    /// Connection state
    state: Arc<RwLock<ConnectionState>>,
    /// Message handler
    handler: Arc<dyn MessageHandler>,
}

impl WsConnection {
    /// Create a new WebSocket connection
    pub fn new(
        id: String,
        addr: SocketAddr,
        stream: WebSocketStream<TcpStream>,
        handler: Arc<dyn MessageHandler>,
    ) -> Self {
        Self {
            id,
            addr,
            stream,
            state: Arc::new(RwLock::new(ConnectionState::Connected)),
            handler,
        }
    }

    /// Get connection state
    pub fn state(&self) -> ConnectionState {
        *self.state.read()
    }

    /// Send a message
    pub async fn send(&mut self, message: WsMessage) -> Result<()> {
        let json = message
            .to_json()
            .map_err(|e| StreamingError::Serialization(e))?;

        self.stream
            .send(Message::Text(json))
            .await
            .map_err(|e| StreamingError::WebSocketConnection(e.to_string()))?;

        Ok(())
    }

    /// Receive and handle messages
    pub async fn handle_messages(&mut self) -> Result<()> {
        // Notify handler of connection
        self.handler.on_connect().await?;

        while let Some(message) = self.stream.next().await {
            match message {
                Ok(Message::Text(text)) => {
                    match WsMessage::from_json(&text) {
                        Ok(ws_msg) => {
                            // Handle the message
                            match self.handler.handle(ws_msg).await {
                                Ok(Some(response)) => {
                                    // Send response if provided
                                    self.send(response).await?;
                                }
                                Ok(None) => {
                                    // No response needed
                                }
                                Err(e) => {
                                    self.handler.on_error(&e).await?;
                                    // Send error message to client
                                    let error_msg = WsMessage::error(
                                        e.category(),
                                        e.to_string(),
                                    );
                                    self.send(error_msg).await.ok();
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to parse WebSocket message: {}", e);
                            let error_msg = WsMessage::error(
                                "parse_error",
                                format!("Invalid message format: {}", e),
                            );
                            self.send(error_msg).await.ok();
                        }
                    }
                }
                Ok(Message::Binary(_)) => {
                    // Binary messages not supported yet
                    let error_msg = WsMessage::error(
                        "unsupported",
                        "Binary messages are not supported",
                    );
                    self.send(error_msg).await.ok();
                }
                Ok(Message::Ping(payload)) => {
                    // Respond to ping
                    self.stream.send(Message::Pong(payload)).await.ok();
                }
                Ok(Message::Pong(_)) => {
                    // Received pong
                }
                Ok(Message::Close(_)) => {
                    // Connection closed by client
                    break;
                }
                Err(e) => {
                    tracing::error!("WebSocket error: {}", e);
                    let error = StreamingError::WebSocketConnection(e.to_string());
                    self.handler.on_error(&error).await?;
                    break;
                }
                _ => {}
            }
        }

        // Update state
        *self.state.write() = ConnectionState::Disconnected;

        // Notify handler of disconnection
        self.handler.on_disconnect().await?;

        Ok(())
    }

    /// Close the connection
    pub async fn close(&mut self) -> Result<()> {
        self.stream
            .close(None)
            .await
            .map_err(|e| StreamingError::WebSocketConnection(e.to_string()))?;

        *self.state.write() = ConnectionState::Disconnected;

        Ok(())
    }
}

/// WebSocket server
pub struct WsServer {
    /// Server address
    addr: SocketAddr,
    /// Message handler factory
    handler_factory: Arc<dyn Fn() -> Arc<dyn MessageHandler> + Send + Sync>,
}

impl WsServer {
    /// Create a new WebSocket server
    pub fn new<F>(addr: SocketAddr, handler_factory: F) -> Self
    where
        F: Fn() -> Arc<dyn MessageHandler> + Send + Sync + 'static,
    {
        Self {
            addr,
            handler_factory: Arc::new(handler_factory),
        }
    }

    /// Start the server
    pub async fn start(self) -> Result<()> {
        let listener = TcpListener::bind(self.addr)
            .await
            .map_err(|e| StreamingError::Io(e))?;

        tracing::info!("WebSocket server listening on {}", self.addr);

        while let Ok((stream, addr)) = listener.accept().await {
            let handler = (self.handler_factory)();
            let connection_id = uuid::Uuid::new_v4().to_string();

            tokio::spawn(async move {
                match accept_async(stream).await {
                    Ok(ws_stream) => {
                        let mut connection = WsConnection::new(
                            connection_id.clone(),
                            addr,
                            ws_stream,
                            handler,
                        );

                        tracing::info!(
                            "New WebSocket connection: {} from {}",
                            connection_id,
                            addr
                        );

                        if let Err(e) = connection.handle_messages().await {
                            tracing::error!(
                                "Error handling connection {}: {}",
                                connection_id,
                                e
                            );
                        }

                        tracing::info!("WebSocket connection closed: {}", connection_id);
                    }
                    Err(e) => {
                        tracing::error!("Failed to accept WebSocket connection: {}", e);
                    }
                }
            });
        }

        Ok(())
    }
}

/// WebSocket server builder
pub struct WsServerBuilder<F>
where
    F: Fn() -> Arc<dyn MessageHandler> + Send + Sync + 'static,
{
    addr: Option<SocketAddr>,
    handler_factory: Option<F>,
}

impl<F> WsServerBuilder<F>
where
    F: Fn() -> Arc<dyn MessageHandler> + Send + Sync + 'static,
{
    /// Create a new server builder
    pub fn new() -> Self {
        Self {
            addr: None,
            handler_factory: None,
        }
    }

    /// Set the server address
    pub fn with_addr(mut self, addr: SocketAddr) -> Self {
        self.addr = Some(addr);
        self
    }

    /// Set the handler factory
    pub fn with_handler_factory(mut self, factory: F) -> Self {
        self.handler_factory = Some(factory);
        self
    }

    /// Build the server
    pub fn build(self) -> Result<WsServer> {
        let addr = self
            .addr
            .ok_or_else(|| StreamingError::Configuration("Server address not set".to_string()))?;

        let handler_factory = self.handler_factory.ok_or_else(|| {
            StreamingError::Configuration("Handler factory not set".to_string())
        })?;

        Ok(WsServer::new(addr, handler_factory))
    }
}
