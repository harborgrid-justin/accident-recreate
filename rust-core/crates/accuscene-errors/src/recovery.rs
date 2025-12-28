//! Error recovery strategies and retry mechanisms

use crate::{AccuSceneError, ErrorCode};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Recovery action that can be taken in response to an error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryAction {
    /// Retry the operation
    Retry,
    /// Use a fallback value or operation
    Fallback,
    /// Skip the operation and continue
    Skip,
    /// Abort the entire process
    Abort,
    /// Ask user for intervention
    UserIntervention,
    /// Use cached data
    UseCache,
    /// Degrade service (reduce functionality)
    Degrade,
}

/// Retry policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts
    pub max_attempts: u32,

    /// Initial delay before first retry
    pub initial_delay: Duration,

    /// Maximum delay between retries
    pub max_delay: Duration,

    /// Backoff multiplier (exponential backoff)
    pub backoff_multiplier: f64,

    /// Jitter factor (0.0 to 1.0) for randomizing delays
    pub jitter: f64,

    /// Whether to retry on all errors or only specific ones
    pub retry_on: RetryCondition,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter: 0.1,
            retry_on: RetryCondition::Transient,
        }
    }
}

impl RetryPolicy {
    /// Creates a new retry policy with custom settings
    pub fn new(max_attempts: u32) -> Self {
        Self {
            max_attempts,
            ..Default::default()
        }
    }

    /// Creates a policy for aggressive retries
    pub fn aggressive() -> Self {
        Self {
            max_attempts: 5,
            initial_delay: Duration::from_millis(50),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 1.5,
            jitter: 0.2,
            retry_on: RetryCondition::Transient,
        }
    }

    /// Creates a policy for conservative retries
    pub fn conservative() -> Self {
        Self {
            max_attempts: 2,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 3.0,
            jitter: 0.05,
            retry_on: RetryCondition::Transient,
        }
    }

    /// Calculates delay for the given attempt number
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        let base_delay = self.initial_delay.as_millis() as f64
            * self.backoff_multiplier.powi(attempt as i32);

        let delay_ms = base_delay.min(self.max_delay.as_millis() as f64);

        // Add jitter
        let jitter_range = delay_ms * self.jitter;
        let jitter = rand::random::<f64>() * jitter_range * 2.0 - jitter_range;
        let final_delay = (delay_ms + jitter).max(0.0);

        Duration::from_millis(final_delay as u64)
    }

    /// Checks if the error should be retried
    pub fn should_retry(&self, error: &AccuSceneError) -> bool {
        match self.retry_on {
            RetryCondition::Always => true,
            RetryCondition::Never => false,
            RetryCondition::Transient => error.code().is_transient(),
            RetryCondition::Custom(ref codes) => codes.contains(&error.code()),
        }
    }
}

/// Condition for when to retry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetryCondition {
    /// Always retry
    Always,
    /// Never retry
    Never,
    /// Retry only transient errors (network, timeout, etc.)
    Transient,
    /// Retry specific error codes
    Custom(Vec<ErrorCode>),
}

/// Extension trait for ErrorCode to determine if error is transient
impl ErrorCode {
    /// Returns whether this error is transient (temporary)
    pub fn is_transient(self) -> bool {
        matches!(
            self,
            ErrorCode::Network
                | ErrorCode::Timeout
                | ErrorCode::Unavailable
                | ErrorCode::RateLimit
                | ErrorCode::ExternalService
        )
    }
}

/// Recovery strategy trait
#[async_trait]
pub trait RecoveryStrategy: Send + Sync {
    /// Attempts to recover from the error
    async fn recover(&self, error: &AccuSceneError) -> Result<RecoveryAction, AccuSceneError>;

    /// Returns whether this strategy can handle the given error
    fn can_handle(&self, error: &AccuSceneError) -> bool;
}

/// Default recovery strategy that suggests actions based on error type
pub struct DefaultRecoveryStrategy {
    retry_policy: RetryPolicy,
}

impl DefaultRecoveryStrategy {
    /// Creates a new default recovery strategy
    pub fn new(retry_policy: RetryPolicy) -> Self {
        Self { retry_policy }
    }
}

impl Default for DefaultRecoveryStrategy {
    fn default() -> Self {
        Self::new(RetryPolicy::default())
    }
}

#[async_trait]
impl RecoveryStrategy for DefaultRecoveryStrategy {
    async fn recover(&self, error: &AccuSceneError) -> Result<RecoveryAction, AccuSceneError> {
        if self.retry_policy.should_retry(error) {
            Ok(RecoveryAction::Retry)
        } else if error.is_recoverable() {
            match error.code() {
                ErrorCode::NotFound => Ok(RecoveryAction::Skip),
                ErrorCode::Validation => Ok(RecoveryAction::UserIntervention),
                ErrorCode::Authentication | ErrorCode::Authorization => {
                    Ok(RecoveryAction::UserIntervention)
                }
                ErrorCode::Cache => Ok(RecoveryAction::UseCache),
                ErrorCode::Unavailable => Ok(RecoveryAction::Degrade),
                _ => Ok(RecoveryAction::Fallback),
            }
        } else {
            Ok(RecoveryAction::Abort)
        }
    }

