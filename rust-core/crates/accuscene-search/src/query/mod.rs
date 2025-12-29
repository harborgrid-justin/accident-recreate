//! Query engine and execution

pub mod builder;
pub mod filters;
pub mod parser;

use crate::config::SearchConfig;
use crate::error::{SearchError, SearchResult};
use crate::index::schema::IndexSchema;
use crate::ranking::SearchResults;
use serde::{Deserialize, Serialize};
use tantivy::query::*;
use tantivy::{Searcher, Term};

pub use builder::QueryBuilder;
pub use filters::SearchFilters;
pub use parser::QueryParser;

/// Search query representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Query {
    pub text: String,
    pub fields: Vec<String>,
    pub fuzzy: bool,
    pub boost: Option<f32>,
    pub operator: QueryOperator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryOperator {
    And,
    Or,
}

impl Default for Query {
    fn default() -> Self {
        Self {
            text: String::new(),
            fields: vec!["title".to_string(), "content".to_string()],
            fuzzy: false,
            boost: None,
            operator: QueryOperator::Or,
        }
    }
}

/// Query executor
pub struct QueryExecutor<'a> {
    searcher: Searcher,
    schema: &'a IndexSchema,
    config: &'a SearchConfig,
}

impl<'a> QueryExecutor<'a> {
    pub fn new(
        searcher: Searcher,
        schema: &'a IndexSchema,
        config: &'a SearchConfig,
    ) -> Self {
        Self {
            searcher,
            schema,
            config,
        }
    }

    /// Execute a search query
    pub async fn execute(
        &self,
        query: &Query,
        filters: Option<SearchFilters>,
    ) -> SearchResult<SearchResults> {
        let tantivy_query = self.build_tantivy_query(query)?;

        // Apply filters if provided
        let final_query: Box<dyn tantivy::query::Query> = if let Some(filters) = filters {
            Box::new(BooleanQuery::new(vec![
                (Occur::Must, tantivy_query),
                (Occur::Must, self.build_filter_query(&filters)?),
            ]))
        } else {
            tantivy_query
        };

        // Execute search with timeout
        let timeout = std::time::Duration::from_millis(self.config.search_timeout_ms);
        let start = std::time::Instant::now();

        let top_docs = self
            .searcher
            .search(
                &*final_query,
                &tantivy::collector::TopDocs::with_limit(self.config.max_results),
            )
            .map_err(|e| SearchError::IndexError(format!("Search failed: {}", e)))?;

        if start.elapsed() > timeout {
            return Err(SearchError::Timeout);
        }

        // Convert to SearchResults
        let mut results = Vec::new();
        for (score, doc_address) in top_docs {
            let doc = self.searcher.doc(doc_address)?;
            results.push((score, doc));
        }

        Ok(SearchResults::new(results, self.schema, self.config))
    }

    fn build_tantivy_query(
        &self,
        query: &Query,
    ) -> SearchResult<Box<dyn tantivy::query::Query>> {
        if query.text.is_empty() {
            return Ok(Box::new(AllQuery));
        }

        let mut sub_queries: Vec<(Occur, Box<dyn tantivy::query::Query>)> = Vec::new();

        let occur = match query.operator {
            QueryOperator::And => Occur::Must,
            QueryOperator::Or => Occur::Should,
        };

        // Get fields to search
        let fields: Vec<_> = if query.fields.is_empty() {
            self.schema.text_fields()
        } else {
            query
                .fields
                .iter()
                .filter_map(|f| self.schema.get_field(f).ok())
                .collect()
        };

        for field in fields {
            if query.fuzzy && self.config.enable_fuzzy {
                // Fuzzy query
                let term = Term::from_field_text(field, &query.text);
                let fuzzy_query = FuzzyTermQuery::new(
                    term,
                    self.config.fuzzy_distance,
                    true,
                );
                sub_queries.push((occur, Box::new(fuzzy_query)));
            } else {
                // Exact term query
                let term = Term::from_field_text(field, &query.text);
                let term_query = TermQuery::new(term, tantivy::schema::IndexRecordOption::Basic);
                sub_queries.push((occur, Box::new(term_query)));
            }
        }

        if sub_queries.is_empty() {
            return Ok(Box::new(AllQuery));
        }

        Ok(Box::new(BooleanQuery::new(sub_queries)))
    }

    fn build_filter_query(
        &self,
        filters: &SearchFilters,
    ) -> SearchResult<Box<dyn tantivy::query::Query>> {
        let mut sub_queries: Vec<(Occur, Box<dyn tantivy::query::Query>)> = Vec::new();

        // Category filters
        if let Some(categories) = &filters.categories {
            let category_queries: Vec<_> = categories
                .iter()
                .map(|cat| {
                    let term = Term::from_field_text(self.schema.category, cat);
                    Box::new(TermQuery::new(
                        term,
                        tantivy::schema::IndexRecordOption::Basic,
                    )) as Box<dyn tantivy::query::Query>
                })
                .map(|q| (Occur::Should, q))
                .collect();

            if !category_queries.is_empty() {
                sub_queries.push((
                    Occur::Must,
                    Box::new(BooleanQuery::new(category_queries)),
                ));
            }
        }

        // Status filters
        if let Some(statuses) = &filters.statuses {
            let status_queries: Vec<_> = statuses
                .iter()
                .map(|status| {
                    let term = Term::from_field_text(self.schema.status, status);
                    Box::new(TermQuery::new(
                        term,
                        tantivy::schema::IndexRecordOption::Basic,
                    )) as Box<dyn tantivy::query::Query>
                })
                .map(|q| (Occur::Should, q))
                .collect();

            if !status_queries.is_empty() {
                sub_queries.push((
                    Occur::Must,
                    Box::new(BooleanQuery::new(status_queries)),
                ));
            }
        }

        // Date range filters
        if let Some(date_range) = &filters.date_range {
            let start_ts = date_range.start.timestamp();
            let end_ts = date_range.end.timestamp();

            let date_query = RangeQuery::new_i64_bounds(
                self.schema.created_at.field_name().to_string(),
                std::ops::Bound::Included(start_ts),
                std::ops::Bound::Included(end_ts),
            );

            sub_queries.push((Occur::Must, Box::new(date_query)));
        }

        if sub_queries.is_empty() {
            Ok(Box::new(AllQuery))
        } else {
            Ok(Box::new(BooleanQuery::new(sub_queries)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_creation() {
        let query = QueryBuilder::new()
            .text("test query")
            .fuzzy(true)
            .build();

        assert_eq!(query.text, "test query");
        assert!(query.fuzzy);
    }
}
