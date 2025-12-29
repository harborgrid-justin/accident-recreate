//! Case management types
//!
//! This module defines structures for managing accident investigation
//! cases, including metadata, status tracking, and team assignments.

use crate::error::{AccuSceneError, Result};
use crate::traits::{Identifiable, MemoryFootprint, Serializable, Timestamped, Validatable};
use crate::types::accident::AccidentScene;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Case status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaseStatus {
    /// Case created but not yet started
    Draft,
    /// Case is under active investigation
    Active,
    /// Case investigation is on hold
    OnHold,
    /// Case under review
    UnderReview,
    /// Case completed
    Completed,
    /// Case archived
    Archived,
    /// Case cancelled
    Cancelled,
}

impl CaseStatus {
    /// Check if case is editable
    pub fn is_editable(&self) -> bool {
        matches!(self, Self::Draft | Self::Active | Self::OnHold)
    }

    /// Check if case is finalized
    pub fn is_finalized(&self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Archived | Self::Cancelled
        )
    }

    /// Get display name
    pub fn display_name(&self) -> &str {
        match self {
            Self::Draft => "Draft",
            Self::Active => "Active",
            Self::OnHold => "On Hold",
            Self::UnderReview => "Under Review",
            Self::Completed => "Completed",
            Self::Archived => "Archived",
            Self::Cancelled => "Cancelled",
        }
    }
}

/// Case priority level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CasePriority {
    /// Low priority
    Low,
    /// Normal priority
    Normal,
    /// High priority
    High,
    /// Critical/urgent priority
    Critical,
}

impl CasePriority {
    /// Get display name
    pub fn display_name(&self) -> &str {
        match self {
            Self::Low => "Low",
            Self::Normal => "Normal",
            Self::High => "High",
            Self::Critical => "Critical",
        }
    }

    /// Get priority score for sorting
    pub fn score(&self) -> u8 {
        match self {
            Self::Low => 1,
            Self::Normal => 2,
            Self::High => 3,
            Self::Critical => 4,
        }
    }
}

/// Case investigator/team member
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Investigator {
    /// Investigator ID
    pub id: String,
    /// Full name
    pub name: String,
    /// Email address
    pub email: Option<String>,
    /// Role (e.g., "Lead Investigator", "Analyst")
    pub role: String,
    /// Date assigned to case
    pub assigned_at: DateTime<Utc>,
}

impl Investigator {
    /// Create a new investigator
    pub fn new(name: String, role: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            email: None,
            role,
            assigned_at: Utc::now(),
        }
    }
}

/// Case metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseMetadata {
    /// Case number or reference
    pub case_number: Option<String>,
    /// Police report number
    pub police_report: Option<String>,
    /// Insurance claim number
    pub insurance_claim: Option<String>,
    /// Court case number
    pub court_case: Option<String>,
    /// Client name or organization
    pub client: Option<String>,
    /// Jurisdiction (city, state, country)
    pub jurisdiction: Option<String>,
    /// Custom tags for categorization
    pub tags: Vec<String>,
    /// Additional notes
    pub notes: Option<String>,
}

impl Default for CaseMetadata {
    fn default() -> Self {
        Self {
            case_number: None,
            police_report: None,
            insurance_claim: None,
            court_case: None,
            client: None,
            jurisdiction: None,
            tags: Vec::new(),
            notes: None,
        }
    }
}