    fn can_handle(&self, _error: &AccuSceneError) -> bool {
        true // Default strategy handles all errors
    }
}

/// Retry executor for operations with automatic retry logic
pub struct RetryExecutor {
    policy: RetryPolicy,
}

impl RetryExecutor {
    /// Creates a new retry executor
    pub fn new(policy: RetryPolicy) -> Self {
        Self { policy }
    }

    /// Executes an async operation with retry logic
    pub async fn execute<F, T, Fut>(&self, mut operation: F) -> Result<T, AccuSceneError>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, AccuSceneError>>,
    {
        let mut attempt = 0;
        let mut last_error = None;

        while attempt < self.policy.max_attempts {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    if !self.policy.should_retry(&error) {
                        return Err(error);
                    }

                    attempt += 1;
                    last_error = Some(error);

                    if attempt < self.policy.max_attempts {
                        let delay = self.policy.calculate_delay(attempt);
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error
            .unwrap_or_else(|| AccuSceneError::internal("Retry failed without error"))
            .with_context(format!("Failed after {} attempts", self.policy.max_attempts)))
    }

    /// Executes a synchronous operation with retry logic
    pub fn execute_sync<F, T>(&self, mut operation: F) -> Result<T, AccuSceneError>
    where
        F: FnMut() -> Result<T, AccuSceneError>,
    {
        let mut attempt = 0;
        let mut last_error = None;

        while attempt < self.policy.max_attempts {
            match operation() {
                Ok(result) => return Ok(result),
                Err(error) => {
                    if !self.policy.should_retry(&error) {
                        return Err(error);
                    }

                    attempt += 1;
                    last_error = Some(error);

                    if attempt < self.policy.max_attempts {
                        let delay = self.policy.calculate_delay(attempt);
                        std::thread::sleep(delay);
                    }
                }
            }
        }

        Err(last_error
            .unwrap_or_else(|| AccuSceneError::internal("Retry failed without error"))
            .with_context(format!("Failed after {} attempts", self.policy.max_attempts)))
    }
}

/// Circuit breaker for preventing cascading failures
pub struct CircuitBreaker {
    failure_threshold: u32,
    success_threshold: u32,
    timeout: Duration,
    state: parking_lot::Mutex<CircuitBreakerState>,
}

#[derive(Debug, Clone)]
struct CircuitBreakerState {
    failures: u32,
    successes: u32,
    last_failure: Option<std::time::Instant>,
    is_open: bool,
}

impl CircuitBreaker {
    /// Creates a new circuit breaker
    pub fn new(failure_threshold: u32, success_threshold: u32, timeout: Duration) -> Self {
        Self {
            failure_threshold,
            success_threshold,
            timeout,
            state: parking_lot::Mutex::new(CircuitBreakerState {
                failures: 0,
                successes: 0,
                last_failure: None,
                is_open: false,
            }),
        }
    }

    /// Executes an operation through the circuit breaker
    pub async fn execute<F, T, Fut>(&self, operation: F) -> Result<T, AccuSceneError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, AccuSceneError>>,
    {
        // Check if circuit is open
        {
            let mut state = self.state.lock();
            if state.is_open {
                if let Some(last_failure) = state.last_failure {
                    if last_failure.elapsed() > self.timeout {
                        // Try to close circuit
                        state.is_open = false;
                        state.failures = 0;
                        state.successes = 0;
                    } else {
                        return Err(AccuSceneError::unavailable("Circuit breaker is open"));
                    }
                }
            }
        }

        // Execute operation
        match operation().await {
            Ok(result) => {
                let mut state = self.state.lock();
                state.successes += 1;
                if state.successes >= self.success_threshold {
                    state.failures = 0;
                    state.is_open = false;
                }
                Ok(result)
            }
            Err(error) => {
                let mut state = self.state.lock();
                state.failures += 1;
                state.successes = 0;
                state.last_failure = Some(std::time::Instant::now());

                if state.failures >= self.failure_threshold {
                    state.is_open = true;
                }

                Err(error)
            }
        }
    }

    /// Returns whether the circuit is open
    pub fn is_open(&self) -> bool {
        self.state.lock().is_open
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_policy_delay() {
        let policy = RetryPolicy::default();
        let delay1 = policy.calculate_delay(0);
        let delay2 = policy.calculate_delay(1);

        assert!(delay2 > delay1);
    }

    #[test]
    fn test_error_is_transient() {
        assert!(ErrorCode::Network.is_transient());
        assert!(ErrorCode::Timeout.is_transient());
        assert!(!ErrorCode::Validation.is_transient());
        assert!(!ErrorCode::Internal.is_transient());
    }

    #[tokio::test]
    async fn test_retry_executor() {
        let executor = RetryExecutor::new(RetryPolicy::new(3));
        let mut attempt = 0;

        let result = executor
            .execute(|| {
                attempt += 1;
                async move {
                    if attempt < 3 {
                        Err(AccuSceneError::network("Temporary failure"))
                    } else {
                        Ok(42)
                    }
                }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempt, 3);
    }
}
