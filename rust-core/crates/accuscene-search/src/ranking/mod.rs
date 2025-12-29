//! Ranking algorithms and search results

pub mod bm25;
pub mod facets;

use crate::config::SearchConfig;
use crate::index::schema::IndexSchema;
use serde::{Deserialize, Serialize};
use tantivy::Document;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    pub total: usize,
    pub hits: Vec<SearchHit>,
    pub facets: Option<Vec<FacetResult>>,
    pub took_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub id: String,
    pub score: f32,
    pub document: serde_json::Value,
    pub highlights: Option<Vec<Highlight>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Highlight {
    pub field: String,
    pub snippet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacetResult {
    pub field: String,
    pub values: Vec<FacetValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacetValue {
    pub value: String,
    pub count: u64,
}

impl SearchResults {
    pub fn new(
        results: Vec<(f32, Document)>,
        schema: &IndexSchema,
        config: &SearchConfig,
    ) -> Self {
        let start = std::time::Instant::now();

        let hits: Vec<SearchHit> = results
            .into_iter()
            .map(|(score, doc)| {
                let id = doc
                    .get_first(schema.id)
                    .and_then(|v| v.as_text())
                    .unwrap_or("")
                    .to_string();

                let document = Self::document_to_json(&doc, schema);

                SearchHit {
                    id,
                    score,
                    document,
                    highlights: None,
                }
            })
            .collect();

        let total = hits.len();
        let took_ms = start.elapsed().as_millis() as u64;

        Self {
            total,
            hits,
            facets: None,
            took_ms,
        }
    }

    pub fn empty() -> Self {
        Self {
            total: 0,
            hits: Vec::new(),
            facets: None,
            took_ms: 0,
        }
    }

    pub fn with_facets(mut self, facets: Vec<FacetResult>) -> Self {
        self.facets = Some(facets);
        self
    }

    pub fn with_highlights(mut self, highlights: Vec<(String, Vec<Highlight>)>) -> Self {
        let highlight_map: std::collections::HashMap<String, Vec<Highlight>> =
            highlights.into_iter().collect();

        for hit in &mut self.hits {
            if let Some(hl) = highlight_map.get(&hit.id) {
                hit.highlights = Some(hl.clone());
            }
        }

        self
    }

    fn document_to_json(doc: &Document, schema: &IndexSchema) -> serde_json::Value {
        let mut json = serde_json::Map::new();

        // Extract all fields
        for (field, field_entry) in schema.tantivy_schema.fields() {
            if let Some(value) = doc.get_first(field) {
                let json_value = match value {
                    tantivy::schema::Value::Str(s) => {
                        serde_json::Value::String(s.to_string())
                    }
                    tantivy::schema::Value::U64(n) => {
                        serde_json::Value::Number((*n).into())
                    }
                    tantivy::schema::Value::I64(n) => {
                        serde_json::Value::Number((*n).into())
                    }
                    tantivy::schema::Value::F64(n) => {
                        serde_json::Number::from_f64(*n)
                            .map(serde_json::Value::Number)
                            .unwrap_or(serde_json::Value::Null)
                    }
                    tantivy::schema::Value::Date(dt) => {
                        let timestamp = dt.into_timestamp_secs();
                        serde_json::Value::Number(timestamp.into())
                    }
                    tantivy::schema::Value::JsonObject(obj) => obj.clone(),
                    _ => serde_json::Value::Null,
                };

                json.insert(field_entry.name().to_string(), json_value);
            }
        }

        serde_json::Value::Object(json)
    }

    pub fn is_empty(&self) -> bool {
        self.total == 0
    }

    pub fn page(&self, page: usize, per_page: usize) -> Vec<&SearchHit> {
        let start = page * per_page;
        let end = std::cmp::min(start + per_page, self.hits.len());

        if start >= self.hits.len() {
            return Vec::new();
        }

        self.hits[start..end].iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_results() {
        let results = SearchResults::empty();
        assert_eq!(results.total, 0);
        assert!(results.is_empty());
    }

    #[test]
    fn test_pagination() {
        let mut results = SearchResults::empty();
        results.hits = (0..50)
            .map(|i| SearchHit {
                id: format!("doc{}", i),
                score: 1.0,
                document: serde_json::json!({}),
                highlights: None,
            })
            .collect();
        results.total = 50;

        let page1 = results.page(0, 10);
        assert_eq!(page1.len(), 10);
        assert_eq!(page1[0].id, "doc0");

        let page2 = results.page(1, 10);
        assert_eq!(page2.len(), 10);
        assert_eq!(page2[0].id, "doc10");
    }
}