/// Complete case representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Case {
    /// Unique identifier
    pub id: String,

    /// Case title
    pub title: String,

    /// Case description
    pub description: Option<String>,

    /// Case status
    pub status: CaseStatus,

    /// Priority level
    pub priority: CasePriority,

    /// Case metadata
    pub metadata: CaseMetadata,

    /// Accident scene associated with this case
    pub scene: AccidentScene,

    /// Assigned investigators
    pub investigators: Vec<Investigator>,

    /// Date case was opened
    pub opened_at: DateTime<Utc>,

    /// Date case was closed (if applicable)
    pub closed_at: Option<DateTime<Utc>>,

    /// Deadline for case completion
    pub deadline: Option<DateTime<Utc>>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl Case {
    /// Create a new case
    pub fn new(title: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            title: title.clone(),
            description: None,
            status: CaseStatus::Draft,
            priority: CasePriority::Normal,
            metadata: CaseMetadata::default(),
            scene: AccidentScene::new(title),
            investigators: Vec::new(),
            opened_at: now,
            closed_at: None,
            deadline: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a case with a scene
    pub fn with_scene(title: String, scene: AccidentScene) -> Self {
        let mut case = Self::new(title);
        case.scene = scene;
        case
    }

    /// Set case status
    pub fn set_status(&mut self, status: CaseStatus) -> Result<()> {
        // Validate status transitions
        match (&self.status, &status) {
            (CaseStatus::Completed | CaseStatus::Archived | CaseStatus::Cancelled, CaseStatus::Draft) => {
                return Err(AccuSceneError::InvalidState(
                    "Cannot revert finalized case to draft".to_string(),
                ));
            }
            _ => {}
        }

        self.status = status;

        // Set closed_at timestamp if finalizing
        if status.is_finalized() && self.closed_at.is_none() {
            self.closed_at = Some(Utc::now());
        }

        self.touch();
        Ok(())
    }

    /// Add an investigator to the case
    pub fn add_investigator(&mut self, investigator: Investigator) {
        self.investigators.push(investigator);
        self.touch();
    }

    /// Remove an investigator by ID
    pub fn remove_investigator(&mut self, investigator_id: &str) -> Result<()> {
        let initial_len = self.investigators.len();
        self.investigators.retain(|inv| inv.id != investigator_id);

        if self.investigators.len() == initial_len {
            return Err(AccuSceneError::not_found("Investigator", investigator_id));
        }

        self.touch();
        Ok(())
    }

    /// Get investigator by ID
    pub fn get_investigator(&self, investigator_id: &str) -> Option<&Investigator> {
        self.investigators.iter().find(|inv| inv.id == investigator_id)
    }

    /// Set priority
    pub fn set_priority(&mut self, priority: CasePriority) {
        self.priority = priority;
        self.touch();
    }

    /// Set deadline
    pub fn set_deadline(&mut self, deadline: DateTime<Utc>) -> Result<()> {
        if deadline < Utc::now() {
            return Err(AccuSceneError::validation(
                "Deadline cannot be in the past",
            ));
        }
        self.deadline = Some(deadline);
        self.touch();
        Ok(())
    }

    /// Check if case is overdue
    pub fn is_overdue(&self) -> bool {
        if let Some(deadline) = self.deadline {
            if !self.status.is_finalized() {
                return Utc::now() > deadline;
            }
        }
        false
    }

    /// Get days until deadline
    pub fn days_until_deadline(&self) -> Option<i64> {
        self.deadline.map(|deadline| {
            let duration = deadline.signed_duration_since(Utc::now());
            duration.num_days()
        })
    }

    /// Get case duration in days
    pub fn duration_days(&self) -> i64 {
        let end = self.closed_at.unwrap_or_else(Utc::now);
        let duration = end.signed_duration_since(self.opened_at);
        duration.num_days()
    }

    /// Check if case can be edited
    pub fn is_editable(&self) -> bool {
        self.status.is_editable()
    }

    /// Add a tag
    pub fn add_tag(&mut self, tag: String) {
        if !self.metadata.tags.contains(&tag) {
            self.metadata.tags.push(tag);
            self.touch();
        }
    }

    /// Remove a tag
    pub fn remove_tag(&mut self, tag: &str) {
        self.metadata.tags.retain(|t| t != tag);
        self.touch();
    }

    /// Get case summary
    pub fn summary(&self) -> CaseSummary {
        CaseSummary {
            id: self.id.clone(),
            title: self.title.clone(),
            status: self.status,
            priority: self.priority,
            vehicle_count: self.scene.vehicle_count(),
            investigator_count: self.investigators.len(),
            is_overdue: self.is_overdue(),
            duration_days: self.duration_days(),
            opened_at: self.opened_at,
            closed_at: self.closed_at,
        }
    }
}

/// Case summary for quick overview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseSummary {
    /// Case ID
    pub id: String,
    /// Case title
    pub title: String,
    /// Current status
    pub status: CaseStatus,
    /// Priority level
    pub priority: CasePriority,
    /// Number of vehicles in scene
    pub vehicle_count: usize,
    /// Number of investigators
    pub investigator_count: usize,
    /// Is case overdue
    pub is_overdue: bool,
    /// Case duration in days
    pub duration_days: i64,
    /// Opened date
    pub opened_at: DateTime<Utc>,
    /// Closed date
    pub closed_at: Option<DateTime<Utc>>,
}

