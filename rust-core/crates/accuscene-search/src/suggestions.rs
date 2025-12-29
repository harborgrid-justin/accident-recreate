//! Auto-complete and query suggestions

use crate::error::{SearchError, SearchResult};
use crate::index::schema::IndexSchema;
use std::collections::HashSet;
use tantivy::Searcher;

pub struct SuggestionEngine<'a> {
    searcher: Searcher,
    schema: &'a IndexSchema,
}

impl<'a> SuggestionEngine<'a> {
    pub fn new(searcher: Searcher, schema: &'a IndexSchema) -> Self {
        Self { searcher, schema }
    }

    /// Generate auto-complete suggestions
    pub async fn suggest(
        &self,
        prefix: &str,
        limit: usize,
    ) -> SearchResult<Vec<String>> {
        if prefix.is_empty() {
            return Ok(Vec::new());
        }

        let prefix_lower = prefix.to_lowercase();
        let mut suggestions = HashSet::new();

        // Search across text fields
        for field in self.schema.text_fields() {
            let field_suggestions = self
                .suggest_from_field(field, &prefix_lower, limit * 2)
                .await?;

            suggestions.extend(field_suggestions);
        }

        // Convert to sorted vector
        let mut results: Vec<String> = suggestions.into_iter().collect();

        // Rank by relevance
        results.sort_by(|a, b| {
            let a_score = self.relevance_score(a, &prefix_lower);
            let b_score = self.relevance_score(b, &prefix_lower);
            b_score.partial_cmp(&a_score).unwrap_or(std::cmp::Ordering::Equal)
        });

        results.truncate(limit);

        Ok(results)
    }

    async fn suggest_from_field(
        &self,
        field: tantivy::schema::Field,
        prefix: &str,
        limit: usize,
    ) -> SearchResult<Vec<String>> {
        let mut suggestions = Vec::new();

        for segment_reader in self.searcher.segment_readers() {
            let inv_index = segment_reader.inverted_index(field)?;
            let mut stream = inv_index.terms().stream()?;

            while let Some(term) = stream.next() {
                let (term_bytes, _) = term?;
                let term_str = std::str::from_utf8(term_bytes)
                    .map_err(|e| SearchError::IndexError(e.to_string()))?;

                if term_str.to_lowercase().starts_with(prefix) {
                    suggestions.push(term_str.to_string());

                    if suggestions.len() >= limit {
                        break;
                    }
                }
            }
        }

        Ok(suggestions)
    }

    fn relevance_score(&self, suggestion: &str, prefix: &str) -> f32 {
        let mut score = 0.0;

        // Exact prefix match
        if suggestion.to_lowercase().starts_with(prefix) {
            score += 10.0;
        }

        // Length penalty (prefer shorter suggestions)
        score -= suggestion.len() as f32 * 0.1;

        // Fuzzy match boost
        let distance = strsim::levenshtein(&suggestion.to_lowercase(), prefix);
        score -= distance as f32;

        score
    }

    /// Get suggestions with fuzzy matching
    pub async fn suggest_fuzzy(
        &self,
        query: &str,
        limit: usize,
        max_distance: u8,
    ) -> SearchResult<Vec<(String, f32)>> {
        let query_lower = query.to_lowercase();
        let mut suggestions = Vec::new();

        for field in self.schema.text_fields() {
            let field_suggestions = self
                .fuzzy_from_field(field, &query_lower, limit * 2, max_distance)
                .await?;

            suggestions.extend(field_suggestions);
        }

        // Sort by similarity score
        suggestions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        suggestions.truncate(limit);

        Ok(suggestions)
    }

    async fn fuzzy_from_field(
        &self,
        field: tantivy::schema::Field,
        query: &str,
        limit: usize,
        max_distance: u8,
    ) -> SearchResult<Vec<(String, f32)>> {
        let mut suggestions = Vec::new();

        for segment_reader in self.searcher.segment_readers() {
            let inv_index = segment_reader.inverted_index(field)?;
            let mut stream = inv_index.terms().stream()?;

            while let Some(term) = stream.next() {
                let (term_bytes, _) = term?;
                let term_str = std::str::from_utf8(term_bytes)
                    .map_err(|e| SearchError::IndexError(e.to_string()))?;

                let distance = strsim::levenshtein(&term_str.to_lowercase(), query);

                if distance <= max_distance as usize {
                    let similarity = 1.0 - (distance as f32 / query.len().max(term_str.len()) as f32);
                    suggestions.push((term_str.to_string(), similarity));

                    if suggestions.len() >= limit {
                        break;
                    }
                }
            }
        }

        Ok(suggestions)
    }

