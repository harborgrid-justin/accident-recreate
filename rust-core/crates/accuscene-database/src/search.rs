//! Full-text search with FTS5
//!
//! Provides high-performance full-text search capabilities using SQLite FTS5,
//! with support for ranked results, snippets, and highlighting.

use crate::error::{DatabaseError, DbResult};
use rusqlite::{params, Connection, Row};
use serde::{Deserialize, Serialize};

/// Search result with ranking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Entity ID
    pub id: String,
    /// Relevance rank (lower is better)
    pub rank: f64,
    /// Snippet with highlighted matches
    pub snippet: Option<String>,
    /// Additional metadata
    pub metadata: serde_json::Value,
}

/// Search options
#[derive(Debug, Clone)]
pub struct SearchOptions {
    /// Maximum number of results
    pub limit: usize,
    /// Offset for pagination
    pub offset: usize,
    /// Include snippets in results
    pub include_snippets: bool,
    /// Snippet context length
    pub snippet_context: usize,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            limit: 20,
            offset: 0,
            include_snippets: true,
            snippet_context: 64,
        }
    }
}

/// Full-text search manager
pub struct SearchManager;

impl SearchManager {
    /// Create a new search manager
    pub fn new() -> Self {
        Self
    }

    /// Search cases
    pub fn search_cases(
        &self,
        conn: &Connection,
        query: &str,
        options: &SearchOptions,
    ) -> DbResult<Vec<SearchResult>> {
        self.search_fts(conn, "cases_fts", "cases", query, options)
    }

    /// Search evidence
    pub fn search_evidence(
        &self,
        conn: &Connection,
        query: &str,
        options: &SearchOptions,
    ) -> DbResult<Vec<SearchResult>> {
        self.search_fts(conn, "evidence_fts", "evidence", query, options)
    }

    /// Generic FTS5 search
    fn search_fts(
        &self,
        conn: &Connection,
        fts_table: &str,
        source_table: &str,
        query: &str,
        options: &SearchOptions,
    ) -> DbResult<Vec<SearchResult>> {
        // Sanitize query for FTS5
        let sanitized_query = self.sanitize_fts_query(query);

        let sql = if options.include_snippets {
            format!(
                "SELECT {}.rowid, rank, snippet({}, -1, '<mark>', '</mark>', '...', {})
                 FROM {} JOIN {} ON {}.rowid = {}.rowid
                 WHERE {} MATCH ?
                 ORDER BY rank
                 LIMIT ? OFFSET ?",
                source_table,
                fts_table,
                options.snippet_context,
                fts_table,
                source_table,
                fts_table,
                source_table,
                fts_table
            )
        } else {
            format!(
                "SELECT {}.rowid, rank
                 FROM {} JOIN {} ON {}.rowid = {}.rowid
                 WHERE {} MATCH ?
                 ORDER BY rank
                 LIMIT ? OFFSET ?",
                source_table, fts_table, source_table, fts_table, source_table, fts_table
            )
        };

        let mut stmt = conn.prepare(&sql).map_err(|e| {
            DatabaseError::SearchError(format!("Failed to prepare search query: {}", e))
        })?;

        let results = stmt
            .query_map(
                params![sanitized_query, options.limit as i64, options.offset as i64],
                |row| {
                    Ok(SearchResult {
                        id: row.get::<_, i64>(0)?.to_string(),
                        rank: row.get(1)?,
                        snippet: if options.include_snippets {
                            Some(row.get(2)?)
                        } else {
                            None
                        },
                        metadata: serde_json::Value::Null,
                    })
                },
            )
            .map_err(|e| DatabaseError::SearchError(format!("Search query failed: {}", e)))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| DatabaseError::SearchError(format!("Failed to collect results: {}", e)))?;

