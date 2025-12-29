//! Session management with refresh tokens
//!
//! Provides secure session management with:
//! - Session creation and validation
//! - Automatic session expiration
//! - Session fingerprinting
//! - Concurrent session limits

use crate::config::SessionConfig;
use crate::error::{SecurityError, SecurityResult};
use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use zeroize::Zeroizing;

/// Session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Session ID
    pub id: String,

    /// User ID
    pub user_id: String,

    /// Created at
    pub created_at: DateTime<Utc>,

    /// Last accessed at
    pub last_accessed_at: DateTime<Utc>,

    /// Expires at
    pub expires_at: DateTime<Utc>,

    /// Session fingerprint (browser, IP, etc.)
    pub fingerprint: Option<String>,

    /// Session data
    #[serde(default)]
    pub data: std::collections::HashMap<String, serde_json::Value>,

    /// Is the session valid
    #[serde(default = "default_true")]
    pub is_valid: bool,
}

fn default_true() -> bool {
    true
}

impl Session {
    /// Create a new session
    pub fn new(user_id: impl Into<String>, expires_in: Duration) -> Self {
        let now = Utc::now();

        Self {
            id: Uuid::new_v4().to_string(),
            user_id: user_id.into(),
            created_at: now,
            last_accessed_at: now,
            expires_at: now + expires_in,
            fingerprint: None,
            data: std::collections::HashMap::new(),
            is_valid: true,
        }
    }

    /// Set fingerprint
    pub fn with_fingerprint(mut self, fingerprint: impl Into<String>) -> Self {
        self.fingerprint = Some(fingerprint.into());
        self
    }

    /// Add session data
    pub fn with_data(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.data.insert(key.into(), value);
        self
    }

    /// Check if session is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Check if session is idle (not accessed for a while)
    pub fn is_idle(&self, idle_timeout: Duration) -> bool {
        Utc::now() > self.last_accessed_at + idle_timeout
    }

    /// Update last accessed time
    pub fn touch(&mut self) {
        self.last_accessed_at = Utc::now();
    }

    /// Invalidate the session
    pub fn invalidate(&mut self) {
        self.is_valid = false;
    }

    /// Extend session expiration
    pub fn extend(&mut self, duration: Duration) {
        self.expires_at = Utc::now() + duration;
    }
}

/// Session manager
#[derive(Debug)]
pub struct SessionManager {
    /// Active sessions (user_id -> list of sessions)
    sessions: Arc<DashMap<String, Vec<Session>>>,

    /// Session lookup by ID
    session_by_id: Arc<DashMap<String, String>>, // session_id -> user_id

    /// Configuration
    config: SessionConfig,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(config: SessionConfig) -> Self {
        Self {
            sessions: Arc::new(DashMap::new()),
            session_by_id: Arc::new(DashMap::new()),
            config,
        }
    }

    /// Create a new session for a user
    pub fn create_session(
        &self,
        user_id: impl Into<String>,
        fingerprint: Option<String>,
    ) -> SecurityResult<Session> {
        let user_id = user_id.into();

        // Check concurrent session limit
        if let Some(user_sessions) = self.sessions.get(&user_id) {
            let active_sessions: Vec<_> = user_sessions
                .iter()
                .filter(|s| s.is_valid && !s.is_expired())
                .collect();

            if active_sessions.len() >= self.config.max_concurrent_sessions as usize {
                return Err(SecurityError::SessionError(
                    "Maximum concurrent sessions exceeded".to_string(),
                ));
            }
        }

        // Create new session
        let mut session = Session::new(
            &user_id,
            Duration::seconds(self.config.timeout_secs),
        );

        if let Some(fp) = fingerprint {
            session = session.with_fingerprint(fp);
        }

        // Store session
        self.session_by_id.insert(session.id.clone(), user_id.clone());

        self.sessions
            .entry(user_id.clone())
            .or_insert_with(Vec::new)
            .push(session.clone());

        Ok(session)
    }

