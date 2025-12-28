//! AccuScene Enterprise Distributed Clustering Support
//!
//! This crate provides comprehensive distributed clustering capabilities for AccuScene Enterprise,
//! including node discovery, membership management, consensus, replication, and failover.
//!
//! # Features
//!
//! - **Node Discovery**: Static configuration and UDP broadcast-based discovery
//! - **Membership Management**: SWIM-like gossip protocol for failure detection
//! - **Consensus**: Raft-lite consensus protocol with leader election
//! - **Data Replication**: Multi-strategy replication with conflict resolution
//! - **Partitioning**: Consistent hashing and range-based data partitioning
//! - **Failover**: Automatic failover with configurable strategies
//! - **Load Balancing**: Multiple strategies including round-robin and least connections
//! - **RPC**: Binary messaging protocol for inter-node communication
//!
//! # Example
//!
//! ```no_run
//! use accuscene_cluster::{ClusterCoordinator, ClusterConfig};
//! use accuscene_cluster::node::NodeId;
//! use std::net::SocketAddr;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create node identity
//! let addr: SocketAddr = "127.0.0.1:7946".parse()?;
//! let node = NodeId::new(addr).with_name("node-1");
//!
//! // Configure cluster
//! let config = ClusterConfig::default();
//!
//! // Create and start coordinator
//! let coordinator = ClusterCoordinator::new(node, config);
//! coordinator.start().await?;
//!
//! // Perform cluster operations
//! coordinator.write("key".to_string(), b"value".to_vec()).await?;
//! let value = coordinator.read("key").await?;
//!
//! // Stop coordinator
//! coordinator.stop().await?;
//! # Ok(())
//! # }
//! ```

pub mod config;
pub mod consensus;
pub mod coordinator;
pub mod discovery;
pub mod error;
pub mod failover;
pub mod load_balancing;
pub mod membership;
pub mod messaging;
pub mod node;
pub mod partitioning;
pub mod replication;

// Re-export primary types
pub use config::{ClusterConfig, ConsistencyLevel, ConflictResolutionStrategy};
pub use consensus::{ConsensusService, LeaderElection, LeaderInfo, LeaderState};
pub use coordinator::ClusterCoordinator;
pub use discovery::{BroadcastDiscovery, DiscoveryService, StaticDiscovery};
pub use error::{ClusterError, Result};
pub use failover::{FailoverConfig, FailoverEvent, FailoverManager, FailoverStrategy};
pub use load_balancing::{LoadBalancer, LoadBalancingStrategy, NodeLoadMetrics};
pub use membership::{GossipMessage, MembershipService, MembershipView};
pub use messaging::{Message, MessageType, RpcService};
pub use node::{
    HealthStatus, Node, NodeCapabilities, NodeId, NodeRegistry, NodeState, NodeVersion,
};
pub use partitioning::{
    ConsistentHashRing, PartitioningService, PartitioningStrategy, RangePartition,
};
pub use replication::{
    ConflictResolution, ConflictResolver, ReplicationService, StateSnapshot, VectorClock,
};

/// Cluster version information.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Protocol version.
pub const PROTOCOL_VERSION: u8 = messaging::PROTOCOL_VERSION;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_protocol_version() {
        assert_eq!(PROTOCOL_VERSION, 1);
    }
}
