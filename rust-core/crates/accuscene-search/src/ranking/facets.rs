//! Faceted search support

use crate::error::{SearchError, SearchResult};
use crate::index::schema::IndexSchema;
use crate::ranking::{FacetResult, FacetValue};
use std::collections::HashMap;
use tantivy::Searcher;

pub struct FacetCollector<'a> {
    schema: &'a IndexSchema,
    field_name: String,
}

impl<'a> FacetCollector<'a> {
    pub fn new(schema: &'a IndexSchema, field_name: &str) -> SearchResult<Self> {
        if !schema.has_field(field_name) {
            return Err(SearchError::InvalidField(field_name.to_string()));
        }

        Ok(Self {
            schema,
            field_name: field_name.to_string(),
        })
    }

    /// Collect facet counts from search results
    pub async fn collect(&self, searcher: &Searcher) -> SearchResult<Vec<(String, u64)>> {
        let field = self.schema.get_field(&self.field_name)?;

        let mut counts: HashMap<String, u64> = HashMap::new();

        // Collect all documents
        for segment_reader in searcher.segment_readers() {
            let inv_index = segment_reader.inverted_index(field)?;

            for term in inv_index.terms().stream()? {
                let term_str = std::str::from_utf8(term?.0)
                    .map_err(|e| SearchError::IndexError(e.to_string()))?;

                let doc_freq = inv_index.doc_freq(&tantivy::Term::from_field_text(field, term_str))?;

                *counts.entry(term_str.to_string()).or_insert(0) += doc_freq as u64;
            }
        }

        // Convert to sorted vector
        let mut results: Vec<(String, u64)> = counts.into_iter().collect();
        results.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by count descending

        Ok(results)
    }

    /// Collect facets for multiple fields
    pub async fn collect_multi(
        schema: &IndexSchema,
        searcher: &Searcher,
        fields: &[&str],
    ) -> SearchResult<Vec<FacetResult>> {
        let mut results = Vec::new();

        for field_name in fields {
            let collector = Self::new(schema, field_name)?;
            let counts = collector.collect(searcher).await?;

            let values = counts
                .into_iter()
                .map(|(value, count)| FacetValue { value, count })
                .collect();

            results.push(FacetResult {
                field: field_name.to_string(),
                values,
            });
        }

        Ok(results)
    }
}

/// Facet aggregation with caching
pub struct CachedFacetCollector {
    cache: dashmap::DashMap<String, (Vec<FacetValue>, std::time::Instant)>,
    ttl: std::time::Duration,
}

impl CachedFacetCollector {
    pub fn new(ttl_secs: u64) -> Self {
        Self {
            cache: dashmap::DashMap::new(),
            ttl: std::time::Duration::from_secs(ttl_secs),
        }
    }

    /// Get facets with caching
    pub async fn get_facets(
        &self,
        schema: &IndexSchema,
        searcher: &Searcher,
        field: &str,
    ) -> SearchResult<Vec<FacetValue>> {
        // Check cache
        if let Some(entry) = self.cache.get(field) {
            let (values, timestamp) = entry.value();
            if timestamp.elapsed() < self.ttl {
                return Ok(values.clone());
            }
        }

        // Collect fresh facets
        let collector = FacetCollector::new(schema, field)?;
        let counts = collector.collect(searcher).await?;

        let values: Vec<FacetValue> = counts
            .into_iter()
            .map(|(value, count)| FacetValue { value, count })
            .collect();

        // Update cache
        self.cache
            .insert(field.to_string(), (values.clone(), std::time::Instant::now()));

        Ok(values)
    }

    /// Clear expired cache entries
    pub fn clear_expired(&self) {
        self.cache.retain(|_, (_, timestamp)| timestamp.elapsed() < self.ttl);
    }

    /// Clear all cache
    pub fn clear_all(&self) {
        self.cache.clear();
    }
}

/// Facet range aggregation (for numeric/date fields)
pub struct RangeFacet {
    pub field: String,
    pub ranges: Vec<(String, f64, f64)>, // (label, min, max)
}

impl RangeFacet {
    pub fn new(field: String) -> Self {
        Self {
            field,
            ranges: Vec::new(),
        }
    }

    pub fn add_range(mut self, label: String, min: f64, max: f64) -> Self {
        self.ranges.push((label, min, max));
        self
    }

    /// Create standard date ranges
    pub fn date_ranges(field: String) -> Self {
        let now = chrono::Utc::now().timestamp() as f64;
        let day = 86400.0;

        Self {
            field,
            ranges: vec![
                ("Last 24 hours".to_string(), now - day, now),
                ("Last 7 days".to_string(), now - 7.0 * day, now),
                ("Last 30 days".to_string(), now - 30.0 * day, now),
                ("Last 90 days".to_string(), now - 90.0 * day, now),
                ("Older".to_string(), 0.0, now - 90.0 * day),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_range_facet_creation() {
        let facet = RangeFacet::new("price".to_string())
            .add_range("Low".to_string(), 0.0, 100.0)
            .add_range("Medium".to_string(), 100.0, 500.0)
            .add_range("High".to_string(), 500.0, f64::MAX);

        assert_eq!(facet.ranges.len(), 3);
    }

    #[test]
    fn test_date_ranges() {
        let facet = RangeFacet::date_ranges("created_at".to_string());
        assert_eq!(facet.ranges.len(), 5);
        assert_eq!(facet.ranges[0].0, "Last 24 hours");
    }

    #[test]
    fn test_cached_facet_collector() {
        let collector = CachedFacetCollector::new(300);
        collector.clear_all();
        assert_eq!(collector.cache.len(), 0);
    }
}
