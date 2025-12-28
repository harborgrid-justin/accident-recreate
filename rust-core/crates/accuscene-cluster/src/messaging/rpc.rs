//! RPC framework for inter-node communication.

use crate::error::{ClusterError, Result};
use crate::messaging::protocol::{Message, MessageType};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::oneshot;
use uuid::Uuid;

/// RPC request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest {
    /// Method name
    pub method: String,

    /// Request parameters
    pub params: Vec<u8>,

    /// Request timeout
    pub timeout_ms: u64,
}

impl RpcRequest {
    /// Create a new RPC request.
    pub fn new<T: Serialize>(method: impl Into<String>, params: &T) -> Result<Self> {
        Ok(Self {
            method: method.into(),
            params: bincode::serialize(params)?,
            timeout_ms: 5000,
        })
    }

    /// Set timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout_ms = timeout.as_millis() as u64;
        self
    }

    /// Decode parameters.
    pub fn decode_params<T: for<'de> Deserialize<'de>>(&self) -> Result<T> {
        bincode::deserialize(&self.params).map_err(Into::into)
    }
}

/// RPC response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse {
    /// Success flag
    pub success: bool,

    /// Response data (if successful)
    pub data: Option<Vec<u8>>,

    /// Error message (if failed)
    pub error: Option<String>,
}

impl RpcResponse {
    /// Create a successful response.
    pub fn success<T: Serialize>(data: &T) -> Result<Self> {
        Ok(Self {
            success: true,
            data: Some(bincode::serialize(data)?),
            error: None,
        })
    }

    /// Create an error response.
    pub fn error(error: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error.into()),
        }
    }

    /// Decode response data.
    pub fn decode_data<T: for<'de> Deserialize<'de>>(&self) -> Result<T> {
        if !self.success {
            return Err(ClusterError::Network(
                self.error.clone().unwrap_or_else(|| "Unknown error".to_string()),
            ));
        }

        if let Some(ref data) = self.data {
            bincode::deserialize(data).map_err(Into::into)
        } else {
            Err(ClusterError::InvalidMessage("No data in response".to_string()))
        }
    }
}

/// RPC handler function type.
pub type RpcHandler = Arc<
    dyn Fn(RpcRequest) -> Result<RpcResponse> + Send + Sync,
>;

/// RPC server for handling incoming requests.
pub struct RpcServer {
    /// Local node ID
    local_id: Uuid,

    /// Registered method handlers
    handlers: Arc<RwLock<HashMap<String, RpcHandler>>>,
}

