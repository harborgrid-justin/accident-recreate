//! Session management with timeout and renewal
//!
//! Provides secure session handling with configurable policies.

use crate::config::SessionConfig;
use crate::error::{Result, SecurityError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Session manager
pub struct SessionManager {
    config: SessionConfig,
    sessions: HashMap<String, Session>,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(config: SessionConfig) -> Self {
        Self {
            config,
            sessions: HashMap::new(),
        }
    }

    /// Create a new session
    pub fn create_session(&mut self, user_id: String, metadata: SessionMetadata) -> Result<Session> {
        // Check concurrent session limit
        if let Some(max) = self.config.max_concurrent_sessions {
            let user_sessions = self.sessions.values()
                .filter(|s| s.user_id == user_id && s.is_valid())
                .count();

            if user_sessions >= max {
                return Err(SecurityError::AuthenticationFailed(
                    "Maximum concurrent sessions reached".to_string(),
                ));
            }
        }

        let session = Session::new(user_id, metadata, &self.config);
        self.sessions.insert(session.id.clone(), session.clone());

        Ok(session)
    }

    /// Get session by ID
    pub fn get_session(&self, session_id: &str) -> Result<&Session> {
        let session = self.sessions.get(session_id)
            .ok_or(SecurityError::SessionNotFound)?;

        if !session.is_valid() {
            return Err(SecurityError::SessionExpired);
        }

        Ok(session)
    }

    /// Get mutable session by ID
    pub fn get_session_mut(&mut self, session_id: &str) -> Result<&mut Session> {
        let session = self.sessions.get_mut(session_id)
            .ok_or(SecurityError::SessionNotFound)?;

        if !session.is_valid() {
            return Err(SecurityError::SessionExpired);
        }

        Ok(session)
    }

    /// Update session activity
    pub fn touch_session(&mut self, session_id: &str) -> Result<()> {
        let session = self.get_session_mut(session_id)?;
        session.touch();
        Ok(())
    }

    /// Renew session
    pub fn renew_session(&mut self, session_id: &str) -> Result<String> {
        let old_session = self.get_session(session_id)?;

        // Create new session with same user
        let new_session = Session::new(
            old_session.user_id.clone(),
            old_session.metadata.clone(),
            &self.config,
        );

        let new_id = new_session.id.clone();

        // Invalidate old session
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.invalidate();
        }

        // Store new session
        self.sessions.insert(new_id.clone(), new_session);

        Ok(new_id)
    }

    /// Invalidate session
    pub fn invalidate_session(&mut self, session_id: &str) -> Result<()> {
        let session = self.sessions.get_mut(session_id)
            .ok_or(SecurityError::SessionNotFound)?;

        session.invalidate();
        Ok(())
    }

    /// Invalidate all sessions for a user
    pub fn invalidate_user_sessions(&mut self, user_id: &str) {
        for session in self.sessions.values_mut() {
            if session.user_id == user_id {
                session.invalidate();
            }
        }
    }

    /// Clean up expired sessions
    pub fn cleanup_expired(&mut self) -> usize {
        let before_count = self.sessions.len();
        self.sessions.retain(|_, session| session.is_valid());
        before_count - self.sessions.len()
    }

    /// Get active session count for user
    pub fn user_session_count(&self, user_id: &str) -> usize {
        self.sessions.values()
            .filter(|s| s.user_id == user_id && s.is_valid())
            .count()
    }
}

/// Session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Session ID
    pub id: String,
    /// User ID
    pub user_id: String,
    /// Session metadata
    pub metadata: SessionMetadata,
    /// Created at
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last activity
    pub last_activity: chrono::DateTime<chrono::Utc>,
    /// Last renewal
    pub last_renewal: chrono::DateTime<chrono::Utc>,
    /// Expires at (absolute timeout)
    pub expires_at: chrono::DateTime<chrono::Utc>,
    /// Invalidated flag
    pub invalidated: bool,
}

