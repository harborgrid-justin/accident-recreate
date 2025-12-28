//! Retry strategies for job execution.

use crate::error::JobError;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Retry strategy trait
pub trait RetryStrategy: Send + Sync {
    /// Calculate the delay before the next retry attempt
    fn next_delay(&self, attempt: u32) -> Option<Duration>;

    /// Check if retry should be attempted
    fn should_retry(&self, attempt: u32, error: &JobError) -> bool;
}

/// No retry strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoRetry;

impl RetryStrategy for NoRetry {
    fn next_delay(&self, _attempt: u32) -> Option<Duration> {
        None
    }

    fn should_retry(&self, _attempt: u32, _error: &JobError) -> bool {
        false
    }
}

/// Fixed delay retry strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixedRetry {
    pub max_attempts: u32,
    pub delay_ms: u64,
}

impl FixedRetry {
    pub fn new(max_attempts: u32, delay_ms: u64) -> Self {
        Self {
            max_attempts,
            delay_ms,
        }
    }
}

impl RetryStrategy for FixedRetry {
    fn next_delay(&self, attempt: u32) -> Option<Duration> {
        if attempt < self.max_attempts {
            Some(Duration::from_millis(self.delay_ms))
        } else {
            None
        }
    }

    fn should_retry(&self, attempt: u32, error: &JobError) -> bool {
        attempt < self.max_attempts && error.is_retryable()
    }
}

/// Exponential backoff retry strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExponentialBackoff {
    pub max_attempts: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub multiplier: f64,
}

impl ExponentialBackoff {
    pub fn new(max_attempts: u32) -> Self {
        Self {
            max_attempts,
            initial_delay_ms: 1000,      // 1 second
            max_delay_ms: 60000,          // 1 minute
            multiplier: 2.0,
        }
    }

    pub fn with_initial_delay(mut self, delay_ms: u64) -> Self {
        self.initial_delay_ms = delay_ms;
        self
    }

    pub fn with_max_delay(mut self, delay_ms: u64) -> Self {
        self.max_delay_ms = delay_ms;
        self
    }

    pub fn with_multiplier(mut self, multiplier: f64) -> Self {
        self.multiplier = multiplier;
        self
    }
}

impl RetryStrategy for ExponentialBackoff {
    fn next_delay(&self, attempt: u32) -> Option<Duration> {
        if attempt >= self.max_attempts {
            return None;
        }

        let delay = (self.initial_delay_ms as f64
            * self.multiplier.powi(attempt as i32))
        .min(self.max_delay_ms as f64);

        Some(Duration::from_millis(delay as u64))
    }

    fn should_retry(&self, attempt: u32, error: &JobError) -> bool {
        attempt < self.max_attempts && error.is_retryable()
    }
}

/// Linear backoff retry strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearBackoff {
    pub max_attempts: u32,
    pub initial_delay_ms: u64,
    pub increment_ms: u64,
    pub max_delay_ms: u64,
}

impl LinearBackoff {
    pub fn new(max_attempts: u32) -> Self {
        Self {
            max_attempts,
            initial_delay_ms: 1000,
            increment_ms: 1000,
            max_delay_ms: 30000,
        }
    }

    pub fn with_increment(mut self, increment_ms: u64) -> Self {
        self.increment_ms = increment_ms;
        self
    }
}

impl RetryStrategy for LinearBackoff {
    fn next_delay(&self, attempt: u32) -> Option<Duration> {
        if attempt >= self.max_attempts {
            return None;
        }

        let delay = (self.initial_delay_ms + self.increment_ms * attempt as u64)
            .min(self.max_delay_ms);

        Some(Duration::from_millis(delay))
    }

    fn should_retry(&self, attempt: u32, error: &JobError) -> bool {
        attempt < self.max_attempts && error.is_retryable()
    }
}

/// Fibonacci backoff retry strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FibonacciBackoff {
    pub max_attempts: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
}

impl FibonacciBackoff {
    pub fn new(max_attempts: u32) -> Self {
        Self {
            max_attempts,
            base_delay_ms: 1000,
            max_delay_ms: 60000,
        }
    }

    fn fibonacci(&self, n: u32) -> u64 {
        match n {
            0 => 0,
            1 => 1,
            _ => {
                let mut a = 0u64;
                let mut b = 1u64;
                for _ in 2..=n {
                    let temp = a + b;
                    a = b;
                    b = temp;
                }
                b
            }
        }
    }
}

