//! Data export for compliance (GDPR data portability)

use crate::error::{SecurityError, SecurityResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Export format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    /// JSON format
    Json,
    /// CSV format
    Csv,
    /// XML format
    Xml,
}

/// Data exporter for GDPR compliance
#[derive(Debug)]
pub struct DataExporter {
    /// Export metadata
    metadata: ExportMetadata,
}

impl DataExporter {
    /// Create a new data exporter
    pub fn new(user_id: impl Into<String>) -> Self {
        Self {
            metadata: ExportMetadata {
                user_id: user_id.into(),
                requested_at: Utc::now(),
                generated_at: None,
                format: ExportFormat::Json,
            },
        }
    }

    /// Set export format
    pub fn with_format(mut self, format: ExportFormat) -> Self {
        self.metadata.format = format;
        self
    }

    /// Export user data
    pub fn export(&mut self, data: UserDataExport) -> SecurityResult<ExportPackage> {
        self.metadata.generated_at = Some(Utc::now());

        let content = match self.metadata.format {
            ExportFormat::Json => self.export_json(&data)?,
            ExportFormat::Csv => self.export_csv(&data)?,
            ExportFormat::Xml => self.export_xml(&data)?,
        };

        Ok(ExportPackage {
            metadata: self.metadata.clone(),
            content,
            checksum: self.calculate_checksum(&content),
        })
    }

    /// Export as JSON
    fn export_json(&self, data: &UserDataExport) -> SecurityResult<String> {
        serde_json::to_string_pretty(data)
            .map_err(|e| SecurityError::SerializationError(e))
    }

    /// Export as CSV (simplified)
    fn export_csv(&self, data: &UserDataExport) -> SecurityResult<String> {
        let mut csv = String::new();

        // Profile section
        csv.push_str("PROFILE\n");
        csv.push_str("Field,Value\n");
        csv.push_str(&format!("User ID,{}\n", data.profile.user_id));
        csv.push_str(&format!("Email,{}\n", data.profile.email));
        csv.push_str(&format!("Name,{}\n", data.profile.name.as_deref().unwrap_or("")));
        csv.push_str(&format!("Created At,{}\n\n", data.profile.created_at));

        // Consents section
        csv.push_str("CONSENTS\n");
        csv.push_str("Purpose,Granted,Recorded At\n");
        for (purpose, record) in &data.consents {
            csv.push_str(&format!("{:?},{},{}\n", purpose, record.granted, record.recorded_at));
        }
        csv.push_str("\n");

        // Activity section
        csv.push_str("ACTIVITY\n");
        csv.push_str(&format!("Total Activities: {}\n", data.activities.len()));

        Ok(csv)
    }

    /// Export as XML (simplified)
    fn export_xml(&self, data: &UserDataExport) -> SecurityResult<String> {
        let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str("<UserDataExport>\n");

        // Profile
        xml.push_str("  <Profile>\n");
        xml.push_str(&format!("    <UserID>{}</UserID>\n", data.profile.user_id));
        xml.push_str(&format!("    <Email>{}</Email>\n", data.profile.email));
        if let Some(name) = &data.profile.name {
            xml.push_str(&format!("    <Name>{}</Name>\n", name));
        }
        xml.push_str(&format!("    <CreatedAt>{}</CreatedAt>\n", data.profile.created_at));
        xml.push_str("  </Profile>\n");

        // Consents
        xml.push_str("  <Consents>\n");
        for (purpose, record) in &data.consents {
            xml.push_str("    <Consent>\n");
            xml.push_str(&format!("      <Purpose>{:?}</Purpose>\n", purpose));
            xml.push_str(&format!("      <Granted>{}</Granted>\n", record.granted));
            xml.push_str(&format!("      <RecordedAt>{}</RecordedAt>\n", record.recorded_at));
            xml.push_str("    </Consent>\n");
        }
        xml.push_str("  </Consents>\n");

        xml.push_str("</UserDataExport>\n");

        Ok(xml)
    }

    /// Calculate checksum for verification
    fn calculate_checksum(&self, content: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        hex::encode(hasher.finalize())
    }
}

/// Export metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetadata {
    /// User ID
    pub user_id: String,

    /// When export was requested
    pub requested_at: DateTime<Utc>,

    /// When export was generated
    pub generated_at: Option<DateTime<Utc>>,

    /// Export format
    pub format: ExportFormat,
}

/// Export package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportPackage {
    /// Export metadata
    pub metadata: ExportMetadata,

    /// Export content
    pub content: String,

    /// SHA-256 checksum
    pub checksum: String,
}

impl ExportPackage {
    /// Verify the checksum
    pub fn verify_checksum(&self) -> bool {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(self.content.as_bytes());
        let calculated = hex::encode(hasher.finalize());

        calculated == self.checksum
    }