    /// Get a session by ID
    pub fn get_session(&self, session_id: &str) -> SecurityResult<Session> {
        // Find user_id from session_id
        let user_id = self
            .session_by_id
            .get(session_id)
            .ok_or_else(|| SecurityError::SessionError("Session not found".to_string()))?
            .clone();

        // Find session in user's sessions
        let sessions = self
            .sessions
            .get(&user_id)
            .ok_or_else(|| SecurityError::SessionError("Session not found".to_string()))?;

        let session = sessions
            .iter()
            .find(|s| s.id == session_id)
            .ok_or_else(|| SecurityError::SessionError("Session not found".to_string()))?;

        Ok(session.clone())
    }

    /// Validate a session
    pub fn validate_session(
        &self,
        session_id: &str,
        fingerprint: Option<&str>,
    ) -> SecurityResult<Session> {
        let mut session = self.get_session(session_id)?;

        // Check if session is valid
        if !session.is_valid {
            return Err(SecurityError::SessionError("Session is invalid".to_string()));
        }

        // Check expiration
        if session.is_expired() {
            self.invalidate_session(session_id)?;
            return Err(SecurityError::SessionExpired);
        }

        // Check idle timeout
        if session.is_idle(Duration::seconds(self.config.idle_timeout_secs)) {
            self.invalidate_session(session_id)?;
            return Err(SecurityError::SessionExpired);
        }

        // Check fingerprint if enabled
        if self.config.enable_fingerprinting {
            if let Some(stored_fp) = &session.fingerprint {
                if let Some(provided_fp) = fingerprint {
                    if stored_fp != provided_fp {
                        return Err(SecurityError::SessionError(
                            "Session fingerprint mismatch".to_string(),
                        ));
                    }
                } else {
                    return Err(SecurityError::SessionError(
                        "Session fingerprint required".to_string(),
                    ));
                }
            }
        }

        // Update last accessed time
        session.touch();
        self.update_session(&session)?;

        Ok(session)
    }

    /// Update a session
    fn update_session(&self, session: &Session) -> SecurityResult<()> {
        let user_id = session.user_id.clone();

        if let Some(mut sessions) = self.sessions.get_mut(&user_id) {
            if let Some(s) = sessions.iter_mut().find(|s| s.id == session.id) {
                *s = session.clone();
                return Ok(());
            }
        }

        Err(SecurityError::SessionError("Session not found".to_string()))
    }

    /// Invalidate a session
    pub fn invalidate_session(&self, session_id: &str) -> SecurityResult<()> {
        let user_id = self
            .session_by_id
            .get(session_id)
            .ok_or_else(|| SecurityError::SessionError("Session not found".to_string()))?
            .clone();

        if let Some(mut sessions) = self.sessions.get_mut(&user_id) {
            if let Some(session) = sessions.iter_mut().find(|s| s.id == session_id) {
                session.invalidate();
                self.session_by_id.remove(session_id);
                return Ok(());
            }
        }

        Err(SecurityError::SessionError("Session not found".to_string()))
    }

    /// Invalidate all sessions for a user
    pub fn invalidate_user_sessions(&self, user_id: &str) -> SecurityResult<()> {
        if let Some(mut sessions) = self.sessions.get_mut(user_id) {
            for session in sessions.iter_mut() {
                session.invalidate();
                self.session_by_id.remove(&session.id);
            }
            Ok(())
        } else {
            Err(SecurityError::SessionError("User has no sessions".to_string()))
        }
    }