    /// Suggest query corrections (did you mean?)
    pub async fn suggest_correction(
        &self,
        query: &str,
    ) -> SearchResult<Option<String>> {
        let fuzzy_suggestions = self.suggest_fuzzy(query, 1, 2).await?;

        if let Some((suggestion, similarity)) = fuzzy_suggestions.first() {
            // Only suggest if similarity is high enough and different from query
            if *similarity > 0.7 && suggestion != query {
                return Ok(Some(suggestion.clone()));
            }
        }

        Ok(None)
    }
}

/// Popular query tracker for trending suggestions
pub struct PopularQueries {
    queries: dashmap::DashMap<String, QueryStats>,
    max_size: usize,
}

#[derive(Clone, Debug)]
struct QueryStats {
    count: u64,
    last_used: std::time::Instant,
}

impl PopularQueries {
    pub fn new(max_size: usize) -> Self {
        Self {
            queries: dashmap::DashMap::new(),
            max_size,
        }
    }

    /// Record a query
    pub fn record(&self, query: &str) {
        let normalized = query.to_lowercase().trim().to_string();

        if normalized.is_empty() {
            return;
        }

        self.queries
            .entry(normalized)
            .and_modify(|stats| {
                stats.count += 1;
                stats.last_used = std::time::Instant::now();
            })
            .or_insert(QueryStats {
                count: 1,
                last_used: std::time::Instant::now(),
            });

        // Trim if too large
        if self.queries.len() > self.max_size {
            self.trim_old_entries();
        }
    }

    /// Get top N popular queries
    pub fn top(&self, n: usize) -> Vec<(String, u64)> {
        let mut queries: Vec<(String, u64)> = self
            .queries
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().count))
            .collect();

        queries.sort_by(|a, b| b.1.cmp(&a.1));
        queries.truncate(n);

        queries
    }

    /// Get trending queries (recent + popular)
    pub fn trending(&self, n: usize) -> Vec<(String, f32)> {
        let now = std::time::Instant::now();

        let mut queries: Vec<(String, f32)> = self
            .queries
            .iter()
            .map(|entry| {
                let recency_score = 1.0 / (1.0 + entry.value().last_used.elapsed().as_secs() as f32 / 3600.0);
                let popularity_score = entry.value().count as f32;
                let combined_score = recency_score * popularity_score.ln();

                (entry.key().clone(), combined_score)
            })
            .collect();

        queries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        queries.truncate(n);

        queries
    }

    fn trim_old_entries(&self) {
        let cutoff = std::time::Instant::now() - std::time::Duration::from_secs(30 * 24 * 3600);

        self.queries
            .retain(|_, stats| stats.last_used > cutoff);
    }

    pub fn clear(&self) {
        self.queries.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_popular_queries() {
        let tracker = PopularQueries::new(100);

        tracker.record("accident report");
        tracker.record("accident report");
        tracker.record("vehicle damage");

        let top = tracker.top(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].0, "accident report");
        assert_eq!(top[0].1, 2);
    }

    #[test]
    fn test_popular_queries_normalization() {
        let tracker = PopularQueries::new(100);

        tracker.record("Test Query");
        tracker.record("test query");
        tracker.record("TEST QUERY");

        let top = tracker.top(1);
        assert_eq!(top.len(), 1);
        assert_eq!(top[0].1, 3);
    }

    #[test]
    fn test_popular_queries_trending() {
        let tracker = PopularQueries::new(100);

        tracker.record("query1");
        tracker.record("query2");
        tracker.record("query2");

        let trending = tracker.trending(2);
        assert_eq!(trending.len(), 2);
    }
}
