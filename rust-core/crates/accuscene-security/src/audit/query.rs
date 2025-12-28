//! Audit log querying and filtering
//!
//! Provides flexible querying capabilities for audit logs.

use crate::audit::event::{AuditEvent, EventResult, EventSeverity, EventType};
use crate::error::Result;
use serde::{Deserialize, Serialize};

/// Audit query builder
#[derive(Debug, Clone, Default)]
pub struct AuditQuery {
    filters: Vec<QueryFilter>,
    sort_by: Option<SortField>,
    sort_order: SortOrder,
    limit: Option<usize>,
    offset: usize,
}

impl AuditQuery {
    /// Create a new query
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by event type
    pub fn event_type(mut self, event_type: EventType) -> Self {
        self.filters.push(QueryFilter::EventType(event_type));
        self
    }

    /// Filter by user ID
    pub fn user_id(mut self, user_id: impl Into<String>) -> Self {
        self.filters.push(QueryFilter::UserId(user_id.into()));
        self
    }

    /// Filter by session ID
    pub fn session_id(mut self, session_id: impl Into<String>) -> Self {
        self.filters
            .push(QueryFilter::SessionId(session_id.into()));
        self
    }

    /// Filter by IP address
    pub fn ip_address(mut self, ip: impl Into<String>) -> Self {
        self.filters.push(QueryFilter::IpAddress(ip.into()));
        self
    }

    /// Filter by severity
    pub fn severity(mut self, severity: EventSeverity) -> Self {
        self.filters.push(QueryFilter::Severity(severity));
        self
    }

    /// Filter by minimum severity
    pub fn min_severity(mut self, severity: EventSeverity) -> Self {
        self.filters.push(QueryFilter::MinSeverity(severity));
        self
    }

    /// Filter by result
    pub fn result(mut self, result: EventResult) -> Self {
        self.filters.push(QueryFilter::Result(result));
        self
    }

    /// Filter by time range
    pub fn time_range(
        mut self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        self.filters.push(QueryFilter::TimeRange { start, end });
        self
    }

    /// Filter by resource type
    pub fn resource_type(mut self, resource_type: impl Into<String>) -> Self {
        self.filters
            .push(QueryFilter::ResourceType(resource_type.into()));
        self
    }

    /// Filter by resource ID
    pub fn resource_id(mut self, resource_id: impl Into<String>) -> Self {
        self.filters
            .push(QueryFilter::ResourceId(resource_id.into()));
        self
    }

    /// Sort by field
    pub fn sort_by(mut self, field: SortField, order: SortOrder) -> Self {
        self.sort_by = Some(field);
        self.sort_order = order;
        self
    }

    /// Limit results
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set offset
    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = offset;
        self
    }

    /// Execute query on a collection of events
    pub fn execute(&self, events: &[AuditEvent]) -> Vec<AuditEvent> {
        let mut results: Vec<AuditEvent> = events
            .iter()
            .filter(|event| self.matches_filters(event))
            .cloned()
            .collect();

        // Sort
        if let Some(field) = &self.sort_by {
            match self.sort_order {
                SortOrder::Ascending => results.sort_by(|a, b| field.compare(a, b)),
                SortOrder::Descending => results.sort_by(|a, b| field.compare(b, a)),
            }
        }

        // Apply offset and limit
        results
            .into_iter()
            .skip(self.offset)
            .take(self.limit.unwrap_or(usize::MAX))
            .collect()
    }

    /// Check if an event matches all filters
    fn matches_filters(&self, event: &AuditEvent) -> bool {
        self.filters.iter().all(|filter| filter.matches(event))
    }
}

/// Query filter
#[derive(Debug, Clone)]
pub enum QueryFilter {
    EventType(EventType),
    UserId(String),
    SessionId(String),
    IpAddress(String),
    Severity(EventSeverity),
    MinSeverity(EventSeverity),
    Result(EventResult),
    TimeRange {
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    },
    ResourceType(String),
    ResourceId(String),
}

impl QueryFilter {
    /// Check if an event matches this filter
    fn matches(&self, event: &AuditEvent) -> bool {
        match self {
            QueryFilter::EventType(et) => event.event_type == *et,
            QueryFilter::UserId(uid) => event.user_id.as_ref() == Some(uid),
            QueryFilter::SessionId(sid) => event.session_id.as_ref() == Some(sid),
            QueryFilter::IpAddress(ip) => event.ip_address.as_ref() == Some(ip),
            QueryFilter::Severity(sev) => event.severity == *sev,
            QueryFilter::MinSeverity(min_sev) => event.severity >= *min_sev,
            QueryFilter::Result(res) => event.result == *res,
            QueryFilter::TimeRange { start, end } => {
                event.timestamp >= *start && event.timestamp <= *end
            }
            QueryFilter::ResourceType(rt) => event
                .resource
                .as_ref()
                .map(|r| &r.resource_type == rt)
                .unwrap_or(false),
            QueryFilter::ResourceId(rid) => event
                .resource
                .as_ref()
                .map(|r| &r.resource_id == rid)
                .unwrap_or(false),
        }
    }
}

