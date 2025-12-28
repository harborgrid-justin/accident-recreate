//! Health check module

pub mod checker;
pub mod probes;

use crate::{HealthConfig, Result};
use parking_lot::RwLock;
use std::sync::Arc;
use tokio::sync::Notify;

pub use checker::{HealthCheck, HealthChecker, HealthStatus};
pub use probes::{LivenessProbe, ReadinessProbe};

/// Health check system
pub struct HealthSystem {
    config: HealthConfig,
    checker: Arc<RwLock<HealthChecker>>,
    liveness: Arc<RwLock<LivenessProbe>>,
    readiness: Arc<RwLock<ReadinessProbe>>,
    shutdown: Arc<Notify>,
    running: Arc<RwLock<bool>>,
}

impl HealthSystem {
    /// Create a new health system
    pub fn new(config: &HealthConfig) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
            checker: Arc::new(RwLock::new(HealthChecker::new())),
            liveness: Arc::new(RwLock::new(LivenessProbe::new())),
            readiness: Arc::new(RwLock::new(ReadinessProbe::new())),
            shutdown: Arc::new(Notify::new()),
            running: Arc::new(RwLock::new(false)),
        })
    }

    /// Start the health check system
    pub async fn start(&self) -> Result<()> {
        if *self.running.read() {
            return Ok(());
        }

        *self.running.write() = true;

        let checker = Arc::clone(&self.checker);
        let liveness = Arc::clone(&self.liveness);
        let readiness = Arc::clone(&self.readiness);
        let shutdown = Arc::clone(&self.shutdown);
        let running = Arc::clone(&self.running);
        let interval = self.config.interval_secs;

        tokio::spawn(async move {
            tracing::info!("Starting health check system");

            loop {
                tokio::select! {
                    _ = shutdown.notified() => {
                        tracing::info!("Health check system shutting down");
                        *running.write() = false;
                        break;
                    }
                    _ = tokio::time::sleep(tokio::time::Duration::from_secs(interval)) => {
                        // Run health checks
                        let status = checker.write().check_all().await;
                        tracing::debug!("Health check status: {:?}", status);

                        // Update probes
                        liveness.write().update(status.is_healthy());
                        readiness.write().update(status.is_healthy());
                    }
                }
            }
        });

        Ok(())
    }

    /// Stop the health check system
    pub async fn stop(&self) -> Result<()> {
        if !*self.running.read() {
            return Ok(());
        }

        self.shutdown.notify_waiters();

        // Wait for shutdown
        while *self.running.read() {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        tracing::info!("Health check system stopped");
        Ok(())
    }

    /// Register a health check
    pub fn register(&self, check: Box<dyn HealthCheck>) {
        self.checker.write().register(check);
    }

    /// Get the health checker
    pub fn checker(&self) -> Arc<RwLock<HealthChecker>> {
        Arc::clone(&self.checker)
    }

    /// Get the liveness probe
    pub fn liveness(&self) -> Arc<RwLock<LivenessProbe>> {
        Arc::clone(&self.liveness)
    }

    /// Get the readiness probe
    pub fn readiness(&self) -> Arc<RwLock<ReadinessProbe>> {
        Arc::clone(&self.readiness)
    }

    /// Check if the system is running
    pub fn is_running(&self) -> bool {
        *self.running.read()
    }

    /// Perform an immediate health check
    pub async fn check_now(&self) -> HealthStatus {
        self.checker.write().check_all().await
    }
}
