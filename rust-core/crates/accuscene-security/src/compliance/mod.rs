//! Compliance framework
//!
//! Implements compliance controls for SOC2, GDPR, and HIPAA.

pub mod gdpr;
pub mod hipaa;
pub mod soc2;

pub use gdpr::{DataContext, GdprService, LawfulBasis};
pub use hipaa::{HipaaService, PhiIdentifier};
pub use soc2::{ComplianceStatus, Control, ControlStatus, Soc2Service, TrustServiceCategory};

/// Main compliance service
pub struct ComplianceService {
    soc2: Soc2Service,
}

impl ComplianceService {
    /// Create a new compliance service
    pub fn new() -> Self {
        Self {
            soc2: Soc2Service::new(),
        }
    }

    /// Get SOC2 service
    pub fn soc2(&self) -> &Soc2Service {
        &self.soc2
    }

    /// Get compliance summary
    pub fn get_summary(&self) -> ComplianceSummary {
        let soc2_status = self.soc2.get_compliance_status();

        ComplianceSummary {
            soc2_compliance: soc2_status.compliance_percentage,
            gdpr_enabled: true,
            hipaa_enabled: false,
        }
    }
}

impl Default for ComplianceService {
    fn default() -> Self {
        Self::new()
    }
}

/// Compliance summary
#[derive(Debug, Clone)]
pub struct ComplianceSummary {
    pub soc2_compliance: u8,
    pub gdpr_enabled: bool,
    pub hipaa_enabled: bool,
}
