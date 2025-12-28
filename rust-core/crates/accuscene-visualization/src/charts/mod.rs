pub mod correlation;
pub mod distribution;
pub mod timeseries;

use serde::{Deserialize, Serialize};

/// Chart data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartData {
    pub title: String,
    pub x_axis: AxisConfig,
    pub y_axis: AxisConfig,
    pub series: Vec<SeriesData>,
}

impl ChartData {
    pub fn new(title: String) -> Self {
        Self {
            title,
            x_axis: AxisConfig::default(),
            y_axis: AxisConfig::default(),
            series: Vec::new(),
        }
    }

    pub fn add_series(&mut self, series: SeriesData) {
        self.series.push(series);
    }

    pub fn with_x_axis(mut self, axis: AxisConfig) -> Self {
        self.x_axis = axis;
        self
    }

    pub fn with_y_axis(mut self, axis: AxisConfig) -> Self {
        self.y_axis = axis;
        self
    }
}

/// Axis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxisConfig {
    pub label: String,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub grid: bool,
    pub tick_format: Option<String>,
}

impl Default for AxisConfig {
    fn default() -> Self {
        Self {
            label: String::new(),
            min: None,
            max: None,
            grid: true,
            tick_format: None,
        }
    }
}

/// Series data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesData {
    pub name: String,
    pub data: Vec<SeriesPoint>,
    pub color: Option<String>,
    pub chart_type: ChartType,
}

impl SeriesData {
    pub fn new(name: String, data: Vec<SeriesPoint>, chart_type: ChartType) -> Self {
        Self {
            name,
            data,
            color: None,
            chart_type,
        }
    }

    pub fn with_color(mut self, color: String) -> Self {
        self.color = Some(color);
        self
    }
}

/// Chart types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ChartType {
    Line,
    Area,
    Bar,
    Scatter,
    Heatmap,
    Radar,
}

/// A point in a series
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesPoint {
    pub x: f64,
    pub y: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl SeriesPoint {
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            x,
            y,
            label: None,
            metadata: None,
        }
    }

    pub fn with_label(mut self, label: String) -> Self {
        self.label = Some(label);
        self
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}
