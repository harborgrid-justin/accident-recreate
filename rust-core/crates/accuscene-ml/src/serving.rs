//! Model serving infrastructure

use crate::error::Result;
use crate::inference::{InferenceEngine, InferenceMetrics};
use crate::model::ModelRegistry;
use dashmap::DashMap;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Semaphore;
use uuid::Uuid;

/// Model serving server
pub struct ServingServer {
    /// Model registry
    registry: Arc<ModelRegistry>,

    /// Active inference engines
    engines: Arc<DashMap<Uuid, Arc<dyn InferenceEngine>>>,

    /// Request semaphore for rate limiting
    semaphore: Arc<Semaphore>,

    /// Server metrics
    metrics: Arc<RwLock<ServerMetrics>>,

    /// Configuration
    config: ServingConfig,
}

impl ServingServer {
    /// Create a new serving server
    pub fn new(registry: Arc<ModelRegistry>, config: ServingConfig) -> Self {
        Self {
            registry,
            engines: Arc::new(DashMap::new()),
            semaphore: Arc::new(Semaphore::new(config.max_concurrent_requests)),
            metrics: Arc::new(RwLock::new(ServerMetrics::new())),
            config,
        }
    }

    /// Load a model for serving
    pub async fn load_model(&self, model_id: &Uuid, engine: Arc<dyn InferenceEngine>) -> Result<()> {
        self.engines.insert(*model_id, engine);
        Ok(())
    }

    /// Unload a model
    pub async fn unload_model(&self, model_id: &Uuid) -> Result<()> {
        self.engines.remove(model_id);
        Ok(())
    }

    /// Get loaded models
    pub fn loaded_models(&self) -> Vec<Uuid> {
        self.engines.iter().map(|entry| *entry.key()).collect()
    }

    /// Get server metrics
    pub fn metrics(&self) -> ServerMetrics {
        self.metrics.read().clone()
    }

    /// Health check
    pub async fn health_check(&self) -> HealthCheckResponse {
        let loaded_models = self.loaded_models().len();
        let metrics = self.metrics();

        HealthCheckResponse {
            status: if loaded_models > 0 {
                "healthy".to_string()
            } else {
                "no models loaded".to_string()
            },
            loaded_models,
            total_requests: metrics.total_requests,
            uptime_seconds: metrics.start_time.elapsed().as_secs(),
        }
    }
}

/// Serving configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServingConfig {
    /// Maximum concurrent requests
    pub max_concurrent_requests: usize,

    /// Request timeout (milliseconds)
    pub request_timeout_ms: u64,

    /// Enable model warm-up
    pub enable_warm_up: bool,

    /// Enable metrics collection
    pub enable_metrics: bool,
}

impl Default for ServingConfig {
    fn default() -> Self {
        Self {
            max_concurrent_requests: 100,
            request_timeout_ms: 30000,
            enable_warm_up: true,
            enable_metrics: true,
        }
    }
}

/// Server metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerMetrics {
    /// Total requests processed
    pub total_requests: u64,

    /// Total errors
    pub total_errors: u64,

    /// Average response time (ms)
    pub avg_response_time_ms: f64,

    /// Start time
    #[serde(skip)]
    pub start_time: Instant,
}

impl ServerMetrics {
    pub fn new() -> Self {
        Self {
            total_requests: 0,
            total_errors: 0,
            avg_response_time_ms: 0.0,
            start_time: Instant::now(),
        }
    }

    pub fn record_request(&mut self, response_time_ms: f64) {
        self.total_requests += 1;
        self.avg_response_time_ms = (self.avg_response_time_ms * (self.total_requests - 1) as f64
            + response_time_ms)
            / self.total_requests as f64;
    }

    pub fn record_error(&mut self) {
        self.total_errors += 1;
    }
}

impl Default for ServerMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub status: String,
    pub loaded_models: usize,
    pub total_requests: u64,
    pub uptime_seconds: u64,
}

/// Model deployment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentConfig {
    /// Model ID
    pub model_id: Uuid,

    /// Deployment name
    pub name: String,

    /// Number of replicas
    pub replicas: usize,

    /// Auto-scaling enabled
    pub auto_scaling: bool,

    /// Minimum replicas
    pub min_replicas: usize,

    /// Maximum replicas
    pub max_replicas: usize,
}

impl DeploymentConfig {
    pub fn new(model_id: Uuid, name: impl Into<String>) -> Self {
        Self {
            model_id,
            name: name.into(),
            replicas: 1,
            auto_scaling: false,
            min_replicas: 1,
            max_replicas: 10,
        }
    }
}
