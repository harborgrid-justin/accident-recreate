//! Query builder for fluent query construction

use crate::query::{Query, QueryOperator};

pub struct QueryBuilder {
    query: Query,
}

impl QueryBuilder {
    pub fn new() -> Self {
        Self {
            query: Query::default(),
        }
    }

    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.query.text = text.into();
        self
    }

    pub fn field(mut self, field: impl Into<String>) -> Self {
        self.query.fields.push(field.into());
        self
    }

    pub fn fields(mut self, fields: Vec<String>) -> Self {
        self.query.fields = fields;
        self
    }

    pub fn fuzzy(mut self, fuzzy: bool) -> Self {
        self.query.fuzzy = fuzzy;
        self
    }

    pub fn boost(mut self, boost: f32) -> Self {
        self.query.boost = Some(boost);
        self
    }

    pub fn operator(mut self, operator: QueryOperator) -> Self {
        self.query.operator = operator;
        self
    }

    pub fn and_operator(mut self) -> Self {
        self.query.operator = QueryOperator::And;
        self
    }

    pub fn or_operator(mut self) -> Self {
        self.query.operator = QueryOperator::Or;
        self
    }

    pub fn build(self) -> Query {
        self.query
    }
}

impl Default for QueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_builder() {
        let query = QueryBuilder::new()
            .text("test query")
            .field("title")
            .field("content")
            .fuzzy(true)
            .boost(2.0)
            .and_operator()
            .build();

        assert_eq!(query.text, "test query");
        assert_eq!(query.fields.len(), 2);
        assert!(query.fuzzy);
        assert_eq!(query.boost, Some(2.0));
        assert!(matches!(query.operator, QueryOperator::And));
    }

    #[test]
    fn test_query_builder_defaults() {
        let query = QueryBuilder::new()
            .text("test")
            .build();

        assert_eq!(query.text, "test");
        assert!(query.fields.is_empty());
        assert!(!query.fuzzy);
    }
}
