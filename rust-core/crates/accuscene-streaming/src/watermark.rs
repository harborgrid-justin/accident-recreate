//! Watermark handling for event-time processing.

use crate::error::{Result, StreamingError};
use chrono::{DateTime, Utc};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, trace};

/// Represents an event timestamp
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp(pub i64);

impl Timestamp {
    /// Create a new timestamp from milliseconds since epoch
    pub fn from_millis(millis: i64) -> Self {
        Self(millis)
    }

    /// Create a timestamp from DateTime
    pub fn from_datetime(dt: DateTime<Utc>) -> Self {
        Self(dt.timestamp_millis())
    }

    /// Get current timestamp
    pub fn now() -> Self {
        Self(Utc::now().timestamp_millis())
    }

    /// Convert to milliseconds
    pub fn as_millis(&self) -> i64 {
        self.0
    }

    /// Convert to DateTime
    pub fn to_datetime(&self) -> DateTime<Utc> {
        DateTime::from_timestamp_millis(self.0).unwrap_or_else(|| Utc::now())
    }

    /// Add duration
    pub fn add(&self, duration: Duration) -> Self {
        Self(self.0 + duration.as_millis() as i64)
    }

    /// Subtract duration
    pub fn sub(&self, duration: Duration) -> Self {
        Self(self.0 - duration.as_millis() as i64)
    }
}

impl From<i64> for Timestamp {
    fn from(millis: i64) -> Self {
        Self(millis)
    }
}

impl From<DateTime<Utc>> for Timestamp {
    fn from(dt: DateTime<Utc>) -> Self {
        Self::from_datetime(dt)
    }
}

/// Watermark represents the progress of event time in a stream
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Watermark {
    /// The timestamp of the watermark
    pub timestamp: Timestamp,
}

impl Watermark {
    /// Create a new watermark
    pub fn new(timestamp: Timestamp) -> Self {
        Self { timestamp }
    }

    /// Create a watermark from milliseconds
    pub fn from_millis(millis: i64) -> Self {
        Self {
            timestamp: Timestamp::from_millis(millis),
        }
    }

    /// Create the minimum watermark (beginning of time)
    pub fn min() -> Self {
        Self {
            timestamp: Timestamp(i64::MIN),
        }
    }

    /// Create the maximum watermark (end of time)
    pub fn max() -> Self {
        Self {
            timestamp: Timestamp(i64::MAX),
        }
    }

    /// Check if an event with the given timestamp is late
    pub fn is_late(&self, event_time: Timestamp, allowed_lateness: Duration) -> bool {
        event_time < self.timestamp.sub(allowed_lateness)
    }
}

/// Watermark strategy for generating watermarks
pub trait WatermarkStrategy: Send + Sync {
    /// Generate a watermark for the given event timestamp
    fn generate(&mut self, event_timestamp: Timestamp) -> Option<Watermark>;

    /// Called when a source becomes idle
    fn on_idle(&mut self) -> Option<Watermark>;
}

/// Bounded out-of-orderness watermark strategy
pub struct BoundedOutOfOrdernessStrategy {
    max_out_of_orderness: Duration,
    last_timestamp: Option<Timestamp>,
    last_watermark: Watermark,
}

impl BoundedOutOfOrdernessStrategy {
    /// Create a new bounded out-of-orderness strategy
    pub fn new(max_out_of_orderness: Duration) -> Self {
        Self {
            max_out_of_orderness,
            last_timestamp: None,
            last_watermark: Watermark::min(),
        }
    }
}

impl WatermarkStrategy for BoundedOutOfOrdernessStrategy {
    fn generate(&mut self, event_timestamp: Timestamp) -> Option<Watermark> {
        // Update last timestamp
        self.last_timestamp = Some(match self.last_timestamp {
            Some(last) => last.max(event_timestamp),
            None => event_timestamp,
        });

        // Generate watermark: current timestamp - max out of orderness
        let watermark = Watermark::new(event_timestamp.sub(self.max_out_of_orderness));

        // Only emit if watermark progresses
        if watermark > self.last_watermark {
            self.last_watermark = watermark;
            Some(watermark)
        } else {
            None
        }
    }

    fn on_idle(&mut self) -> Option<Watermark> {
        // When idle, advance watermark to last seen timestamp
        if let Some(last) = self.last_timestamp {
            let watermark = Watermark::new(last);
            if watermark > self.last_watermark {
                self.last_watermark = watermark;
                return Some(watermark);
            }
        }
        None
    }
}

/// Periodic watermark strategy
pub struct PeriodicWatermarkStrategy {
    interval: Duration,
    last_event_time: Option<Timestamp>,
    last_watermark_time: Option<std::time::Instant>,
}

impl PeriodicWatermarkStrategy {
    /// Create a new periodic watermark strategy
    pub fn new(interval: Duration) -> Self {
        Self {
            interval,
            last_event_time: None,
            last_watermark_time: None,
        }
    }
}

impl WatermarkStrategy for PeriodicWatermarkStrategy {
    fn generate(&mut self, event_timestamp: Timestamp) -> Option<Watermark> {
        self.last_event_time = Some(match self.last_event_time {
            Some(last) => last.max(event_timestamp),
            None => event_timestamp,
        });

        let now = std::time::Instant::now();
        let should_emit = match self.last_watermark_time {
            Some(last) => now.duration_since(last) >= self.interval,
            None => true,
        };

        if should_emit {
            self.last_watermark_time = Some(now);
            self.last_event_time.map(Watermark::new)
        } else {
            None
        }
    }