    /// Get all active sessions for a user
    pub fn get_user_sessions(&self, user_id: &str) -> Vec<Session> {
        self.sessions
            .get(user_id)
            .map(|sessions| {
                sessions
                    .iter()
                    .filter(|s| s.is_valid && !s.is_expired())
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Clean up expired sessions
    pub fn cleanup_expired_sessions(&self) -> usize {
        let mut cleaned = 0;

        for mut entry in self.sessions.iter_mut() {
            let sessions = entry.value_mut();
            let original_len = sessions.len();

            sessions.retain(|s| {
                let should_keep = s.is_valid && !s.is_expired();
                if !should_keep {
                    self.session_by_id.remove(&s.id);
                }
                should_keep
            });

            cleaned += original_len - sessions.len();
        }

        cleaned
    }

    /// Get session count
    pub fn session_count(&self) -> usize {
        self.session_by_id.len()
    }

    /// Extend session expiration
    pub fn extend_session(
        &self,
        session_id: &str,
        duration: Duration,
    ) -> SecurityResult<()> {
        let user_id = self
            .session_by_id
            .get(session_id)
            .ok_or_else(|| SecurityError::SessionError("Session not found".to_string()))?
            .clone();

        if let Some(mut sessions) = self.sessions.get_mut(&user_id) {
            if let Some(session) = sessions.iter_mut().find(|s| s.id == session_id) {
                session.extend(duration);
                return Ok(());
            }
        }

        Err(SecurityError::SessionError("Session not found".to_string()))
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new(SessionConfig::default())
    }
}

/// Generate a session fingerprint from request information
pub fn generate_fingerprint(
    ip: &str,
    user_agent: &str,
) -> String {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(ip.as_bytes());
    hasher.update(user_agent.as_bytes());
    let result = hasher.finalize();

    base64::encode(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> SessionConfig {
        SessionConfig {
            timeout_secs: 3600,
            idle_timeout_secs: 1800,
            max_concurrent_sessions: 5,
            enable_fingerprinting: true,
        }
    }

    #[test]
    fn test_session_creation() {
        let manager = SessionManager::new(create_test_config());
        let session = manager.create_session("user123", None).unwrap();

        assert_eq!(session.user_id, "user123");
        assert!(session.is_valid);
        assert!(!session.is_expired());
    }

    #[test]
    fn test_session_validation() {
        let manager = SessionManager::new(create_test_config());
        let session = manager.create_session("user123", Some("fingerprint".to_string())).unwrap();

        let validated = manager
            .validate_session(&session.id, Some("fingerprint"))
            .unwrap();

        assert_eq!(validated.user_id, "user123");
    }

    #[test]
    fn test_session_invalidation() {
        let manager = SessionManager::new(create_test_config());
        let session = manager.create_session("user123", None).unwrap();

        manager.invalidate_session(&session.id).unwrap();

        let result = manager.get_session(&session.id);
        assert!(result.is_ok()); // Session exists but is invalid
        assert!(!result.unwrap().is_valid);
    }

    #[test]
    fn test_concurrent_session_limit() {
        let manager = SessionManager::new(create_test_config());

        // Create max sessions
        for _ in 0..5 {
            manager.create_session("user123", None).unwrap();
        }

        // Try to create one more
        let result = manager.create_session("user123", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_fingerprint_mismatch() {
        let manager = SessionManager::new(create_test_config());
        let session = manager.create_session("user123", Some("fingerprint1".to_string())).unwrap();

        let result = manager.validate_session(&session.id, Some("fingerprint2"));
        assert!(result.is_err());
    }

    #[test]
    fn test_user_sessions() {
        let manager = SessionManager::new(create_test_config());

        manager.create_session("user123", None).unwrap();
        manager.create_session("user123", None).unwrap();

        let sessions = manager.get_user_sessions("user123");
        assert_eq!(sessions.len(), 2);
    }

    #[test]
    fn test_cleanup_expired_sessions() {
        let mut config = create_test_config();
        config.timeout_secs = 1; // 1 second timeout

        let manager = SessionManager::new(config);
        manager.create_session("user123", None).unwrap();

        std::thread::sleep(std::time::Duration::from_secs(2));

        let cleaned = manager.cleanup_expired_sessions();
        assert_eq!(cleaned, 1);
    }

    #[test]
    fn test_generate_fingerprint() {
        let fp1 = generate_fingerprint("192.168.1.1", "Mozilla/5.0");
        let fp2 = generate_fingerprint("192.168.1.1", "Mozilla/5.0");
        let fp3 = generate_fingerprint("192.168.1.2", "Mozilla/5.0");

        assert_eq!(fp1, fp2);
        assert_ne!(fp1, fp3);
    }
}
