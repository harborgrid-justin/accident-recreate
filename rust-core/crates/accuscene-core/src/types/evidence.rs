//! Evidence tracking types
//!
//! This module defines structures for tracking and managing
//! evidence associated with accident investigations.

use crate::error::{AccuSceneError, Result};
use crate::traits::{Identifiable, MemoryFootprint, Serializable, Timestamped, Validatable};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Type of evidence
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceType {
    /// Photograph
    Photo,
    /// Video recording
    Video,
    /// Audio recording
    Audio,
    /// Document (report, citation, etc.)
    Document,
    /// Physical evidence (skid marks, debris, etc.)
    Physical,
    /// Witness statement
    WitnessStatement,
    /// Expert report
    ExpertReport,
    /// Diagram or sketch
    Diagram,
    /// 3D scan or model
    ThreeDModel,
    /// GPS/telemetry data
    TelemetryData,
    /// Medical report
    MedicalReport,
    /// Vehicle inspection report
    VehicleInspection,
    /// Weather report
    WeatherReport,
    /// Other evidence type
    Other,
}

impl EvidenceType {
    /// Get display name
    pub fn display_name(&self) -> &str {
        match self {
            Self::Photo => "Photograph",
            Self::Video => "Video",
            Self::Audio => "Audio Recording",
            Self::Document => "Document",
            Self::Physical => "Physical Evidence",
            Self::WitnessStatement => "Witness Statement",
            Self::ExpertReport => "Expert Report",
            Self::Diagram => "Diagram",
            Self::ThreeDModel => "3D Model",
            Self::TelemetryData => "Telemetry Data",
            Self::MedicalReport => "Medical Report",
            Self::VehicleInspection => "Vehicle Inspection",
            Self::WeatherReport => "Weather Report",
            Self::Other => "Other",
        }
    }

    /// Check if evidence type supports file attachments
    pub fn supports_files(&self) -> bool {
        !matches!(self, Self::Physical)
    }

    /// Get typical file extensions for this evidence type
    pub fn typical_extensions(&self) -> Vec<&str> {
        match self {
            Self::Photo => vec!["jpg", "jpeg", "png", "heic", "raw"],
            Self::Video => vec!["mp4", "mov", "avi", "mkv"],
            Self::Audio => vec!["mp3", "wav", "m4a", "aac"],
            Self::Document => vec!["pdf", "doc", "docx", "txt"],
            Self::ThreeDModel => vec!["obj", "fbx", "stl", "ply"],
            Self::TelemetryData => vec!["csv", "json", "xml"],
            Self::Diagram => vec!["svg", "png", "jpg", "pdf"],
            _ => vec!["pdf", "jpg", "png"],
        }
    }
}

/// Evidence collection method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CollectionMethod {
    /// Collected at scene
    AtScene,
    /// Obtained from police/authorities
    FromPolice,
    /// Obtained from witness
    FromWitness,
    /// Obtained from vehicle owner
    FromVehicleOwner,
    /// Generated through analysis
    Generated,
    /// Obtained from third party
    ThirdParty,
    /// Other method
    Other,
}

/// Chain of custody entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustodyEntry {
    /// Entry ID
    pub id: String,
    /// Person who took custody
    pub custodian: String,
    /// Timestamp when custody was taken
    pub timestamp: DateTime<Utc>,
    /// Purpose of custody transfer
    pub purpose: String,
    /// Location where custody was transferred
    pub location: Option<String>,
    /// Additional notes
    pub notes: Option<String>,
}

impl CustodyEntry {
    /// Create a new custody entry
    pub fn new(custodian: String, purpose: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            custodian,
            timestamp: Utc::now(),
            purpose,
            location: None,
            notes: None,
        }
    }
}

/// Evidence metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceMetadata {
    /// Collection method
    pub collection_method: CollectionMethod,
    /// Collector name
    pub collected_by: Option<String>,
    /// Collection location
    pub collection_location: Option<String>,
    /// Source of evidence
    pub source: Option<String>,
    /// Associated vehicle ID (if applicable)
    pub vehicle_id: Option<String>,
    /// File path or URL
    pub file_path: Option<String>,
    /// File size in bytes
    pub file_size_bytes: Option<u64>,
    /// File format/extension
    pub file_format: Option<String>,
    /// Checksum for integrity verification
    pub checksum: Option<String>,
    /// Additional custom metadata
    pub custom_fields: std::collections::HashMap<String, String>,
}