impl Session {
    /// Create a new session
    pub fn new(user_id: String, metadata: SessionMetadata, config: &SessionConfig) -> Self {
        let now = chrono::Utc::now();
        let expires_at = now + chrono::Duration::seconds(config.absolute_timeout_secs as i64);

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            user_id,
            metadata,
            created_at: now,
            last_activity: now,
            last_renewal: now,
            expires_at,
            invalidated: false,
        }
    }

    /// Check if session is still valid
    pub fn is_valid(&self) -> bool {
        if self.invalidated {
            return false;
        }

        let now = chrono::Utc::now();
        now < self.expires_at
    }

    /// Check if session has been idle too long
    pub fn is_idle(&self, idle_timeout_secs: u64) -> bool {
        let now = chrono::Utc::now();
        let idle_duration = now.signed_duration_since(self.last_activity);
        idle_duration.num_seconds() as u64 > idle_timeout_secs
    }

    /// Check if session needs renewal
    pub fn needs_renewal(&self, renewal_interval_secs: u64) -> bool {
        let now = chrono::Utc::now();
        let since_renewal = now.signed_duration_since(self.last_renewal);
        since_renewal.num_seconds() as u64 > renewal_interval_secs
    }

    /// Update last activity timestamp
    pub fn touch(&mut self) {
        self.last_activity = chrono::Utc::now();
    }

    /// Invalidate session
    pub fn invalidate(&mut self) {
        self.invalidated = true;
    }

    /// Get session age in seconds
    pub fn age(&self) -> i64 {
        let now = chrono::Utc::now();
        now.signed_duration_since(self.created_at).num_seconds()
    }

    /// Get idle time in seconds
    pub fn idle_time(&self) -> i64 {
        let now = chrono::Utc::now();
        now.signed_duration_since(self.last_activity).num_seconds()
    }
}

/// Session metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    /// IP address
    pub ip_address: String,
    /// User agent
    pub user_agent: Option<String>,
    /// Device fingerprint
    pub device_fingerprint: Option<String>,
    /// Geographic location
    pub location: Option<GeoLocation>,
    /// Additional custom data
    pub custom_data: HashMap<String, String>,
}

impl SessionMetadata {
    /// Create basic session metadata
    pub fn basic(ip_address: String) -> Self {
        Self {
            ip_address,
            user_agent: None,
            device_fingerprint: None,
            location: None,
            custom_data: HashMap::new(),
        }
    }
}

/// Geographic location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoLocation {
    /// Country code
    pub country: Option<String>,
    /// Region/state
    pub region: Option<String>,
    /// City
    pub city: Option<String>,
    /// Latitude
    pub latitude: Option<f64>,
    /// Longitude
    pub longitude: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> SessionConfig {
        SessionConfig {
            timeout_secs: 3600,
            idle_timeout_secs: 1800,
            absolute_timeout_secs: 28800,
            max_concurrent_sessions: Some(3),
            require_renewal: true,
            renewal_interval_secs: 900,
        }
    }

    #[test]
    fn test_session_creation() {
        let mut manager = SessionManager::new(test_config());
        let metadata = SessionMetadata::basic("192.168.1.1".to_string());

        let session = manager.create_session("user123".to_string(), metadata).unwrap();

        assert!(!session.id.is_empty());
        assert_eq!(session.user_id, "user123");
        assert!(session.is_valid());
    }

    #[test]
    fn test_session_retrieval() {
        let mut manager = SessionManager::new(test_config());
        let metadata = SessionMetadata::basic("192.168.1.1".to_string());

        let session = manager.create_session("user123".to_string(), metadata).unwrap();
        let session_id = session.id.clone();

        let retrieved = manager.get_session(&session_id).unwrap();
        assert_eq!(retrieved.id, session_id);
    }

    #[test]
    fn test_session_touch() {
        let mut manager = SessionManager::new(test_config());
        let metadata = SessionMetadata::basic("192.168.1.1".to_string());

        let session = manager.create_session("user123".to_string(), metadata).unwrap();
        let session_id = session.id.clone();

        std::thread::sleep(std::time::Duration::from_millis(100));

        manager.touch_session(&session_id).unwrap();
        let updated = manager.get_session(&session_id).unwrap();

        assert!(updated.last_activity > session.created_at);
    }

    #[test]
    fn test_session_invalidation() {
        let mut manager = SessionManager::new(test_config());
        let metadata = SessionMetadata::basic("192.168.1.1".to_string());

        let session = manager.create_session("user123".to_string(), metadata).unwrap();
        let session_id = session.id.clone();

        manager.invalidate_session(&session_id).unwrap();

        let result = manager.get_session(&session_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_concurrent_session_limit() {
        let mut manager = SessionManager::new(test_config());
        let metadata = SessionMetadata::basic("192.168.1.1".to_string());

        // Create 3 sessions (the limit)
        for _ in 0..3 {
            manager.create_session("user123".to_string(), metadata.clone()).unwrap();
        }

        // Fourth session should fail
        let result = manager.create_session("user123".to_string(), metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_user_session_invalidation() {
        let mut manager = SessionManager::new(test_config());
        let metadata = SessionMetadata::basic("192.168.1.1".to_string());

        manager.create_session("user123".to_string(), metadata.clone()).unwrap();
        manager.create_session("user123".to_string(), metadata).unwrap();

        assert_eq!(manager.user_session_count("user123"), 2);

        manager.invalidate_user_sessions("user123");

        assert_eq!(manager.user_session_count("user123"), 0);
    }
}
