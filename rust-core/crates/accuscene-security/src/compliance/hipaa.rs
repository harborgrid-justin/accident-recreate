//! HIPAA compliance helpers
//!
//! Implements HIPAA requirements for protecting health information.

use crate::error::Result;
use serde::{Deserialize, Serialize};

/// HIPAA compliance service
pub struct HipaaService;

impl HipaaService {
    /// Validate access to Protected Health Information (PHI)
    pub fn validate_phi_access(
        user_role: &str,
        minimum_necessary: bool,
    ) -> Result<()> {
        // Verify "minimum necessary" standard
        if !minimum_necessary {
            return Err(crate::error::SecurityError::HipaaViolation(
                "Access does not meet minimum necessary standard".to_string(),
            ));
        }

        // Verify user has appropriate role
        let authorized_roles = ["physician", "nurse", "admin", "researcher"];
        if !authorized_roles.contains(&user_role) {
            return Err(crate::error::SecurityError::HipaaViolation(
                format!("Role {} not authorized for PHI access", user_role),
            ));
        }

        Ok(())
    }

    /// Check if data is considered PHI
    pub fn is_phi(data_type: PhiIdentifier) -> bool {
        // All 18 HIPAA identifiers are considered PHI
        matches!(
            data_type,
            PhiIdentifier::Name
                | PhiIdentifier::GeographicSubdivision
                | PhiIdentifier::DateElement
                | PhiIdentifier::PhoneNumber
                | PhiIdentifier::Email
                | PhiIdentifier::SocialSecurity
                | PhiIdentifier::MedicalRecord
                | PhiIdentifier::HealthPlan
                | PhiIdentifier::AccountNumber
                | PhiIdentifier::Certificate
                | PhiIdentifier::VehicleIdentifier
                | PhiIdentifier::DeviceIdentifier
                | PhiIdentifier::WebUrl
                | PhiIdentifier::IpAddress
                | PhiIdentifier::Biometric
                | PhiIdentifier::PhotoImage
                | PhiIdentifier::OtherIdentifier
        )
    }

    /// De-identify PHI (Safe Harbor method)
    pub fn deidentify_phi(phi_type: PhiIdentifier, value: &str) -> String {
        match phi_type {
            PhiIdentifier::Name => "[REDACTED NAME]".to_string(),
            PhiIdentifier::DateElement => "[REDACTED DATE]".to_string(),
            PhiIdentifier::PhoneNumber => "[REDACTED PHONE]".to_string(),
            PhiIdentifier::Email => "[REDACTED EMAIL]".to_string(),
            PhiIdentifier::SocialSecurity => "[REDACTED SSN]".to_string(),
            _ => "[REDACTED]".to_string(),
        }
    }
}

/// HIPAA PHI identifiers (18 identifiers)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhiIdentifier {
    Name,
    GeographicSubdivision,
    DateElement,
    PhoneNumber,
    Email,
    SocialSecurity,
    MedicalRecord,
    HealthPlan,
    AccountNumber,
    Certificate,
    VehicleIdentifier,
    DeviceIdentifier,
    WebUrl,
    IpAddress,
    Biometric,
    PhotoImage,
    OtherIdentifier,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phi_access_validation() {
        assert!(HipaaService::validate_phi_access("physician", true).is_ok());
        assert!(HipaaService::validate_phi_access("physician", false).is_err());
        assert!(HipaaService::validate_phi_access("unauthorized", true).is_err());
    }

    #[test]
    fn test_is_phi() {
        assert!(HipaaService::is_phi(PhiIdentifier::Name));
        assert!(HipaaService::is_phi(PhiIdentifier::SocialSecurity));
    }

    #[test]
    fn test_deidentify_phi() {
        let deidentified = HipaaService::deidentify_phi(PhiIdentifier::Name, "John Doe");
        assert_eq!(deidentified, "[REDACTED NAME]");
    }
}
