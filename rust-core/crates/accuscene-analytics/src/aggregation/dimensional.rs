//! Multi-dimensional aggregations and rollups

use super::{AggregationOp, AggregationResult, Aggregator, MeanAggregator, SumAggregator};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// A dimension for multi-dimensional analysis
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Dimension {
    pub name: String,
    pub value: DimensionValue,
}

impl Dimension {
    pub fn new(name: impl Into<String>, value: DimensionValue) -> Self {
        Self {
            name: name.into(),
            value,
        }
    }

    pub fn string(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: DimensionValue::String(value.into()),
        }
    }

    pub fn int(name: impl Into<String>, value: i64) -> Self {
        Self {
            name: name.into(),
            value: DimensionValue::Int(value),
        }
    }

    pub fn float(name: impl Into<String>, value: f64) -> Self {
        Self {
            name: name.into(),
            value: DimensionValue::Float(value.to_bits()),
        }
    }

    pub fn bool(name: impl Into<String>, value: bool) -> Self {
        Self {
            name: name.into(),
            value: DimensionValue::Bool(value),
        }
    }
}

/// Value types for dimensions
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DimensionValue {
    String(String),
    Int(i64),
    Float(u64), // Stored as bits for Hash/Eq
    Bool(bool),
}

impl DimensionValue {
    pub fn as_string(&self) -> String {
        match self {
            Self::String(s) => s.clone(),
            Self::Int(i) => i.to_string(),
            Self::Float(bits) => f64::from_bits(*bits).to_string(),
            Self::Bool(b) => b.to_string(),
        }
    }
}

/// Key for multi-dimensional aggregations
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DimensionKey {
    dimensions: Vec<Dimension>,
}

impl DimensionKey {
    pub fn new(dimensions: Vec<Dimension>) -> Self {
        let mut dimensions = dimensions;
        dimensions.sort_by(|a, b| a.name.cmp(&b.name));
        Self { dimensions }
    }

    pub fn dimensions(&self) -> &[Dimension] {
        &self.dimensions
    }

    pub fn to_string(&self) -> String {
        self.dimensions
            .iter()
            .map(|d| format!("{}={}", d.name, d.value.as_string()))
            .collect::<Vec<_>>()
            .join(",")
    }

    /// Get all possible rollup keys (excluding subsets of dimensions)
    pub fn rollups(&self) -> Vec<DimensionKey> {
        let mut rollups = Vec::new();
        let n = self.dimensions.len();

        // Generate all non-empty subsets
        for i in 1..(1 << n) {
            let mut subset = Vec::new();
            for j in 0..n {
                if (i & (1 << j)) != 0 {
                    subset.push(self.dimensions[j].clone());
                }
            }
            if !subset.is_empty() && subset.len() < n {
                rollups.push(DimensionKey::new(subset));
            }
        }

        rollups
    }
}

/// Multi-dimensional aggregator
pub struct DimensionalAggregator {
    operation: AggregationOp,
    aggregations: Arc<DashMap<DimensionKey, Box<dyn Aggregator + Send + Sync>>>,
    enable_rollups: bool,
}

impl DimensionalAggregator {
    pub fn new(operation: AggregationOp) -> Self {
        Self {
            operation,
            aggregations: Arc::new(DashMap::new()),
            enable_rollups: false,
        }
    }

    pub fn with_rollups(mut self, enable: bool) -> Self {
        self.enable_rollups = enable;
        self
    }

    /// Add a value with dimensions
    pub fn add(&self, dimensions: Vec<Dimension>, value: f64) {
        let key = DimensionKey::new(dimensions);

        // Add to the specific dimension combination
        self.aggregations
            .entry(key.clone())
            .or_insert_with(|| self.create_aggregator())
            .add(value);

        // Add to rollups if enabled
        if self.enable_rollups {
            for rollup_key in key.rollups() {
                self.aggregations
                    .entry(rollup_key)
                    .or_insert_with(|| self.create_aggregator())
                    .add(value);
            }
        }
    }

    /// Get all aggregation results
    pub fn results(&self) -> Vec<(DimensionKey, AggregationResult)> {
        self.aggregations
            .iter()
            .map(|entry| {
                let key = entry.key().clone();
                let agg = entry.value();
                let result = AggregationResult::new(self.operation, agg.result(), agg.count());
                (key, result)
            })
            .collect()
    }

    /// Get result for a specific dimension combination
    pub fn result_for(&self, dimensions: Vec<Dimension>) -> Option<AggregationResult> {
        let key = DimensionKey::new(dimensions);
        self.aggregations.get(&key).map(|agg| {
            AggregationResult::new(self.operation, agg.result(), agg.count())
        })
    }

