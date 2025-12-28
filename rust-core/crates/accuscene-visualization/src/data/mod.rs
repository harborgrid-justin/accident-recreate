pub mod aggregation;
pub mod interpolation;
pub mod sampling;

use serde::{Deserialize, Serialize};

/// A data point with x and y coordinates
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DataPoint {
    pub x: f64,
    pub y: f64,
}

impl DataPoint {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// A time-series data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    pub timestamp: i64,
    pub value: f64,
}

impl TimeSeriesPoint {
    pub fn new(timestamp: i64, value: f64) -> Self {
        Self { timestamp, value }
    }
}

/// Multi-dimensional data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiDimPoint {
    pub values: Vec<f64>,
}

impl MultiDimPoint {
    pub fn new(values: Vec<f64>) -> Self {
        Self { values }
    }

    pub fn dimension(&self) -> usize {
        self.values.len()
    }
}

/// A data series with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSeries {
    pub name: String,
    pub data: Vec<DataPoint>,
    pub color: Option<String>,
    pub line_style: Option<LineStyle>,
}

impl DataSeries {
    pub fn new(name: String, data: Vec<DataPoint>) -> Self {
        Self {
            name,
            data,
            color: None,
            line_style: None,
        }
    }

    pub fn with_color(mut self, color: String) -> Self {
        self.color = Some(color);
        self
    }

    pub fn with_line_style(mut self, style: LineStyle) -> Self {
        self.line_style = Some(style);
        self
    }
}

/// Line style for charts
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LineStyle {
    Solid,
    Dashed,
    Dotted,
    DashDot,
}

/// Statistical summary of a dataset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalSummary {
    pub count: usize,
    pub mean: f64,
    pub median: f64,
    pub std_dev: f64,
    pub variance: f64,
    pub min: f64,
    pub max: f64,
    pub quartiles: Quartiles,
}

/// Quartile values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quartiles {
    pub q1: f64,
    pub q2: f64,
    pub q3: f64,
}

/// Histogram bin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramBin {
    pub start: f64,
    pub end: f64,
    pub count: usize,
    pub frequency: f64,
}

/// Correlation matrix entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationEntry {
    pub variable1: String,
    pub variable2: String,
    pub correlation: f64,
    pub p_value: Option<f64>,
}
