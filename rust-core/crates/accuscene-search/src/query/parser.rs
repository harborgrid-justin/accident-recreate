//! Query DSL parser

use crate::error::{SearchError, SearchResult};
use crate::query::{Query, QueryOperator};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryDSL {
    pub query: String,
    pub fields: Option<Vec<String>>,
    pub fuzzy: Option<bool>,
    pub boost: Option<f32>,
    pub operator: Option<String>,
}

pub struct QueryParser;

impl QueryParser {
    /// Parse a query DSL into a Query
    pub fn parse(dsl: QueryDSL) -> SearchResult<Query> {
        let operator = match dsl.operator.as_deref() {
            Some("AND") | Some("and") => QueryOperator::And,
            Some("OR") | Some("or") | None => QueryOperator::Or,
            Some(op) => {
                return Err(SearchError::QueryParseError(format!(
                    "Invalid operator: {}",
                    op
                )))
            }
        };

        Ok(Query {
            text: dsl.query,
            fields: dsl.fields.unwrap_or_default(),
            fuzzy: dsl.fuzzy.unwrap_or(false),
            boost: dsl.boost,
            operator,
        })
    }

    /// Parse a simple text query
    pub fn parse_text(text: &str) -> SearchResult<Query> {
        Ok(Query {
            text: text.to_string(),
            ..Default::default()
        })
    }

    /// Parse a Lucene-style query string
    pub fn parse_lucene(query_str: &str) -> SearchResult<Query> {
        // Simple Lucene-style parsing
        let mut text = String::new();
        let mut fields = Vec::new();
        let mut fuzzy = false;
        let mut operator = QueryOperator::Or;

        let tokens: Vec<&str> = query_str.split_whitespace().collect();

        for token in tokens {
            if token.contains(':') {
                // Field-specific query: field:value
                let parts: Vec<&str> = token.splitn(2, ':').collect();
                if parts.len() == 2 {
                    fields.push(parts[0].to_string());
                    text.push_str(parts[1]);
                    text.push(' ');
                }
            } else if token.ends_with('~') {
                // Fuzzy query indicator
                fuzzy = true;
                text.push_str(token.trim_end_matches('~'));
                text.push(' ');
            } else if token.eq_ignore_ascii_case("AND") {
                operator = QueryOperator::And;
            } else if token.eq_ignore_ascii_case("OR") {
                operator = QueryOperator::Or;
            } else {
                text.push_str(token);
                text.push(' ');
            }
        }

        Ok(Query {
            text: text.trim().to_string(),
            fields,
            fuzzy,
            boost: None,
            operator,
        })
    }

    /// Parse advanced query with filters
    pub fn parse_advanced(json: &str) -> SearchResult<Query> {
        let dsl: QueryDSL = serde_json::from_str(json)
            .map_err(|e| SearchError::QueryParseError(e.to_string()))?;

        Self::parse(dsl)
    }

    /// Validate a query
    pub fn validate(query: &Query) -> SearchResult<()> {
        if query.text.is_empty() && query.fields.is_empty() {
            return Err(SearchError::InvalidQuery(
                "Query must have text or fields".to_string(),
            ));
        }

        if let Some(boost) = query.boost {
            if boost <= 0.0 {
                return Err(SearchError::InvalidQuery(
                    "Boost must be positive".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Sanitize query text to prevent injection
    pub fn sanitize(text: &str) -> String {
        text.chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '-' || *c == '_')
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_text() {
        let query = QueryParser::parse_text("hello world").unwrap();
        assert_eq!(query.text, "hello world");
    }

    #[test]
    fn test_parse_lucene() {
        let query = QueryParser::parse_lucene("title:test fuzzy~").unwrap();
        assert!(query.fuzzy);
        assert!(query.fields.contains(&"title".to_string()));
    }

    #[test]
    fn test_parse_dsl() {
        let dsl = QueryDSL {
            query: "search query".to_string(),
            fields: Some(vec!["title".to_string()]),
            fuzzy: Some(true),
            boost: Some(2.0),
            operator: Some("AND".to_string()),
        };

        let query = QueryParser::parse(dsl).unwrap();
        assert_eq!(query.text, "search query");
        assert!(query.fuzzy);
        assert_eq!(query.boost, Some(2.0));
    }

    #[test]
    fn test_sanitize() {
        let sanitized = QueryParser::sanitize("test<script>alert(1)</script>");
        assert!(!sanitized.contains('<'));
        assert!(!sanitized.contains('>'));
    }
}
