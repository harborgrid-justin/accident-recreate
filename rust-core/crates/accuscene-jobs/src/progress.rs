//! Job progress tracking for the AccuScene job system.

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Job progress tracker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobProgress {
    pub job_id: String,
    pub current: u64,
    pub total: u64,
    pub message: String,
    pub percentage: f64,
    pub updated_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

impl JobProgress {
    /// Create a new progress tracker
    pub fn new(job_id: String, total: u64) -> Self {
        Self {
            job_id,
            current: 0,
            total,
            message: String::new(),
            percentage: 0.0,
            updated_at: Utc::now(),
            metadata: serde_json::Value::Null,
        }
    }

    /// Update progress
    pub fn update(&mut self, current: u64, message: impl Into<String>) {
        self.current = current.min(self.total);
        self.message = message.into();
        self.percentage = if self.total > 0 {
            (self.current as f64 / self.total as f64) * 100.0
        } else {
            0.0
        };
        self.updated_at = Utc::now();
    }

    /// Increment progress by one
    pub fn increment(&mut self, message: impl Into<String>) {
        self.update(self.current + 1, message);
    }

    /// Check if progress is complete
    pub fn is_complete(&self) -> bool {
        self.current >= self.total
    }

    /// Set metadata
    pub fn set_metadata(&mut self, metadata: serde_json::Value) {
        self.metadata = metadata;
        self.updated_at = Utc::now();
    }
}

/// Thread-safe progress tracker
#[derive(Debug, Clone)]
pub struct ProgressTracker {
    inner: Arc<RwLock<JobProgress>>,
}

impl ProgressTracker {
    /// Create a new progress tracker
    pub fn new(job_id: String, total: u64) -> Self {
        Self {
            inner: Arc::new(RwLock::new(JobProgress::new(job_id, total))),
        }
    }

    /// Update progress
    pub fn update(&self, current: u64, message: impl Into<String>) {
        self.inner.write().update(current, message);
    }

    /// Increment progress
    pub fn increment(&self, message: impl Into<String>) {
        self.inner.write().increment(message);
    }

    /// Get current progress
    pub fn get(&self) -> JobProgress {
        self.inner.read().clone()
    }

    /// Check if complete
    pub fn is_complete(&self) -> bool {
        self.inner.read().is_complete()
    }

    /// Set metadata
    pub fn set_metadata(&self, metadata: serde_json::Value) {
        self.inner.write().set_metadata(metadata);
    }
}

/// Progress callback type
pub type ProgressCallback = Arc<dyn Fn(JobProgress) + Send + Sync>;

/// Progress reporter with callbacks
#[derive(Clone)]
pub struct ProgressReporter {
    tracker: ProgressTracker,
    callbacks: Arc<RwLock<Vec<ProgressCallback>>>,
}

impl ProgressReporter {
    /// Create a new progress reporter
    pub fn new(job_id: String, total: u64) -> Self {
        Self {
            tracker: ProgressTracker::new(job_id, total),
            callbacks: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Add a progress callback
    pub fn add_callback(&self, callback: ProgressCallback) {
        self.callbacks.write().push(callback);
    }

    /// Update progress and notify callbacks
    pub fn update(&self, current: u64, message: impl Into<String>) {
        self.tracker.update(current, message);
        self.notify();
    }

    /// Increment progress and notify callbacks
    pub fn increment(&self, message: impl Into<String>) {
        self.tracker.increment(message);
        self.notify();
    }

    /// Get current progress
    pub fn get(&self) -> JobProgress {
        self.tracker.get()
    }

    /// Notify all callbacks
    fn notify(&self) {
        let progress = self.get();
        let callbacks = self.callbacks.read();
        for callback in callbacks.iter() {
            callback(progress.clone());
        }
    }
}

impl std::fmt::Debug for ProgressReporter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProgressReporter")
            .field("tracker", &self.tracker)
            .field("callbacks_count", &self.callbacks.read().len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_update() {
        let mut progress = JobProgress::new("job-1".to_string(), 100);
        progress.update(50, "Half done");

        assert_eq!(progress.current, 50);
        assert_eq!(progress.total, 100);
        assert_eq!(progress.percentage, 50.0);
        assert_eq!(progress.message, "Half done");
        assert!(!progress.is_complete());
    }

    #[test]
    fn test_progress_increment() {
        let mut progress = JobProgress::new("job-1".to_string(), 10);
        for i in 0..10 {
            progress.increment(format!("Step {}", i + 1));
        }

        assert_eq!(progress.current, 10);
        assert!(progress.is_complete());
    }

    #[test]
    fn test_progress_tracker() {
        let tracker = ProgressTracker::new("job-1".to_string(), 100);
        tracker.update(25, "Quarter done");

        let progress = tracker.get();
        assert_eq!(progress.current, 25);
        assert_eq!(progress.percentage, 25.0);
    }

    #[test]
    fn test_progress_reporter() {
        let reporter = ProgressReporter::new("job-1".to_string(), 100);
        let called = Arc::new(RwLock::new(false));
        let called_clone = called.clone();

        reporter.add_callback(Arc::new(move |_| {
            *called_clone.write() = true;
        }));

        reporter.update(50, "Half done");
        assert!(*called.read());
    }
}
