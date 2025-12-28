//! Connection heartbeat and health monitoring.

use crate::error::Result;
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::Instant;

/// Heartbeat status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeartbeatStatus {
    /// Healthy - receiving heartbeats
    Healthy,
    /// Warning - missed some heartbeats
    Warning,
    /// Unhealthy - too many missed heartbeats
    Unhealthy,
    /// Timeout - heartbeat timeout exceeded
    Timeout,
}

/// Heartbeat monitor for tracking connection health
pub struct HeartbeatMonitor {
    /// Last heartbeat time
    last_heartbeat: Arc<RwLock<Instant>>,
    /// Heartbeat timeout
    timeout: Duration,
    /// Warning threshold
    warning_threshold: Duration,
    /// Current status
    status: Arc<RwLock<HeartbeatStatus>>,
    /// Missed heartbeat count
    missed_count: Arc<RwLock<usize>>,
}

impl HeartbeatMonitor {
    /// Create a new heartbeat monitor
    pub fn new(timeout: Duration) -> Self {
        Self {
            last_heartbeat: Arc::new(RwLock::new(Instant::now())),
            timeout,
            warning_threshold: timeout / 2,
            status: Arc::new(RwLock::new(HeartbeatStatus::Healthy)),
            missed_count: Arc::new(RwLock::new(0)),
        }
    }

    /// Create a monitor with custom thresholds
    pub fn with_thresholds(timeout: Duration, warning_threshold: Duration) -> Self {
        Self {
            last_heartbeat: Arc::new(RwLock::new(Instant::now())),
            timeout,
            warning_threshold,
            status: Arc::new(RwLock::new(HeartbeatStatus::Healthy)),
            missed_count: Arc::new(RwLock::new(0)),
        }
    }

    /// Record a heartbeat
    pub fn beat(&self) {
        *self.last_heartbeat.write() = Instant::now();
        *self.status.write() = HeartbeatStatus::Healthy;
        *self.missed_count.write() = 0;
    }

    /// Check heartbeat status
    pub fn check(&self) -> HeartbeatStatus {
        let elapsed = self.last_heartbeat.read().elapsed();

        let new_status = if elapsed >= self.timeout {
            HeartbeatStatus::Timeout
        } else if elapsed >= self.warning_threshold {
            HeartbeatStatus::Warning
        } else {
            HeartbeatStatus::Healthy
        };

        *self.status.write() = new_status;
        new_status
    }

    /// Get current status
    pub fn status(&self) -> HeartbeatStatus {
        *self.status.read()
    }

    /// Get time since last heartbeat
    pub fn time_since_last_beat(&self) -> Duration {
        self.last_heartbeat.read().elapsed()
    }

    /// Get missed heartbeat count
    pub fn missed_count(&self) -> usize {
        *self.missed_count.read()
    }

    /// Increment missed count
    pub fn increment_missed(&self) {
        *self.missed_count.write() += 1;
    }

    /// Check if timed out
    pub fn is_timeout(&self) -> bool {
        self.check() == HeartbeatStatus::Timeout
    }

    /// Reset the monitor
    pub fn reset(&self) {
        *self.last_heartbeat.write() = Instant::now();
        *self.status.write() = HeartbeatStatus::Healthy;
        *self.missed_count.write() = 0;
    }
}

/// Heartbeat sender for periodic heartbeat messages
pub struct HeartbeatSender {
    /// Heartbeat interval
    interval: Duration,
    /// Send callback
    send_fn: Arc<dyn Fn() -> Result<()> + Send + Sync>,
}

impl HeartbeatSender {
    /// Create a new heartbeat sender
    pub fn new<F>(interval: Duration, send_fn: F) -> Self
    where
        F: Fn() -> Result<()> + Send + Sync + 'static,
    {
        Self {
            interval,
            send_fn: Arc::new(send_fn),
        }
    }

    /// Start sending heartbeats
    pub async fn start(self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(self.interval);

            loop {
                interval.tick().await;

                if let Err(e) = (self.send_fn)() {
                    tracing::error!("Failed to send heartbeat: {}", e);
                    break;
                }
            }
        })
    }
}

