//! Distributed coordination and orchestration.

use crate::config::ClusterConfig;
use crate::consensus::ConsensusService;
use crate::discovery::DiscoveryService;
use crate::error::Result;
use crate::failover::{FailoverConfig, FailoverManager};
use crate::load_balancing::{LoadBalancer, LoadBalancingStrategy};
use crate::membership::MembershipService;
use crate::messaging::RpcService;
use crate::node::{Node, NodeId, NodeRegistry, NodeState};
use crate::partitioning::PartitioningService;
use crate::replication::ReplicationService;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

/// Cluster coordinator managing all distributed components.
pub struct ClusterCoordinator {
    /// Local node identity
    local_node: NodeId,

    /// Cluster configuration
    config: ClusterConfig,

    /// Node registry
    registry: Arc<NodeRegistry>,

    /// Discovery service
    discovery: Arc<RwLock<DiscoveryService>>,

    /// Membership service
    membership: Arc<RwLock<MembershipService>>,

    /// Consensus service
    consensus: Arc<ConsensusService>,

    /// Replication service
    replication: Arc<ReplicationService>,

    /// Partitioning service
    partitioning: Arc<RwLock<PartitioningService>>,

    /// RPC service
    rpc: Arc<RpcService>,

    /// Load balancer
    load_balancer: Arc<LoadBalancer>,

    /// Failover manager
    failover: Arc<RwLock<FailoverManager>>,

    /// Running state
    running: Arc<RwLock<bool>>,
}

impl ClusterCoordinator {
    /// Create a new cluster coordinator.
    pub fn new(local_node: NodeId, config: ClusterConfig) -> Self {
        let local_id = local_node.id;

        // Initialize services
        let registry = Arc::new(NodeRegistry::new());

        let discovery = Arc::new(RwLock::new(DiscoveryService::new(
            local_node.clone(),
            config.cluster_name.clone(),
            config.discovery.clone(),
        )));

        let membership = Arc::new(RwLock::new(MembershipService::new(
            local_id,
            config.membership.clone(),
        )));

        let consensus = Arc::new(ConsensusService::new(local_id, config.consensus.clone()));

        let replication = Arc::new(ReplicationService::new(
            local_id,
            config.replication.clone(),
        ));

        let partitioning = Arc::new(RwLock::new(PartitioningService::with_consistent_hash(
            150,
            config.replication.replication_factor,
        )));

        let rpc = Arc::new(RpcService::new(local_id));

        let load_balancer = Arc::new(LoadBalancer::new(LoadBalancingStrategy::LeastConnections));

        let failover_config = FailoverConfig::default();
        let failover = Arc::new(RwLock::new(FailoverManager::new(
            local_id,
            failover_config,
            Arc::clone(consensus.leader_election()),
        )));

        Self {
            local_node,
            config,
            registry,
            discovery,
            membership,
            consensus,
            replication,
            partitioning,
            rpc,
            load_balancer,
            failover,
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start the cluster coordinator.
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if *running {
            return Ok(());
        }
        *running = true;
        drop(running);

        info!("Starting cluster coordinator for node {}", self.local_node.id);

        // Register local node
        let local = Node::new(self.local_node.clone());
        self.registry.upsert(local.clone());

        // Start discovery
        self.discovery.read().await.start().await?;
        info!("Discovery service started");

        // Discover initial nodes
        let discovered = self.discovery.read().await.discover_all().await?;
        info!("Discovered {} nodes", discovered.len());

        // Join membership
        self.membership.read().await.start().await?;
        self.membership.read().await.join(local).await?;
        info!("Membership service started");

        // Start failover
        self.failover.read().await.start().await?;
        info!("Failover manager started");

        // Add local node to partitioning
        self.partitioning.write().await.add_node(self.local_node.id);
        info!("Partitioning service initialized");

        // Register RPC handlers
        self.register_rpc_handlers();
        info!("RPC handlers registered");

        info!("Cluster coordinator started successfully");

        Ok(())
    }

    /// Stop the cluster coordinator.
    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if !*running {
            return Ok(());
        }
        *running = false;

        info!("Stopping cluster coordinator");

        // Leave membership
        self.membership.read().await.leave().await?;
        self.membership.read().await.stop().await;

        // Stop discovery
        self.discovery.read().await.stop().await;

        // Stop failover
        self.failover.read().await.stop().await;

        info!("Cluster coordinator stopped");

        Ok(())
    }

