//! Temporal aggregations - time-based grouping and rollups

use super::{AggregationOp, AggregationResult, Aggregator, MeanAggregator, SumAggregator};
use chrono::{DateTime, Datelike, Duration, Timelike, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Temporal bucket for time-based aggregations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TemporalBucket {
    Minute { year: i32, month: u32, day: u32, hour: u32, minute: u32 },
    Hour { year: i32, month: u32, day: u32, hour: u32 },
    Day { year: i32, month: u32, day: u32 },
    Week { year: i32, week: u32 },
    Month { year: i32, month: u32 },
    Year { year: i32 },
}

impl TemporalBucket {
    pub fn from_datetime(dt: DateTime<Utc>, resolution: TemporalResolution) -> Self {
        match resolution {
            TemporalResolution::Minute => Self::Minute {
                year: dt.year(),
                month: dt.month(),
                day: dt.day(),
                hour: dt.hour(),
                minute: dt.minute(),
            },
            TemporalResolution::Hour => Self::Hour {
                year: dt.year(),
                month: dt.month(),
                day: dt.day(),
                hour: dt.hour(),
            },
            TemporalResolution::Day => Self::Day {
                year: dt.year(),
                month: dt.month(),
                day: dt.day(),
            },
            TemporalResolution::Week => {
                let week = dt.iso_week().week();
                Self::Week {
                    year: dt.year(),
                    week,
                }
            }
            TemporalResolution::Month => Self::Month {
                year: dt.year(),
                month: dt.month(),
            },
            TemporalResolution::Year => Self::Year { year: dt.year() },
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::Minute { year, month, day, hour, minute } => {
                format!("{:04}-{:02}-{:02} {:02}:{:02}", year, month, day, hour, minute)
            }
            Self::Hour { year, month, day, hour } => {
                format!("{:04}-{:02}-{:02} {:02}:00", year, month, day, hour)
            }
            Self::Day { year, month, day } => {
                format!("{:04}-{:02}-{:02}", year, month, day)
            }
            Self::Week { year, week } => {
                format!("{:04}-W{:02}", year, week)
            }
            Self::Month { year, month } => {
                format!("{:04}-{:02}", year, month)
            }
            Self::Year { year } => {
                format!("{:04}", year)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemporalResolution {
    Minute,
    Hour,
    Day,
    Week,
    Month,
    Year,
}

/// Temporal aggregator for time-series data
pub struct TemporalAggregator {
    resolution: TemporalResolution,
    operation: AggregationOp,
    buckets: Arc<DashMap<TemporalBucket, Box<dyn Aggregator + Send + Sync>>>,
}

impl TemporalAggregator {
    pub fn new(resolution: TemporalResolution, operation: AggregationOp) -> Self {
        Self {
            resolution,
            operation,
            buckets: Arc::new(DashMap::new()),
        }
    }

    /// Add a value with a timestamp
    pub fn add(&self, timestamp: DateTime<Utc>, value: f64) {
        let bucket = TemporalBucket::from_datetime(timestamp, self.resolution);

        self.buckets
            .entry(bucket)
            .or_insert_with(|| self.create_aggregator())
            .add(value);
    }

    /// Get aggregation results for all buckets
    pub fn results(&self) -> Vec<(TemporalBucket, AggregationResult)> {
        self.buckets
            .iter()
            .map(|entry| {
                let bucket = *entry.key();
                let agg = entry.value();
                let result = AggregationResult::new(self.operation, agg.result(), agg.count());
                (bucket, result)
            })
            .collect()
    }

    /// Get results within a time range
    pub fn results_in_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<(TemporalBucket, AggregationResult)> {
        let start_bucket = TemporalBucket::from_datetime(start, self.resolution);
        let end_bucket = TemporalBucket::from_datetime(end, self.resolution);

        self.results()
            .into_iter()
            .filter(|(bucket, _)| bucket >= &start_bucket && bucket <= &end_bucket)
            .collect()
    }

    /// Get result for a specific bucket
    pub fn result_for_bucket(&self, bucket: TemporalBucket) -> Option<AggregationResult> {
        self.buckets.get(&bucket).map(|agg| {
            AggregationResult::new(self.operation, agg.result(), agg.count())
        })
    }

    /// Clear all buckets
    pub fn clear(&self) {
        self.buckets.clear();
    }

    /// Get the number of buckets
    pub fn bucket_count(&self) -> usize {
        self.buckets.len()
    }

    fn create_aggregator(&self) -> Box<dyn Aggregator + Send + Sync> {
        match self.operation {
            AggregationOp::Sum | AggregationOp::Count => Box::new(SumAggregator::default()),
            AggregationOp::Mean => Box::new(MeanAggregator::default()),
            _ => Box::new(SumAggregator::default()),
        }
    }
}

/// Rollup aggregator for hierarchical time-based aggregations
pub struct RollupAggregator {
    aggregators: Vec<(TemporalResolution, Arc<TemporalAggregator>)>,
}

impl RollupAggregator {
    pub fn new(operation: AggregationOp) -> Self {
        let resolutions = vec![
            TemporalResolution::Minute,
            TemporalResolution::Hour,
            TemporalResolution::Day,
            TemporalResolution::Week,
            TemporalResolution::Month,
        ];

        let aggregators = resolutions
            .into_iter()
            .map(|res| (res, Arc::new(TemporalAggregator::new(res, operation))))
            .collect();

        Self { aggregators }
    }

    /// Add a value to all resolution levels
    pub fn add(&self, timestamp: DateTime<Utc>, value: f64) {
        for (_, agg) in &self.aggregators {
            agg.add(timestamp, value);
        }
    }

    /// Get aggregator for a specific resolution
    pub fn get(&self, resolution: TemporalResolution) -> Option<Arc<TemporalAggregator>> {
        self.aggregators
            .iter()
            .find(|(res, _)| *res == resolution)
            .map(|(_, agg)| Arc::clone(agg))
    }

    /// Clear all aggregators
    pub fn clear(&self) {
        for (_, agg) in &self.aggregators {
            agg.clear();
        }
    }
}

/// Time-based windowing for streaming aggregations
pub struct SlidingWindowAggregator {
    window_duration: Duration,
    operation: AggregationOp,
    values: Arc<DashMap<DateTime<Utc>, f64>>,
}

impl SlidingWindowAggregator {
    pub fn new(window_duration: Duration, operation: AggregationOp) -> Self {
        Self {
            window_duration,
            operation,
            values: Arc::new(DashMap::new()),
        }
    }

    /// Add a value
    pub fn add(&self, value: f64) {
        self.add_with_timestamp(Utc::now(), value);
    }

    /// Add a value with a specific timestamp
    pub fn add_with_timestamp(&self, timestamp: DateTime<Utc>, value: f64) {
        self.values.insert(timestamp, value);
        self.cleanup();
    }

    /// Get the current aggregation result
    pub fn result(&self) -> Option<f64> {
        self.cleanup();

        let values: Vec<f64> = self.values.iter().map(|entry| *entry.value()).collect();

        if values.is_empty() {
            return None;
        }

        Some(match self.operation {
            AggregationOp::Sum => values.iter().sum(),
            AggregationOp::Count => values.len() as f64,
            AggregationOp::Mean => values.iter().sum::<f64>() / values.len() as f64,
            AggregationOp::Min => values.iter().cloned().fold(f64::INFINITY, f64::min),
            AggregationOp::Max => values.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
            _ => values.iter().sum(),
        })
    }

    fn cleanup(&self) {
        let cutoff = Utc::now() - self.window_duration;
        self.values.retain(|timestamp, _| *timestamp > cutoff);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temporal_bucket() {
        let dt = Utc::now();
        let bucket = TemporalBucket::from_datetime(dt, TemporalResolution::Hour);
        assert!(bucket.to_string().len() > 0);
    }

    #[test]
    fn test_temporal_aggregator() {
        let agg = TemporalAggregator::new(TemporalResolution::Hour, AggregationOp::Sum);

        let now = Utc::now();
        agg.add(now, 10.0);
        agg.add(now, 20.0);

        let results = agg.results();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1.value, 30.0);
    }
}
