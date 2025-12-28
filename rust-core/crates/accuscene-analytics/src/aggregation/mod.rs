//! Aggregation framework for multi-dimensional data analysis

pub mod dimensional;
pub mod spatial;
pub mod temporal;

pub use dimensional::{DimensionalAggregator, Dimension, DimensionValue};
pub use spatial::{SpatialAggregator, SpatialGrid, SpatialPoint};
pub use temporal::{TemporalAggregator, TemporalBucket};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Common aggregation operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AggregationOp {
    Sum,
    Count,
    Mean,
    Min,
    Max,
    StdDev,
    Variance,
}

/// Aggregation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationResult {
    pub operation: AggregationOp,
    pub value: f64,
    pub count: usize,
    pub timestamp: DateTime<Utc>,
}

impl AggregationResult {
    pub fn new(operation: AggregationOp, value: f64, count: usize) -> Self {
        Self {
            operation,
            value,
            count,
            timestamp: Utc::now(),
        }
    }
}

/// Generic aggregator trait
pub trait Aggregator: Send + Sync {
    /// Add a value to the aggregation
    fn add(&mut self, value: f64);

    /// Get the aggregation result
    fn result(&self) -> f64;

    /// Reset the aggregator
    fn reset(&mut self);

    /// Get the number of values aggregated
    fn count(&self) -> usize;
}

/// Sum aggregator
#[derive(Debug, Clone, Default)]
pub struct SumAggregator {
    sum: f64,
    count: usize,
}

impl Aggregator for SumAggregator {
    fn add(&mut self, value: f64) {
        self.sum += value;
        self.count += 1;
    }

    fn result(&self) -> f64 {
        self.sum
    }

    fn reset(&mut self) {
        self.sum = 0.0;
        self.count = 0;
    }

    fn count(&self) -> usize {
        self.count
    }
}

/// Mean aggregator
#[derive(Debug, Clone, Default)]
pub struct MeanAggregator {
    sum: f64,
    count: usize,
}

impl Aggregator for MeanAggregator {
    fn add(&mut self, value: f64) {
        self.sum += value;
        self.count += 1;
    }

    fn result(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.sum / self.count as f64
        }
    }

    fn reset(&mut self) {
        self.sum = 0.0;
        self.count = 0;
    }

    fn count(&self) -> usize {
        self.count
    }
}

/// Min/Max aggregator
#[derive(Debug, Clone)]
pub struct MinMaxAggregator {
    min: f64,
    max: f64,
    count: usize,
}

impl Default for MinMaxAggregator {
    fn default() -> Self {
        Self {
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
            count: 0,
        }
    }
}

impl MinMaxAggregator {
    pub fn min(&self) -> f64 {
        self.min
    }

    pub fn max(&self) -> f64 {
        self.max
    }
}

impl Aggregator for MinMaxAggregator {
    fn add(&mut self, value: f64) {
        if value < self.min {
            self.min = value;
        }
        if value > self.max {
            self.max = value;
        }
        self.count += 1;
    }

    fn result(&self) -> f64 {
        self.max - self.min
    }

    fn reset(&mut self) {
        self.min = f64::INFINITY;
        self.max = f64::NEG_INFINITY;
        self.count = 0;
    }

    fn count(&self) -> usize {
        self.count
    }
}
