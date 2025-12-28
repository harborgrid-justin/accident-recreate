//! Node identity and metadata.

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use uuid::Uuid;

/// Unique node identifier and metadata.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId {
    /// Unique node UUID
    pub id: Uuid,

    /// Node address
    pub addr: SocketAddr,

    /// Node name (optional)
    pub name: Option<String>,

    /// Node datacenter/region
    pub datacenter: Option<String>,

    /// Node rack/zone
    pub rack: Option<String>,

    /// Node metadata tags
    pub tags: Vec<(String, String)>,
}

impl NodeId {
    /// Create a new node identity.
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            id: Uuid::new_v4(),
            addr,
            name: None,
            datacenter: None,
            rack: None,
            tags: Vec::new(),
        }
    }

    /// Create with name.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set datacenter.
    pub fn with_datacenter(mut self, datacenter: impl Into<String>) -> Self {
        self.datacenter = Some(datacenter.into());
        self
    }

    /// Set rack.
    pub fn with_rack(mut self, rack: impl Into<String>) -> Self {
        self.rack = Some(rack.into());
        self
    }

    /// Add metadata tag.
    pub fn with_tag(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.tags.push((key.into(), value.into()));
        self
    }

    /// Get tag value.
    pub fn get_tag(&self, key: &str) -> Option<&str> {
        self.tags
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }
}

/// Node capabilities and resources.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeCapabilities {
    /// CPU cores available
    pub cpu_cores: u32,

    /// Memory in MB
    pub memory_mb: u64,

    /// Disk space in MB
    pub disk_mb: u64,

    /// Maximum concurrent connections
    pub max_connections: u32,

    /// Supported features
    pub features: Vec<String>,
}

impl Default for NodeCapabilities {
    fn default() -> Self {
        Self {
            cpu_cores: num_cpus::get() as u32,
            memory_mb: 4096,
            disk_mb: 10240,
            max_connections: 1000,
            features: vec![
                "consensus".to_string(),
                "replication".to_string(),
                "partitioning".to_string(),
            ],
        }
    }
}

/// Node version information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeVersion {
    /// Software version
    pub version: String,

    /// Protocol version
    pub protocol_version: u32,

    /// Build timestamp
    pub build_timestamp: Option<String>,
}

impl Default for NodeVersion {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            protocol_version: 1,
            build_timestamp: Some(chrono::Utc::now().to_rfc3339()),
        }
    }
}

// Add num_cpus dependency helper
mod num_cpus {
    pub fn get() -> usize {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
    }
}
