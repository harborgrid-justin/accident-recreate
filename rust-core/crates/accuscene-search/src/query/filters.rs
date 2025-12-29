//! Filter types and combinations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchFilters {
    pub categories: Option<Vec<String>>,
    pub statuses: Option<Vec<String>>,
    pub severities: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub created_by: Option<Vec<String>>,
    pub date_range: Option<DateRange>,
    pub custom: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl SearchFilters {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn builder() -> SearchFiltersBuilder {
        SearchFiltersBuilder::new()
    }

    /// Check if filters are empty
    pub fn is_empty(&self) -> bool {
        self.categories.is_none()
            && self.statuses.is_none()
            && self.severities.is_none()
            && self.tags.is_none()
            && self.created_by.is_none()
            && self.date_range.is_none()
            && self.custom.is_none()
    }

    /// Merge with another filter set (AND operation)
    pub fn merge(&mut self, other: SearchFilters) {
        if let Some(cats) = other.categories {
            self.categories = Some(
                self.categories
                    .get_or_insert_with(Vec::new)
                    .iter()
                    .filter(|c| cats.contains(c))
                    .cloned()
                    .collect(),
            );
        }

        if let Some(stats) = other.statuses {
            self.statuses = Some(
                self.statuses
                    .get_or_insert_with(Vec::new)
                    .iter()
                    .filter(|s| stats.contains(s))
                    .cloned()
                    .collect(),
            );
        }

        if let Some(sevs) = other.severities {
            self.severities = Some(
                self.severities
                    .get_or_insert_with(Vec::new)
                    .iter()
                    .filter(|s| sevs.contains(s))
                    .cloned()
                    .collect(),
            );
        }

        if other.date_range.is_some() {
            self.date_range = other.date_range;
        }
    }

    /// Combine with another filter set (OR operation)
    pub fn combine(&mut self, other: SearchFilters) {
        if let Some(cats) = other.categories {
            self.categories
                .get_or_insert_with(Vec::new)
                .extend(cats);
        }

        if let Some(stats) = other.statuses {
            self.statuses
                .get_or_insert_with(Vec::new)
                .extend(stats);
        }

        if let Some(sevs) = other.severities {
            self.severities
                .get_or_insert_with(Vec::new)
                .extend(sevs);
        }

        if let Some(tags) = other.tags {
            self.tags
                .get_or_insert_with(Vec::new)
                .extend(tags);
        }
    }
}

pub struct SearchFiltersBuilder {
    filters: SearchFilters,
}

impl SearchFiltersBuilder {
    pub fn new() -> Self {
        Self {
            filters: SearchFilters::default(),
        }
    }

    pub fn category(mut self, category: impl Into<String>) -> Self {
        self.filters
            .categories
            .get_or_insert_with(Vec::new)
            .push(category.into());
        self
    }

    pub fn categories(mut self, categories: Vec<String>) -> Self {
        self.filters.categories = Some(categories);
        self
    }

    pub fn status(mut self, status: impl Into<String>) -> Self {
        self.filters
            .statuses
            .get_or_insert_with(Vec::new)
            .push(status.into());
        self
    }

    pub fn statuses(mut self, statuses: Vec<String>) -> Self {
        self.filters.statuses = Some(statuses);
        self
    }

    pub fn severity(mut self, severity: impl Into<String>) -> Self {
        self.filters
            .severities
            .get_or_insert_with(Vec::new)
            .push(severity.into());
        self
    }

    pub fn severities(mut self, severities: Vec<String>) -> Self {
        self.filters.severities = Some(severities);
        self
    }

    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.filters
            .tags
            .get_or_insert_with(Vec::new)
            .push(tag.into());
        self
    }

    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.filters.tags = Some(tags);
        self
    }

    pub fn created_by(mut self, user: impl Into<String>) -> Self {
        self.filters
            .created_by
            .get_or_insert_with(Vec::new)
            .push(user.into());
        self
    }

    pub fn date_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.filters.date_range = Some(DateRange { start, end });
        self
    }

    pub fn last_n_days(mut self, days: i64) -> Self {
        let end = Utc::now();
        let start = end - chrono::Duration::days(days);
        self.filters.date_range = Some(DateRange { start, end });
        self
    }

    pub fn custom(mut self, custom: serde_json::Value) -> Self {
        self.filters.custom = Some(custom);
        self
    }

    pub fn build(self) -> SearchFilters {
        self.filters
    }
}

impl Default for SearchFiltersBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_builder() {
        let filters = SearchFiltersBuilder::new()
            .category("accident")
            .status("open")
            .severity("high")
            .last_n_days(7)
            .build();

        assert!(filters.categories.is_some());
        assert!(filters.statuses.is_some());
        assert!(filters.date_range.is_some());
    }

    #[test]
    fn test_filter_merge() {
        let mut filter1 = SearchFiltersBuilder::new()
            .categories(vec!["cat1".to_string(), "cat2".to_string()])
            .build();

        let filter2 = SearchFiltersBuilder::new()
            .categories(vec!["cat2".to_string(), "cat3".to_string()])
            .build();

        filter1.merge(filter2);

        let cats = filter1.categories.unwrap();
        assert_eq!(cats.len(), 1);
        assert!(cats.contains(&"cat2".to_string()));
    }

    #[test]
    fn test_filter_combine() {
        let mut filter1 = SearchFiltersBuilder::new()
            .category("cat1")
            .build();

        let filter2 = SearchFiltersBuilder::new()
            .category("cat2")
            .build();

        filter1.combine(filter2);

        let cats = filter1.categories.unwrap();
        assert_eq!(cats.len(), 2);
    }
}
