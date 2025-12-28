//! GDPR compliance helpers
//!
//! Implements GDPR requirements for data protection and privacy.

use crate::error::Result;
use serde::{Deserialize, Serialize};

/// GDPR compliance service
pub struct GdprService;

impl GdprService {
    /// Validate data processing lawful basis
    pub fn validate_lawful_basis(basis: LawfulBasis) -> Result<()> {
        // Validate that a lawful basis exists for processing
        match basis {
            LawfulBasis::Consent => Ok(()),
            LawfulBasis::Contract => Ok(()),
            LawfulBasis::LegalObligation => Ok(()),
            LawfulBasis::VitalInterests => Ok(()),
            LawfulBasis::PublicTask => Ok(()),
            LawfulBasis::LegitimateInterests => Ok(()),
        }
    }

    /// Check if data subject has right to erasure ("right to be forgotten")
    pub fn check_right_to_erasure(context: &DataContext) -> bool {
        // Simplified logic - in production, would check various conditions
        context.consent_given && !context.legal_obligation
    }

    /// Check if data subject has right to data portability
    pub fn check_right_to_portability(basis: &LawfulBasis) -> bool {
        matches!(basis, LawfulBasis::Consent | LawfulBasis::Contract)
    }

    /// Anonymize personal data
    pub fn anonymize_data(data: &str) -> String {
        // Simplified anonymization - in production would use proper techniques
        let len = data.len();
        if len <= 2 {
            "*".repeat(len)
        } else {
            format!("{}***{}", &data[..1], &data[len - 1..])
        }
    }

    /// Pseudonymize personal data
    pub fn pseudonymize_data(data: &str, key: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        hasher.update(key.as_bytes());
        hex::encode(&hasher.finalize()[..8])
    }
}

/// Lawful basis for processing under GDPR
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LawfulBasis {
    Consent,
    Contract,
    LegalObligation,
    VitalInterests,
    PublicTask,
    LegitimateInterests,
}

/// Data processing context
#[derive(Debug, Clone)]
pub struct DataContext {
    pub consent_given: bool,
    pub legal_obligation: bool,
    pub vital_interests: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lawful_basis_validation() {
        assert!(GdprService::validate_lawful_basis(LawfulBasis::Consent).is_ok());
    }

    #[test]
    fn test_right_to_erasure() {
        let context = DataContext {
            consent_given: true,
            legal_obligation: false,
            vital_interests: false,
        };
        assert!(GdprService::check_right_to_erasure(&context));
    }

    #[test]
    fn test_anonymization() {
        let data = "user@example.com";
        let anonymized = GdprService::anonymize_data(data);
        assert_ne!(anonymized, data);
        assert!(anonymized.contains("***"));
    }

    #[test]
    fn test_pseudonymization() {
        let data = "user@example.com";
        let pseudo = GdprService::pseudonymize_data(data, "secret-key");
        assert_ne!(pseudo, data);
        assert_eq!(pseudo.len(), 16); // 8 bytes = 16 hex chars
    }
}