    /// Get results filtered by a dimension
    pub fn results_where(
        &self,
        dimension_name: &str,
        dimension_value: &DimensionValue,
    ) -> Vec<(DimensionKey, AggregationResult)> {
        self.results()
            .into_iter()
            .filter(|(key, _)| {
                key.dimensions()
                    .iter()
                    .any(|d| d.name == dimension_name && &d.value == dimension_value)
            })
            .collect()
    }

    /// Get top N results by value
    pub fn top_n(&self, n: usize) -> Vec<(DimensionKey, AggregationResult)> {
        let mut results = self.results();
        results.sort_by(|a, b| b.1.value.partial_cmp(&a.1.value).unwrap());
        results.into_iter().take(n).collect()
    }

    /// Clear all aggregations
    pub fn clear(&self) {
        self.aggregations.clear();
    }

    /// Get the number of dimension combinations
    pub fn combination_count(&self) -> usize {
        self.aggregations.len()
    }

    fn create_aggregator(&self) -> Box<dyn Aggregator + Send + Sync> {
        match self.operation {
            AggregationOp::Sum | AggregationOp::Count => Box::new(SumAggregator::default()),
            AggregationOp::Mean => Box::new(MeanAggregator::default()),
            _ => Box::new(SumAggregator::default()),
        }
    }
}

/// Cube aggregator for OLAP-style analytics
pub struct CubeAggregator {
    aggregators: Vec<Arc<DimensionalAggregator>>,
    dimension_names: Vec<String>,
}

impl CubeAggregator {
    pub fn new(dimension_names: Vec<String>, operation: AggregationOp) -> Self {
        let mut aggregators = Vec::new();

        // Create aggregators for all dimension combinations
        let n = dimension_names.len();
        for i in 1..=(1 << n) {
            let agg = Arc::new(DimensionalAggregator::new(operation));
            aggregators.push(agg);
        }

        Self {
            aggregators,
            dimension_names,
        }
    }

    /// Add a value with all dimensions
    pub fn add(&self, dimensions: Vec<Dimension>, value: f64) {
        // Add to all relevant aggregators
        for agg in &self.aggregators {
            agg.add(dimensions.clone(), value);
        }
    }

    /// Slice: fix one dimension, vary others
    pub fn slice(&self, dimension: Dimension) -> Vec<(DimensionKey, AggregationResult)> {
        let mut results = Vec::new();

        for agg in &self.aggregators {
            for (key, result) in agg.results() {
                if key.dimensions().iter().any(|d| d == &dimension) {
                    results.push((key, result));
                }
            }
        }

        results
    }

    /// Dice: filter by multiple dimensions
    pub fn dice(&self, dimensions: Vec<Dimension>) -> Vec<(DimensionKey, AggregationResult)> {
        let mut results = Vec::new();

        for agg in &self.aggregators {
            for (key, result) in agg.results() {
                if dimensions.iter().all(|d| key.dimensions().contains(d)) {
                    results.push((key, result));
                }
            }
        }

        results
    }

    /// Drill down: increase granularity
    pub fn drill_down(&self, from_dimensions: Vec<Dimension>, add_dimension: Dimension) -> Vec<(DimensionKey, AggregationResult)> {
        let mut new_dims = from_dimensions;
        new_dims.push(add_dimension);

        let key = DimensionKey::new(new_dims);

        let mut results = Vec::new();
        for agg in &self.aggregators {
            if let Some(result) = agg.result_for(key.dimensions().to_vec()) {
                results.push((key.clone(), result));
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dimension_key() {
        let dims = vec![
            Dimension::string("country", "US"),
            Dimension::string("city", "NYC"),
        ];

        let key = DimensionKey::new(dims);
        assert_eq!(key.dimensions().len(), 2);
    }

    #[test]
    fn test_dimensional_aggregator() {
        let agg = DimensionalAggregator::new(AggregationOp::Sum);

        agg.add(vec![Dimension::string("region", "west")], 10.0);
        agg.add(vec![Dimension::string("region", "west")], 20.0);
        agg.add(vec![Dimension::string("region", "east")], 15.0);

        let results = agg.results();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_rollups() {
        let dims = vec![
            Dimension::string("country", "US"),
            Dimension::string("state", "CA"),
        ];

        let key = DimensionKey::new(dims);
        let rollups = key.rollups();

        assert!(rollups.len() > 0);
    }
}