    fn on_idle(&mut self) -> Option<Watermark> {
        self.last_event_time.map(Watermark::new)
    }
}

/// Watermark tracker that combines watermarks from multiple sources
#[derive(Clone)]
pub struct WatermarkTracker {
    inner: Arc<RwLock<WatermarkTrackerInner>>,
}

struct WatermarkTrackerInner {
    source_watermarks: BTreeMap<String, Watermark>,
    global_watermark: Watermark,
}

impl WatermarkTracker {
    /// Create a new watermark tracker
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(WatermarkTrackerInner {
                source_watermarks: BTreeMap::new(),
                global_watermark: Watermark::min(),
            })),
        }
    }

    /// Update watermark for a source
    pub async fn update(&self, source_id: &str, watermark: Watermark) -> Result<()> {
        let mut inner = self.inner.write().await;

        // Update source watermark
        inner.source_watermarks.insert(source_id.to_string(), watermark);

        // Compute global watermark (minimum of all sources)
        let new_global = inner
            .source_watermarks
            .values()
            .min()
            .copied()
            .unwrap_or(Watermark::min());

        if new_global > inner.global_watermark {
            trace!(
                "Global watermark advanced: {:?} -> {:?}",
                inner.global_watermark,
                new_global
            );
            inner.global_watermark = new_global;
        }

        Ok(())
    }

    /// Get the current global watermark
    pub async fn get(&self) -> Watermark {
        self.inner.read().await.global_watermark
    }

    /// Remove a source from tracking
    pub async fn remove_source(&self, source_id: &str) -> Result<()> {
        let mut inner = self.inner.write().await;
        inner.source_watermarks.remove(source_id);

        // Recompute global watermark
        let new_global = inner
            .source_watermarks
            .values()
            .min()
            .copied()
            .unwrap_or(Watermark::min());

        if new_global > inner.global_watermark {
            inner.global_watermark = new_global;
        }

        Ok(())
    }

    /// Get watermark for a specific source
    pub async fn get_source_watermark(&self, source_id: &str) -> Option<Watermark> {
        self.inner
            .read()
            .await
            .source_watermarks
            .get(source_id)
            .copied()
    }

    /// Check if all sources have advanced beyond a timestamp
    pub async fn all_advanced_beyond(&self, timestamp: Timestamp) -> bool {
        let inner = self.inner.read().await;
        inner
            .source_watermarks
            .values()
            .all(|w| w.timestamp >= timestamp)
    }
}

impl Default for WatermarkTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Watermark alignment that waits for watermarks to align
pub struct WatermarkAlignment {
    tracker: WatermarkTracker,
    alignment_timeout: Duration,
}

impl WatermarkAlignment {
    /// Create a new watermark alignment
    pub fn new(timeout: Duration) -> Self {
        Self {
            tracker: WatermarkTracker::new(),
            alignment_timeout: timeout,
        }
    }

    /// Wait for watermark to advance to at least the given timestamp
    pub async fn wait_for(&self, timestamp: Timestamp) -> Result<()> {
        let start = std::time::Instant::now();

        loop {
            let current = self.tracker.get().await;
            if current.timestamp >= timestamp {
                return Ok(());
            }

            if start.elapsed() >= self.alignment_timeout {
                return Err(StreamingError::Watermark(format!(
                    "Watermark alignment timeout waiting for {:?}",
                    timestamp
                )));
            }

            // Wait a bit before checking again
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    /// Get the tracker
    pub fn tracker(&self) -> &WatermarkTracker {
        &self.tracker
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp() {
        let ts1 = Timestamp::from_millis(1000);
        let ts2 = Timestamp::from_millis(2000);

        assert!(ts1 < ts2);
        assert_eq!(ts1.add(Duration::from_secs(1)), ts2);
    }

    #[test]
    fn test_watermark() {
        let wm1 = Watermark::from_millis(1000);
        let wm2 = Watermark::from_millis(2000);

        assert!(wm1 < wm2);
        assert!(!wm1.is_late(Timestamp::from_millis(1500), Duration::from_secs(1)));
        assert!(wm2.is_late(Timestamp::from_millis(500), Duration::from_secs(1)));
    }

    #[test]
    fn test_bounded_out_of_orderness_strategy() {
        let mut strategy = BoundedOutOfOrdernessStrategy::new(Duration::from_secs(5));

        let wm1 = strategy.generate(Timestamp::from_millis(10000));
        assert!(wm1.is_some());
        assert_eq!(wm1.unwrap().timestamp.as_millis(), 5000);

        let wm2 = strategy.generate(Timestamp::from_millis(15000));
        assert!(wm2.is_some());
        assert_eq!(wm2.unwrap().timestamp.as_millis(), 10000);
    }

    #[tokio::test]
    async fn test_watermark_tracker() {
        let tracker = WatermarkTracker::new();

        tracker
            .update("source1", Watermark::from_millis(1000))
            .await
            .unwrap();
        assert_eq!(tracker.get().await.timestamp.as_millis(), 1000);

        tracker
            .update("source2", Watermark::from_millis(500))
            .await
            .unwrap();
        // Global watermark should be minimum
        assert_eq!(tracker.get().await.timestamp.as_millis(), 500);

        tracker
            .update("source2", Watermark::from_millis(1500))
            .await
            .unwrap();
        assert_eq!(tracker.get().await.timestamp.as_millis(), 1000);
    }
}
