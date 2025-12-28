//! Backpressure handling strategies for stream processing.

use crate::config::BackpressureConfig;
use crate::error::{Result, StreamingError};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::time::{timeout, Duration};
use tracing::{debug, warn};

/// Backpressure controller for managing flow control
#[derive(Clone)]
pub struct BackpressureController {
    config: BackpressureConfig,
    current_load: Arc<AtomicUsize>,
    capacity: usize,
    semaphore: Arc<Semaphore>,
}

impl BackpressureController {
    /// Create a new backpressure controller
    pub fn new(capacity: usize, config: BackpressureConfig) -> Self {
        Self {
            config,
            current_load: Arc::new(AtomicUsize::new(0)),
            capacity,
            semaphore: Arc::new(Semaphore::new(capacity)),
        }
    }

    /// Acquire permission to process an item
    pub async fn acquire(&self) -> Result<BackpressurePermit> {
        match self.config.strategy {
            crate::config::BackpressureStrategy::Block => {
                // Wait for a permit with timeout
                match timeout(self.config.timeout, self.semaphore.acquire()).await {
                    Ok(Ok(permit)) => {
                        self.current_load.fetch_add(1, Ordering::SeqCst);
                        Ok(BackpressurePermit {
                            _permit: Some(permit.into()),
                            controller: self.clone(),
                        })
                    }
                    Ok(Err(e)) => Err(StreamingError::Backpressure(format!(
                        "Failed to acquire permit: {}",
                        e
                    ))),
                    Err(_) => Err(StreamingError::Timeout(format!(
                        "Backpressure timeout after {:?}",
                        self.config.timeout
                    ))),
                }
            }
            crate::config::BackpressureStrategy::DropOldest => {
                // Always succeed, let the buffer handle dropping
                self.current_load.fetch_add(1, Ordering::SeqCst);
                Ok(BackpressurePermit {
                    _permit: None,
                    controller: self.clone(),
                })
            }
            crate::config::BackpressureStrategy::DropNewest => {
                // Check if we're over capacity
                let current = self.current_load.load(Ordering::SeqCst);
                if current >= self.capacity {
                    return Err(StreamingError::Backpressure(
                        "Capacity exceeded, dropping newest".to_string(),
                    ));
                }
                self.current_load.fetch_add(1, Ordering::SeqCst);
                Ok(BackpressurePermit {
                    _permit: None,
                    controller: self.clone(),
                })
            }
            crate::config::BackpressureStrategy::Fail => {
                // Fail immediately if over capacity
                let current = self.current_load.load(Ordering::SeqCst);
                if current >= self.capacity {
                    return Err(StreamingError::BufferOverflow {
                        capacity: self.capacity,
                    });
                }
                self.current_load.fetch_add(1, Ordering::SeqCst);
                Ok(BackpressurePermit {
                    _permit: None,
                    controller: self.clone(),
                })
            }
        }
    }

    /// Get current load
    pub fn current_load(&self) -> usize {
        self.current_load.load(Ordering::SeqCst)
    }

    /// Get capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get load ratio (0.0 to 1.0)
    pub fn load_ratio(&self) -> f64 {
        self.current_load() as f64 / self.capacity as f64
    }

    /// Check if we're above high watermark
    pub fn is_high_water(&self) -> bool {
        self.load_ratio() >= self.config.high_watermark
    }

    /// Check if we're below low watermark
    pub fn is_low_water(&self) -> bool {
        self.load_ratio() <= self.config.low_watermark
    }

    /// Check if backpressure should be applied
    pub fn should_apply_backpressure(&self) -> bool {
        self.is_high_water()
    }

    /// Release backpressure (internal use)
    fn release(&self) {
        let prev = self.current_load.fetch_sub(1, Ordering::SeqCst);
        debug!("Backpressure released, load: {} -> {}", prev, prev - 1);

        if prev == (self.config.high_watermark * self.capacity as f64) as usize {
            debug!("Dropped below high watermark");
        }
    }
}

/// A permit that represents permission to process an item
pub struct BackpressurePermit {
    _permit: Option<tokio::sync::OwnedSemaphorePermit>,
    controller: BackpressureController,
}

impl Drop for BackpressurePermit {
    fn drop(&mut self) {
        self.controller.release();
    }
}

/// Adaptive backpressure that adjusts based on processing time
pub struct AdaptiveBackpressure {
    controller: BackpressureController,
    avg_processing_time: Arc<AtomicUsize>, // in microseconds
    target_latency: Duration,
}

