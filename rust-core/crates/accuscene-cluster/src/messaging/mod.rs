//! Inter-node messaging infrastructure.

pub mod protocol;
pub mod rpc;

pub use protocol::{Message, MessageHeader, MessageType, PROTOCOL_VERSION};
pub use rpc::{RpcClient, RpcRequest, RpcResponse, RpcServer, RpcService};
