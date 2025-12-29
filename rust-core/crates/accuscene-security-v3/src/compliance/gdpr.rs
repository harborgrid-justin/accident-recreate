//! GDPR compliance utilities

use crate::error::{SecurityError, SecurityResult};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// GDPR compliance manager
#[derive(Debug)]
pub struct GdprCompliance {
    /// Consent manager
    consent_manager: Arc<ConsentManager>,

    /// Data subject requests
    dsr_manager: Arc<DsrManager>,
}

impl GdprCompliance {
    /// Create a new GDPR compliance manager
    pub fn new() -> Self {
        Self {
            consent_manager: Arc::new(ConsentManager::new()),
            dsr_manager: Arc::new(DsrManager::new()),
        }
    }

    /// Get consent manager
    pub fn consent_manager(&self) -> &Arc<ConsentManager> {
        &self.consent_manager
    }

    /// Get DSR manager
    pub fn dsr_manager(&self) -> &Arc<DsrManager> {
        &self.dsr_manager
    }

    /// Submit a data subject request
    pub fn submit_request(&self, request: DataSubjectRequest) -> SecurityResult<String> {
        self.dsr_manager.submit_request(request)
    }

    /// Get request status
    pub fn get_request_status(&self, request_id: &str) -> Option<RequestStatus> {
        self.dsr_manager.get_request_status(request_id)
    }
}

impl Default for GdprCompliance {
    fn default() -> Self {
        Self::new()
    }
}

/// Consent manager for GDPR
#[derive(Debug)]
pub struct ConsentManager {
    consents: Arc<DashMap<String, UserConsent>>,
}

impl ConsentManager {
    /// Create a new consent manager
    pub fn new() -> Self {
        Self {
            consents: Arc::new(DashMap::new()),
        }
    }

    /// Record user consent
    pub fn record_consent(
        &self,
        user_id: impl Into<String>,
        purpose: ConsentPurpose,
        granted: bool,
    ) -> SecurityResult<()> {
        let user_id = user_id.into();

        let mut consent = self.consents
            .entry(user_id.clone())
            .or_insert_with(|| UserConsent::new(&user_id));

        consent.record(purpose, granted);

        Ok(())
    }

    /// Check if user has consented for a purpose
    pub fn has_consent(&self, user_id: &str, purpose: ConsentPurpose) -> bool {
        self.consents
            .get(user_id)
            .map(|c| c.has_consent(purpose))
            .unwrap_or(false)
    }

    /// Get user consent record
    pub fn get_consent(&self, user_id: &str) -> Option<UserConsent> {
        self.consents.get(user_id).map(|c| c.clone())
    }

    /// Withdraw consent
    pub fn withdraw_consent(
        &self,
        user_id: &str,
        purpose: ConsentPurpose,
    ) -> SecurityResult<()> {
        if let Some(mut consent) = self.consents.get_mut(user_id) {
            consent.record(purpose, false);
            Ok(())
        } else {
            Err(SecurityError::ComplianceError(
                "User consent record not found".to_string(),
            ))
        }
    }

    /// Get all consents for audit
    pub fn get_all_consents(&self) -> Vec<UserConsent> {
        self.consents.iter().map(|entry| entry.value().clone()).collect()
    }
}

impl Default for ConsentManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Consent purposes under GDPR
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConsentPurpose {
    /// Marketing communications
    Marketing,
    /// Analytics and performance monitoring
    Analytics,
    /// Personalization
    Personalization,
    /// Third-party sharing
    ThirdPartySharing,
    /// Essential service functionality
    Essential,
    /// Custom purpose
    Custom,
}

/// User consent record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConsent {
    /// User ID
    pub user_id: String,

    /// Consent records by purpose
    pub consents: HashMap<ConsentPurpose, ConsentRecord>,

    /// Created at
    pub created_at: DateTime<Utc>,

    /// Updated at
    pub updated_at: DateTime<Utc>,
}

impl UserConsent {
    /// Create new user consent
    pub fn new(user_id: impl Into<String>) -> Self {
        let now = Utc::now();

        Self {
            user_id: user_id.into(),
            consents: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Record consent
    pub fn record(&mut self, purpose: ConsentPurpose, granted: bool) {
        self.consents.insert(
            purpose,
            ConsentRecord {
                granted,
                recorded_at: Utc::now(),
            },
        );
        self.updated_at = Utc::now();
    }

    /// Check if user has consented
    pub fn has_consent(&self, purpose: ConsentPurpose) -> bool {
        self.consents
            .get(&purpose)
            .map(|c| c.granted)
            .unwrap_or(false)
    }
}

/// Individual consent record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentRecord {
    /// Whether consent was granted
    pub granted: bool,

    /// When consent was recorded
    pub recorded_at: DateTime<Utc>,
}

/// Data subject request types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequestType {
    /// Right to access (Article 15)
    Access,
    /// Right to rectification (Article 16)
    Rectification,
    /// Right to erasure / "Right to be forgotten" (Article 17)
    Erasure,
    /// Right to restriction of processing (Article 18)
    Restriction,
    /// Right to data portability (Article 20)
    Portability,
    /// Right to object (Article 21)
    Objection,
}