impl AdaptiveBackpressure {
    /// Create a new adaptive backpressure controller
    pub fn new(
        capacity: usize,
        config: BackpressureConfig,
        target_latency: Duration,
    ) -> Self {
        Self {
            controller: BackpressureController::new(capacity, config),
            avg_processing_time: Arc::new(AtomicUsize::new(0)),
            target_latency,
        }
    }

    /// Acquire permission with adaptive behavior
    pub async fn acquire(&self) -> Result<AdaptivePermit> {
        let permit = self.controller.acquire().await?;

        Ok(AdaptivePermit {
            permit,
            start_time: std::time::Instant::now(),
            avg_processing_time: self.avg_processing_time.clone(),
        })
    }

    /// Get average processing time
    pub fn avg_processing_time(&self) -> Duration {
        Duration::from_micros(self.avg_processing_time.load(Ordering::SeqCst) as u64)
    }

    /// Check if we're meeting latency targets
    pub fn is_meeting_target(&self) -> bool {
        self.avg_processing_time() <= self.target_latency
    }

    /// Get recommended capacity adjustment
    pub fn recommended_capacity_adjustment(&self) -> i32 {
        let avg = self.avg_processing_time();
        if avg > self.target_latency * 2 {
            // Significantly over target, reduce capacity
            -10
        } else if avg > self.target_latency {
            // Slightly over target, reduce capacity
            -5
        } else if avg < self.target_latency / 2 {
            // Well under target, increase capacity
            10
        } else if avg < self.target_latency {
            // Slightly under target, increase capacity
            5
        } else {
            // At target
            0
        }
    }
}

/// Adaptive backpressure permit
pub struct AdaptivePermit {
    permit: BackpressurePermit,
    start_time: std::time::Instant,
    avg_processing_time: Arc<AtomicUsize>,
}

impl Drop for AdaptivePermit {
    fn drop(&mut self) {
        let elapsed = self.start_time.elapsed();
        let elapsed_micros = elapsed.as_micros() as usize;

        // Update moving average (simple exponential moving average with alpha=0.2)
        let old_avg = self.avg_processing_time.load(Ordering::SeqCst);
        let new_avg = if old_avg == 0 {
            elapsed_micros
        } else {
            (old_avg * 4 + elapsed_micros) / 5
        };

        self.avg_processing_time.store(new_avg, Ordering::SeqCst);

        if elapsed > Duration::from_secs(1) {
            warn!("Slow processing detected: {:?}", elapsed);
        }
    }
}

/// Rate limiter for controlling throughput
pub struct RateLimiter {
    permits_per_second: usize,
    semaphore: Arc<Semaphore>,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(permits_per_second: usize) -> Self {
        let semaphore = Arc::new(Semaphore::new(permits_per_second));

        // Spawn a task to refill permits
        let semaphore_clone = semaphore.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            loop {
                interval.tick().await;
                // Add permits back to the semaphore
                semaphore_clone.add_permits(permits_per_second);
            }
        });

        Self {
            permits_per_second,
            semaphore,
        }
    }

    /// Acquire a permit to proceed
    pub async fn acquire(&self) -> Result<()> {
        self.semaphore
            .acquire()
            .await
            .map_err(|e| StreamingError::Backpressure(format!("Rate limit error: {}", e)))?;
        Ok(())
    }

    /// Try to acquire a permit without blocking
    pub fn try_acquire(&self) -> Result<()> {
        self.semaphore
            .try_acquire()
            .map_err(|e| StreamingError::Backpressure(format!("Rate limit exceeded: {}", e)))?;
        Ok(())
    }

    /// Get the configured rate
    pub fn rate(&self) -> usize {
        self.permits_per_second
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_backpressure_controller() {
        let config = BackpressureConfig {
            strategy: crate::config::BackpressureStrategy::Block,
            high_watermark: 0.8,
            low_watermark: 0.2,
            timeout: Duration::from_secs(5),
        };

        let controller = BackpressureController::new(10, config);

        assert_eq!(controller.current_load(), 0);
        assert!(!controller.is_high_water());

        let mut permits = vec![];
        for _ in 0..8 {
            permits.push(controller.acquire().await.unwrap());
        }

        assert_eq!(controller.current_load(), 8);
        assert!(controller.is_high_water());

        drop(permits);

        assert_eq!(controller.current_load(), 0);
        assert!(controller.is_low_water());
    }

    #[tokio::test]
    async fn test_adaptive_backpressure() {
        let config = BackpressureConfig::default();
        let adaptive = AdaptiveBackpressure::new(10, config, Duration::from_millis(100));

        let permit = adaptive.acquire().await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;
        drop(permit);

        // Processing time should be recorded
        assert!(adaptive.avg_processing_time() > Duration::from_millis(40));
    }
}
