//! Advanced filtering for database queries

use serde::{Deserialize, Serialize};

/// Filter condition for queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterCondition {
    pub field: String,
    pub operator: FilterOperator,
    pub value: FilterValue,
}

impl FilterCondition {
    pub fn new(field: impl Into<String>, operator: FilterOperator, value: FilterValue) -> Self {
        Self {
            field: field.into(),
            operator,
            value,
        }
    }

    /// Convert filter condition to SQL
    pub fn to_sql(&self) -> String {
        match &self.operator {
            FilterOperator::Equals => format!("{} = {}", self.field, self.value.to_sql()),
            FilterOperator::NotEquals => format!("{} != {}", self.field, self.value.to_sql()),
            FilterOperator::GreaterThan => format!("{} > {}", self.field, self.value.to_sql()),
            FilterOperator::GreaterThanOrEqual => format!("{} >= {}", self.field, self.value.to_sql()),
            FilterOperator::LessThan => format!("{} < {}", self.field, self.value.to_sql()),
            FilterOperator::LessThanOrEqual => format!("{} <= {}", self.field, self.value.to_sql()),
            FilterOperator::Like => format!("{} LIKE {}", self.field, self.value.to_sql()),
            FilterOperator::NotLike => format!("{} NOT LIKE {}", self.field, self.value.to_sql()),
            FilterOperator::In => {
                if let FilterValue::Array(values) = &self.value {
                    let sql_values: Vec<String> = values.iter().map(|v| v.to_sql()).collect();
                    format!("{} IN ({})", self.field, sql_values.join(", "))
                } else {
                    format!("{} IN ({})", self.field, self.value.to_sql())
                }
            }
            FilterOperator::NotIn => {
                if let FilterValue::Array(values) = &self.value {
                    let sql_values: Vec<String> = values.iter().map(|v| v.to_sql()).collect();
                    format!("{} NOT IN ({})", self.field, sql_values.join(", "))
                } else {
                    format!("{} NOT IN ({})", self.field, self.value.to_sql())
                }
            }
            FilterOperator::IsNull => format!("{} IS NULL", self.field),
            FilterOperator::IsNotNull => format!("{} IS NOT NULL", self.field),
            FilterOperator::Between => {
                if let FilterValue::Range(start, end) = &self.value {
                    format!("{} BETWEEN {} AND {}", self.field, start.to_sql(), end.to_sql())
                } else {
                    panic!("BETWEEN operator requires a Range value")
                }
            }
        }
    }
}

/// Filter operators
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FilterOperator {
    Equals,
    NotEquals,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Like,
    NotLike,
    In,
    NotIn,
    IsNull,
    IsNotNull,
    Between,
}

/// Filter value types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Null,
    Array(Vec<FilterValue>),
    Range(Box<FilterValue>, Box<FilterValue>),
}

impl FilterValue {
    /// Convert filter value to SQL representation
    pub fn to_sql(&self) -> String {
        match self {
            FilterValue::String(s) => format!("'{}'", s.replace('\'', "''")),
            FilterValue::Integer(i) => i.to_string(),
            FilterValue::Float(f) => f.to_string(),
            FilterValue::Boolean(b) => if *b { "1" } else { "0" }.to_string(),
            FilterValue::Null => "NULL".to_string(),
            FilterValue::Array(values) => {
                let sql_values: Vec<String> = values.iter().map(|v| v.to_sql()).collect();
                sql_values.join(", ")
            }
            FilterValue::Range(_, _) => {
                panic!("Range values should be handled by the operator")
            }
        }
    }
}

/// Filter builder for combining multiple conditions
pub struct Filter {
    conditions: Vec<FilterCondition>,
    logic: FilterLogic,
}

impl Filter {
    /// Create a new filter with AND logic
    pub fn and() -> Self {
        Self {
            conditions: Vec::new(),
            logic: FilterLogic::And,
        }
    }

    /// Create a new filter with OR logic
    pub fn or() -> Self {
        Self {
            conditions: Vec::new(),
            logic: FilterLogic::Or,
        }
    }

    /// Add a condition to the filter
    pub fn add(mut self, condition: FilterCondition) -> Self {
        self.conditions.push(condition);
        self
    }

    /// Convert filter to SQL WHERE clause
    pub fn to_sql(&self) -> String {
        if self.conditions.is_empty() {
            return String::new();
        }

        let sql_conditions: Vec<String> = self.conditions.iter().map(|c| c.to_sql()).collect();

        match self.logic {
            FilterLogic::And => sql_conditions.join(" AND "),
            FilterLogic::Or => format!("({})", sql_conditions.join(" OR ")),
        }
    }
}

/// Filter logic for combining conditions
#[derive(Debug, Clone, Copy)]
pub enum FilterLogic {
    And,
    Or,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equals_filter() {
        let condition = FilterCondition::new(
            "status",
            FilterOperator::Equals,
            FilterValue::String("open".to_string()),
        );

        assert_eq!(condition.to_sql(), "status = 'open'");
    }

    #[test]
    fn test_in_filter() {
        let condition = FilterCondition::new(
            "priority",
            FilterOperator::In,
            FilterValue::Array(vec![
                FilterValue::String("high".to_string()),
                FilterValue::String("critical".to_string()),
            ]),
        );

        assert_eq!(condition.to_sql(), "priority IN ('high', 'critical')");
    }

    #[test]
    fn test_between_filter() {
        let condition = FilterCondition::new(
            "created_at",
            FilterOperator::Between,
            FilterValue::Range(
                Box::new(FilterValue::String("2024-01-01".to_string())),
                Box::new(FilterValue::String("2024-12-31".to_string())),
            ),
        );

        assert_eq!(condition.to_sql(), "created_at BETWEEN '2024-01-01' AND '2024-12-31'");
    }

    #[test]
    fn test_filter_builder() {
        let filter = Filter::and()
            .add(FilterCondition::new(
                "status",
                FilterOperator::Equals,
                FilterValue::String("open".to_string()),
            ))
            .add(FilterCondition::new(
                "priority",
                FilterOperator::Equals,
                FilterValue::String("high".to_string()),
            ));

        assert_eq!(filter.to_sql(), "status = 'open' AND priority = 'high'");
    }
}
