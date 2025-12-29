//! Data retention policies and management

use crate::error::{SecurityError, SecurityResult};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Data retention policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRetentionPolicy {
    /// Policy ID
    pub id: String,

    /// Policy name
    pub name: String,

    /// Data type this policy applies to
    pub data_type: String,

    /// Retention period in days
    pub retention_days: u32,

    /// Whether to delete or archive after retention period
    pub action: RetentionAction,

    /// Legal hold (prevents deletion even after retention period)
    pub legal_hold: bool,

    /// Created at
    pub created_at: DateTime<Utc>,

    /// Updated at
    pub updated_at: DateTime<Utc>,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Action to take after retention period
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetentionAction {
    /// Delete the data permanently
    Delete,
    /// Archive the data (move to cold storage)
    Archive,
    /// Anonymize the data (remove PII)
    Anonymize,
}

impl DataRetentionPolicy {
    /// Create a new retention policy
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        data_type: impl Into<String>,
        retention_days: u32,
        action: RetentionAction,
    ) -> Self {
        let now = Utc::now();

        Self {
            id: id.into(),
            name: name.into(),
            data_type: data_type.into(),
            retention_days,
            action,
            legal_hold: false,
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        }
    }

    /// Set legal hold
    pub fn with_legal_hold(mut self, hold: bool) -> Self {
        self.legal_hold = hold;
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Check if data should be retained based on creation date
    pub fn should_retain(&self, created_at: DateTime<Utc>) -> bool {
        if self.legal_hold {
            return true;
        }

        let expiry = created_at + Duration::days(self.retention_days as i64);
        Utc::now() < expiry
    }

    /// Get expiry date for data created at a given time
    pub fn get_expiry_date(&self, created_at: DateTime<Utc>) -> DateTime<Utc> {
        created_at + Duration::days(self.retention_days as i64)
    }
}

/// Retention manager
#[derive(Debug)]
pub struct RetentionManager {
    policies: HashMap<String, DataRetentionPolicy>,
    default_retention_days: u32,
}

impl RetentionManager {
    /// Create a new retention manager
    pub fn new(default_retention_days: u32) -> Self {
        Self {
            policies: HashMap::new(),
            default_retention_days,
        }
    }

    /// Add a retention policy
    pub fn add_policy(&mut self, policy: DataRetentionPolicy) {
        self.policies.insert(policy.data_type.clone(), policy);
    }

    /// Get policy for a data type
    pub fn get_policy(&self, data_type: &str) -> Option<&DataRetentionPolicy> {
        self.policies.get(data_type)
    }

    /// Get or create default policy for a data type
    pub fn get_or_default_policy(&self, data_type: &str) -> DataRetentionPolicy {
        self.policies
            .get(data_type)
            .cloned()
            .unwrap_or_else(|| {
                DataRetentionPolicy::new(
                    format!("default_{}", data_type),
                    format!("Default policy for {}", data_type),
                    data_type,
                    self.default_retention_days,
                    RetentionAction::Archive,
                )
            })
    }

    /// Check if data should be retained
    pub fn should_retain(
        &self,
        data_type: &str,
        created_at: DateTime<Utc>,
    ) -> bool {
        let policy = self.get_or_default_policy(data_type);
        policy.should_retain(created_at)
    }

    /// Get action to take for expired data
    pub fn get_expiry_action(&self, data_type: &str) -> RetentionAction {
        self.policies
            .get(data_type)
            .map(|p| p.action)
            .unwrap_or(RetentionAction::Archive)
    }

    /// Find all data types that need action
    pub fn find_expired_data_types(&self) -> Vec<String> {
        self.policies
            .iter()
            .filter(|(_, policy)| !policy.legal_hold)
            .map(|(data_type, _)| data_type.clone())
            .collect()
    }

    /// Set legal hold on a data type
    pub fn set_legal_hold(&mut self, data_type: &str, hold: bool) -> SecurityResult<()> {
        if let Some(policy) = self.policies.get_mut(data_type) {
            policy.legal_hold = hold;
            policy.updated_at = Utc::now();
            Ok(())
        } else {
            Err(SecurityError::ComplianceError(format!(
                "No policy found for data type: {}",
                data_type
            )))
        }
    }

    /// Get all policies
    pub fn all_policies(&self) -> Vec<&DataRetentionPolicy> {
        self.policies.values().collect()
    }
}

impl Default for RetentionManager {
    fn default() -> Self {
        let mut manager = Self::new(2555); // 7 years default

        // Add standard policies
        manager.add_policy(
            DataRetentionPolicy::new(
                "user_data",
                "User Personal Data",
                "user_data",
                365, // 1 year
                RetentionAction::Anonymize,
            )
        );

        manager.add_policy(
            DataRetentionPolicy::new(
                "audit_logs",
                "Audit Logs",
                "audit_logs",
                2555, // 7 years for compliance
                RetentionAction::Archive,
            )
        );

        manager.add_policy(
            DataRetentionPolicy::new(
                "financial_records",
                "Financial Records",
                "financial_records",
                2555, // 7 years
                RetentionAction::Archive,
            )
        );

        manager.add_policy(
            DataRetentionPolicy::new(
                "session_data",
                "Session Data",
                "session_data",
                30, // 30 days
                RetentionAction::Delete,
            )
        );

        manager
    }
}

/// Data item with retention tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetainedData {
    /// Data ID
    pub id: String,

    /// Data type
    pub data_type: String,

    /// Created at
    pub created_at: DateTime<Utc>,

    /// Expiry date (based on retention policy)
    pub expires_at: DateTime<Utc>,

    /// Whether data is on legal hold
    pub legal_hold: bool,

    /// Data payload (would be encrypted in production)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl RetainedData {
    /// Create new retained data
    pub fn new(
        id: impl Into<String>,
        data_type: impl Into<String>,
        policy: &DataRetentionPolicy,
        data: Option<serde_json::Value>,
    ) -> Self {
        let created_at = Utc::now();
        let expires_at = policy.get_expiry_date(created_at);

        Self {
            id: id.into(),
            data_type: data_type.into(),
            created_at,
            expires_at,
            legal_hold: false,
            data,
        }
    }

    /// Check if data has expired
    pub fn is_expired(&self) -> bool {
        if self.legal_hold {
            return false;
        }

        Utc::now() > self.expires_at
    }

    /// Set legal hold
    pub fn set_legal_hold(&mut self, hold: bool) {
        self.legal_hold = hold;
    }

    /// Extend retention period
    pub fn extend_retention(&mut self, additional_days: u32) {
        self.expires_at = self.expires_at + Duration::days(additional_days as i64);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retention_policy() {
        let policy = DataRetentionPolicy::new(
            "test_policy",
            "Test Policy",
            "test_data",
            30,
            RetentionAction::Delete,
        );

        let created = Utc::now() - Duration::days(20);
        assert!(policy.should_retain(created));

        let old_created = Utc::now() - Duration::days(40);
        assert!(!policy.should_retain(old_created));
    }

    #[test]
    fn test_legal_hold() {
        let policy = DataRetentionPolicy::new(
            "test_policy",
            "Test Policy",
            "test_data",
            30,
            RetentionAction::Delete,
        )
        .with_legal_hold(true);

        let old_created = Utc::now() - Duration::days(365);
        assert!(policy.should_retain(old_created)); // Should retain due to legal hold
    }

    #[test]
    fn test_retention_manager() {
        let manager = RetentionManager::default();

        let policy = manager.get_policy("audit_logs").unwrap();
        assert_eq!(policy.retention_days, 2555);

        let recent = Utc::now() - Duration::days(100);
        assert!(manager.should_retain("audit_logs", recent));
    }

    #[test]
    fn test_retained_data() {
        let policy = DataRetentionPolicy::new(
            "test",
            "Test",
            "test_data",
            30,
            RetentionAction::Delete,
        );

        let data = RetainedData::new(
            "data123",
            "test_data",
            &policy,
            Some(serde_json::json!({"key": "value"})),
        );

        assert!(!data.is_expired());
        assert!(data.expires_at > Utc::now());
    }

    #[test]
    fn test_extend_retention() {
        let policy = DataRetentionPolicy::new(
            "test",
            "Test",
            "test_data",
            30,
            RetentionAction::Delete,
        );

        let mut data = RetainedData::new("data123", "test_data", &policy, None);
        let original_expiry = data.expires_at;

        data.extend_retention(30);

        assert!(data.expires_at > original_expiry);
    }
}
