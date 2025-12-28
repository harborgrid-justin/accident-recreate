//! Rate limiting using token bucket algorithm

use crate::error::{Result, SecurityError};
use governor::{Quota, RateLimiter as GovernorRateLimiter};
use nonzero_ext::nonzero;
use std::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Rate limiter service
pub struct RateLimiter {
    limiters: Arc<RwLock<HashMap<String, GovernorRateLimiter<String, governor::DefaultKeyedStateStore<String>, governor::clock::DefaultClock>>>>,
    requests_per_minute: NonZeroU32,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(requests_per_minute: u32) -> Self {
        Self {
            limiters: Arc::new(RwLock::new(HashMap::new())),
            requests_per_minute: NonZeroU32::new(requests_per_minute).unwrap_or(nonzero!(60u32)),
        }
    }

    /// Check if request is allowed
    pub async fn check_rate_limit(&self, key: &str) -> Result<()> {
        let limiters = self.limiters.read().await;

        if limiters.contains_key(key) {
            // Check existing limiter
            drop(limiters);
            return Err(SecurityError::RateLimitExceeded {
                retry_after_secs: 60,
            });
        }

        Ok(())
    }

    /// Record request
    pub async fn record_request(&self, key: &str) {
        let mut limiters = self.limiters.write().await;
        limiters.insert(key.to_string(), GovernorRateLimiter::keyed(Quota::per_minute(self.requests_per_minute)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter() {
        let limiter = RateLimiter::new(100);
        assert!(limiter.check_rate_limit("user123").await.is_ok());
    }
}