impl RpcServer {
    /// Create a new RPC server.
    pub fn new(local_id: Uuid) -> Self {
        Self {
            local_id,
            handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a method handler.
    pub fn register<F>(&self, method: impl Into<String>, handler: F)
    where
        F: Fn(RpcRequest) -> Result<RpcResponse> + Send + Sync + 'static,
    {
        let mut handlers = self.handlers.write();
        handlers.insert(method.into(), Arc::new(handler));
    }

    /// Handle an RPC request.
    pub fn handle_request(&self, request: RpcRequest) -> RpcResponse {
        let handlers = self.handlers.read();

        if let Some(handler) = handlers.get(&request.method) {
            match handler(request) {
                Ok(response) => response,
                Err(e) => RpcResponse::error(e.to_string()),
            }
        } else {
            RpcResponse::error(format!("Method not found: {}", request.method))
        }
    }

    /// Handle incoming message.
    pub fn handle_message(&self, message: Message) -> Result<Message> {
        if message.header.message_type != MessageType::Request {
            return Err(ClusterError::InvalidMessage(
                "Expected request message".to_string(),
            ));
        }

        let request: RpcRequest = message.decode_payload()?;
        let response = self.handle_request(request);

        Message::new(MessageType::Response, self.local_id, &response)
    }
}

/// Pending RPC call.
struct PendingCall {
    sender: oneshot::Sender<Result<RpcResponse>>,
}

/// RPC client for making requests.
pub struct RpcClient {
    /// Local node ID
    local_id: Uuid,

    /// Pending calls
    pending: Arc<RwLock<HashMap<Uuid, PendingCall>>>,

    /// Default timeout
    default_timeout: Duration,
}

impl RpcClient {
    /// Create a new RPC client.
    pub fn new(local_id: Uuid) -> Self {
        Self {
            local_id,
            pending: Arc::new(RwLock::new(HashMap::new())),
            default_timeout: Duration::from_secs(5),
        }
    }

    /// Set default timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.default_timeout = timeout;
        self
    }

    /// Make an RPC call.
    pub async fn call<T, R>(
        &self,
        target: Uuid,
        method: impl Into<String>,
        params: &T,
    ) -> Result<R>
    where
        T: Serialize,
        R: for<'de> Deserialize<'de>,
    {
        let request = RpcRequest::new(method, params)?.with_timeout(self.default_timeout);

        let message = Message::new(MessageType::Request, self.local_id, &request)?;
        let message_id = message.header.message_id;

        // Create pending call
        let (tx, rx) = oneshot::channel();
        self.pending.write().insert(
            message_id,
            PendingCall { sender: tx },
        );

        // In a real implementation, send the message to the target node
        // For now, we'll simulate an error
        drop(message); // Prevent unused variable warning

        // Wait for response with timeout
        match tokio::time::timeout(self.default_timeout, rx).await {
            Ok(Ok(Ok(response))) => response.decode_data(),
            Ok(Ok(Err(e))) => Err(e),
            Ok(Err(_)) => Err(ClusterError::Network("Channel closed".to_string())),
            Err(_) => Err(ClusterError::Timeout("RPC call timed out".to_string())),
        }
    }

    /// Handle response message.
    pub fn handle_response(&self, message: Message) -> Result<()> {
        if message.header.message_type != MessageType::Response {
            return Err(ClusterError::InvalidMessage(
                "Expected response message".to_string(),
            ));
        }

        let response: RpcResponse = message.decode_payload()?;
        let message_id = message.header.message_id;

        let mut pending = self.pending.write();
        if let Some(call) = pending.remove(&message_id) {
            let _ = call.sender.send(Ok(response));
        }

        Ok(())
    }

    /// Cancel a pending call.
    pub fn cancel(&self, message_id: &Uuid) -> bool {
        self.pending.write().remove(message_id).is_some()
    }

    /// Get pending call count.
    pub fn pending_count(&self) -> usize {
        self.pending.read().len()
    }
}

/// RPC service combining server and client.
pub struct RpcService {
    /// RPC server
    pub server: Arc<RpcServer>,

    /// RPC client
    pub client: Arc<RpcClient>,
}

impl RpcService {
    /// Create a new RPC service.
    pub fn new(local_id: Uuid) -> Self {
        Self {
            server: Arc::new(RpcServer::new(local_id)),
            client: Arc::new(RpcClient::new(local_id)),
        }
    }

    /// Register a method handler.
    pub fn register<F>(&self, method: impl Into<String>, handler: F)
    where
        F: Fn(RpcRequest) -> Result<RpcResponse> + Send + Sync + 'static,
    {
        self.server.register(method, handler);
    }

    /// Make an RPC call.
    pub async fn call<T, R>(
        &self,
        target: Uuid,
        method: impl Into<String>,
        params: &T,
    ) -> Result<R>
    where
        T: Serialize,
        R: for<'de> Deserialize<'de>,
    {
        self.client.call(target, method, params).await
    }

    /// Handle incoming message.
    pub fn handle_message(&self, message: Message) -> Result<Option<Message>> {
        match message.header.message_type {
            MessageType::Request => {
                let response = self.server.handle_message(message)?;
                Ok(Some(response))
            }
            MessageType::Response => {
                self.client.handle_response(message)?;
                Ok(None)
            }
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestParams {
        value: i32,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestResult {
        result: i32,
    }

    #[test]
    fn test_rpc_request() {
        let params = TestParams { value: 42 };
        let request = RpcRequest::new("test_method", &params).unwrap();

        assert_eq!(request.method, "test_method");

        let decoded: TestParams = request.decode_params().unwrap();
        assert_eq!(decoded, params);
    }

    #[test]
    fn test_rpc_response() {
        let data = TestResult { result: 100 };
        let response = RpcResponse::success(&data).unwrap();

        assert!(response.success);

        let decoded: TestResult = response.decode_data().unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_rpc_server() {
        let local_id = Uuid::new_v4();
        let server = RpcServer::new(local_id);

        server.register("double", |req: RpcRequest| {
            let params: TestParams = req.decode_params()?;
            let result = TestResult {
                result: params.value * 2,
            };
            RpcResponse::success(&result)
        });

        let request = RpcRequest::new("double", &TestParams { value: 21 }).unwrap();
        let response = server.handle_request(request);

        assert!(response.success);
        let result: TestResult = response.decode_data().unwrap();
        assert_eq!(result.result, 42);
    }
}
