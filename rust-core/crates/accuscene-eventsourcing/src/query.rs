//! Query handling for CQRS read models.

use crate::error::{EventSourcingError, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use uuid::Uuid;

/// Trait for queries in the CQRS pattern.
pub trait Query: Send + Sync + Debug + Clone {
    /// The result type of this query.
    type Result: Send + Sync;

    /// Returns the query type identifier.
    fn query_type(&self) -> &'static str;

    /// Validates the query.
    fn validate(&self) -> Result<()> {
        Ok(())
    }
}

/// Query envelope with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryEnvelope<Q>
where
    Q: Query,
{
    /// Unique query identifier.
    pub query_id: Uuid,

    /// Query type.
    pub query_type: String,

    /// The actual query payload.
    pub payload: Q,

    /// User or service that issued the query.
    pub issuer: Option<String>,

    /// Correlation ID for tracking.
    pub correlation_id: Option<Uuid>,

    /// Timestamp when the query was created.
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl<Q> QueryEnvelope<Q>
where
    Q: Query,
{
    /// Creates a new query envelope.
    pub fn new(payload: Q) -> Self {
        Self {
            query_id: Uuid::new_v4(),
            query_type: payload.query_type().to_string(),
            payload,
            issuer: None,
            correlation_id: None,
            created_at: chrono::Utc::now(),
        }
    }

    /// Sets the issuer.
    pub fn with_issuer(mut self, issuer: impl Into<String>) -> Self {
        self.issuer = Some(issuer.into());
        self
    }

    /// Sets the correlation ID.
    pub fn with_correlation_id(mut self, id: Uuid) -> Self {
        self.correlation_id = Some(id);
        self
    }

    /// Validates the query.
    pub fn validate(&self) -> Result<()> {
        self.payload.validate()
    }
}

/// Trait for query handlers.
#[async_trait]
pub trait QueryHandler<Q>: Send + Sync
where
    Q: Query,
{
    /// Handles a query and returns the result.
    async fn handle(&self, query: Q) -> Result<Q::Result>;

    /// Validates a query before handling.
    async fn validate(&self, query: &Q) -> Result<()> {
        query.validate()
    }
}

/// Result of query execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult<T> {
    /// Query identifier.
    pub query_id: Uuid,

    /// Whether the query succeeded.
    pub success: bool,

    /// The query result data.
    pub data: Option<T>,

    /// Error message if failed.
    pub error: Option<String>,

    /// Query execution time in milliseconds.
    pub execution_time_ms: Option<u64>,
}

impl<T> QueryResult<T> {
    /// Creates a successful result.
    pub fn success(query_id: Uuid, data: T) -> Self {
        Self {
            query_id,
            success: true,
            data: Some(data),
            error: None,
            execution_time_ms: None,
        }
    }

    /// Creates a failed result.
    pub fn failure(query_id: Uuid, error: impl Into<String>) -> Self {
        Self {
            query_id,
            success: false,
            data: None,
            error: Some(error.into()),
            execution_time_ms: None,
        }
    }

    /// Sets the execution time.
    pub fn with_execution_time(mut self, ms: u64) -> Self {
        self.execution_time_ms = Some(ms);
        self
    }
}

/// Pagination parameters for queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    /// Page number (0-indexed).
    pub page: usize,

    /// Number of items per page.
    pub page_size: usize,

    /// Total number of items.
    pub total: Option<usize>,
}

impl Pagination {
    /// Creates new pagination parameters.
    pub fn new(page: usize, page_size: usize) -> Self {
        Self {
            page,
            page_size,
            total: None,
        }
    }

    /// Sets the total number of items.
    pub fn with_total(mut self, total: usize) -> Self {
        self.total = Some(total);
        self
    }

    /// Returns the offset for database queries.
    pub fn offset(&self) -> usize {
        self.page * self.page_size
    }

    /// Returns the limit for database queries.
    pub fn limit(&self) -> usize {
        self.page_size
    }

    /// Returns the total number of pages.
    pub fn total_pages(&self) -> Option<usize> {
        self.total
            .map(|t| (t + self.page_size - 1) / self.page_size)
    }

    /// Returns whether there is a next page.
    pub fn has_next(&self) -> Option<bool> {
        self.total_pages()
            .map(|total_pages| self.page + 1 < total_pages)
    }

    /// Returns whether there is a previous page.
    pub fn has_previous(&self) -> bool {
        self.page > 0
    }
}

impl Default for Pagination {
    fn default() -> Self {
        Self::new(0, 20)
    }
}

/// Paginated result wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResult<T> {
    /// The items in this page.
    pub items: Vec<T>,

    /// Pagination information.
    pub pagination: Pagination,
}

impl<T> PaginatedResult<T> {
    /// Creates a new paginated result.
    pub fn new(items: Vec<T>, pagination: Pagination) -> Self {
        Self { items, pagination }
    }
}

/// Sorting parameters for queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sort {
    /// Field to sort by.
    pub field: String,

    /// Sort direction.
    pub direction: SortDirection,
}