impl Identifiable for Case {
    type Id = String;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn set_id(&mut self, id: Self::Id) {
        self.id = id;
        self.touch();
    }

    fn with_new_id(mut self) -> Self {
        self.id = Uuid::new_v4().to_string();
        self
    }
}

impl Timestamped for Case {
    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}

impl Validatable for Case {
    fn validate(&self) -> Result<()> {
        if self.title.is_empty() {
            return Err(AccuSceneError::validation_field(
                "Case title cannot be empty",
                "title",
            ));
        }

        self.scene.validate()?;

        if let Some(deadline) = self.deadline {
            if deadline < self.opened_at {
                return Err(AccuSceneError::validation_field(
                    "Deadline cannot be before case opened date",
                    "deadline",
                ));
            }
        }

        if let Some(closed_at) = self.closed_at {
            if closed_at < self.opened_at {
                return Err(AccuSceneError::validation_field(
                    "Close date cannot be before open date",
                    "closed_at",
                ));
            }
        }

        Ok(())
    }
}

impl Serializable for Case {}
impl Serializable for CaseSummary {}
impl Serializable for Investigator {}
impl Serializable for CaseMetadata {}

impl MemoryFootprint for Case {
    fn memory_footprint(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.title.capacity()
            + self.description.as_ref().map(|s| s.capacity()).unwrap_or(0)
            + self.scene.memory_footprint()
            + self.investigators.len() * std::mem::size_of::<Investigator>()
            + self
                .metadata
                .tags
                .iter()
                .map(|t| t.capacity())
                .sum::<usize>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_creation() {
        let case = Case::new("Test Case".to_string());
        assert!(case.validate().is_ok());
        assert_eq!(case.status, CaseStatus::Draft);
        assert_eq!(case.priority, CasePriority::Normal);
    }

    #[test]
    fn test_case_status_change() {
        let mut case = Case::new("Test".to_string());

        case.set_status(CaseStatus::Active).unwrap();
        assert_eq!(case.status, CaseStatus::Active);

        case.set_status(CaseStatus::Completed).unwrap();
        assert!(case.closed_at.is_some());
        assert!(!case.is_editable());
    }

    #[test]
    fn test_investigator_management() {
        let mut case = Case::new("Test".to_string());
        let investigator = Investigator::new("John Doe".to_string(), "Lead".to_string());
        let inv_id = investigator.id.clone();

        case.add_investigator(investigator);
        assert_eq!(case.investigators.len(), 1);

        case.remove_investigator(&inv_id).unwrap();
        assert_eq!(case.investigators.len(), 0);
    }

    #[test]
    fn test_deadline() {
        let mut case = Case::new("Test".to_string());
        let future = Utc::now() + chrono::Duration::days(7);

        case.set_deadline(future).unwrap();
        assert!(!case.is_overdue());

        let days = case.days_until_deadline().unwrap();
        assert!(days >= 6 && days <= 7);
    }

    #[test]
    fn test_tags() {
        let mut case = Case::new("Test".to_string());

        case.add_tag("intersection".to_string());
        case.add_tag("high-speed".to_string());
        assert_eq!(case.metadata.tags.len(), 2);

        case.remove_tag("intersection");
        assert_eq!(case.metadata.tags.len(), 1);
    }
}
