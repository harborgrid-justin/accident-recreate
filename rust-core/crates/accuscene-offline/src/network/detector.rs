use crate::config::NetworkConfig;
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Network state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkState {
    /// Online with good connection
    Online,

    /// Online but poor connection
    Degraded,

    /// Offline
    Offline,

    /// Unknown state
    Unknown,
}

/// Network quality metrics
#[derive(Debug, Clone)]
pub struct NetworkQuality {
    /// Latency in milliseconds
    pub latency_ms: u64,

    /// Bandwidth estimate in bytes per second
    pub bandwidth_bps: u64,

    /// Packet loss rate (0.0 - 1.0)
    pub packet_loss: f32,

    /// Network quality score (0.0 - 1.0)
    pub score: f32,
}

impl NetworkQuality {
    /// Create a new network quality metric
    pub fn new() -> Self {
        Self {
            latency_ms: 0,
            bandwidth_bps: 0,
            packet_loss: 0.0,
            score: 0.0,
        }
    }

    /// Calculate quality score
    pub fn calculate_score(&mut self) {
        // Simple scoring algorithm
        let latency_score = if self.latency_ms < 50 {
            1.0
        } else if self.latency_ms < 150 {
            0.8
        } else if self.latency_ms < 300 {
            0.5
        } else if self.latency_ms < 1000 {
            0.2
        } else {
            0.0
        };

        let packet_loss_score = 1.0 - self.packet_loss;

        self.score = (latency_score * 0.7 + packet_loss_score * 0.3).max(0.0).min(1.0);
    }

    /// Check if quality is acceptable
    pub fn is_acceptable(&self, threshold: f32) -> bool {
        self.score >= threshold
    }
}

impl Default for NetworkQuality {
    fn default() -> Self {
        Self::new()
    }
}

/// Network detector for monitoring connection state
pub struct NetworkDetector {
    /// Configuration
    config: NetworkConfig,

    /// Current network state
    state: Arc<RwLock<NetworkState>>,

    /// Network quality metrics
    quality: Arc<RwLock<NetworkQuality>>,

    /// Last check timestamp
    last_check: Arc<RwLock<Option<Instant>>>,

    /// HTTP client for connectivity checks
    client: reqwest::Client,
}

impl NetworkDetector {
    /// Create a new network detector
    pub fn new(config: NetworkConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(config.connect_timeout_ms))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            config,
            state: Arc::new(RwLock::new(NetworkState::Unknown)),
            quality: Arc::new(RwLock::new(NetworkQuality::default())),
            last_check: Arc::new(RwLock::new(None)),
            client,
        }
    }

    /// Check if online
    pub async fn is_online(&self) -> bool {
        self.check_connectivity().await;
        matches!(*self.state.read(), NetworkState::Online | NetworkState::Degraded)
    }

    /// Get current network state
    pub fn state(&self) -> NetworkState {
        *self.state.read()
    }

    /// Get network quality
    pub fn quality(&self) -> NetworkQuality {
        self.quality.read().clone()
    }

    /// Check network connectivity
    pub async fn check_connectivity(&self) {
        // Check if we need to update based on check interval
        let should_check = {
            let last_check = self.last_check.read();
            match *last_check {
                None => true,
                Some(instant) => {
                    instant.elapsed().as_millis() >= self.config.network_check_interval_ms as u128
                }
            }
        };

        if !should_check {
            return;
        }

        // Perform connectivity check
        let start = Instant::now();

        let result = self
            .client
            .get(&format!("{}/health", self.config.api_endpoint))
            .send()
            .await;

        let latency = start.elapsed().as_millis() as u64;

        match result {
            Ok(response) if response.status().is_success() => {
                let mut quality = self.quality.write();
                quality.latency_ms = latency;
                quality.packet_loss = 0.0;
                quality.calculate_score();

                let state = if quality.is_acceptable(self.config.min_network_quality) {
                    NetworkState::Online
                } else {
                    NetworkState::Degraded
                };

                *self.state.write() = state;
            }
            Ok(_) => {
                *self.state.write() = NetworkState::Degraded;

                let mut quality = self.quality.write();
                quality.latency_ms = latency;
                quality.calculate_score();
            }
            Err(_) => {
                *self.state.write() = NetworkState::Offline;

                let mut quality = self.quality.write();
                quality.packet_loss = 1.0;
                quality.score = 0.0;
            }
        }

        *self.last_check.write() = Some(Instant::now());
    }

    /// Force set network state (for testing)
    pub fn set_state(&self, state: NetworkState) {
        *self.state.write() = state;
    }

    /// Wait for network to become online
    pub async fn wait_for_online(&self, timeout: Duration) -> bool {
        let start = Instant::now();

        while start.elapsed() < timeout {
            if self.is_online().await {
                return true;
            }

            tokio::time::sleep(Duration::from_millis(1000)).await;
        }

        false
    }

    /// Start monitoring network state in background
    pub fn start_monitoring(&self) -> tokio::task::JoinHandle<()> {
        let detector = Self {
            config: self.config.clone(),
            state: Arc::clone(&self.state),
            quality: Arc::clone(&self.quality),
            last_check: Arc::clone(&self.last_check),
            client: self.client.clone(),
        };

        tokio::spawn(async move {
            loop {
                detector.check_connectivity().await;
                tokio::time::sleep(Duration::from_millis(detector.config.network_check_interval_ms)).await;
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_quality_calculation() {
        let mut quality = NetworkQuality::new();
        quality.latency_ms = 50;
        quality.packet_loss = 0.0;
        quality.calculate_score();

        assert!(quality.score > 0.7);
        assert!(quality.is_acceptable(0.5));
    }

    #[tokio::test]
    async fn test_network_detector() {
        let config = NetworkConfig::default();
        let detector = NetworkDetector::new(config);

        assert_eq!(detector.state(), NetworkState::Unknown);
    }
}
