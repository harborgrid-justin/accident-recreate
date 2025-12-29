//! Session Management Module

pub mod token;
pub mod refresh;

use crate::{SSOUser, SSOError, SSOResult, AuthenticationResult, config::SessionConfig};
use chrono::{Utc, Duration, DateTime};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub use token::TokenManager;
pub use refresh::RefreshTokenManager;

/// Session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Session ID
    pub id: Uuid,

    /// User ID
    pub user_id: String,

    /// User email
    pub email: String,

    /// Created timestamp
    pub created_at: DateTime<Utc>,

    /// Last accessed timestamp
    pub last_accessed: DateTime<Utc>,

    /// Expires at timestamp
    pub expires_at: DateTime<Utc>,

    /// IP address
    pub ip_address: Option<String>,

    /// User agent
    pub user_agent: Option<String>,

    /// MFA verified
    pub mfa_verified: bool,

    /// Session metadata
    pub metadata: serde_json::Value,
}

impl Session {
    /// Check if session is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Check if session needs refresh
    pub fn needs_refresh(&self, threshold_seconds: i64) -> bool {
        let threshold = Duration::seconds(threshold_seconds);
        Utc::now() + threshold > self.expires_at
    }
}

/// Session manager
pub struct SessionManager {
    config: SessionConfig,
    token_manager: TokenManager,
    refresh_manager: RefreshTokenManager,
    sessions: Arc<RwLock<HashMap<Uuid, Session>>>,
    user_sessions: Arc<RwLock<HashMap<String, Vec<Uuid>>>>,
}

impl SessionManager {
    /// Create new session manager
    pub fn new(config: SessionConfig) -> Self {
        Self {
            token_manager: TokenManager::new(config.clone()),
            refresh_manager: RefreshTokenManager::new(config.clone()),
            config,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            user_sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create new session
    pub async fn create_session(&self, user: &SSOUser) -> SSOResult<AuthenticationResult> {
        let session_id = Uuid::new_v4();
        let now = Utc::now();

        // Create session
        let session = Session {
            id: session_id,
            user_id: user.id.clone(),
            email: user.email.clone(),
            created_at: now,
            last_accessed: now,
            expires_at: now + Duration::seconds(self.config.session_timeout),
            ip_address: None,
            user_agent: None,
            mfa_verified: false, // Will be set to true after MFA verification
            metadata: user.metadata.clone(),
        };

        // Store session
        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(session_id, session.clone());
        }

        // Track user sessions
        {
            let mut user_sessions = self.user_sessions.write().await;
            let sessions = user_sessions.entry(user.id.clone()).or_insert_with(Vec::new);
            sessions.push(session_id);

            // Enforce max sessions per user
            if sessions.len() > self.config.max_sessions_per_user {
                let old_session_id = sessions.remove(0);
                // Remove old session
                let mut all_sessions = self.sessions.write().await;
                all_sessions.remove(&old_session_id);
            }
        }

        // Generate tokens
        let access_token = self.token_manager.create_access_token(user, &session_id)?;
        let refresh_token = self.refresh_manager.create_refresh_token(user, &session_id)?;

        Ok(AuthenticationResult {
            user: user.clone(),
            access_token,
            refresh_token: Some(refresh_token),
            expires_at: now + Duration::seconds(self.config.access_token_expiry),
            id_token: None,
            session_id,
            mfa_required: false, // TODO: Check if user has MFA enabled
            mfa_token: None,
        })
    }

    /// Validate access token and return user
    pub async fn validate_token(&self, token: &str) -> SSOResult<SSOUser> {
        self.token_manager.validate_access_token(token).await
    }

    /// Refresh session using refresh token
    pub async fn refresh_session(&self, refresh_token: &str) -> SSOResult<AuthenticationResult> {
        // Validate refresh token
        let (user, session_id) = self.refresh_manager.validate_refresh_token(refresh_token).await?;

        // Check if session exists and is valid
        {
            let sessions = self.sessions.read().await;
            let session = sessions.get(&session_id)
                .ok_or(SSOError::SessionNotFound)?;

            if session.is_expired() {
                return Err(SSOError::SessionExpired);
            }
        }

        // Update session last accessed
        {
            let mut sessions = self.sessions.write().await;
            if let Some(session) = sessions.get_mut(&session_id) {
                session.last_accessed = Utc::now();
            }
        }

        // Generate new tokens
        let now = Utc::now();
        let access_token = self.token_manager.create_access_token(&user, &session_id)?;
        let new_refresh_token = self.refresh_manager.create_refresh_token(&user, &session_id)?;

        Ok(AuthenticationResult {
            user,
            access_token,
            refresh_token: Some(new_refresh_token),
            expires_at: now + Duration::seconds(self.config.access_token_expiry),
            id_token: None,
            session_id,
            mfa_required: false,
            mfa_token: None,
        })
    }

    /// Get session by ID
    pub async fn get_session(&self, session_id: &Uuid) -> SSOResult<Session> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id)
            .cloned()
            .ok_or(SSOError::SessionNotFound)
    }

    /// Invalidate session
    pub async fn invalidate_session(&self, session_id: &Uuid) -> SSOResult<()> {
        // Remove session
        let user_id = {
            let mut sessions = self.sessions.write().await;
            sessions.remove(session_id)
                .map(|s| s.user_id)
                .ok_or(SSOError::SessionNotFound)?
        };

        // Remove from user sessions
        {
            let mut user_sessions = self.user_sessions.write().await;
            if let Some(sessions) = user_sessions.get_mut(&user_id) {
                sessions.retain(|id| id != session_id);
            }
        }

        Ok(())
    }

    /// Invalidate all sessions for a user
    pub async fn invalidate_user_sessions(&self, user_id: &str) -> SSOResult<()> {
        let session_ids = {
            let mut user_sessions = self.user_sessions.write().await;
            user_sessions.remove(user_id).unwrap_or_default()
        };

        let mut sessions = self.sessions.write().await;
        for session_id in session_ids {
            sessions.remove(&session_id);
        }

        Ok(())
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired_sessions(&self) -> usize {
        let now = Utc::now();
        let mut sessions = self.sessions.write().await;
        let mut user_sessions = self.user_sessions.write().await;

        let expired_sessions: Vec<Uuid> = sessions
            .iter()
            .filter(|(_, session)| session.is_expired())
            .map(|(id, _)| *id)
            .collect();

        let count = expired_sessions.len();

        for session_id in expired_sessions {
            if let Some(session) = sessions.remove(&session_id) {
                if let Some(user_session_ids) = user_sessions.get_mut(&session.user_id) {
                    user_session_ids.retain(|id| id != &session_id);
                }
            }
        }

        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_expiration() {
        let session = Session {
            id: Uuid::new_v4(),
            user_id: "user123".to_string(),
            email: "user@example.com".to_string(),
            created_at: Utc::now() - Duration::hours(2),
            last_accessed: Utc::now() - Duration::hours(1),
            expires_at: Utc::now() - Duration::minutes(30),
            ip_address: None,
            user_agent: None,
            mfa_verified: true,
            metadata: serde_json::json!({}),
        };

        assert!(session.is_expired());
    }
}