impl Default for EvidenceMetadata {
    fn default() -> Self {
        Self {
            collection_method: CollectionMethod::Other,
            collected_by: None,
            collection_location: None,
            source: None,
            vehicle_id: None,
            file_path: None,
            file_size_bytes: None,
            file_format: None,
            checksum: None,
            custom_fields: std::collections::HashMap::new(),
        }
    }
}

/// Evidence item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    /// Unique identifier
    pub id: String,

    /// Evidence title/name
    pub title: String,

    /// Description of evidence
    pub description: Option<String>,

    /// Evidence type
    pub evidence_type: EvidenceType,

    /// Evidence metadata
    pub metadata: EvidenceMetadata,

    /// Chain of custody records
    pub chain_of_custody: Vec<CustodyEntry>,

    /// Tags for categorization
    pub tags: Vec<String>,

    /// Date/time evidence was collected
    pub collected_at: DateTime<Utc>,

    /// Is evidence admissible in court
    pub admissible: bool,

    /// Importance/relevance rating (0-10)
    pub relevance_score: u8,

    /// Notes about the evidence
    pub notes: Option<String>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl Evidence {
    /// Create a new evidence item
    pub fn new(title: String, evidence_type: EvidenceType) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            title,
            description: None,
            evidence_type,
            metadata: EvidenceMetadata::default(),
            chain_of_custody: Vec::new(),
            tags: Vec::new(),
            collected_at: now,
            admissible: true,
            relevance_score: 5,
            notes: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create evidence with metadata
    pub fn with_metadata(
        title: String,
        evidence_type: EvidenceType,
        metadata: EvidenceMetadata,
    ) -> Self {
        let mut evidence = Self::new(title, evidence_type);
        evidence.metadata = metadata;
        evidence
    }

    /// Add a custody entry
    pub fn add_custody_entry(&mut self, entry: CustodyEntry) {
        self.chain_of_custody.push(entry);
        self.touch();
    }

    /// Transfer custody to a new person
    pub fn transfer_custody(&mut self, custodian: String, purpose: String) {
        let entry = CustodyEntry::new(custodian, purpose);
        self.add_custody_entry(entry);
    }

    /// Get current custodian
    pub fn current_custodian(&self) -> Option<&str> {
        self.chain_of_custody
            .last()
            .map(|entry| entry.custodian.as_str())
    }

    /// Add a tag
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.touch();
        }
    }

    /// Remove a tag
    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.retain(|t| t != tag);
        self.touch();
    }

    /// Set relevance score (0-10)
    pub fn set_relevance(&mut self, score: u8) -> Result<()> {
        if score > 10 {
            return Err(AccuSceneError::validation(
                "Relevance score must be between 0 and 10",
            ));
        }
        self.relevance_score = score;
        self.touch();
        Ok(())
    }

    /// Set admissibility
    pub fn set_admissible(&mut self, admissible: bool) {
        self.admissible = admissible;
        self.touch();
    }

    /// Attach a file
    pub fn attach_file(&mut self, file_path: String, file_size: u64, format: String) {
        self.metadata.file_path = Some(file_path);
        self.metadata.file_size_bytes = Some(file_size);
        self.metadata.file_format = Some(format);
        self.touch();
    }

    /// Set file checksum for integrity
    pub fn set_checksum(&mut self, checksum: String) {
        self.metadata.checksum = Some(checksum);
        self.touch();
    }

    /// Verify file checksum
    pub fn verify_checksum(&self, provided_checksum: &str) -> bool {
        self.metadata
            .checksum
            .as_ref()
            .map(|c| c == provided_checksum)
            .unwrap_or(false)
    }

    /// Get human-readable file size
    pub fn file_size_string(&self) -> Option<String> {
        self.metadata.file_size_bytes.map(|bytes| {
            if bytes < 1024 {
                format!("{} B", bytes)
            } else if bytes < 1024 * 1024 {
                format!("{:.2} KB", bytes as f64 / 1024.0)
            } else if bytes < 1024 * 1024 * 1024 {
                format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
            } else {
                format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
            }
        })
    }

    /// Get chain of custody count
    pub fn custody_transfers(&self) -> usize {
        self.chain_of_custody.len()
    }

    /// Check if evidence has file attached
    pub fn has_file(&self) -> bool {
        self.metadata.file_path.is_some()
    }

    /// Get summary of evidence
    pub fn summary(&self) -> EvidenceSummary {
        EvidenceSummary {
            id: self.id.clone(),
            title: self.title.clone(),
            evidence_type: self.evidence_type,
            collected_at: self.collected_at,
            admissible: self.admissible,
            relevance_score: self.relevance_score,
            has_file: self.has_file(),
            custody_transfers: self.custody_transfers(),
        }
    }
}

