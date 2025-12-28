use crate::config::RetryConfig;
use crate::error::{OfflineError, Result};
use std::time::Duration;

/// Retry policy with exponential backoff
pub struct RetryPolicy {
    config: RetryConfig,
}

impl RetryPolicy {
    /// Create a new retry policy
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }

    /// Execute a function with retry logic
    pub async fn execute<F, Fut, T>(&self, mut f: F) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut attempt = 0;
        let mut last_error = None;

        while attempt <= self.config.max_attempts {
            match f().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    // Check if error is retryable
                    let should_retry = self.should_retry(&e, attempt);

                    if !should_retry {
                        return Err(e);
                    }

                    last_error = Some(e);
                    attempt += 1;

                    if attempt <= self.config.max_attempts {
                        let delay = self.config.calculate_delay(attempt);
                        tracing::debug!(
                            "Retry attempt {}/{} after {:?}",
                            attempt,
                            self.config.max_attempts,
                            delay
                        );
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            OfflineError::Internal("Retry failed without error".to_string())
        }))
    }

    /// Check if error should be retried
    fn should_retry(&self, error: &OfflineError, attempt: usize) -> bool {
        if attempt >= self.config.max_attempts {
            return false;
        }

        match error {
            OfflineError::Network(_) => self.config.retry_on_network_error,
            OfflineError::NetworkUnavailable => self.config.retry_on_network_error,
            OfflineError::Timeout(_) => self.config.retry_on_timeout,
            OfflineError::Conflict(_) => self.config.retry_on_conflict,
            OfflineError::VersionMismatch { .. } => self.config.retry_on_conflict,
            OfflineError::LockTimeout => true,
            OfflineError::RateLimited(_) => true,
            _ => false,
        }
    }

    /// Calculate backoff delay for attempt
    pub fn calculate_delay(&self, attempt: usize) -> Duration {
        self.config.calculate_delay(attempt)
    }

    /// Execute with exponential backoff
    pub async fn execute_with_backoff<F, Fut, T>(
        &self,
        operation_name: &str,
        f: F,
    ) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        tracing::debug!("Executing {} with retry policy", operation_name);
        self.execute(f).await
    }
}

/// Retry decorator for async functions
pub struct Retryable<F> {
    func: F,
    policy: RetryPolicy,
}

impl<F> Retryable<F> {
    /// Create a new retryable function
    pub fn new(func: F, policy: RetryPolicy) -> Self {
        Self { func, policy }
    }

    /// Execute the retryable function
    pub async fn execute<T, Fut>(&mut self) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        self.policy.execute(&mut self.func).await
    }
}

/// Circuit breaker for preventing cascading failures
pub struct CircuitBreaker {
    /// Failure threshold before opening circuit
    failure_threshold: usize,

    /// Success threshold before closing circuit
    success_threshold: usize,

    /// Timeout before attempting to close circuit
    timeout: Duration,

    /// Current state
    state: parking_lot::RwLock<CircuitState>,

    /// Consecutive failures
    consecutive_failures: parking_lot::RwLock<usize>,

    /// Consecutive successes (in half-open state)
    consecutive_successes: parking_lot::RwLock<usize>,

    /// Last failure time
    last_failure: parking_lot::RwLock<Option<std::time::Instant>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(failure_threshold: usize, success_threshold: usize, timeout: Duration) -> Self {
        Self {
            failure_threshold,
            success_threshold,
            timeout,
            state: parking_lot::RwLock::new(CircuitState::Closed),
            consecutive_failures: parking_lot::RwLock::new(0),
            consecutive_successes: parking_lot::RwLock::new(0),
            last_failure: parking_lot::RwLock::new(None),
        }
    }

    /// Execute a function through circuit breaker
    pub async fn execute<F, Fut, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        // Check circuit state
        self.check_state();

        let state = *self.state.read();

        if state == CircuitState::Open {
            return Err(OfflineError::InvalidState(
                "Circuit breaker is open".to_string()
            ));
        }

        match f().await {
            Ok(result) => {
                self.on_success();
                Ok(result)
            }
            Err(e) => {
                self.on_failure();
                Err(e)
            }
        }
    }

    /// Check and update circuit state based on timeout
    fn check_state(&self) {
        let state = *self.state.read();

        if state == CircuitState::Open {
            if let Some(last_failure) = *self.last_failure.read() {
                if last_failure.elapsed() >= self.timeout {
                    *self.state.write() = CircuitState::HalfOpen;
                    *self.consecutive_successes.write() = 0;
                }
            }
        }
    }

    /// Handle successful execution
    fn on_success(&self) {
        *self.consecutive_failures.write() = 0;

        let state = *self.state.read();

        if state == CircuitState::HalfOpen {
            let mut successes = self.consecutive_successes.write();
            *successes += 1;

            if *successes >= self.success_threshold {
                *self.state.write() = CircuitState::Closed;
                *successes = 0;
            }
        }
    }

    /// Handle failed execution
    fn on_failure(&self) {
        *self.last_failure.write() = Some(std::time::Instant::now());

        let mut failures = self.consecutive_failures.write();
        *failures += 1;

        if *failures >= self.failure_threshold {
            *self.state.write() = CircuitState::Open;
        }
    }

    /// Get current state
    pub fn is_open(&self) -> bool {
        *self.state.read() == CircuitState::Open
    }

    /// Reset circuit breaker
    pub fn reset(&self) {
        *self.state.write() = CircuitState::Closed;
        *self.consecutive_failures.write() = 0;
        *self.consecutive_successes.write() = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_retry_policy() {
        let config = RetryConfig {
            max_attempts: 3,
            initial_delay_ms: 10,
            max_delay_ms: 100,
            multiplier: 2.0,
            jitter: 0.0,
            retry_on_network_error: true,
            retry_on_timeout: true,
            retry_on_conflict: true,
        };

        let policy = RetryPolicy::new(config);
        let mut attempt_count = 0;

        let result = policy
            .execute(|| async {
                attempt_count += 1;
                if attempt_count < 3 {
                    Err(OfflineError::NetworkUnavailable)
                } else {
                    Ok(42)
                }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempt_count, 3);
    }

    #[tokio::test]
    async fn test_circuit_breaker() {
        let breaker = CircuitBreaker::new(3, 2, Duration::from_millis(100));

        // Simulate failures
        for _ in 0..3 {
            let _ = breaker
                .execute(|| async { Err::<(), _>(OfflineError::NetworkUnavailable) })
                .await;
        }

        assert!(breaker.is_open());

        // Circuit should be open
        let result = breaker.execute(|| async { Ok::<_, OfflineError>(42) }).await;
        assert!(result.is_err());

        // Wait for timeout
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Should be half-open, allow one request
        let result = breaker.execute(|| async { Ok::<_, OfflineError>(42) }).await;
        assert!(result.is_ok());
    }
}