impl RetryStrategy for FibonacciBackoff {
    fn next_delay(&self, attempt: u32) -> Option<Duration> {
        if attempt >= self.max_attempts {
            return None;
        }

        let fib = self.fibonacci(attempt + 1);
        let delay = (self.base_delay_ms * fib).min(self.max_delay_ms);

        Some(Duration::from_millis(delay))
    }

    fn should_retry(&self, attempt: u32, error: &JobError) -> bool {
        attempt < self.max_attempts && error.is_retryable()
    }
}

/// Retry policy combining strategy with jitter
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    strategy: RetryStrategyType,
    jitter: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetryStrategyType {
    None(NoRetry),
    Fixed(FixedRetry),
    Exponential(ExponentialBackoff),
    Linear(LinearBackoff),
    Fibonacci(FibonacciBackoff),
}

impl RetryPolicy {
    pub fn none() -> Self {
        Self {
            strategy: RetryStrategyType::None(NoRetry),
            jitter: false,
        }
    }

    pub fn fixed(max_attempts: u32, delay_ms: u64) -> Self {
        Self {
            strategy: RetryStrategyType::Fixed(FixedRetry::new(max_attempts, delay_ms)),
            jitter: false,
        }
    }

    pub fn exponential(max_attempts: u32) -> Self {
        Self {
            strategy: RetryStrategyType::Exponential(ExponentialBackoff::new(max_attempts)),
            jitter: true,
        }
    }

    pub fn linear(max_attempts: u32) -> Self {
        Self {
            strategy: RetryStrategyType::Linear(LinearBackoff::new(max_attempts)),
            jitter: false,
        }
    }

    pub fn fibonacci(max_attempts: u32) -> Self {
        Self {
            strategy: RetryStrategyType::Fibonacci(FibonacciBackoff::new(max_attempts)),
            jitter: false,
        }
    }

    pub fn with_jitter(mut self) -> Self {
        self.jitter = true;
        self
    }

    pub fn next_delay(&self, attempt: u32) -> Option<Duration> {
        let delay = match &self.strategy {
            RetryStrategyType::None(s) => s.next_delay(attempt),
            RetryStrategyType::Fixed(s) => s.next_delay(attempt),
            RetryStrategyType::Exponential(s) => s.next_delay(attempt),
            RetryStrategyType::Linear(s) => s.next_delay(attempt),
            RetryStrategyType::Fibonacci(s) => s.next_delay(attempt),
        };

        if self.jitter {
            delay.map(|d| {
                let jitter = (rand::random::<f64>() * 0.2 - 0.1) + 1.0; // ±10%
                Duration::from_millis((d.as_millis() as f64 * jitter) as u64)
            })
        } else {
            delay
        }
    }

    pub fn should_retry(&self, attempt: u32, error: &JobError) -> bool {
        match &self.strategy {
            RetryStrategyType::None(s) => s.should_retry(attempt, error),
            RetryStrategyType::Fixed(s) => s.should_retry(attempt, error),
            RetryStrategyType::Exponential(s) => s.should_retry(attempt, error),
            RetryStrategyType::Linear(s) => s.should_retry(attempt, error),
            RetryStrategyType::Fibonacci(s) => s.should_retry(attempt, error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_retry() {
        let strategy = FixedRetry::new(3, 1000);
        assert_eq!(strategy.next_delay(0), Some(Duration::from_millis(1000)));
        assert_eq!(strategy.next_delay(2), Some(Duration::from_millis(1000)));
        assert_eq!(strategy.next_delay(3), None);
    }

    #[test]
    fn test_exponential_backoff() {
        let strategy = ExponentialBackoff::new(5);
        let delay1 = strategy.next_delay(0).unwrap();
        let delay2 = strategy.next_delay(1).unwrap();
        assert!(delay2 > delay1);
    }

    #[test]
    fn test_fibonacci_backoff() {
        let strategy = FibonacciBackoff::new(5);
        assert_eq!(strategy.fibonacci(0), 0);
        assert_eq!(strategy.fibonacci(1), 1);
        assert_eq!(strategy.fibonacci(5), 5);
        assert_eq!(strategy.fibonacci(10), 55);
    }

    #[test]
    fn test_retry_policy() {
        let policy = RetryPolicy::exponential(3);
        let error = JobError::ExecutionFailed("test".to_string());
        assert!(policy.should_retry(0, &error));
        assert!(policy.should_retry(2, &error));
        assert!(!policy.should_retry(3, &error));
    }
}
