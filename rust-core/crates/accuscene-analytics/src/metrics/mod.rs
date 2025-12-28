//! Metrics framework for tracking system and business metrics

pub mod counter;
pub mod gauge;
pub mod histogram;
pub mod timeseries;

pub use counter::{Counter, RateCounter};
pub use gauge::Gauge;
pub use histogram::Histogram;
pub use timeseries::{TimeSeries, TimeSeriesPoint};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Common trait for all metrics
pub trait Metric: Send + Sync {
    /// Get the metric name
    fn name(&self) -> &str;

    /// Get metric tags
    fn tags(&self) -> &HashMap<String, String>;

    /// Reset the metric
    fn reset(&mut self);

    /// Get the last update time
    fn last_updated(&self) -> DateTime<Utc>;
}

/// Metric metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricMetadata {
    pub name: String,
    pub description: String,
    pub unit: String,
    pub tags: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
}

impl MetricMetadata {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            unit: String::new(),
            tags: HashMap::new(),
            created_at: Utc::now(),
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    pub fn with_unit(mut self, unit: impl Into<String>) -> Self {
        self.unit = unit.into();
        self
    }

    pub fn with_tag(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.tags.insert(key.into(), value.into());
        self
    }
}
