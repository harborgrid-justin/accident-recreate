//! Analytics query DSL

use crate::aggregation::{AggregationOp, DimensionValue};
use crate::error::{AnalyticsError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Query builder for analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Query {
    pub metric: String,
    pub filters: Vec<Filter>,
    pub aggregations: Vec<Aggregation>,
    pub group_by: Vec<String>,
    pub time_range: Option<TimeRange>,
    pub limit: Option<usize>,
    pub order_by: Option<OrderBy>,
}

impl Query {
    pub fn new(metric: impl Into<String>) -> Self {
        Self {
            metric: metric.into(),
            filters: Vec::new(),
            aggregations: Vec::new(),
            group_by: Vec::new(),
            time_range: None,
            limit: None,
            order_by: None,
        }
    }

    pub fn filter(mut self, filter: Filter) -> Self {
        self.filters.push(filter);
        self
    }

    pub fn aggregate(mut self, agg: Aggregation) -> Self {
        self.aggregations.push(agg);
        self
    }

    pub fn group_by(mut self, dimension: impl Into<String>) -> Self {
        self.group_by.push(dimension.into());
        self
    }

    pub fn time_range(mut self, range: TimeRange) -> Self {
        self.time_range = Some(range);
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn order_by(mut self, order: OrderBy) -> Self {
        self.order_by = Some(order);
        self
    }

    pub fn validate(&self) -> Result<()> {
        if self.metric.is_empty() {
            return Err(AnalyticsError::Query("Metric name cannot be empty".to_string()));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filter {
    pub dimension: String,
    pub operator: FilterOperator,
    pub value: FilterValue,
}

impl Filter {
    pub fn eq(dimension: impl Into<String>, value: impl Into<FilterValue>) -> Self {
        Self {
            dimension: dimension.into(),
            operator: FilterOperator::Equals,
            value: value.into(),
        }
    }

    pub fn ne(dimension: impl Into<String>, value: impl Into<FilterValue>) -> Self {
        Self {
            dimension: dimension.into(),
            operator: FilterOperator::NotEquals,
            value: value.into(),
        }
    }

    pub fn gt(dimension: impl Into<String>, value: impl Into<FilterValue>) -> Self {
        Self {
            dimension: dimension.into(),
            operator: FilterOperator::GreaterThan,
            value: value.into(),
        }
    }

    pub fn lt(dimension: impl Into<String>, value: impl Into<FilterValue>) -> Self {
        Self {
            dimension: dimension.into(),
            operator: FilterOperator::LessThan,
            value: value.into(),
        }
    }

    pub fn contains(dimension: impl Into<String>, value: String) -> Self {
        Self {
            dimension: dimension.into(),
            operator: FilterOperator::Contains,
            value: FilterValue::String(value),
        }
    }

    pub fn in_list(dimension: impl Into<String>, values: Vec<FilterValue>) -> Self {
        Self {
            dimension: dimension.into(),
            operator: FilterOperator::In,
            value: FilterValue::List(values),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterOperator {
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    GreaterOrEqual,
    LessOrEqual,
    Contains,
    In,
    Between,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterValue {
    String(String),
    Number(f64),
    Boolean(bool),
    List(Vec<FilterValue>),
}

impl From<String> for FilterValue {
    fn from(s: String) -> Self {
        FilterValue::String(s)
    }
}

impl From<&str> for FilterValue {
    fn from(s: &str) -> Self {
        FilterValue::String(s.to_string())
    }
}

impl From<f64> for FilterValue {
    fn from(n: f64) -> Self {
        FilterValue::Number(n)
    }
}

impl From<i64> for FilterValue {
    fn from(n: i64) -> Self {
        FilterValue::Number(n as f64)
    }
}

impl From<bool> for FilterValue {
    fn from(b: bool) -> Self {
        FilterValue::Boolean(b)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Aggregation {
    pub operation: AggregationOp,
    pub field: Option<String>,
    pub alias: Option<String>,
}

impl Aggregation {
    pub fn sum(field: impl Into<String>) -> Self {
        Self {
            operation: AggregationOp::Sum,
            field: Some(field.into()),
            alias: None,
        }
    }

    pub fn count() -> Self {
        Self {
            operation: AggregationOp::Count,
            field: None,
            alias: None,
        }
    }

    pub fn mean(field: impl Into<String>) -> Self {
        Self {
            operation: AggregationOp::Mean,
            field: Some(field.into()),
            alias: None,
        }
    }

    pub fn min(field: impl Into<String>) -> Self {
        Self {
            operation: AggregationOp::Min,
            field: Some(field.into()),
            alias: None,
        }
    }

    pub fn max(field: impl Into<String>) -> Self {
        Self {
            operation: AggregationOp::Max,
            field: Some(field.into()),
            alias: None,
        }
    }

    pub fn alias(mut self, alias: impl Into<String>) -> Self {
        self.alias = Some(alias.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl TimeRange {
    pub fn new(start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        Self { start, end }
    }

    pub fn last_hours(hours: i64) -> Self {
        let end = Utc::now();
        let start = end - chrono::Duration::hours(hours);
        Self { start, end }
    }

    pub fn last_days(days: i64) -> Self {
        let end = Utc::now();
        let start = end - chrono::Duration::days(days);
        Self { start, end }
    }

    pub fn today() -> Self {
        let now = Utc::now();
        let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
        let start = DateTime::from_naive_utc_and_offset(start, Utc);
        Self { start, end: now }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBy {
    pub field: String,
    pub direction: OrderDirection,
}

impl OrderBy {
    pub fn asc(field: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            direction: OrderDirection::Ascending,
        }
    }

    pub fn desc(field: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            direction: OrderDirection::Descending,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderDirection {
    Ascending,
    Descending,
}

/// Query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub rows: Vec<ResultRow>,
    pub metadata: QueryMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultRow {
    pub dimensions: Vec<(String, DimensionValue)>,
    pub metrics: Vec<(String, f64)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryMetadata {
    pub query_time_ms: u64,
    pub row_count: usize,
    pub was_truncated: bool,
}

/// Query executor (trait for implementing actual query execution)
pub trait QueryExecutor: Send + Sync {
    fn execute(&self, query: &Query) -> Result<QueryResult>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_builder() {
        let query = Query::new("accidents")
            .filter(Filter::eq("severity", "high"))
            .filter(Filter::gt("speed", 50.0))
            .aggregate(Aggregation::count())
            .aggregate(Aggregation::mean("impact_force").alias("avg_force"))
            .group_by("location")
            .time_range(TimeRange::last_days(7))
            .limit(100)
            .order_by(OrderBy::desc("count"));

        assert_eq!(query.metric, "accidents");
        assert_eq!(query.filters.len(), 2);
        assert_eq!(query.aggregations.len(), 2);
        assert_eq!(query.group_by.len(), 1);
        assert!(query.time_range.is_some());
        assert_eq!(query.limit, Some(100));
    }

    #[test]
    fn test_filter_builder() {
        let filter1 = Filter::eq("status", "active");
        assert_eq!(filter1.operator, FilterOperator::Equals);

        let filter2 = Filter::gt("value", 100.0);
        assert_eq!(filter2.operator, FilterOperator::GreaterThan);
    }
}
