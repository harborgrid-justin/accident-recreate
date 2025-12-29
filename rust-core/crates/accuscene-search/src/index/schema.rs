//! Index schema definitions

use crate::error::{SearchError, SearchResult};
use serde_json::Value;
use tantivy::schema::*;
use tantivy::{doc, Document};

/// Index schema for AccuScene documents
pub struct IndexSchema {
    pub tantivy_schema: Schema,
    pub id: Field,
    pub title: Field,
    pub content: Field,
    pub description: Field,
    pub category: Field,
    pub status: Field,
    pub severity: Field,
    pub location: Field,
    pub tags: Field,
    pub created_at: Field,
    pub updated_at: Field,
    pub created_by: Field,
    pub metadata: Field,
}

impl Default for IndexSchema {
    fn default() -> Self {
        let mut schema_builder = Schema::builder();

        // Document ID (indexed, stored)
        let id = schema_builder.add_text_field("id", STRING | STORED);

        // Full-text searchable fields
        let text_options = TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer("en_stem")
                    .set_index_option(IndexRecordOption::WithFreqsAndPositions),
            )
            .set_stored();

        let title = schema_builder.add_text_field("title", text_options.clone());
        let content = schema_builder.add_text_field("content", text_options.clone());
        let description = schema_builder.add_text_field("description", text_options.clone());

        // Faceted fields (for filtering and aggregation)
        let facet_options = TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer("raw")
                    .set_index_option(IndexRecordOption::Basic),
            )
            .set_stored();

        let category = schema_builder.add_text_field("category", facet_options.clone());
        let status = schema_builder.add_text_field("status", facet_options.clone());
        let severity = schema_builder.add_text_field("severity", facet_options.clone());
        let location = schema_builder.add_text_field("location", TEXT | STORED);

        // Tags (multi-value)
        let tags = schema_builder.add_text_field("tags", facet_options.clone());

        // Timestamp fields
        let date_options = NumericOptions::default()
            .set_indexed()
            .set_stored()
            .set_fast();

        let created_at = schema_builder.add_date_field("created_at", date_options.clone());
        let updated_at = schema_builder.add_date_field("updated_at", date_options);

        // User fields
        let created_by = schema_builder.add_text_field("created_by", facet_options);

        // Metadata (JSON)
        let metadata = schema_builder.add_json_field("metadata", STORED);

        Self {
            tantivy_schema: schema_builder.build(),
            id,
            title,
            content,
            description,
            category,
            status,
            severity,
            location,
            tags,
            created_at,
            updated_at,
            created_by,
            metadata,
        }
    }
}

impl IndexSchema {
    /// Create a Tantivy document from a JSON value
    pub fn create_document<T: serde::Serialize>(
        &self,
        id: &str,
        data: T,
    ) -> SearchResult<Document> {
        let json = serde_json::to_value(data)
            .map_err(|e| SearchError::SerializationError(e.to_string()))?;

        let mut doc = doc!(
            self.id => id
        );

        // Extract and add fields from JSON
        if let Value::Object(map) = json {
            // Text fields
            if let Some(Value::String(title)) = map.get("title") {
                doc.add_text(self.title, title);
            }

            if let Some(Value::String(content)) = map.get("content") {
                doc.add_text(self.content, content);
            }

            if let Some(Value::String(description)) = map.get("description") {
                doc.add_text(self.description, description);
            }

            // Faceted fields
            if let Some(Value::String(category)) = map.get("category") {
                doc.add_text(self.category, category);
            }

            if let Some(Value::String(status)) = map.get("status") {
                doc.add_text(self.status, status);
            }

            if let Some(Value::String(severity)) = map.get("severity") {
                doc.add_text(self.severity, severity);
            }

            if let Some(Value::String(location)) = map.get("location") {
                doc.add_text(self.location, location);
            }

            // Tags (array)
            if let Some(Value::Array(tags)) = map.get("tags") {
                for tag in tags {
                    if let Value::String(tag_str) = tag {
                        doc.add_text(self.tags, tag_str);
                    }
                }
            }

            // Dates
            if let Some(created_at) = map.get("created_at") {
                if let Some(date) = Self::parse_date(created_at) {
                    doc.add_date(self.created_at, date);
                }
            }

            if let Some(updated_at) = map.get("updated_at") {
                if let Some(date) = Self::parse_date(updated_at) {
                    doc.add_date(self.updated_at, date);
                }
            }

            // User
            if let Some(Value::String(created_by)) = map.get("created_by") {
                doc.add_text(self.created_by, created_by);
            }

            // Metadata
            if let Some(metadata) = map.get("metadata") {
                doc.add_json_object(self.metadata, metadata.clone());
            }
        }

        Ok(doc)
    }

    fn parse_date(value: &Value) -> Option<tantivy::DateTime> {
        match value {
            Value::String(s) => {
                chrono::DateTime::parse_from_rfc3339(s)
                    .ok()
                    .and_then(|dt| tantivy::DateTime::from_timestamp_secs(dt.timestamp()))
            }
            Value::Number(n) => {
                n.as_i64()
                    .and_then(|ts| tantivy::DateTime::from_timestamp_secs(ts))
            }
            _ => None,
        }
    }

    /// Get field by name
    pub fn get_field(&self, name: &str) -> SearchResult<Field> {
        self.tantivy_schema
            .get_field(name)
            .ok_or_else(|| SearchError::InvalidField(name.to_string()))
    }

    /// Check if a field exists
    pub fn has_field(&self, name: &str) -> bool {
        self.tantivy_schema.get_field(name).is_some()
    }

    /// Get all text fields for searching
    pub fn text_fields(&self) -> Vec<Field> {
        vec![self.title, self.content, self.description, self.location]
    }

    /// Get all facet fields
    pub fn facet_fields(&self) -> Vec<(&str, Field)> {
        vec![
            ("category", self.category),
            ("status", self.status),
            ("severity", self.severity),
            ("created_by", self.created_by),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_schema_creation() {
        let schema = IndexSchema::default();
        assert!(schema.has_field("id"));
        assert!(schema.has_field("title"));
        assert!(schema.has_field("content"));
    }

    #[test]
    fn test_document_creation() {
        let schema = IndexSchema::default();
        let data = json!({
            "title": "Test Document",
            "content": "This is test content",
            "category": "accident",
            "tags": ["tag1", "tag2"]
        });

        let doc = schema.create_document("doc1", data).unwrap();
        assert!(!doc.field_values().is_empty());
    }
}
