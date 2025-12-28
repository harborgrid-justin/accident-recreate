//! Case-related events for accident reconstruction.

use crate::event::Event;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Case created event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CaseCreated {
    /// Unique case identifier.
    pub case_id: String,

    /// Case number or reference.
    pub case_number: String,

    /// Case title.
    pub title: String,

    /// Case description.
    pub description: Option<String>,

    /// User who created the case.
    pub created_by: String,

    /// Timestamp when created.
    pub created_at: DateTime<Utc>,

    /// Case metadata.
    pub metadata: HashMap<String, String>,
}

impl Event for CaseCreated {
    fn event_type(&self) -> &'static str {
        "CaseCreated"
    }

    fn aggregate_id(&self) -> &str {
        &self.case_id
    }

    fn aggregate_type(&self) -> &'static str {
        "Case"
    }
}

/// Case updated event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CaseUpdated {
    /// Case identifier.
    pub case_id: String,

    /// Updated title.
    pub title: Option<String>,

    /// Updated description.
    pub description: Option<String>,

    /// User who updated the case.
    pub updated_by: String,

    /// Timestamp when updated.
    pub updated_at: DateTime<Utc>,

    /// Fields that were changed.
    pub changed_fields: Vec<String>,
}

impl Event for CaseUpdated {
    fn event_type(&self) -> &'static str {
        "CaseUpdated"
    }

    fn aggregate_id(&self) -> &str {
        &self.case_id
    }

    fn aggregate_type(&self) -> &'static str {
        "Case"
    }
}

/// Case status changed event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CaseStatusChanged {
    /// Case identifier.
    pub case_id: String,

    /// Previous status.
    pub old_status: CaseStatus,

    /// New status.
    pub new_status: CaseStatus,

    /// User who changed the status.
    pub changed_by: String,

    /// Timestamp when changed.
    pub changed_at: DateTime<Utc>,

    /// Reason for status change.
    pub reason: Option<String>,
}

impl Event for CaseStatusChanged {
    fn event_type(&self) -> &'static str {
        "CaseStatusChanged"
    }

    fn aggregate_id(&self) -> &str {
        &self.case_id
    }

    fn aggregate_type(&self) -> &'static str {
        "Case"
    }
}

/// Case archived event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CaseArchived {
    /// Case identifier.
    pub case_id: String,

    /// User who archived the case.
    pub archived_by: String,

    /// Timestamp when archived.
    pub archived_at: DateTime<Utc>,

    /// Reason for archiving.
    pub reason: Option<String>,
}

impl Event for CaseArchived {
    fn event_type(&self) -> &'static str {
        "CaseArchived"
    }

    fn aggregate_id(&self) -> &str {
        &self.case_id
    }

    fn aggregate_type(&self) -> &'static str {
        "Case"
    }
}

/// Case deleted event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CaseDeleted {
    /// Case identifier.
    pub case_id: String,

    /// User who deleted the case.
    pub deleted_by: String,

    /// Timestamp when deleted.
    pub deleted_at: DateTime<Utc>,

    /// Whether this is a soft delete.
    pub soft_delete: bool,
}

impl Event for CaseDeleted {
    fn event_type(&self) -> &'static str {
        "CaseDeleted"
    }

    fn aggregate_id(&self) -> &str {
        &self.case_id
    }

    fn aggregate_type(&self) -> &'static str {
        "Case"
    }
}

/// Case participant added event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CaseParticipantAdded {
    /// Case identifier.
    pub case_id: String,

    /// Participant identifier.
    pub participant_id: String,

    /// Participant name.
    pub name: String,

    /// Participant role.
    pub role: ParticipantRole,

    /// Contact information.
    pub contact: Option<String>,

    /// Added by.
    pub added_by: String,

    /// Timestamp when added.
    pub added_at: DateTime<Utc>,
}

impl Event for CaseParticipantAdded {
    fn event_type(&self) -> &'static str {
        "CaseParticipantAdded"
    }

    fn aggregate_id(&self) -> &str {
        &self.case_id
    }

    fn aggregate_type(&self) -> &'static str {
        "Case"
    }
}

/// Case participant removed event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CaseParticipantRemoved {
    /// Case identifier.
    pub case_id: String,

    /// Participant identifier.
    pub participant_id: String,

    /// Removed by.
    pub removed_by: String,

    /// Timestamp when removed.
    pub removed_at: DateTime<Utc>,

    /// Reason for removal.
    pub reason: Option<String>,
}

impl Event for CaseParticipantRemoved {
    fn event_type(&self) -> &'static str {
        "CaseParticipantRemoved"
    }

    fn aggregate_id(&self) -> &str {
        &self.case_id
    }

    fn aggregate_type(&self) -> &'static str {
        "Case"
    }
}

/// Case evidence added event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CaseEvidenceAdded {
    /// Case identifier.
    pub case_id: String,

    /// Evidence identifier.
    pub evidence_id: String,

    /// Evidence type.
    pub evidence_type: EvidenceType,

    /// File path or reference.
    pub reference: String,

    /// Description.
    pub description: Option<String>,

    /// Added by.
    pub added_by: String,

    /// Timestamp when added.
    pub added_at: DateTime<Utc>,

    /// Evidence metadata.
    pub metadata: HashMap<String, String>,
}

impl Event for CaseEvidenceAdded {
    fn event_type(&self) -> &'static str {
        "CaseEvidenceAdded"
    }

    fn aggregate_id(&self) -> &str {
        &self.case_id
    }

    fn aggregate_type(&self) -> &'static str {
        "Case"
    }
}

/// Case status enumeration.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CaseStatus {
    /// Case is draft.
    Draft,

    /// Case is open and active.
    Open,

    /// Case is under review.
    InReview,

    /// Case is closed.
    Closed,

    /// Case is on hold.
    OnHold,

    /// Case is archived.
    Archived,
}

/// Participant role enumeration.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ParticipantRole {
    /// Lead investigator.
    LeadInvestigator,

    /// Assistant investigator.
    AssistantInvestigator,

    /// Witness.
    Witness,

    /// Expert witness.
    ExpertWitness,

    /// Attorney.
    Attorney,

    /// Insurance adjuster.
    InsuranceAdjuster,

    /// Other role.
    Other,
}

/// Evidence type enumeration.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EvidenceType {
    /// Photo evidence.
    Photo,

    /// Video evidence.
    Video,

    /// Document.
    Document,

    /// Audio recording.
    Audio,

    /// 3D scan data.
    Scan3D,

    /// Police report.
    PoliceReport,

    /// Medical record.
    MedicalRecord,

    /// Other evidence.
    Other,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_created_event() {
        let event = CaseCreated {
            case_id: "case-123".to_string(),
            case_number: "2024-001".to_string(),
            title: "Highway Collision".to_string(),
            description: Some("Multi-vehicle collision".to_string()),
            created_by: "user-456".to_string(),
            created_at: Utc::now(),
            metadata: HashMap::new(),
        };

        assert_eq!(event.event_type(), "CaseCreated");
        assert_eq!(event.aggregate_id(), "case-123");
        assert_eq!(event.aggregate_type(), "Case");
    }

    #[test]
    fn test_case_status_changed_event() {
        let event = CaseStatusChanged {
            case_id: "case-123".to_string(),
            old_status: CaseStatus::Draft,
            new_status: CaseStatus::Open,
            changed_by: "user-456".to_string(),
            changed_at: Utc::now(),
            reason: Some("Case ready for investigation".to_string()),
        };

        assert_eq!(event.event_type(), "CaseStatusChanged");
    }
}
