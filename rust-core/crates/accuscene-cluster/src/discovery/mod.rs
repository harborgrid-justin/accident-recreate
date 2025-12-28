//! Node discovery mechanisms.

pub mod broadcast;
pub mod static_config;

pub use broadcast::BroadcastDiscovery;
pub use static_config::StaticDiscovery;

use crate::config::DiscoveryConfig;
use crate::error::Result;
use crate::node::NodeId;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// Unified discovery service.
pub struct DiscoveryService {
    /// Static discovery
    static_discovery: Option<StaticDiscovery>,

    /// Broadcast discovery
    broadcast_discovery: Option<Arc<RwLock<BroadcastDiscovery>>>,
}

impl DiscoveryService {
    /// Create a new discovery service.
    pub fn new(local_node: NodeId, cluster_name: String, config: DiscoveryConfig) -> Self {
        let static_discovery = if !config.seed_nodes.is_empty() {
            Some(StaticDiscovery::new(config.seed_nodes.clone()))
        } else {
            None
        };

        let broadcast_discovery = if config.enable_broadcast {
            Some(Arc::new(RwLock::new(BroadcastDiscovery::new(
                local_node,
                cluster_name,
                config.broadcast_port,
                config.broadcast_interval,
            ))))
        } else {
            None
        };

        Self {
            static_discovery,
            broadcast_discovery,
        }
    }

    /// Start discovery services.
    pub async fn start(&self) -> Result<()> {
        info!("Starting discovery services");

        if let Some(ref broadcast) = self.broadcast_discovery {
            broadcast.read().await.start().await?;
        }

        Ok(())
    }

    /// Stop discovery services.
    pub async fn stop(&self) {
        info!("Stopping discovery services");

        if let Some(ref broadcast) = self.broadcast_discovery {
            broadcast.read().await.stop().await;
        }
    }

    /// Discover all nodes.
    pub async fn discover_all(&self) -> Result<Vec<NodeId>> {
        let mut all_nodes = Vec::new();

        // Get nodes from static configuration
        if let Some(ref static_disc) = self.static_discovery {
            let nodes = static_disc.discover().await?;
            all_nodes.extend(nodes);
        }

        // Get nodes from broadcast discovery
        if let Some(ref broadcast) = self.broadcast_discovery {
            let nodes = broadcast.read().await.discovered_nodes().await;
            all_nodes.extend(nodes);
        }

        // Deduplicate by node ID
        all_nodes.sort_by_key(|n| n.id);
        all_nodes.dedup_by_key(|n| n.id);

        Ok(all_nodes)
    }
}
