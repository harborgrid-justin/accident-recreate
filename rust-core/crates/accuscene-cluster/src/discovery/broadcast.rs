//! UDP broadcast-based node discovery.

use crate::error::Result;
use crate::node::NodeId;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::sync::RwLock;
use tokio::time;
use tracing::{debug, error, info, warn};

/// Broadcast discovery message.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DiscoveryMessage {
    /// Node identity
    node_id: NodeId,

    /// Cluster name
    cluster_name: String,

    /// Protocol version
    protocol_version: u32,

    /// Timestamp
    timestamp: i64,
}

/// UDP broadcast discovery.
pub struct BroadcastDiscovery {
    /// Local node ID
    local_node: NodeId,

    /// Cluster name
    cluster_name: String,

    /// Broadcast port
    port: u16,

    /// Broadcast interval
    interval: Duration,

    /// Discovered nodes
    discovered: Arc<RwLock<Vec<NodeId>>>,

    /// Running flag
    running: Arc<RwLock<bool>>,
}

impl BroadcastDiscovery {
    /// Create a new broadcast discovery.
    pub fn new(
        local_node: NodeId,
        cluster_name: String,
        port: u16,
        interval: Duration,
    ) -> Self {
        Self {
            local_node,
            cluster_name,
            port,
            interval,
            discovered: Arc::new(RwLock::new(Vec::new())),
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start discovery service.
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if *running {
            return Ok(());
        }
        *running = true;
        drop(running);

        info!("Starting broadcast discovery on port {}", self.port);

        // Start broadcaster
        let broadcaster = self.clone_for_task();
        tokio::spawn(async move {
            if let Err(e) = broadcaster.broadcast_loop().await {
                error!("Broadcast loop error: {}", e);
            }
        });

        // Start listener
        let listener = self.clone_for_task();
        tokio::spawn(async move {
            if let Err(e) = listener.listen_loop().await {
                error!("Listen loop error: {}", e);
            }
        });

        Ok(())
    }

    /// Stop discovery service.
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        info!("Stopped broadcast discovery");
    }

    /// Get discovered nodes.
    pub async fn discovered_nodes(&self) -> Vec<NodeId> {
        self.discovered.read().await.clone()
    }

    /// Broadcast loop.
    async fn broadcast_loop(&self) -> Result<()> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.set_broadcast(true)?;

        let broadcast_addr = SocketAddr::new(
            IpAddr::V4(Ipv4Addr::BROADCAST),
            self.port,
        );

        let message = DiscoveryMessage {
            node_id: self.local_node.clone(),
            cluster_name: self.cluster_name.clone(),
            protocol_version: 1,
            timestamp: chrono::Utc::now().timestamp(),
        };

        let mut interval = time::interval(self.interval);

        loop {
            interval.tick().await;

            let running = self.running.read().await;
            if !*running {
                break;
            }
            drop(running);

            // Update timestamp
            let mut msg = message.clone();
            msg.timestamp = chrono::Utc::now().timestamp();

            // Serialize and broadcast
            match bincode::serialize(&msg) {
                Ok(data) => {
                    if let Err(e) = socket.send_to(&data, broadcast_addr).await {
                        warn!("Failed to broadcast: {}", e);
                    } else {
                        debug!("Broadcast discovery message");
                    }
                }
                Err(e) => {
                    warn!("Failed to serialize discovery message: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Listen loop.
    async fn listen_loop(&self) -> Result<()> {
        let socket = UdpSocket::bind(format!("0.0.0.0:{}", self.port)).await?;
        let mut buf = vec![0u8; 65536];

        info!("Listening for broadcasts on port {}", self.port);

        loop {
            let running = self.running.read().await;
            if !*running {
                break;
            }
            drop(running);

            match socket.recv_from(&mut buf).await {
                Ok((len, addr)) => {
                    if let Err(e) = self.handle_message(&buf[..len], addr).await {
                        debug!("Failed to handle discovery message: {}", e);
                    }
                }
                Err(e) => {
                    warn!("Failed to receive broadcast: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Handle received discovery message.
    async fn handle_message(&self, data: &[u8], _addr: SocketAddr) -> Result<()> {
        let message: DiscoveryMessage = bincode::deserialize(data)?;

        // Ignore own messages
        if message.node_id.id == self.local_node.id {
            return Ok(());
        }

        // Check cluster name
        if message.cluster_name != self.cluster_name {
            debug!(
                "Ignoring message from different cluster: {}",
                message.cluster_name
            );
            return Ok(());
        }

        // Check protocol version
        if message.protocol_version != 1 {
            warn!(
                "Ignoring message with unsupported protocol version: {}",
                message.protocol_version
            );
            return Ok(());
        }

        // Add to discovered nodes
        let mut discovered = self.discovered.write().await;
        if !discovered.iter().any(|n| n.id == message.node_id.id) {
            info!("Discovered new node via broadcast: {}", message.node_id.addr);
            discovered.push(message.node_id);
        }

        Ok(())
    }

    /// Clone for async task.
    fn clone_for_task(&self) -> Self {
        Self {
            local_node: self.local_node.clone(),
            cluster_name: self.cluster_name.clone(),
            port: self.port,
            interval: self.interval,
            discovered: Arc::clone(&self.discovered),
            running: Arc::clone(&self.running),
        }
    }
}

impl Drop for BroadcastDiscovery {
    fn drop(&mut self) {
        // Note: Cannot use async in Drop, handled by explicit stop()
    }
}
