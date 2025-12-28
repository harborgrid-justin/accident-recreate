//! Cluster configuration settings.

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::time::Duration;

/// Comprehensive cluster configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterConfig {
    /// Node bind address
    pub bind_addr: SocketAddr,

    /// Cluster name (for isolation)
    pub cluster_name: String,

    /// Discovery configuration
    pub discovery: DiscoveryConfig,

    /// Membership configuration
    pub membership: MembershipConfig,

    /// Consensus configuration
    pub consensus: ConsensusConfig,

    /// Replication configuration
    pub replication: ReplicationConfig,

    /// Health check configuration
    pub health: HealthConfig,
}

impl Default for ClusterConfig {
    fn default() -> Self {
        Self {
            bind_addr: "127.0.0.1:7946".parse().unwrap(),
            cluster_name: "accuscene-cluster".to_string(),
            discovery: DiscoveryConfig::default(),
            membership: MembershipConfig::default(),
            consensus: ConsensusConfig::default(),
            replication: ReplicationConfig::default(),
            health: HealthConfig::default(),
        }
    }
}

/// Node discovery configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    /// Enable UDP broadcast discovery
    pub enable_broadcast: bool,

    /// Broadcast port
    pub broadcast_port: u16,

    /// Broadcast interval
    pub broadcast_interval: Duration,

    /// Static seed nodes
    pub seed_nodes: Vec<SocketAddr>,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            enable_broadcast: true,
            broadcast_port: 7947,
            broadcast_interval: Duration::from_secs(30),
            seed_nodes: Vec::new(),
        }
    }
}

/// Cluster membership configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MembershipConfig {
    /// Gossip protocol interval
    pub gossip_interval: Duration,

    /// Gossip fanout (number of nodes to gossip with)
    pub gossip_fanout: usize,

    /// Suspicion timeout before marking node as failed
    pub suspicion_timeout: Duration,

    /// Number of indirect probes for failure detection
    pub indirect_probes: usize,

    /// Maximum transmission limit before declaring node dead
    pub max_transmissions: usize,
}

impl Default for MembershipConfig {
    fn default() -> Self {
        Self {
            gossip_interval: Duration::from_millis(500),
            gossip_fanout: 3,
            suspicion_timeout: Duration::from_secs(5),
            indirect_probes: 3,
            max_transmissions: 15,
        }
    }
}

/// Consensus protocol configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    /// Election timeout range (min, max)
    pub election_timeout: (Duration, Duration),

    /// Heartbeat interval
    pub heartbeat_interval: Duration,

    /// Leader lease duration
    pub leader_lease: Duration,

    /// Enable pre-vote to prevent disruptive elections
    pub enable_prevote: bool,

    /// Maximum log entries per append
    pub max_log_entries: usize,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            election_timeout: (Duration::from_millis(150), Duration::from_millis(300)),
            heartbeat_interval: Duration::from_millis(50),
            leader_lease: Duration::from_millis(500),
            enable_prevote: true,
            max_log_entries: 1000,
        }
    }
}

/// Data replication configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationConfig {
    /// Replication factor
    pub replication_factor: usize,

    /// Write consistency level
    pub write_consistency: ConsistencyLevel,

    /// Read consistency level
    pub read_consistency: ConsistencyLevel,

    /// Enable anti-entropy for repair
    pub enable_anti_entropy: bool,

    /// Anti-entropy interval
    pub anti_entropy_interval: Duration,

    /// Conflict resolution strategy
    pub conflict_resolution: ConflictResolutionStrategy,
}

impl Default for ReplicationConfig {
    fn default() -> Self {
        Self {
            replication_factor: 3,
            write_consistency: ConsistencyLevel::Quorum,
            read_consistency: ConsistencyLevel::Quorum,
            enable_anti_entropy: true,
            anti_entropy_interval: Duration::from_secs(60),
            conflict_resolution: ConflictResolutionStrategy::LastWriteWins,
        }
    }
}

/// Health check configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthConfig {
    /// Health check interval
    pub check_interval: Duration,

    /// Health check timeout
    pub check_timeout: Duration,

    /// Failure threshold before marking unhealthy
    pub failure_threshold: u32,

    /// Success threshold before marking healthy
    pub success_threshold: u32,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(5),
            check_timeout: Duration::from_secs(2),
            failure_threshold: 3,
            success_threshold: 2,
        }
    }
}

/// Consistency level for read/write operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsistencyLevel {
    /// Any single node
    One,
    /// Quorum (majority) of nodes
    Quorum,
    /// All nodes
    All,
}

impl ConsistencyLevel {
    /// Calculate required nodes for this consistency level.
    pub fn required_nodes(&self, total_nodes: usize) -> usize {
        match self {
            ConsistencyLevel::One => 1,
            ConsistencyLevel::Quorum => (total_nodes / 2) + 1,
            ConsistencyLevel::All => total_nodes,
        }
    }
}

/// Conflict resolution strategies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictResolutionStrategy {
    /// Last write wins based on timestamp
    LastWriteWins,
    /// Vector clock based resolution
    VectorClock,
    /// Custom application-level resolution
    Custom,
}