/// Evidence summary for quick overview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceSummary {
    /// Evidence ID
    pub id: String,
    /// Title
    pub title: String,
    /// Type of evidence
    pub evidence_type: EvidenceType,
    /// Collection date
    pub collected_at: DateTime<Utc>,
    /// Is admissible
    pub admissible: bool,
    /// Relevance score
    pub relevance_score: u8,
    /// Has attached file
    pub has_file: bool,
    /// Number of custody transfers
    pub custody_transfers: usize,
}

impl Identifiable for Evidence {
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

impl Timestamped for Evidence {
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

impl Validatable for Evidence {
    fn validate(&self) -> Result<()> {
        if self.title.is_empty() {
            return Err(AccuSceneError::validation_field(
                "Evidence title cannot be empty",
                "title",
            ));
        }

        if self.relevance_score > 10 {
            return Err(AccuSceneError::validation_field(
                "Relevance score must be between 0 and 10",
                "relevance_score",
            ));
        }

        if self.collected_at > Utc::now() {
            return Err(AccuSceneError::validation_field(
                "Collection date cannot be in the future",
                "collected_at",
            ));
        }

        Ok(())
    }
}

impl Serializable for Evidence {}
impl Serializable for EvidenceSummary {}
impl Serializable for CustodyEntry {}
impl Serializable for EvidenceMetadata {}

impl MemoryFootprint for Evidence {
    fn memory_footprint(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.title.capacity()
            + self.description.as_ref().map(|s| s.capacity()).unwrap_or(0)
            + self.tags.iter().map(|t| t.capacity()).sum::<usize>()
            + self.chain_of_custody.len() * std::mem::size_of::<CustodyEntry>()
            + self
                .metadata
                .custom_fields
                .iter()
                .map(|(k, v)| k.capacity() + v.capacity())
                .sum::<usize>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evidence_creation() {
        let evidence = Evidence::new("Skid marks photo".to_string(), EvidenceType::Photo);
        assert!(evidence.validate().is_ok());
        assert_eq!(evidence.evidence_type, EvidenceType::Photo);
    }

    #[test]
    fn test_custody_chain() {
        let mut evidence = Evidence::new("Test Evidence".to_string(), EvidenceType::Document);

        evidence.transfer_custody("John Doe".to_string(), "Initial collection".to_string());
        assert_eq!(evidence.custody_transfers(), 1);
        assert_eq!(evidence.current_custodian(), Some("John Doe"));

        evidence.transfer_custody("Jane Smith".to_string(), "Analysis".to_string());
        assert_eq!(evidence.custody_transfers(), 2);
        assert_eq!(evidence.current_custodian(), Some("Jane Smith"));
    }

    #[test]
    fn test_tags() {
        let mut evidence = Evidence::new("Test".to_string(), EvidenceType::Photo);

        evidence.add_tag("critical".to_string());
        evidence.add_tag("scene".to_string());
        assert_eq!(evidence.tags.len(), 2);

        evidence.remove_tag("scene");
        assert_eq!(evidence.tags.len(), 1);
    }

    #[test]
    fn test_relevance_score() {
        let mut evidence = Evidence::new("Test".to_string(), EvidenceType::Photo);

        assert!(evidence.set_relevance(8).is_ok());
        assert_eq!(evidence.relevance_score, 8);

        assert!(evidence.set_relevance(11).is_err());
    }

    #[test]
    fn test_file_attachment() {
        let mut evidence = Evidence::new("Photo".to_string(), EvidenceType::Photo);

        evidence.attach_file("/path/to/photo.jpg".to_string(), 1024 * 500, "jpg".to_string());
        assert!(evidence.has_file());

        let size_str = evidence.file_size_string().unwrap();
        assert!(size_str.contains("KB"));
    }

    #[test]
    fn test_checksum() {
        let mut evidence = Evidence::new("Document".to_string(), EvidenceType::Document);

        let checksum = "abc123def456";
        evidence.set_checksum(checksum.to_string());

        assert!(evidence.verify_checksum(checksum));
        assert!(!evidence.verify_checksum("wrong"));
    }
}
