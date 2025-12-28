//! WebSocket message handlers.

use crate::error::{Result, StreamingError};
use crate::websocket::protocol::WsMessage;
use async_trait::async_trait;
use std::sync::Arc;

/// Message handler trait
#[async_trait]
pub trait MessageHandler: Send + Sync {
    /// Handle an incoming message
    async fn handle(&self, message: WsMessage) -> Result<Option<WsMessage>>;

    /// Handle connection established
    async fn on_connect(&self) -> Result<()> {
        Ok(())
    }

    /// Handle connection closed
    async fn on_disconnect(&self) -> Result<()> {
        Ok(())
    }

    /// Handle error
    async fn on_error(&self, error: &StreamingError) -> Result<()> {
        tracing::error!("WebSocket error: {}", error);
        Ok(())
    }
}

/// Default message handler
pub struct DefaultMessageHandler;

impl DefaultMessageHandler {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultMessageHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MessageHandler for DefaultMessageHandler {
    async fn handle(&self, message: WsMessage) -> Result<Option<WsMessage>> {
        match message {
            WsMessage::Ping { timestamp } => {
                // Respond to ping with pong
                Ok(Some(WsMessage::pong(timestamp)))
            }
            WsMessage::Pong { .. } => {
                // Handle pong (heartbeat confirmation)
                Ok(None)
            }
            _ => {
                // Default: no response
                Ok(None)
            }
        }
    }
}

/// Composite message handler that chains multiple handlers
pub struct CompositeHandler {
    handlers: Vec<Arc<dyn MessageHandler>>,
}

impl CompositeHandler {
    /// Create a new composite handler
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    /// Add a handler
    pub fn add_handler(mut self, handler: Arc<dyn MessageHandler>) -> Self {
        self.handlers.push(handler);
        self
    }

    /// Add multiple handlers
    pub fn add_handlers(mut self, handlers: Vec<Arc<dyn MessageHandler>>) -> Self {
        self.handlers.extend(handlers);
        self
    }
}

impl Default for CompositeHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MessageHandler for CompositeHandler {
    async fn handle(&self, message: WsMessage) -> Result<Option<WsMessage>> {
        for handler in &self.handlers {
            if let Some(response) = handler.handle(message.clone()).await? {
                return Ok(Some(response));
            }
        }
        Ok(None)
    }

    async fn on_connect(&self) -> Result<()> {
        for handler in &self.handlers {
            handler.on_connect().await?;
        }
        Ok(())
    }

    async fn on_disconnect(&self) -> Result<()> {
        for handler in &self.handlers {
            handler.on_disconnect().await?;
        }
        Ok(())
    }

    async fn on_error(&self, error: &StreamingError) -> Result<()> {
        for handler in &self.handlers {
            handler.on_error(error).await?;
        }
        Ok(())
    }
}

/// Handler builder
pub struct HandlerBuilder {
    handlers: Vec<Arc<dyn MessageHandler>>,
}

impl HandlerBuilder {
    /// Create a new handler builder
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    /// Add a handler
    pub fn with_handler(mut self, handler: Arc<dyn MessageHandler>) -> Self {
        self.handlers.push(handler);
        self
    }

    /// Build the composite handler
    pub fn build(self) -> CompositeHandler {
        CompositeHandler {
            handlers: self.handlers,
        }
    }
}

impl Default for HandlerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_default_handler_ping_pong() {
        let handler = DefaultMessageHandler::new();
        let ping = WsMessage::ping();

        let response = handler.handle(ping).await.unwrap();
        assert!(response.is_some());

        if let Some(WsMessage::Pong { .. }) = response {
            // Success
        } else {
            panic!("Expected Pong response");
        }
    }

    #[tokio::test]
    async fn test_composite_handler() {
        let handler1 = Arc::new(DefaultMessageHandler::new());
        let handler2 = Arc::new(DefaultMessageHandler::new());

        let composite = CompositeHandler::new()
            .add_handler(handler1)
            .add_handler(handler2);

        let ping = WsMessage::ping();
        let response = composite.handle(ping).await.unwrap();
        assert!(response.is_some());
    }
}
