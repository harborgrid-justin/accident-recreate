//! User presence tracking system.

use crate::error::{Result, StreamingError};
use crate::event::{PresenceStatus, UserId};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

/// User presence information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceInfo {
    /// User ID
    pub user_id: UserId,
    /// Current status
    pub status: PresenceStatus,
    /// Last activity timestamp
    pub last_active: DateTime<Utc>,
    /// Additional metadata
    pub metadata: Option<serde_json::Value>,
    /// Session start time
    pub session_start: DateTime<Utc>,
}

impl PresenceInfo {
    /// Create a new presence info
    pub fn new(user_id: impl Into<UserId>) -> Self {
        let now = Utc::now();
        Self {
            user_id: user_id.into(),
            status: PresenceStatus::Online,
            last_active: now,
            metadata: None,
            session_start: now,
        }
    }

    /// Update status
    pub fn update_status(&mut self, status: PresenceStatus) {
        self.status = status;
        self.last_active = Utc::now();
    }

    /// Mark as active
    pub fn mark_active(&mut self) {
        self.status = PresenceStatus::Online;
        self.last_active = Utc::now();
    }

    /// Set metadata
    pub fn set_metadata(&mut self, metadata: serde_json::Value) {
        self.metadata = Some(metadata);
        self.last_active = Utc::now();
    }

    /// Get session duration
    pub fn session_duration(&self) -> chrono::Duration {
        Utc::now() - self.session_start
    }

    /// Get idle duration
    pub fn idle_duration(&self) -> chrono::Duration {
        Utc::now() - self.last_active
    }
}

/// Presence tracker for managing user presence
pub struct PresenceTracker {
    /// Active user presences
    presences: Arc<DashMap<UserId, PresenceInfo>>,
    /// Idle timeout duration
    idle_timeout: Duration,
    /// Offline timeout duration
    offline_timeout: Duration,
}

impl PresenceTracker {
    /// Create a new presence tracker
    pub fn new() -> Self {
        Self {
            presences: Arc::new(DashMap::new()),
            idle_timeout: Duration::from_secs(300), // 5 minutes
            offline_timeout: Duration::from_secs(600), // 10 minutes
        }
    }

    /// Create a tracker with custom timeouts
    pub fn with_timeouts(idle_timeout: Duration, offline_timeout: Duration) -> Self {
        Self {
            presences: Arc::new(DashMap::new()),
            idle_timeout,
            offline_timeout,
        }
    }

    /// Update user presence
    pub fn update(&self, user_id: impl Into<UserId>, status: PresenceStatus) -> Result<()> {
        let user_id = user_id.into();

        if let Some(mut presence) = self.presences.get_mut(&user_id) {
            presence.update_status(status);
        } else {
            let mut info = PresenceInfo::new(user_id.clone());
            info.status = status;
            self.presences.insert(user_id, info);
        }

        Ok(())
    }

    /// Mark user as active
    pub fn mark_active(&self, user_id: impl Into<UserId>) -> Result<()> {
        let user_id = user_id.into();

        if let Some(mut presence) = self.presences.get_mut(&user_id) {
            presence.mark_active();
        } else {
            self.presences.insert(user_id.clone(), PresenceInfo::new(user_id));
        }

        Ok(())
    }

    /// Set user metadata
    pub fn set_metadata(
        &self,
        user_id: impl Into<UserId>,
        metadata: serde_json::Value,
    ) -> Result<()> {
        let user_id = user_id.into();

        if let Some(mut presence) = self.presences.get_mut(&user_id) {
            presence.set_metadata(metadata);
        } else {
            let mut info = PresenceInfo::new(user_id.clone());
            info.metadata = Some(metadata);
            self.presences.insert(user_id, info);
        }

        Ok(())
    }

    /// Get user presence
    pub fn get(&self, user_id: &UserId) -> Result<PresenceInfo> {
        self.presences
            .get(user_id)
            .map(|p| p.clone())
            .ok_or_else(|| StreamingError::Presence(format!("User {} not found", user_id)))
    }

    /// Remove user presence
    pub fn remove(&self, user_id: &UserId) -> Result<()> {
        self.presences.remove(user_id);
        Ok(())
    }

