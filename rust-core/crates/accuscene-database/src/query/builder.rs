//! Type-safe query builder for AccuScene database

use crate::query::filter::FilterCondition;
use crate::query::pagination::Pagination;

/// Query builder for constructing SQL queries
pub struct QueryBuilder {
    table: String,
    select: Vec<String>,
    where_clause: Vec<String>,
    order_by: Vec<String>,
    limit: Option<usize>,
    offset: Option<usize>,
    joins: Vec<String>,
}

impl QueryBuilder {
    /// Create a new query builder for a table
    pub fn new(table: impl Into<String>) -> Self {
        Self {
            table: table.into(),
            select: vec!["*".to_string()],
            where_clause: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            offset: None,
            joins: Vec::new(),
        }
    }

    /// Specify columns to select
    pub fn select(mut self, columns: &[&str]) -> Self {
        self.select = columns.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Add a WHERE condition
    pub fn where_clause(mut self, condition: impl Into<String>) -> Self {
        self.where_clause.push(condition.into());
        self
    }

    /// Add multiple WHERE conditions (AND)
    pub fn where_all(mut self, conditions: Vec<impl Into<String>>) -> Self {
        for condition in conditions {
            self.where_clause.push(condition.into());
        }
        self
    }

    /// Add a filter condition
    pub fn filter(mut self, condition: &FilterCondition) -> Self {
        self.where_clause.push(condition.to_sql());
        self
    }

    /// Add an ORDER BY clause
    pub fn order_by(mut self, column: &str, direction: OrderDirection) -> Self {
        self.order_by.push(format!("{} {}", column, direction.as_str()));
        self
    }

    /// Set LIMIT
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set OFFSET
    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Apply pagination
    pub fn paginate(mut self, pagination: &Pagination) -> Self {
        self.limit = Some(pagination.page_size);
        self.offset = Some((pagination.page - 1) * pagination.page_size);
        self
    }

    /// Add a JOIN clause
    pub fn join(mut self, join_clause: impl Into<String>) -> Self {
        self.joins.push(join_clause.into());
        self
    }

    /// Add an INNER JOIN
    pub fn inner_join(mut self, table: &str, on_clause: &str) -> Self {
        self.joins.push(format!("INNER JOIN {} ON {}", table, on_clause));
        self
    }

    /// Add a LEFT JOIN
    pub fn left_join(mut self, table: &str, on_clause: &str) -> Self {
        self.joins.push(format!("LEFT JOIN {} ON {}", table, on_clause));
        self
    }

    /// Build the SQL query
    pub fn build(&self) -> String {
        let mut query = format!("SELECT {} FROM {}", self.select.join(", "), self.table);

        // Add JOINs
        if !self.joins.is_empty() {
            query.push_str(&format!(" {}", self.joins.join(" ")));
        }

        // Add WHERE clause
        if !self.where_clause.is_empty() {
            query.push_str(&format!(" WHERE {}", self.where_clause.join(" AND ")));
        }

        // Add ORDER BY
        if !self.order_by.is_empty() {
            query.push_str(&format!(" ORDER BY {}", self.order_by.join(", ")));
        }

        // Add LIMIT
        if let Some(limit) = self.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        // Add OFFSET
        if let Some(offset) = self.offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }

        query
    }

    /// Build a COUNT query
    pub fn build_count(&self) -> String {
        let mut query = format!("SELECT COUNT(*) FROM {}", self.table);

        // Add JOINs
        if !self.joins.is_empty() {
            query.push_str(&format!(" {}", self.joins.join(" ")));
        }

        // Add WHERE clause
        if !self.where_clause.is_empty() {
            query.push_str(&format!(" WHERE {}", self.where_clause.join(" AND ")));
        }

        query
    }
}

/// Order direction for ORDER BY clauses
#[derive(Debug, Clone, Copy)]
pub enum OrderDirection {
    Asc,
    Desc,
}

impl OrderDirection {
    pub fn as_str(&self) -> &str {
        match self {
            OrderDirection::Asc => "ASC",
            OrderDirection::Desc => "DESC",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_query() {
        let query = QueryBuilder::new("users")
            .select(&["id", "email", "username"])
            .build();

        assert_eq!(query, "SELECT id, email, username FROM users");
    }

    #[test]
    fn test_query_with_where() {
        let query = QueryBuilder::new("users")
            .where_clause("email = 'test@example.com'")
            .build();

        assert_eq!(query, "SELECT * FROM users WHERE email = 'test@example.com'");
    }

    #[test]
    fn test_query_with_order_and_limit() {
        let query = QueryBuilder::new("cases")
            .order_by("created_at", OrderDirection::Desc)
            .limit(10)
            .build();

        assert_eq!(query, "SELECT * FROM cases ORDER BY created_at DESC LIMIT 10");
    }

    #[test]
    fn test_query_with_join() {
        let query = QueryBuilder::new("cases")
            .inner_join("users", "cases.created_by = users.id")
            .select(&["cases.*", "users.email"])
            .build();

        assert_eq!(
            query,
            "SELECT cases.*, users.email FROM cases INNER JOIN users ON cases.created_by = users.id"
        );
    }
}