impl Sort {
    /// Creates new sort parameters.
    pub fn new(field: impl Into<String>, direction: SortDirection) -> Self {
        Self {
            field: field.into(),
            direction,
        }
    }

    /// Creates ascending sort.
    pub fn asc(field: impl Into<String>) -> Self {
        Self::new(field, SortDirection::Ascending)
    }

    /// Creates descending sort.
    pub fn desc(field: impl Into<String>) -> Self {
        Self::new(field, SortDirection::Descending)
    }
}

/// Sort direction.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SortDirection {
    /// Ascending order.
    Ascending,

    /// Descending order.
    Descending,
}

/// Filter for queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filter {
    /// Field to filter on.
    pub field: String,

    /// Filter operator.
    pub operator: FilterOperator,

    /// Filter value.
    pub value: FilterValue,
}

impl Filter {
    /// Creates a new filter.
    pub fn new(field: impl Into<String>, operator: FilterOperator, value: FilterValue) -> Self {
        Self {
            field: field.into(),
            operator,
            value,
        }
    }

    /// Creates an equals filter.
    pub fn equals(field: impl Into<String>, value: impl Into<String>) -> Self {
        Self::new(
            field,
            FilterOperator::Equals,
            FilterValue::String(value.into()),
        )
    }

    /// Creates a contains filter.
    pub fn contains(field: impl Into<String>, value: impl Into<String>) -> Self {
        Self::new(
            field,
            FilterOperator::Contains,
            FilterValue::String(value.into()),
        )
    }
}

/// Filter operator.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FilterOperator {
    /// Equal to.
    Equals,

    /// Not equal to.
    NotEquals,

    /// Greater than.
    GreaterThan,

    /// Greater than or equal to.
    GreaterThanOrEqual,

    /// Less than.
    LessThan,

    /// Less than or equal to.
    LessThanOrEqual,

    /// Contains (for strings).
    Contains,

    /// Starts with (for strings).
    StartsWith,

    /// Ends with (for strings).
    EndsWith,

    /// In a list of values.
    In,

    /// Not in a list of values.
    NotIn,
}

/// Filter value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterValue {
    /// String value.
    String(String),

    /// Integer value.
    Int(i64),

    /// Float value.
    Float(f64),

    /// Boolean value.
    Bool(bool),

    /// List of values.
    List(Vec<FilterValue>),

    /// Null value.
    Null,
}

/// Query options combining pagination, sorting, and filtering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryOptions {
    /// Pagination parameters.
    pub pagination: Option<Pagination>,

    /// Sorting parameters.
    pub sort: Vec<Sort>,

    /// Filters.
    pub filters: Vec<Filter>,
}

impl QueryOptions {
    /// Creates new query options.
    pub fn new() -> Self {
        Self {
            pagination: None,
            sort: Vec::new(),
            filters: Vec::new(),
        }
    }

    /// Sets pagination.
    pub fn with_pagination(mut self, pagination: Pagination) -> Self {
        self.pagination = Some(pagination);
        self
    }

    /// Adds a sort.
    pub fn add_sort(mut self, sort: Sort) -> Self {
        self.sort.push(sort);
        self
    }

    /// Adds a filter.
    pub fn add_filter(mut self, filter: Filter) -> Self {
        self.filters.push(filter);
        self
    }
}

impl Default for QueryOptions {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct TestQuery {
        id: String,
    }

    impl Query for TestQuery {
        type Result = String;

        fn query_type(&self) -> &'static str {
            "TestQuery"
        }
    }

    #[test]
    fn test_query_envelope() {
        let query = TestQuery {
            id: "test-1".to_string(),
        };

        let envelope = QueryEnvelope::new(query.clone())
            .with_issuer("user-123")
            .with_correlation_id(Uuid::new_v4());

        assert_eq!(envelope.query_type, "TestQuery");
        assert!(envelope.issuer.is_some());
        assert!(envelope.correlation_id.is_some());
    }

    #[test]
    fn test_pagination() {
        let pagination = Pagination::new(2, 10).with_total(100);

        assert_eq!(pagination.offset(), 20);
        assert_eq!(pagination.limit(), 10);
        assert_eq!(pagination.total_pages(), Some(10));
        assert_eq!(pagination.has_next(), Some(true));
        assert!(pagination.has_previous());
    }

    #[test]
    fn test_sort() {
        let sort = Sort::asc("name");
        assert_eq!(sort.field, "name");
        assert_eq!(sort.direction, SortDirection::Ascending);

        let sort = Sort::desc("created_at");
        assert_eq!(sort.field, "created_at");
        assert_eq!(sort.direction, SortDirection::Descending);
    }

    #[test]
    fn test_filter() {
        let filter = Filter::equals("status", "active");
        assert_eq!(filter.field, "status");
        assert_eq!(filter.operator, FilterOperator::Equals);
    }

    #[test]
    fn test_query_options() {
        let options = QueryOptions::new()
            .with_pagination(Pagination::new(0, 20))
            .add_sort(Sort::asc("name"))
            .add_filter(Filter::equals("status", "active"));

        assert!(options.pagination.is_some());
        assert_eq!(options.sort.len(), 1);
        assert_eq!(options.filters.len(), 1);
    }
}
