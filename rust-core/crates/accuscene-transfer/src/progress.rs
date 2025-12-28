use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Progress state for transfer operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferProgress {
    /// Unique operation ID
    pub id: String,
    /// Total items to process
    pub total: u64,
    /// Items processed so far
    pub processed: u64,
    /// Bytes processed
    pub bytes_processed: u64,
    /// Total bytes (if known)
    pub total_bytes: Option<u64>,
    /// Current status
    pub status: ProgressStatus,
    /// Current stage/step
    pub stage: String,
    /// Error message if failed
    pub error: Option<String>,
    /// Estimated time remaining (seconds)
    pub eta_seconds: Option<u64>,
    /// Start time
    pub started_at: chrono::DateTime<chrono::Utc>,
    /// Updated time
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProgressStatus {
    Pending,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

impl TransferProgress {
    /// Create new progress tracker
    pub fn new(total: u64) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            total,
            processed: 0,
            bytes_processed: 0,
            total_bytes: None,
            status: ProgressStatus::Pending,
            stage: "Initializing".to_string(),
            error: None,
            eta_seconds: None,
            started_at: now,
            updated_at: now,
        }
    }

    /// Calculate percentage complete
    pub fn percentage(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.processed as f64 / self.total as f64) * 100.0
        }
    }

    /// Update progress
    pub fn update(&mut self, processed: u64, stage: Option<String>) {
        self.processed = processed;
        if let Some(stage) = stage {
            self.stage = stage;
        }
        self.updated_at = chrono::Utc::now();
        self.calculate_eta();
    }

    /// Calculate ETA based on current progress
    fn calculate_eta(&mut self) {
        if self.processed == 0 || self.total == 0 {
            self.eta_seconds = None;
            return;
        }

        let elapsed = (chrono::Utc::now() - self.started_at).num_seconds() as u64;
        if elapsed == 0 {
            self.eta_seconds = None;
            return;
        }

        let rate = self.processed as f64 / elapsed as f64;
        let remaining = self.total - self.processed;
        self.eta_seconds = Some((remaining as f64 / rate) as u64);
    }

    /// Mark as completed
    pub fn complete(&mut self) {
        self.processed = self.total;
        self.status = ProgressStatus::Completed;
        self.updated_at = chrono::Utc::now();
        self.eta_seconds = Some(0);
    }

    /// Mark as failed
    pub fn fail(&mut self, error: String) {
        self.status = ProgressStatus::Failed;
        self.error = Some(error);
        self.updated_at = chrono::Utc::now();
    }

    /// Mark as cancelled
    pub fn cancel(&mut self) {
        self.status = ProgressStatus::Cancelled;
        self.updated_at = chrono::Utc::now();
    }
}

/// Progress tracker that can be shared across async tasks
#[derive(Clone)]
pub struct ProgressTracker {
    progress: Arc<RwLock<TransferProgress>>,
}

impl ProgressTracker {
    /// Create new tracker
    pub fn new(total: u64) -> Self {
        Self {
            progress: Arc::new(RwLock::new(TransferProgress::new(total))),
        }
    }

    /// Get current progress
    pub async fn get(&self) -> TransferProgress {
        self.progress.read().await.clone()
    }

    /// Update progress
    pub async fn update(&self, processed: u64, stage: Option<String>) {
        let mut p = self.progress.write().await;
        p.update(processed, stage);
    }

    /// Increment progress
    pub async fn increment(&self, amount: u64) {
        let mut p = self.progress.write().await;
        p.processed += amount;
        p.updated_at = chrono::Utc::now();
        p.calculate_eta();
    }

    /// Set total bytes
    pub async fn set_total_bytes(&self, bytes: u64) {
        let mut p = self.progress.write().await;
        p.total_bytes = Some(bytes);
    }

    /// Add bytes processed
    pub async fn add_bytes(&self, bytes: u64) {
        let mut p = self.progress.write().await;
        p.bytes_processed += bytes;
    }

    /// Start tracking
    pub async fn start(&self) {
        let mut p = self.progress.write().await;
        p.status = ProgressStatus::Running;
        p.started_at = chrono::Utc::now();
        p.updated_at = chrono::Utc::now();
    }

    /// Complete tracking
    pub async fn complete(&self) {
        let mut p = self.progress.write().await;
        p.complete();
    }

    /// Fail tracking
    pub async fn fail(&self, error: String) {
        let mut p = self.progress.write().await;
        p.fail(error);
    }

    /// Cancel tracking
    pub async fn cancel(&self) {
        let mut p = self.progress.write().await;
        p.cancel();
    }

    /// Check if cancelled
    pub async fn is_cancelled(&self) -> bool {
        self.progress.read().await.status == ProgressStatus::Cancelled
    }
}