/// Sort field
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortField {
    Timestamp,
    EventType,
    Severity,
    UserId,
}

impl SortField {
    fn compare(&self, a: &AuditEvent, b: &AuditEvent) -> std::cmp::Ordering {
        match self {
            SortField::Timestamp => a.timestamp.cmp(&b.timestamp),
            SortField::EventType => format!("{:?}", a.event_type).cmp(&format!("{:?}", b.event_type)),
            SortField::Severity => a.severity.cmp(&b.severity),
            SortField::UserId => a.user_id.cmp(&b.user_id),
        }
    }
}

/// Sort order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl Default for SortOrder {
    fn default() -> Self {
        SortOrder::Descending
    }
}

/// Query result with pagination info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    /// Matching events
    pub events: Vec<AuditEvent>,
    /// Total count (before pagination)
    pub total_count: usize,
    /// Offset used
    pub offset: usize,
    /// Limit used
    pub limit: Option<usize>,
}

impl QueryResult {
    /// Create from query execution
    pub fn from_query(query: &AuditQuery, all_events: &[AuditEvent]) -> Self {
        // Count total matches before pagination
        let total_count = all_events
            .iter()
            .filter(|event| query.matches_filters(event))
            .count();

        // Execute query with pagination
        let events = query.execute(all_events);

        Self {
            events,
            total_count,
            offset: query.offset,
            limit: query.limit,
        }
    }

    /// Check if there are more results
    pub fn has_more(&self) -> bool {
        self.offset + self.events.len() < self.total_count
    }

    /// Get next offset for pagination
    pub fn next_offset(&self) -> Option<usize> {
        if self.has_more() {
            Some(self.offset + self.events.len())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_events() -> Vec<AuditEvent> {
        vec![
            AuditEvent::new(EventType::AuthLogin, "login".to_string())
                .with_user("user1".to_string())
                .with_severity(EventSeverity::Info),
            AuditEvent::new(EventType::AuthLogout, "logout".to_string())
                .with_user("user1".to_string())
                .with_severity(EventSeverity::Info),
            AuditEvent::new(EventType::CaseCreated, "case.create".to_string())
                .with_user("user2".to_string())
                .with_severity(EventSeverity::Info),
            AuditEvent::new(EventType::SecurityThreatDetected, "threat".to_string())
                .with_severity(EventSeverity::Critical),
        ]
    }

    #[test]
    fn test_filter_by_event_type() {
        let events = create_test_events();
        let query = AuditQuery::new().event_type(EventType::AuthLogin);
        let results = query.execute(&events);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].event_type, EventType::AuthLogin);
    }

    #[test]
    fn test_filter_by_user() {
        let events = create_test_events();
        let query = AuditQuery::new().user_id("user1");
        let results = query.execute(&events);

        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|e| e.user_id.as_ref() == Some(&"user1".to_string())));
    }

    #[test]
    fn test_filter_by_severity() {
        let events = create_test_events();
        let query = AuditQuery::new().severity(EventSeverity::Critical);
        let results = query.execute(&events);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].severity, EventSeverity::Critical);
    }

    #[test]
    fn test_filter_min_severity() {
        let events = create_test_events();
        let query = AuditQuery::new().min_severity(EventSeverity::Warning);
        let results = query.execute(&events);

        assert!(results.iter().all(|e| e.severity >= EventSeverity::Warning));
    }

    #[test]
    fn test_multiple_filters() {
        let events = create_test_events();
        let query = AuditQuery::new()
            .user_id("user1")
            .event_type(EventType::AuthLogin);
        let results = query.execute(&events);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].event_type, EventType::AuthLogin);
        assert_eq!(results[0].user_id.as_ref(), Some(&"user1".to_string()));
    }

    #[test]
    fn test_limit_and_offset() {
        let events = create_test_events();
        let query = AuditQuery::new().limit(2).offset(1);
        let results = query.execute(&events);

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_sorting() {
        let events = create_test_events();
        let query = AuditQuery::new().sort_by(SortField::Severity, SortOrder::Descending);
        let results = query.execute(&events);

        // Critical should be first
        assert_eq!(results[0].severity, EventSeverity::Critical);
    }

    #[test]
    fn test_query_result() {
        let events = create_test_events();
        let query = AuditQuery::new().limit(2);
        let result = QueryResult::from_query(&query, &events);

        assert_eq!(result.total_count, 4);
        assert_eq!(result.events.len(), 2);
        assert!(result.has_more());
        assert_eq!(result.next_offset(), Some(2));
    }
}