        Ok(results)
    }

    /// Sanitize FTS5 query to prevent syntax errors
    fn sanitize_fts_query(&self, query: &str) -> String {
        // Remove or escape special FTS5 characters
        let mut sanitized = query
            .replace('"', "")
            .replace('*', "")
            .replace('(', "")
            .replace(')', "");

        // Add wildcard suffix for prefix matching
        if !sanitized.is_empty() {
            sanitized = format!("{}*", sanitized.trim());
        }

        sanitized
    }

    /// Advanced search with boolean operators
    pub fn advanced_search(
        &self,
        conn: &Connection,
        fts_table: &str,
        source_table: &str,
        query: &SearchQuery,
        options: &SearchOptions,
    ) -> DbResult<Vec<SearchResult>> {
        let fts_query = query.to_fts5_query();
        self.search_fts(conn, fts_table, source_table, &fts_query, options)
    }

    /// Get search suggestions based on partial input
    pub fn suggest(
        &self,
        conn: &Connection,
        fts_table: &str,
        source_table: &str,
        partial: &str,
        limit: usize,
    ) -> DbResult<Vec<String>> {
        let query = format!("{}*", partial.trim());

        let sql = format!(
            "SELECT DISTINCT snippet({}, -1, '', '', '', 10)
             FROM {}
             WHERE {} MATCH ?
             LIMIT ?",
            fts_table, fts_table, fts_table
        );

        let mut stmt = conn.prepare(&sql).map_err(|e| {
            DatabaseError::SearchError(format!("Failed to prepare suggestion query: {}", e))
        })?;

        let suggestions = stmt
            .query_map(params![query, limit as i64], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| DatabaseError::SearchError(format!("Failed to get suggestions: {}", e)))?;

        Ok(suggestions)
    }
}

impl Default for SearchManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Advanced search query builder
#[derive(Debug, Clone)]
pub enum SearchQuery {
    /// Simple term search
    Term(String),
    /// Phrase search (exact match)
    Phrase(String),
    /// AND operator
    And(Vec<SearchQuery>),
    /// OR operator
    Or(Vec<SearchQuery>),
    /// NOT operator
    Not(Box<SearchQuery>),
    /// NEAR operator (terms within N tokens)
    Near(Vec<String>, usize),
}

impl SearchQuery {
    /// Convert search query to FTS5 query syntax
    pub fn to_fts5_query(&self) -> String {
        match self {
            SearchQuery::Term(term) => format!("{}*", term),
            SearchQuery::Phrase(phrase) => format!("\"{}\"", phrase),
            SearchQuery::And(queries) => {
                let parts: Vec<String> = queries.iter().map(|q| q.to_fts5_query()).collect();
                parts.join(" AND ")
            }
            SearchQuery::Or(queries) => {
                let parts: Vec<String> = queries.iter().map(|q| q.to_fts5_query()).collect();
                format!("({})", parts.join(" OR "))
            }
            SearchQuery::Not(query) => format!("NOT {}", query.to_fts5_query()),
            SearchQuery::Near(terms, distance) => {
                format!("NEAR({})", terms.join(&format!(" ", )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_fts_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE cases (id INTEGER PRIMARY KEY, title TEXT, description TEXT);
             CREATE VIRTUAL TABLE cases_fts USING fts5(title, description, content=cases, content_rowid=rowid);
             INSERT INTO cases (title, description) VALUES ('Test Case', 'This is a test description');
             INSERT INTO cases_fts(rowid, title, description) SELECT rowid, title, description FROM cases;",
        )
        .unwrap();
        conn
    }

    #[test]
    fn test_search_manager() {
        let conn = setup_fts_db();
        let manager = SearchManager::new();
        let options = SearchOptions::default();

        let results = manager.search_cases(&conn, "test", &options).unwrap();
        assert!(results.len() > 0);
    }

    #[test]
    fn test_sanitize_query() {
        let manager = SearchManager::new();
        let sanitized = manager.sanitize_fts_query("test*query\"");
        assert!(!sanitized.contains('"'));
        assert!(sanitized.ends_with('*'));
    }

    #[test]
    fn test_search_query_builder() {
        let query = SearchQuery::And(vec![
            SearchQuery::Term("accident".to_string()),
            SearchQuery::Phrase("traffic collision".to_string()),
        ]);

        let fts_query = query.to_fts5_query();
        assert!(fts_query.contains("AND"));
        assert!(fts_query.contains("\"traffic collision\""));
    }
}
