//! Rate limiting and throttling for job execution.

use crate::error::{JobError, Result};
use parking_lot::RwLock;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Rate limiter using token bucket algorithm
#[derive(Clone)]
pub struct RateLimiter {
    config: RateLimitConfig,
    state: Arc<RwLock<RateLimiterState>>,
}

#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub max_tokens: u32,
    pub refill_rate: u32,
    pub refill_interval: Duration,
}

impl RateLimitConfig {
    /// Create a new rate limit config
    pub fn new(max_requests: u32, window_secs: u64) -> Self {
        Self {
            max_tokens: max_requests,
            refill_rate: max_requests,
            refill_interval: Duration::from_secs(window_secs),
        }
    }

    /// Requests per second
    pub fn per_second(requests: u32) -> Self {
        Self::new(requests, 1)
    }

    /// Requests per minute
    pub fn per_minute(requests: u32) -> Self {
        Self::new(requests, 60)
    }

    /// Requests per hour
    pub fn per_hour(requests: u32) -> Self {
        Self::new(requests, 3600)
    }
}

struct RateLimiterState {
    tokens: u32,
    last_refill: Instant,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            state: Arc::new(RwLock::new(RateLimiterState {
                tokens: config.max_tokens,
                last_refill: Instant::now(),
            })),
            config,
        }
    }

    /// Try to acquire a token (non-blocking)
    pub fn try_acquire(&self) -> Result<()> {
        let mut state = self.state.write();
        self.refill_tokens(&mut state);

        if state.tokens > 0 {
            state.tokens -= 1;
            Ok(())
        } else {
            Err(JobError::RateLimitExceeded {
                limit: self.config.max_tokens,
                window_secs: self.config.refill_interval.as_secs(),
            })
        }
    }

    /// Acquire a token (blocking until available)
    pub async fn acquire(&self) -> Result<()> {
        loop {
            match self.try_acquire() {
                Ok(()) => return Ok(()),
                Err(_) => {
                    // Wait a bit before retrying
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        }
    }

    /// Refill tokens based on elapsed time
    fn refill_tokens(&self, state: &mut RateLimiterState) {
        let now = Instant::now();
        let elapsed = now.duration_since(state.last_refill);

        if elapsed >= self.config.refill_interval {
            let refills = (elapsed.as_secs() / self.config.refill_interval.as_secs()) as u32;
            let tokens_to_add = refills * self.config.refill_rate;
            state.tokens = (state.tokens + tokens_to_add).min(self.config.max_tokens);
            state.last_refill = now;
        }
    }

    /// Get current token count
    pub fn available_tokens(&self) -> u32 {
        let mut state = self.state.write();
        self.refill_tokens(&mut state);
        state.tokens
    }

    /// Get configuration
    pub fn config(&self) -> &RateLimitConfig {
        &self.config
    }
}

/// Sliding window rate limiter
#[derive(Clone)]
pub struct SlidingWindowLimiter {
    max_requests: u32,
    window: Duration,
    requests: Arc<RwLock<VecDeque<Instant>>>,
}

impl SlidingWindowLimiter {
    /// Create a new sliding window limiter
    pub fn new(max_requests: u32, window_secs: u64) -> Self {
        Self {
            max_requests,
            window: Duration::from_secs(window_secs),
            requests: Arc::new(RwLock::new(VecDeque::new())),
        }
    }

    /// Try to acquire permission
    pub fn try_acquire(&self) -> Result<()> {
        let mut requests = self.requests.write();
        let now = Instant::now();

        // Remove old requests outside the window
        while let Some(&first) = requests.front() {
            if now.duration_since(first) > self.window {
                requests.pop_front();
            } else {
                break;
            }
        }

        if requests.len() < self.max_requests as usize {
            requests.push_back(now);
            Ok(())
        } else {
            Err(JobError::RateLimitExceeded {
                limit: self.max_requests,
                window_secs: self.window.as_secs(),
            })
        }
    }

    /// Acquire permission (blocking)
    pub async fn acquire(&self) -> Result<()> {
        loop {
            match self.try_acquire() {
                Ok(()) => return Ok(()),
                Err(_) => {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        }
    }

    /// Get current request count in window
    pub fn current_count(&self) -> usize {
        let mut requests = self.requests.write();
        let now = Instant::now();

        // Clean up old requests
        while let Some(&first) = requests.front() {
            if now.duration_since(first) > self.window {
                requests.pop_front();
            } else {
                break;
            }
        }

        requests.len()
    }
}

/// Job throttler for controlling execution rate
pub struct JobThrottler {
    limiter: RateLimiter,
}

impl JobThrottler {
    /// Create a new job throttler
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            limiter: RateLimiter::new(config),
        }
    }

    /// Throttle job execution
    pub async fn throttle<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce() -> T,
    {
        self.limiter.acquire().await?;
        Ok(f())
    }

    /// Throttle async job execution
    pub async fn throttle_async<F, Fut, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = T>,
    {
        self.limiter.acquire().await?;
        Ok(f().await)
    }

    /// Get available capacity
    pub fn available_capacity(&self) -> u32 {
        self.limiter.available_tokens()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_basic() {
        let limiter = RateLimiter::new(RateLimitConfig::per_second(5));

        // Should be able to acquire 5 tokens
        for _ in 0..5 {
            assert!(limiter.try_acquire().is_ok());
        }

        // 6th should fail
        assert!(limiter.try_acquire().is_err());
    }

    #[test]
    fn test_rate_limiter_available_tokens() {
        let limiter = RateLimiter::new(RateLimitConfig::per_second(10));

        assert_eq!(limiter.available_tokens(), 10);

        limiter.try_acquire().unwrap();
        assert_eq!(limiter.available_tokens(), 9);
    }

    #[tokio::test]
    async fn test_rate_limiter_acquire() {
        let limiter = RateLimiter::new(RateLimitConfig::per_second(2));

        limiter.acquire().await.unwrap();
        limiter.acquire().await.unwrap();

        // This would block if we didn't have a short timeout
        let result = tokio::time::timeout(
            Duration::from_millis(50),
            limiter.acquire()
        ).await;

        assert!(result.is_err()); // Timeout
    }

    #[test]
    fn test_sliding_window_limiter() {
        let limiter = SlidingWindowLimiter::new(3, 1);

        assert!(limiter.try_acquire().is_ok());
        assert!(limiter.try_acquire().is_ok());
        assert!(limiter.try_acquire().is_ok());
        assert!(limiter.try_acquire().is_err());

        assert_eq!(limiter.current_count(), 3);
    }

    #[tokio::test]
    async fn test_job_throttler() {
        let throttler = JobThrottler::new(RateLimitConfig::per_second(5));

        let result = throttler.throttle(|| 42).await.unwrap();
        assert_eq!(result, 42);

        let result = throttler.throttle_async(|| async { "test" }).await.unwrap();
        assert_eq!(result, "test");
    }
}