/// Heartbeat configuration
#[derive(Debug, Clone)]
pub struct HeartbeatConfig {
    /// Heartbeat interval (how often to send)
    pub interval: Duration,
    /// Heartbeat timeout (when to consider connection dead)
    pub timeout: Duration,
    /// Warning threshold
    pub warning_threshold: Duration,
}

impl Default for HeartbeatConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(90),
            warning_threshold: Duration::from_secs(60),
        }
    }
}

impl HeartbeatConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_warning_threshold(mut self, threshold: Duration) -> Self {
        self.warning_threshold = threshold;
        self
    }
}

/// Heartbeat manager combining sender and monitor
pub struct HeartbeatManager {
    monitor: Arc<HeartbeatMonitor>,
    config: HeartbeatConfig,
}

impl HeartbeatManager {
    /// Create a new heartbeat manager
    pub fn new(config: HeartbeatConfig) -> Self {
        let monitor = Arc::new(HeartbeatMonitor::with_thresholds(
            config.timeout,
            config.warning_threshold,
        ));

        Self { monitor, config }
    }

    /// Get the monitor
    pub fn monitor(&self) -> Arc<HeartbeatMonitor> {
        self.monitor.clone()
    }

    /// Create a sender
    pub fn create_sender<F>(&self, send_fn: F) -> HeartbeatSender
    where
        F: Fn() -> Result<()> + Send + Sync + 'static,
    {
        HeartbeatSender::new(self.config.interval, send_fn)
    }

    /// Start monitoring
    pub fn start_monitoring(
        self: Arc<Self>,
        on_timeout: impl Fn() + Send + Sync + 'static,
    ) -> tokio::task::JoinHandle<()> {
        let on_timeout = Arc::new(on_timeout);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));

            loop {
                interval.tick().await;

                let status = self.monitor.check();

                match status {
                    HeartbeatStatus::Timeout => {
                        tracing::warn!("Heartbeat timeout detected");
                        on_timeout();
                        break;
                    }
                    HeartbeatStatus::Warning => {
                        tracing::warn!("Heartbeat warning: slow response");
                    }
                    _ => {}
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heartbeat_monitor() {
        let monitor = HeartbeatMonitor::new(Duration::from_secs(10));

        monitor.beat();
        assert_eq!(monitor.status(), HeartbeatStatus::Healthy);

        // Simulate timeout
        std::thread::sleep(Duration::from_millis(100));
        let elapsed = monitor.time_since_last_beat();
        assert!(elapsed >= Duration::from_millis(100));
    }

    #[test]
    fn test_heartbeat_status() {
        let monitor = HeartbeatMonitor::with_thresholds(
            Duration::from_millis(100),
            Duration::from_millis(50),
        );

        monitor.beat();
        assert_eq!(monitor.check(), HeartbeatStatus::Healthy);

        std::thread::sleep(Duration::from_millis(60));
        assert_eq!(monitor.check(), HeartbeatStatus::Warning);

        std::thread::sleep(Duration::from_millis(50));
        assert_eq!(monitor.check(), HeartbeatStatus::Timeout);
    }

    #[test]
    fn test_heartbeat_reset() {
        let monitor = HeartbeatMonitor::new(Duration::from_secs(1));

        monitor.beat();
        monitor.increment_missed();
        assert_eq!(monitor.missed_count(), 1);

        monitor.reset();
        assert_eq!(monitor.missed_count(), 0);
        assert_eq!(monitor.status(), HeartbeatStatus::Healthy);
    }

    #[tokio::test]
    async fn test_heartbeat_sender() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let sender = HeartbeatSender::new(Duration::from_millis(50), move || {
            counter_clone.fetch_add(1, Ordering::SeqCst);
            Ok(())
        });

        let handle = sender.start().await;

        tokio::time::sleep(Duration::from_millis(200)).await;
        handle.abort();

        let count = counter.load(Ordering::SeqCst);
        assert!(count >= 3); // Should have sent at least 3 heartbeats
    }

    #[test]
    fn test_heartbeat_config() {
        let config = HeartbeatConfig::new()
            .with_interval(Duration::from_secs(15))
            .with_timeout(Duration::from_secs(60))
            .with_warning_threshold(Duration::from_secs(45));

        assert_eq!(config.interval, Duration::from_secs(15));
        assert_eq!(config.timeout, Duration::from_secs(60));
        assert_eq!(config.warning_threshold, Duration::from_secs(45));
    }
}