    /// Check if coordinator is running.
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    /// Get local node ID.
    pub fn local_id(&self) -> Uuid {
        self.local_node.id
    }

    /// Check if this node is the cluster leader.
    pub fn is_leader(&self) -> bool {
        self.consensus.is_leader()
    }

    /// Get current cluster leader.
    pub fn current_leader(&self) -> Option<Uuid> {
        self.consensus.current_leader()
    }

    /// Get cluster size.
    pub async fn cluster_size(&self) -> usize {
        self.membership.read().await.view().count()
    }

    /// Get active nodes.
    pub async fn active_nodes(&self) -> Vec<Uuid> {
        self.membership
            .read()
            .await
            .view()
            .active_members()
            .iter()
            .map(|m| m.node_id)
            .collect()
    }

    /// Write data with replication.
    pub async fn write(&self, key: String, value: Vec<u8>) -> Result<()> {
        // Determine target nodes using partitioning
        let nodes = self.partitioning.read().await.get_nodes(&key);

        // Write locally
        self.replication.write(key.clone(), value.clone()).await?;

        // Replicate to peer nodes
        let peers: Vec<_> = nodes
            .iter()
            .filter(|&&id| id != self.local_node.id)
            .copied()
            .collect();

        if !peers.is_empty() {
            self.replication.replicate_to_peers(&peers).await?;
        }

        Ok(())
    }

    /// Read data.
    pub async fn read(&self, key: &str) -> Result<Option<Vec<u8>>> {
        self.replication.read(key).await
    }

    /// Propose a consensus value.
    pub async fn propose(&self, data: Vec<u8>) -> Result<u64> {
        self.consensus.propose(data).await
    }

    /// Get node registry.
    pub fn registry(&self) -> &NodeRegistry {
        &self.registry
    }

    /// Get consensus service.
    pub fn consensus(&self) -> &ConsensusService {
        &self.consensus
    }

    /// Get replication service.
    pub fn replication(&self) -> &ReplicationService {
        &self.replication
    }

    /// Get RPC service.
    pub fn rpc(&self) -> &RpcService {
        &self.rpc
    }

    /// Get load balancer.
    pub fn load_balancer(&self) -> &LoadBalancer {
        &self.load_balancer
    }

    /// Register RPC handlers.
    fn register_rpc_handlers(&self) {
        use crate::messaging::{RpcRequest, RpcResponse};

        // Health check handler
        self.rpc.register("health", |_req: RpcRequest| {
            RpcResponse::success(&"healthy")
        });

        // Ping handler
        self.rpc.register("ping", |_req: RpcRequest| {
            RpcResponse::success(&"pong")
        });

        // Add more handlers as needed
    }

    /// Handle node join.
    pub async fn handle_node_join(&self, node: Node) -> Result<()> {
        info!("Node joining: {}", node.id.id);

        // Add to registry
        self.registry.upsert(node.clone());

        // Add to partitioning
        self.partitioning.write().await.add_node(node.id.id);

        // Add to load balancer
        self.load_balancer.add_node(node.id.id, 1);

        Ok(())
    }

    /// Handle node leave.
    pub async fn handle_node_leave(&self, node_id: Uuid) -> Result<()> {
        info!("Node leaving: {}", node_id);

        // Update state
        if let Some(mut node) = self.registry.get(&node_id) {
            node.state = NodeState::Leaving;
            self.registry.upsert(node);
        }

        // Remove from partitioning
        self.partitioning.write().await.remove_node(&node_id);

        // Remove from load balancer
        self.load_balancer.remove_node(&node_id);

        // Eventually remove from registry
        self.registry.remove(&node_id);

        Ok(())
    }

    /// Rebalance cluster.
    pub async fn rebalance(&self) -> Result<()> {
        info!("Rebalancing cluster");

        let active_nodes = self.active_nodes().await;

        // Rebalance partitions if using range partitioning
        if let Some(range_map) = self.partitioning.write().await.range_map_mut() {
            range_map.rebalance(&active_nodes);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;

    #[tokio::test]
    async fn test_coordinator_lifecycle() {
        let local_node = NodeId::new("127.0.0.1:7946".parse::<SocketAddr>().unwrap());
        let config = ClusterConfig::default();

        let coordinator = ClusterCoordinator::new(local_node, config);

        assert!(!coordinator.is_running().await);

        coordinator.start().await.unwrap();
        assert!(coordinator.is_running().await);

        coordinator.stop().await.unwrap();
        assert!(!coordinator.is_running().await);
    }
}