/// Data subject request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSubjectRequest {
    /// Request ID
    pub id: String,

    /// User ID
    pub user_id: String,

    /// Request type
    pub request_type: RequestType,

    /// Request details
    pub details: Option<String>,

    /// Submitted at
    pub submitted_at: DateTime<Utc>,

    /// Status
    pub status: RequestStatus,

    /// Completed at
    pub completed_at: Option<DateTime<Utc>>,
}

impl DataSubjectRequest {
    /// Create a new data subject request
    pub fn new(
        user_id: impl Into<String>,
        request_type: RequestType,
        details: Option<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            user_id: user_id.into(),
            request_type,
            details,
            submitted_at: Utc::now(),
            status: RequestStatus::Pending,
            completed_at: None,
        }
    }
}

/// Request status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequestStatus {
    /// Request is pending
    Pending,
    /// Request is being processed
    Processing,
    /// Request is completed
    Completed,
    /// Request was rejected
    Rejected,
}

/// Data subject request manager
#[derive(Debug)]
pub struct DsrManager {
    requests: Arc<DashMap<String, DataSubjectRequest>>,
}

impl DsrManager {
    /// Create a new DSR manager
    pub fn new() -> Self {
        Self {
            requests: Arc::new(DashMap::new()),
        }
    }

    /// Submit a data subject request
    pub fn submit_request(&self, request: DataSubjectRequest) -> SecurityResult<String> {
        let id = request.id.clone();
        self.requests.insert(id.clone(), request);
        Ok(id)
    }

    /// Get request status
    pub fn get_request_status(&self, request_id: &str) -> Option<RequestStatus> {
        self.requests.get(request_id).map(|r| r.status)
    }

    /// Update request status
    pub fn update_status(
        &self,
        request_id: &str,
        status: RequestStatus,
    ) -> SecurityResult<()> {
        if let Some(mut request) = self.requests.get_mut(request_id) {
            request.status = status;
            if status == RequestStatus::Completed || status == RequestStatus::Rejected {
                request.completed_at = Some(Utc::now());
            }
            Ok(())
        } else {
            Err(SecurityError::ComplianceError(
                "Request not found".to_string(),
            ))
        }
    }

    /// Get all requests for a user
    pub fn get_user_requests(&self, user_id: &str) -> Vec<DataSubjectRequest> {
        self.requests
            .iter()
            .filter(|entry| entry.value().user_id == user_id)
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get all pending requests
    pub fn get_pending_requests(&self) -> Vec<DataSubjectRequest> {
        self.requests
            .iter()
            .filter(|entry| entry.value().status == RequestStatus::Pending)
            .map(|entry| entry.value().clone())
            .collect()
    }
}

impl Default for DsrManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consent_recording() {
        let manager = ConsentManager::new();

        manager
            .record_consent("user123", ConsentPurpose::Marketing, true)
            .unwrap();

        assert!(manager.has_consent("user123", ConsentPurpose::Marketing));
        assert!(!manager.has_consent("user123", ConsentPurpose::Analytics));
    }

    #[test]
    fn test_consent_withdrawal() {
        let manager = ConsentManager::new();

        manager
            .record_consent("user123", ConsentPurpose::Marketing, true)
            .unwrap();

        assert!(manager.has_consent("user123", ConsentPurpose::Marketing));

        manager
            .withdraw_consent("user123", ConsentPurpose::Marketing)
            .unwrap();

        assert!(!manager.has_consent("user123", ConsentPurpose::Marketing));
    }

    #[test]
    fn test_data_subject_request() {
        let compliance = GdprCompliance::new();

        let request = DataSubjectRequest::new(
            "user123",
            RequestType::Access,
            Some("I want to see my data".to_string()),
        );

        let request_id = compliance.submit_request(request).unwrap();

        let status = compliance.get_request_status(&request_id).unwrap();
        assert_eq!(status, RequestStatus::Pending);
    }

    #[test]
    fn test_request_status_update() {
        let manager = DsrManager::new();

        let request = DataSubjectRequest::new("user123", RequestType::Erasure, None);
        let request_id = manager.submit_request(request).unwrap();

        manager
            .update_status(&request_id, RequestStatus::Processing)
            .unwrap();

        assert_eq!(
            manager.get_request_status(&request_id).unwrap(),
            RequestStatus::Processing
        );
    }

    #[test]
    fn test_get_user_requests() {
        let manager = DsrManager::new();

        manager
            .submit_request(DataSubjectRequest::new("user123", RequestType::Access, None))
            .unwrap();

        manager
            .submit_request(DataSubjectRequest::new("user123", RequestType::Erasure, None))
            .unwrap();

        let requests = manager.get_user_requests("user123");
        assert_eq!(requests.len(), 2);
    }
}
