//! Time series data structures for tracking values over time

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;

/// A single point in a time series
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
}

impl TimeSeriesPoint {
    pub fn new(timestamp: DateTime<Utc>, value: f64) -> Self {
        Self { timestamp, value }
    }

    pub fn now(value: f64) -> Self {
        Self {
            timestamp: Utc::now(),
            value,
        }
    }
}

/// Time series with windowed retention
#[derive(Debug)]
pub struct TimeSeries {
    name: String,
    points: Arc<RwLock<VecDeque<TimeSeriesPoint>>>,
    max_points: usize,
    retention_seconds: i64,
}

impl TimeSeries {
    pub fn new(name: impl Into<String>, max_points: usize, retention_seconds: i64) -> Self {
        Self {
            name: name.into(),
            points: Arc::new(RwLock::new(VecDeque::with_capacity(max_points))),
            max_points,
            retention_seconds,
        }
    }

    /// Add a data point
    pub fn add(&self, value: f64) {
        self.add_point(TimeSeriesPoint::now(value));
    }

    /// Add a data point with a specific timestamp
    pub fn add_point(&self, point: TimeSeriesPoint) {
        let mut points = self.points.write();

        // Remove old points
        self.cleanup(&mut points);

        // Add new point
        points.push_back(point);

        // Enforce max points limit
        if points.len() > self.max_points {
            points.pop_front();
        }
    }

    /// Get all points in the series
    pub fn points(&self) -> Vec<TimeSeriesPoint> {
        let mut points = self.points.write();
        self.cleanup(&mut points);
        points.iter().cloned().collect()
    }

    /// Get points within a time range
    pub fn points_in_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<TimeSeriesPoint> {
        let mut points = self.points.write();
        self.cleanup(&mut points);

        points
            .iter()
            .filter(|p| p.timestamp >= start && p.timestamp <= end)
            .cloned()
            .collect()
    }

    /// Get the latest value
    pub fn latest(&self) -> Option<TimeSeriesPoint> {
        let points = self.points.read();
        points.back().cloned()
    }

    /// Get the number of points
    pub fn len(&self) -> usize {
        let points = self.points.read();
        points.len()
    }

    /// Check if the series is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Calculate the mean of all points
    pub fn mean(&self) -> Option<f64> {
        let points = self.points.read();
        if points.is_empty() {
            return None;
        }

        let sum: f64 = points.iter().map(|p| p.value).sum();
        Some(sum / points.len() as f64)
    }

    /// Calculate the sum of all points
    pub fn sum(&self) -> f64 {
        let points = self.points.read();
        points.iter().map(|p| p.value).sum()
    }

    /// Get the minimum value
    pub fn min(&self) -> Option<f64> {
        let points = self.points.read();
        points.iter().map(|p| p.value).min_by(|a, b| a.partial_cmp(b).unwrap())
    }

    /// Get the maximum value
    pub fn max(&self) -> Option<f64> {
        let points = self.points.read();
        points.iter().map(|p| p.value).max_by(|a, b| a.partial_cmp(b).unwrap())
    }

    /// Calculate the rate of change (per second)
    pub fn rate(&self) -> Option<f64> {
        let points = self.points.read();
        if points.len() < 2 {
            return None;
        }

        let first = points.front()?;
        let last = points.back()?;

        let value_diff = last.value - first.value;
        let time_diff = (last.timestamp - first.timestamp).num_seconds();

        if time_diff == 0 {
            return None;
        }

        Some(value_diff / time_diff as f64)
    }

    /// Calculate moving average with window size
    pub fn moving_average(&self, window: usize) -> Vec<TimeSeriesPoint> {
        let points = self.points.read();
        if points.len() < window {
            return Vec::new();
        }

        let mut result = Vec::new();
        for i in window - 1..points.len() {
            let window_points: Vec<&TimeSeriesPoint> =
                points.iter().skip(i + 1 - window).take(window).collect();

            let sum: f64 = window_points.iter().map(|p| p.value).sum();
            let avg = sum / window as f64;

            result.push(TimeSeriesPoint {
                timestamp: points[i].timestamp,
                value: avg,
            });
        }

        result
    }

    /// Detect trend (positive, negative, or stable)
    pub fn trend(&self) -> Trend {
        let points = self.points.read();
        if points.len() < 2 {
            return Trend::Stable;
        }

        // Simple linear regression
        let n = points.len() as f64;
        let x_mean = (n - 1.0) / 2.0;
        let y_mean = points.iter().map(|p| p.value).sum::<f64>() / n;

        let mut numerator = 0.0;
        let mut denominator = 0.0;

        for (i, point) in points.iter().enumerate() {
            let x = i as f64;
            numerator += (x - x_mean) * (point.value - y_mean);
            denominator += (x - x_mean).powi(2);
        }

        if denominator == 0.0 {
            return Trend::Stable;
        }

        let slope = numerator / denominator;

        if slope > 0.01 {
            Trend::Increasing
        } else if slope < -0.01 {
            Trend::Decreasing
        } else {
            Trend::Stable
        }
    }

    fn cleanup(&self, points: &mut VecDeque<TimeSeriesPoint>) {
        if self.retention_seconds <= 0 {
            return;
        }

        let cutoff = Utc::now() - chrono::Duration::seconds(self.retention_seconds);
        while let Some(front) = points.front() {
            if front.timestamp < cutoff {
                points.pop_front();
            } else {
                break;
            }
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Trend {
    Increasing,
    Decreasing,
    Stable,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeseries() {
        let ts = TimeSeries::new("test", 100, 3600);

        ts.add(10.0);
        ts.add(20.0);
        ts.add(30.0);

        assert_eq!(ts.len(), 3);
        assert_eq!(ts.sum(), 60.0);
        assert_eq!(ts.mean(), Some(20.0));
    }

    #[test]
    fn test_moving_average() {
        let ts = TimeSeries::new("test", 100, 3600);

        for i in 1..=10 {
            ts.add(i as f64);
        }

        let ma = ts.moving_average(3);
        assert!(!ma.is_empty());
    }
}
