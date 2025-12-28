//! Static node configuration discovery.

use crate::error::Result;
use crate::node::NodeId;
use std::net::SocketAddr;
use tracing::{debug, info};

/// Static configuration-based discovery.
pub struct StaticDiscovery {
    /// Configured seed nodes
    seed_nodes: Vec<SocketAddr>,
}

impl StaticDiscovery {
    /// Create a new static discovery.
    pub fn new(seed_nodes: Vec<SocketAddr>) -> Self {
        info!("Static discovery initialized with {} seed nodes", seed_nodes.len());
        Self { seed_nodes }
    }

    /// Discover nodes from static configuration.
    pub async fn discover(&self) -> Result<Vec<NodeId>> {
        debug!("Discovering nodes from static configuration");

        let mut nodes = Vec::new();
        for addr in &self.seed_nodes {
            // Create node ID from address
            let node_id = NodeId::new(*addr);
            nodes.push(node_id);
        }

        info!("Discovered {} nodes from static configuration", nodes.len());
        Ok(nodes)
    }

    /// Add a seed node.
    pub fn add_seed(&mut self, addr: SocketAddr) {
        if !self.seed_nodes.contains(&addr) {
            self.seed_nodes.push(addr);
            debug!("Added seed node: {}", addr);
        }
    }

    /// Remove a seed node.
    pub fn remove_seed(&mut self, addr: &SocketAddr) -> bool {
        if let Some(pos) = self.seed_nodes.iter().position(|a| a == addr) {
            self.seed_nodes.remove(pos);
            debug!("Removed seed node: {}", addr);
            true
        } else {
            false
        }
    }

    /// Get all seed nodes.
    pub fn seed_nodes(&self) -> &[SocketAddr] {
        &self.seed_nodes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_static_discovery() {
        let seeds = vec![
            "127.0.0.1:7946".parse().unwrap(),
            "127.0.0.1:7947".parse().unwrap(),
        ];

        let discovery = StaticDiscovery::new(seeds);
        let nodes = discovery.discover().await.unwrap();

        assert_eq!(nodes.len(), 2);
    }
}