    /// Get all online users
    pub fn get_online_users(&self) -> Vec<UserId> {
        self.presences
            .iter()
            .filter(|entry| entry.status == PresenceStatus::Online)
            .map(|entry| entry.user_id.clone())
            .collect()
    }

    /// Get all presences
    pub fn get_all(&self) -> Vec<PresenceInfo> {
        self.presences
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Update idle and offline statuses based on timeouts
    pub fn update_timeouts(&self) {
        let now = Utc::now();

        for mut entry in self.presences.iter_mut() {
            let presence = entry.value_mut();

            let idle_duration = now - presence.last_active;

            // Check offline timeout
            if idle_duration.num_seconds() as u64 >= self.offline_timeout.as_secs() {
                if presence.status != PresenceStatus::Offline {
                    presence.status = PresenceStatus::Offline;
                }
            }
            // Check idle timeout
            else if idle_duration.num_seconds() as u64 >= self.idle_timeout.as_secs() {
                if presence.status == PresenceStatus::Online {
                    presence.status = PresenceStatus::Idle;
                }
            }
        }
    }

    /// Start automatic timeout updates
    pub fn start_timeout_updater(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));

            loop {
                interval.tick().await;
                self.update_timeouts();
            }
        })
    }

    /// Get total user count
    pub fn count(&self) -> usize {
        self.presences.len()
    }

    /// Clear all presences
    pub fn clear(&self) {
        self.presences.clear();
    }
}

impl Default for PresenceTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Presence configuration
#[derive(Debug, Clone)]
pub struct PresenceConfig {
    /// Idle timeout
    pub idle_timeout: Duration,
    /// Offline timeout
    pub offline_timeout: Duration,
    /// Update interval
    pub update_interval: Duration,
}

impl Default for PresenceConfig {
    fn default() -> Self {
        Self {
            idle_timeout: Duration::from_secs(300),
            offline_timeout: Duration::from_secs(600),
            update_interval: Duration::from_secs(30),
        }
    }
}

impl PresenceConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_idle_timeout(mut self, timeout: Duration) -> Self {
        self.idle_timeout = timeout;
        self
    }

    pub fn with_offline_timeout(mut self, timeout: Duration) -> Self {
        self.offline_timeout = timeout;
        self
    }

    pub fn with_update_interval(mut self, interval: Duration) -> Self {
        self.update_interval = interval;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_presence_info() {
        let mut info = PresenceInfo::new("user123");
        assert_eq!(info.user_id, "user123");
        assert_eq!(info.status, PresenceStatus::Online);

        info.update_status(PresenceStatus::Away);
        assert_eq!(info.status, PresenceStatus::Away);
    }

    #[test]
    fn test_presence_tracker() {
        let tracker = PresenceTracker::new();

        tracker.mark_active("user1").unwrap();
        tracker.mark_active("user2").unwrap();

        assert_eq!(tracker.count(), 2);

        let presence = tracker.get(&"user1".to_string()).unwrap();
        assert_eq!(presence.user_id, "user1");
        assert_eq!(presence.status, PresenceStatus::Online);
    }

    #[test]
    fn test_update_status() {
        let tracker = PresenceTracker::new();

        tracker.mark_active("user1").unwrap();
        tracker.update("user1", PresenceStatus::Away).unwrap();

        let presence = tracker.get(&"user1".to_string()).unwrap();
        assert_eq!(presence.status, PresenceStatus::Away);
    }

    #[test]
    fn test_set_metadata() {
        let tracker = PresenceTracker::new();

        tracker.mark_active("user1").unwrap();
        tracker
            .set_metadata("user1", serde_json::json!({"foo": "bar"}))
            .unwrap();

        let presence = tracker.get(&"user1".to_string()).unwrap();
        assert!(presence.metadata.is_some());
    }

    #[test]
    fn test_get_online_users() {
        let tracker = PresenceTracker::new();

        tracker.mark_active("user1").unwrap();
        tracker.mark_active("user2").unwrap();
        tracker.update("user2", PresenceStatus::Away).unwrap();

        let online = tracker.get_online_users();
        assert_eq!(online.len(), 1);
        assert_eq!(online[0], "user1");
    }
}
