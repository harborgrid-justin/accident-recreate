//! Screen reader announcement system for live updates

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

/// Announcement priority levels (maps to ARIA live politeness)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AnnouncementPriority {
    /// Polite - announce when user is idle
    Low,
    /// Polite - normal priority
    Normal,
    /// Assertive - interrupt user
    High,
    /// Assertive - critical announcement
    Critical,
}

impl AnnouncementPriority {
    /// Convert to ARIA live politeness level
    pub fn to_aria_live(&self) -> &'static str {
        match self {
            Self::Low | Self::Normal => "polite",
            Self::High | Self::Critical => "assertive",
        }
    }
}

/// Screen reader announcement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Announcement {
    /// Announcement message
    pub message: String,
    /// Priority level
    pub priority: AnnouncementPriority,
    /// Region where announcement should appear
    pub region: LiveRegion,
    /// Whether the announcement is atomic (read as whole)
    pub atomic: bool,
    /// Timestamp
    pub timestamp: u64,
}

impl Announcement {
    /// Create a new announcement
    pub fn new(message: impl Into<String>, priority: AnnouncementPriority) -> Self {
        Self {
            message: message.into(),
            priority,
            region: LiveRegion::Status,
            atomic: true,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    /// Set the live region
    pub fn with_region(mut self, region: LiveRegion) -> Self {
        self.region = region;
        self
    }

    /// Set whether announcement is atomic
    pub fn with_atomic(mut self, atomic: bool) -> Self {
        self.atomic = atomic;
        self
    }
}

/// ARIA live regions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LiveRegion {
    /// Status messages (polite)
    Status,
    /// Alert messages (assertive)
    Alert,
    /// Log messages (additions only)
    Log,
    /// Timer updates
    Timer,
    /// Marquee/ticker updates
    Marquee,
}

impl LiveRegion {
    /// Get ARIA role for this region
    pub fn aria_role(&self) -> &'static str {
        match self {
            Self::Status => "status",
            Self::Alert => "alert",
            Self::Log => "log",
            Self::Timer => "timer",
            Self::Marquee => "marquee",
        }
    }

    /// Get default ARIA live politeness
    pub fn default_live(&self) -> &'static str {
        match self {
            Self::Status | Self::Log | Self::Timer | Self::Marquee => "polite",
            Self::Alert => "assertive",
        }
    }
}

/// Screen reader announcer
#[derive(Clone)]
pub struct ScreenReaderAnnouncer {
    /// Announcement queue
    queue: Arc<Mutex<VecDeque<Announcement>>>,
    /// Broadcast channel for announcements
    tx: broadcast::Sender<Announcement>,
    /// Maximum queue size
    max_queue_size: usize,
}

impl ScreenReaderAnnouncer {
    /// Create a new screen reader announcer
    pub fn new(max_queue_size: usize) -> Self {
        let (tx, _) = broadcast::channel(100);

        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            tx,
            max_queue_size,
        }
    }

    /// Announce a message
    pub fn announce(&self, announcement: Announcement) -> Result<()> {
        let mut queue = self.queue.lock().unwrap();

        // Remove oldest if queue is full
        if queue.len() >= self.max_queue_size {
            queue.pop_front();
        }

        // Add to queue
        queue.push_back(announcement.clone());

        // Broadcast to listeners
        let _ = self.tx.send(announcement);

        Ok(())
    }

    /// Announce a simple message
    pub fn announce_message(
        &self,
        message: impl Into<String>,
        priority: AnnouncementPriority,
    ) -> Result<()> {
        let announcement = Announcement::new(message, priority);
        self.announce(announcement)
    }

    /// Announce success message
    pub fn announce_success(&self, message: impl Into<String>) -> Result<()> {
        self.announce_message(message, AnnouncementPriority::Normal)
    }

    /// Announce error message
    pub fn announce_error(&self, message: impl Into<String>) -> Result<()> {
        let announcement = Announcement::new(message, AnnouncementPriority::Critical)
            .with_region(LiveRegion::Alert);
        self.announce(announcement)
    }

    /// Announce warning message
    pub fn announce_warning(&self, message: impl Into<String>) -> Result<()> {
        let announcement = Announcement::new(message, AnnouncementPriority::High)
            .with_region(LiveRegion::Alert);
        self.announce(announcement)
    }

    /// Announce info message
    pub fn announce_info(&self, message: impl Into<String>) -> Result<()> {
        self.announce_message(message, AnnouncementPriority::Low)
    }

    /// Get next announcement from queue
    pub fn next(&self) -> Option<Announcement> {
        let mut queue = self.queue.lock().unwrap();
        queue.pop_front()
    }

    /// Get all pending announcements
    pub fn drain(&self) -> Vec<Announcement> {
        let mut queue = self.queue.lock().unwrap();
        queue.drain(..).collect()
    }

    /// Clear all announcements
    pub fn clear(&self) {
        let mut queue = self.queue.lock().unwrap();
        queue.clear();
    }

    /// Subscribe to announcements
    pub fn subscribe(&self) -> broadcast::Receiver<Announcement> {
        self.tx.subscribe()
    }

    /// Get queue size
    pub fn queue_size(&self) -> usize {
        let queue = self.queue.lock().unwrap();
        queue.len()
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        let queue = self.queue.lock().unwrap();
        queue.is_empty()
    }
}

impl Default for ScreenReaderAnnouncer {
    fn default() -> Self {
        Self::new(50)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_announcement_creation() {
        let announcement = Announcement::new("Test message", AnnouncementPriority::Normal);
        assert_eq!(announcement.message, "Test message");
        assert_eq!(announcement.priority, AnnouncementPriority::Normal);
        assert!(announcement.atomic);
    }

    #[test]
    fn test_announcer() {
        let announcer = ScreenReaderAnnouncer::new(10);

        announcer.announce_success("Operation completed").unwrap();
        announcer.announce_error("An error occurred").unwrap();

        assert_eq!(announcer.queue_size(), 2);

        let first = announcer.next().unwrap();
        assert_eq!(first.message, "Operation completed");

        assert_eq!(announcer.queue_size(), 1);
    }

    #[test]
    fn test_max_queue_size() {
        let announcer = ScreenReaderAnnouncer::new(3);

        announcer.announce_message("Message 1", AnnouncementPriority::Normal).unwrap();
        announcer.announce_message("Message 2", AnnouncementPriority::Normal).unwrap();
        announcer.announce_message("Message 3", AnnouncementPriority::Normal).unwrap();
        announcer.announce_message("Message 4", AnnouncementPriority::Normal).unwrap();

        assert_eq!(announcer.queue_size(), 3);

        let first = announcer.next().unwrap();
        assert_eq!(first.message, "Message 2"); // Message 1 was removed
    }

    #[test]
    fn test_priority_aria_live() {
        assert_eq!(AnnouncementPriority::Low.to_aria_live(), "polite");
        assert_eq!(AnnouncementPriority::Normal.to_aria_live(), "polite");
        assert_eq!(AnnouncementPriority::High.to_aria_live(), "assertive");
        assert_eq!(AnnouncementPriority::Critical.to_aria_live(), "assertive");
    }
}