    /// Save to file
    pub fn save_to_file(&self, path: &str) -> SecurityResult<()> {
        std::fs::write(path, &self.content)
            .map_err(|e| SecurityError::IoError(e))
    }
}

/// User data export structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDataExport {
    /// User profile
    pub profile: UserProfile,

    /// Consent records
    pub consents: HashMap<super::gdpr::ConsentPurpose, super::gdpr::ConsentRecord>,

    /// Activity history
    pub activities: Vec<ActivityRecord>,

    /// User preferences
    pub preferences: HashMap<String, serde_json::Value>,

    /// Custom data
    pub custom_data: HashMap<String, serde_json::Value>,
}

impl UserDataExport {
    /// Create a new user data export
    pub fn new(user_id: impl Into<String>, email: impl Into<String>) -> Self {
        Self {
            profile: UserProfile {
                user_id: user_id.into(),
                email: email.into(),
                name: None,
                phone: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                metadata: HashMap::new(),
            },
            consents: HashMap::new(),
            activities: Vec::new(),
            preferences: HashMap::new(),
            custom_data: HashMap::new(),
        }
    }

    /// Add consent record
    pub fn add_consent(
        mut self,
        purpose: super::gdpr::ConsentPurpose,
        record: super::gdpr::ConsentRecord,
    ) -> Self {
        self.consents.insert(purpose, record);
        self
    }

    /// Add activity
    pub fn add_activity(mut self, activity: ActivityRecord) -> Self {
        self.activities.push(activity);
        self
    }

    /// Add preference
    pub fn add_preference(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.preferences.insert(key.into(), value);
        self
    }

    /// Add custom data
    pub fn add_custom_data(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.custom_data.insert(key.into(), value);
        self
    }
}

/// User profile for export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    /// User ID
    pub user_id: String,

    /// Email address
    pub email: String,

    /// Full name
    pub name: Option<String>,

    /// Phone number
    pub phone: Option<String>,

    /// Created at
    pub created_at: DateTime<Utc>,

    /// Updated at
    pub updated_at: DateTime<Utc>,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Activity record for export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityRecord {
    /// Activity type
    pub activity_type: String,

    /// Activity description
    pub description: String,

    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// Additional data
    pub data: HashMap<String, serde_json::Value>,
}

impl ActivityRecord {
    /// Create a new activity record
    pub fn new(
        activity_type: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            activity_type: activity_type.into(),
            description: description.into(),
            timestamp: Utc::now(),
            data: HashMap::new(),
        }
    }

    /// Add data
    pub fn with_data(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.data.insert(key.into(), value);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compliance::gdpr::{ConsentPurpose, ConsentRecord};

    #[test]
    fn test_json_export() {
        let data = UserDataExport::new("user123", "user@example.com")
            .add_consent(
                ConsentPurpose::Marketing,
                ConsentRecord {
                    granted: true,
                    recorded_at: Utc::now(),
                },
            )
            .add_activity(ActivityRecord::new("login", "User logged in"));

        let mut exporter = DataExporter::new("user123").with_format(ExportFormat::Json);
        let package = exporter.export(data).unwrap();

        assert!(package.content.contains("user123"));
        assert!(package.content.contains("user@example.com"));
        assert!(package.verify_checksum());
    }

    #[test]
    fn test_csv_export() {
        let data = UserDataExport::new("user123", "user@example.com");

        let mut exporter = DataExporter::new("user123").with_format(ExportFormat::Csv);
        let package = exporter.export(data).unwrap();

        assert!(package.content.contains("PROFILE"));
        assert!(package.content.contains("user123"));
        assert!(package.verify_checksum());
    }

    #[test]
    fn test_xml_export() {
        let data = UserDataExport::new("user123", "user@example.com");

        let mut exporter = DataExporter::new("user123").with_format(ExportFormat::Xml);
        let package = exporter.export(data).unwrap();

        assert!(package.content.contains("<?xml"));
        assert!(package.content.contains("<UserDataExport>"));
        assert!(package.content.contains("user123"));
        assert!(package.verify_checksum());
    }

    #[test]
    fn test_checksum_verification() {
        let data = UserDataExport::new("user123", "user@example.com");
        let mut exporter = DataExporter::new("user123");
        let mut package = exporter.export(data).unwrap();

        assert!(package.verify_checksum());

        // Tamper with content
        package.content.push_str("tampered");

        assert!(!package.verify_checksum());
    }

    #[test]
    fn test_export_package_metadata() {
        let data = UserDataExport::new("user123", "user@example.com");
        let mut exporter = DataExporter::new("user123");
        let package = exporter.export(data).unwrap();

        assert_eq!(package.metadata.user_id, "user123");
        assert!(package.metadata.generated_at.is_some());
        assert_eq!(package.metadata.format, ExportFormat::Json);
    }
}
