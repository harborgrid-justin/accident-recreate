//! Load balancing strategies for distributed requests.

use crate::node::Node;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Load balancing strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadBalancingStrategy {
    /// Round-robin selection
    RoundRobin,
    /// Random selection
    Random,
    /// Least connections
    LeastConnections,
    /// Weighted round-robin
    WeightedRoundRobin,
    /// Least response time
    LeastResponseTime,
}

/// Node load metrics.
#[derive(Debug, Clone)]
pub struct NodeLoadMetrics {
    /// Node ID
    pub node_id: Uuid,

    /// Active connections
    pub active_connections: usize,

    /// Average response time (ms)
    pub avg_response_time: f64,

    /// CPU utilization (0.0 - 1.0)
    pub cpu_utilization: f64,

    /// Memory utilization (0.0 - 1.0)
    pub memory_utilization: f64,

    /// Request count
    pub request_count: u64,

    /// Weight for weighted strategies
    pub weight: u32,
}

impl NodeLoadMetrics {
    /// Create new metrics for a node.
    pub fn new(node_id: Uuid) -> Self {
        Self {
            node_id,
            active_connections: 0,
            avg_response_time: 0.0,
            cpu_utilization: 0.0,
            memory_utilization: 0.0,
            request_count: 0,
            weight: 1,
        }
    }

    /// Calculate overall load score (0.0 - 1.0, lower is better).
    pub fn load_score(&self) -> f64 {
        let connection_score = self.active_connections as f64 / 100.0;
        let cpu_score = self.cpu_utilization;
        let memory_score = self.memory_utilization;
        let response_score = (self.avg_response_time / 1000.0).min(1.0);

        (connection_score + cpu_score + memory_score + response_score) / 4.0
    }
}

/// Load balancer.
pub struct LoadBalancer {
    /// Balancing strategy
    strategy: LoadBalancingStrategy,

    /// Node metrics
    metrics: Arc<RwLock<HashMap<Uuid, NodeLoadMetrics>>>,

    /// Round-robin counter
    round_robin_index: Arc<RwLock<usize>>,

    /// Weighted round-robin state
    weighted_state: Arc<RwLock<WeightedRRState>>,
}

impl LoadBalancer {
    /// Create a new load balancer.
    pub fn new(strategy: LoadBalancingStrategy) -> Self {
        Self {
            strategy,
            metrics: Arc::new(RwLock::new(HashMap::new())),
            round_robin_index: Arc::new(RwLock::new(0)),
            weighted_state: Arc::new(RwLock::new(WeightedRRState::new())),
        }
    }

    /// Add a node to the load balancer.
    pub fn add_node(&self, node_id: Uuid, weight: u32) {
        let mut metrics = NodeLoadMetrics::new(node_id);
        metrics.weight = weight;
        self.metrics.write().insert(node_id, metrics);
    }

    /// Remove a node from the load balancer.
    pub fn remove_node(&self, node_id: &Uuid) {
        self.metrics.write().remove(node_id);
    }

    /// Update node metrics.
    pub fn update_metrics(&self, node_id: Uuid, metrics: NodeLoadMetrics) {
        self.metrics.write().insert(node_id, metrics);
    }

    /// Select a node based on the strategy.
    pub fn select_node(&self, available_nodes: &[Uuid]) -> Option<Uuid> {
        if available_nodes.is_empty() {
            return None;
        }

        match self.strategy {
            LoadBalancingStrategy::RoundRobin => self.round_robin(available_nodes),
            LoadBalancingStrategy::Random => self.random_select(available_nodes),
            LoadBalancingStrategy::LeastConnections => self.least_connections(available_nodes),
            LoadBalancingStrategy::WeightedRoundRobin => {
                self.weighted_round_robin(available_nodes)
            }
            LoadBalancingStrategy::LeastResponseTime => self.least_response_time(available_nodes),
        }
    }

    /// Round-robin selection.
    fn round_robin(&self, nodes: &[Uuid]) -> Option<Uuid> {
        let mut index = self.round_robin_index.write();
        let selected = nodes.get(*index % nodes.len()).copied();
        *index = (*index + 1) % nodes.len();
        selected
    }

    /// Random selection.
    fn random_select(&self, nodes: &[Uuid]) -> Option<Uuid> {
        use std::collections::hash_map::RandomState;
        use std::hash::{BuildHasher, Hash, Hasher};

        let hasher = RandomState::new();
        let mut h = hasher.build_hasher();
        std::time::SystemTime::now().hash(&mut h);
        let random_index = h.finish() as usize % nodes.len();

        nodes.get(random_index).copied()
    }

    /// Least connections selection.
    fn least_connections(&self, nodes: &[Uuid]) -> Option<Uuid> {
        let metrics = self.metrics.read();

        nodes
            .iter()
            .min_by_key(|&&node_id| {
                metrics
                    .get(&node_id)
                    .map(|m| m.active_connections)
                    .unwrap_or(0)
            })
            .copied()
    }

