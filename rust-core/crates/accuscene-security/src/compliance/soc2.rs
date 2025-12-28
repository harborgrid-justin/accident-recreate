//! SOC 2 compliance controls
//!
//! Implements SOC 2 Trust Services Criteria controls.

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// SOC 2 compliance service
pub struct Soc2Service {
    controls: HashMap<String, Control>,
}

impl Soc2Service {
    /// Create a new SOC 2 service with default controls
    pub fn new() -> Self {
        let mut service = Self {
            controls: HashMap::new(),
        };
        service.initialize_controls();
        service
    }

    /// Initialize SOC 2 controls
    fn initialize_controls(&mut self) {
        // CC6.1 - Logical and Physical Access Controls
        self.add_control(Control {
            id: "CC6.1".to_string(),
            category: TrustServiceCategory::CommonCriteria,
            title: "Logical and Physical Access Controls".to_string(),
            description: "The entity implements logical access security software, infrastructure, and architectures over protected information assets to protect them from security events to meet the entity's objectives.".to_string(),
            requirements: vec![
                "Implement authentication mechanisms".to_string(),
                "Enforce password policies".to_string(),
                "Implement MFA where appropriate".to_string(),
                "Log and monitor access attempts".to_string(),
            ],
            status: ControlStatus::Implemented,
        });

        // CC6.2 - User Access Management
        self.add_control(Control {
            id: "CC6.2".to_string(),
            category: TrustServiceCategory::CommonCriteria,
            title: "Prior to Issuing System Credentials and Granting System Access".to_string(),
            description: "Prior to issuing system credentials and granting system access, the entity registers and authorizes new internal and external users whose access is administered by the entity.".to_string(),
            requirements: vec![
                "Implement user registration process".to_string(),
                "Require management approval for access".to_string(),
                "Assign appropriate roles and permissions".to_string(),
            ],
            status: ControlStatus::Implemented,
        });

        // CC6.3 - User Access Removal
        self.add_control(Control {
            id: "CC6.3".to_string(),
            category: TrustServiceCategory::CommonCriteria,
            title: "User Access Modification and Removal".to_string(),
            description: "The entity authorizes, modifies, or removes access to data, software, functions, and other protected information assets based on roles, responsibilities, or the system design and changes.".to_string(),
            requirements: vec![
                "Review and update access permissions regularly".to_string(),
                "Remove access promptly when no longer needed".to_string(),
                "Implement segregation of duties".to_string(),
            ],
            status: ControlStatus::Implemented,
        });

        // CC7.2 - Audit Logging
        self.add_control(Control {
            id: "CC7.2".to_string(),
            category: TrustServiceCategory::CommonCriteria,
            title: "System Monitoring".to_string(),
            description: "The entity monitors system components and the operation of those components for anomalies that are indicative of malicious acts, natural disasters, and errors affecting the entity's ability to meet its objectives.".to_string(),
            requirements: vec![
                "Log security-relevant events".to_string(),
                "Monitor logs for anomalies".to_string(),
                "Retain logs per retention policy".to_string(),
                "Protect log integrity".to_string(),
            ],
            status: ControlStatus::Implemented,
        });

        // CC7.3 - Incident Response
        self.add_control(Control {
            id: "CC7.3".to_string(),
            category: TrustServiceCategory::CommonCriteria,
            title: "Incident Response".to_string(),
            description: "The entity evaluates security events to determine whether they could or have resulted in a failure of the entity to meet its objectives and, if so, takes actions to prevent or address such failures.".to_string(),
            requirements: vec![
                "Implement incident detection mechanisms".to_string(),
                "Define incident response procedures".to_string(),
                "Escalate critical incidents".to_string(),
            ],
            status: ControlStatus::Implemented,
        });
    }

    /// Add a control
    pub fn add_control(&mut self, control: Control) {
        self.controls.insert(control.id.clone(), control);
    }

    /// Get a control by ID
    pub fn get_control(&self, id: &str) -> Option<&Control> {
        self.controls.get(id)
    }

    /// Get all controls
    pub fn get_all_controls(&self) -> Vec<&Control> {
        self.controls.values().collect()
    }

    /// Get controls by category
    pub fn get_controls_by_category(&self, category: TrustServiceCategory) -> Vec<&Control> {
        self.controls
            .values()
            .filter(|c| c.category == category)
            .collect()
    }

    /// Get compliance status
    pub fn get_compliance_status(&self) -> ComplianceStatus {
        let total = self.controls.len();
        let implemented = self
            .controls
            .values()
            .filter(|c| matches!(c.status, ControlStatus::Implemented))
            .count();
        let in_progress = self
            .controls
            .values()
            .filter(|c| matches!(c.status, ControlStatus::InProgress))
            .count();
        let not_implemented = self
            .controls
            .values()
            .filter(|c| matches!(c.status, ControlStatus::NotImplemented))
            .count();

        ComplianceStatus {
            total_controls: total,
            implemented,
            in_progress,
            not_implemented,
            compliance_percentage: (implemented as f64 / total as f64 * 100.0) as u8,
        }
    }

    /// Verify control implementation
    pub fn verify_control(&self, control_id: &str) -> Result<bool> {
        let control = self.get_control(control_id)
            .ok_or_else(|| crate::error::SecurityError::Soc2Violation(
                format!("Control {} not found", control_id)
            ))?;

        Ok(matches!(control.status, ControlStatus::Implemented))
    }
}

impl Default for Soc2Service {
    fn default() -> Self {
        Self::new()
    }
}

/// SOC 2 control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Control {
    /// Control ID (e.g., "CC6.1")
    pub id: String,
    /// Trust Service Category
    pub category: TrustServiceCategory,
    /// Control title
    pub title: String,
    /// Control description
    pub description: String,
    /// Control requirements
    pub requirements: Vec<String>,
    /// Implementation status
    pub status: ControlStatus,
}

/// Trust Service Categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TrustServiceCategory {
    /// Common Criteria (CC)
    CommonCriteria,
    /// Availability (A)
    Availability,
    /// Confidentiality (C)
    Confidentiality,
    /// Processing Integrity (PI)
    ProcessingIntegrity,
    /// Privacy (P)
    Privacy,
}

/// Control implementation status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlStatus {
    Implemented,
    InProgress,
    NotImplemented,
}

/// Compliance status summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceStatus {
    /// Total number of controls
    pub total_controls: usize,
    /// Number of implemented controls
    pub implemented: usize,
    /// Number of controls in progress
    pub in_progress: usize,
    /// Number of not implemented controls
    pub not_implemented: usize,
    /// Compliance percentage
    pub compliance_percentage: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_soc2_service_creation() {
        let service = Soc2Service::new();
        assert!(!service.controls.is_empty());
    }

    #[test]
    fn test_get_control() {
        let service = Soc2Service::new();
        let control = service.get_control("CC6.1");
        assert!(control.is_some());
        assert_eq!(control.unwrap().id, "CC6.1");
    }

    #[test]
    fn test_compliance_status() {
        let service = Soc2Service::new();
        let status = service.get_compliance_status();
        assert!(status.total_controls > 0);
        assert_eq!(status.compliance_percentage, 100);
    }

    #[test]
    fn test_verify_control() {
        let service = Soc2Service::new();
        assert!(service.verify_control("CC6.1").unwrap());
    }
}