    /// Weighted round-robin selection.
    fn weighted_round_robin(&self, nodes: &[Uuid]) -> Option<Uuid> {
        let metrics = self.metrics.read();
        let mut state = self.weighted_state.write();

        // Build weight map
        let mut weights: Vec<(Uuid, u32)> = nodes
            .iter()
            .filter_map(|&id| {
                metrics.get(&id).map(|m| (id, m.weight))
            })
            .collect();

        if weights.is_empty() {
            return None;
        }

        weights.sort_by_key(|(id, _)| *id);

        state.select(&weights)
    }

    /// Least response time selection.
    fn least_response_time(&self, nodes: &[Uuid]) -> Option<Uuid> {
        let metrics = self.metrics.read();

        nodes
            .iter()
            .min_by(|&&a, &&b| {
                let score_a = metrics.get(&a).map(|m| m.load_score()).unwrap_or(1.0);
                let score_b = metrics.get(&b).map(|m| m.load_score()).unwrap_or(1.0);
                score_a.partial_cmp(&score_b).unwrap()
            })
            .copied()
    }

    /// Increment active connections for a node.
    pub fn increment_connections(&self, node_id: &Uuid) {
        if let Some(metrics) = self.metrics.write().get_mut(node_id) {
            metrics.active_connections += 1;
            metrics.request_count += 1;
        }
    }

    /// Decrement active connections for a node.
    pub fn decrement_connections(&self, node_id: &Uuid) {
        if let Some(metrics) = self.metrics.write().get_mut(node_id) {
            if metrics.active_connections > 0 {
                metrics.active_connections -= 1;
            }
        }
    }

    /// Update response time for a node.
    pub fn update_response_time(&self, node_id: &Uuid, response_time_ms: f64) {
        if let Some(metrics) = self.metrics.write().get_mut(node_id) {
            // Exponential moving average
            let alpha = 0.3;
            metrics.avg_response_time =
                alpha * response_time_ms + (1.0 - alpha) * metrics.avg_response_time;
        }
    }

    /// Get metrics for a node.
    pub fn get_metrics(&self, node_id: &Uuid) -> Option<NodeLoadMetrics> {
        self.metrics.read().get(node_id).cloned()
    }

    /// Get all metrics.
    pub fn all_metrics(&self) -> Vec<NodeLoadMetrics> {
        self.metrics.read().values().cloned().collect()
    }

    /// Get strategy.
    pub fn strategy(&self) -> LoadBalancingStrategy {
        self.strategy
    }
}

/// Weighted round-robin state.
#[derive(Debug)]
struct WeightedRRState {
    current_weight: HashMap<Uuid, i32>,
}

impl WeightedRRState {
    fn new() -> Self {
        Self {
            current_weight: HashMap::new(),
        }
    }

    fn select(&mut self, weights: &[(Uuid, u32)]) -> Option<Uuid> {
        if weights.is_empty() {
            return None;
        }

        let total_weight: i32 = weights.iter().map(|(_, w)| *w as i32).sum();

        // Update current weights
        let mut max_node = None;
        let mut max_weight = i32::MIN;

        for (node_id, weight) in weights {
            let current = self.current_weight.entry(*node_id).or_insert(0);
            *current += *weight as i32;

            if *current > max_weight {
                max_weight = *current;
                max_node = Some(*node_id);
            }
        }

        // Decrease selected node's weight
        if let Some(node_id) = max_node {
            if let Some(current) = self.current_weight.get_mut(&node_id) {
                *current -= total_weight;
            }
        }

        max_node
    }
}

impl Default for LoadBalancer {
    fn default() -> Self {
        Self::new(LoadBalancingStrategy::RoundRobin)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_robin() {
        let lb = LoadBalancer::new(LoadBalancingStrategy::RoundRobin);

        let node1 = Uuid::new_v4();
        let node2 = Uuid::new_v4();
        let node3 = Uuid::new_v4();

        lb.add_node(node1, 1);
        lb.add_node(node2, 1);
        lb.add_node(node3, 1);

        let nodes = vec![node1, node2, node3];

        let selected1 = lb.select_node(&nodes).unwrap();
        let selected2 = lb.select_node(&nodes).unwrap();
        let selected3 = lb.select_node(&nodes).unwrap();
        let selected4 = lb.select_node(&nodes).unwrap();

        assert_eq!(selected1, nodes[0]);
        assert_eq!(selected2, nodes[1]);
        assert_eq!(selected3, nodes[2]);
        assert_eq!(selected4, nodes[0]); // Wraps around
    }

    #[test]
    fn test_least_connections() {
        let lb = LoadBalancer::new(LoadBalancingStrategy::LeastConnections);

        let node1 = Uuid::new_v4();
        let node2 = Uuid::new_v4();

        lb.add_node(node1, 1);
        lb.add_node(node2, 1);

        lb.increment_connections(&node1);
        lb.increment_connections(&node1);

        let nodes = vec![node1, node2];
        let selected = lb.select_node(&nodes).unwrap();

        assert_eq!(selected, node2); // node2 has fewer connections
    }
}
