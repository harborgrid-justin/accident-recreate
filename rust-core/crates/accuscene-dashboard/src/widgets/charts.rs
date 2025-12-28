//! Chart widget implementation
//!
//! Supports various chart types for data visualization

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};

use super::{Widget, WidgetConfig, WidgetData, InteractionEvent, ExportFormat};
use crate::error::{WidgetError, WidgetResult};

/// Chart type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChartType {
    /// Line chart
    Line,
    /// Bar chart
    Bar,
    /// Area chart
    Area,
    /// Pie chart
    Pie,
    /// Donut chart
    Donut,
    /// Scatter plot
    Scatter,
    /// Heatmap
    Heatmap,
    /// Radar/Spider chart
    Radar,
}

impl ChartType {
    /// Get chart type from string
    pub fn from_str(s: &str) -> WidgetResult<Self> {
        match s.to_lowercase().as_str() {
            "line" => Ok(ChartType::Line),
            "bar" => Ok(ChartType::Bar),
            "area" => Ok(ChartType::Area),
            "pie" => Ok(ChartType::Pie),
            "donut" => Ok(ChartType::Donut),
            "scatter" => Ok(ChartType::Scatter),
            "heatmap" => Ok(ChartType::Heatmap),
            "radar" => Ok(ChartType::Radar),
            _ => Err(WidgetError::invalid_type(format!("Unknown chart type: {}", s))),
        }
    }
}

/// Data point for charts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    /// X-axis value (can be time, category, or number)
    pub x: JsonValue,

    /// Y-axis value
    pub y: f64,

    /// Additional metadata
    pub metadata: Option<JsonValue>,
}

impl DataPoint {
    /// Create a new data point
    pub fn new(x: JsonValue, y: f64) -> Self {
        Self {
            x,
            y,
            metadata: None,
        }
    }

    /// Create with metadata
    pub fn with_metadata(x: JsonValue, y: f64, metadata: JsonValue) -> Self {
        Self {
            x,
            y,
            metadata: Some(metadata),
        }
    }
}

/// Data series for multi-series charts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSeries {
    /// Series name
    pub name: String,

    /// Data points
    pub data: Vec<DataPoint>,

    /// Series color (hex)
    pub color: Option<String>,

    /// Series type (for mixed charts)
    pub series_type: Option<ChartType>,

    /// Visibility
    pub visible: bool,
}

impl DataSeries {
    /// Create a new data series
    pub fn new(name: String, data: Vec<DataPoint>) -> Self {
        Self {
            name,
            data,
            color: None,
            series_type: None,
            visible: true,
        }
    }

    /// Set color
    pub fn with_color(mut self, color: String) -> Self {
        self.color = Some(color);
        self
    }

    /// Set series type
    pub fn with_type(mut self, chart_type: ChartType) -> Self {
        self.series_type = Some(chart_type);
        self
    }
}

/// Axis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxisConfig {
    /// Axis label
    pub label: Option<String>,

    /// Axis type (linear, logarithmic, time, category)
    pub axis_type: String,

    /// Show grid lines
    pub show_grid: bool,

    /// Minimum value
    pub min: Option<f64>,

    /// Maximum value
    pub max: Option<f64>,

    /// Show axis
    pub show_axis: bool,
}

impl Default for AxisConfig {
    fn default() -> Self {
        Self {
            label: None,
            axis_type: "linear".to_string(),
            show_grid: true,
            min: None,
            max: None,
            show_axis: true,
        }
    }
}

/// Legend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegendConfig {
    /// Show legend
    pub show: bool,

    /// Position (top, bottom, left, right)
    pub position: String,

    /// Alignment (start, center, end)
    pub align: String,
}

impl Default for LegendConfig {
    fn default() -> Self {
        Self {
            show: true,
            position: "bottom".to_string(),
            align: "center".to_string(),
        }
    }
}

/// Tooltip configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TooltipConfig {
    /// Enable tooltip
    pub enabled: bool,

    /// Tooltip format string
    pub format: Option<String>,

    /// Shared tooltip (for multiple series)
    pub shared: bool,

    /// Sort values in tooltip
    pub sorted: bool,
}

impl Default for TooltipConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            format: None,
            shared: true,
            sorted: false,
        }
    }
}

/// Chart configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartConfig {
    /// Chart type
    pub chart_type: ChartType,

    /// Data series
    pub series: Vec<DataSeries>,

    /// X-axis configuration
    pub x_axis: AxisConfig,

    /// Y-axis configuration
    pub y_axis: AxisConfig,

    /// Legend configuration
    pub legend: LegendConfig,

    /// Tooltip configuration
    pub tooltip: TooltipConfig,

    /// Enable zoom
    pub zoom_enabled: bool,

    /// Enable pan
    pub pan_enabled: bool,

    /// Enable animations
    pub animations_enabled: bool,

    /// Stacked mode (for bar/area charts)
    pub stacked: bool,

    /// Color palette
    pub color_palette: Vec<String>,
}

impl ChartConfig {
    /// Create a new chart configuration
    pub fn new(chart_type: ChartType) -> Self {
        Self {
            chart_type,
            series: Vec::new(),
            x_axis: AxisConfig::default(),
            y_axis: AxisConfig::default(),
            legend: LegendConfig::default(),
            tooltip: TooltipConfig::default(),
            zoom_enabled: true,
            pan_enabled: true,
            animations_enabled: true,
            stacked: false,
            color_palette: vec![
                "#1976d2".to_string(),
                "#dc004e".to_string(),
                "#4caf50".to_string(),
                "#ff9800".to_string(),
                "#9c27b0".to_string(),
                "#00bcd4".to_string(),
            ],
        }
    }

    /// Add a data series
    pub fn add_series(&mut self, series: DataSeries) {
        self.series.push(series);
    }

    /// Get total data points across all series
    pub fn total_data_points(&self) -> usize {
        self.series.iter().map(|s| s.data.len()).sum()
    }
}

/// Chart widget implementation
pub struct ChartWidget {
    config: WidgetConfig,
    data: Option<WidgetData>,
    chart_config: ChartConfig,
}

impl ChartWidget {
    /// Create a new chart widget
    pub fn new(config: WidgetConfig, chart_config: ChartConfig) -> Self {
        Self {
            config,
            data: None,
            chart_config,
        }
    }

    /// Update chart data
    pub fn update_series(&mut self, series: Vec<DataSeries>) {
        self.chart_config.series = series;
    }

    /// Add a series
    pub fn add_series(&mut self, series: DataSeries) {
        self.chart_config.add_series(series);
    }

    /// Parse chart data from JSON
    fn parse_chart_data(&self, data: &JsonValue) -> WidgetResult<Vec<DataSeries>> {
        let series_array = data["series"]
            .as_array()
            .ok_or_else(|| WidgetError::data_error("Expected 'series' array in data"))?;

        let mut series_list = Vec::new();

        for series_data in series_array {
            let name = series_data["name"]
                .as_str()
                .ok_or_else(|| WidgetError::missing_field("name"))?
                .to_string();

            let data_array = series_data["data"]
                .as_array()
                .ok_or_else(|| WidgetError::missing_field("data"))?;

            let mut data_points = Vec::new();
            for point in data_array {
                let x = point["x"].clone();
                let y = point["y"]
                    .as_f64()
                    .ok_or_else(|| WidgetError::data_error("y value must be a number"))?;

                let metadata = point.get("metadata").cloned();
                let data_point = if let Some(meta) = metadata {
                    DataPoint::with_metadata(x, y, meta)
                } else {
                    DataPoint::new(x, y)
                };

                data_points.push(data_point);
            }

            let mut series = DataSeries::new(name, data_points);

            if let Some(color) = series_data["color"].as_str() {
                series.color = Some(color.to_string());
            }

            series_list.push(series);
        }

        Ok(series_list)
    }

    /// Serialize chart config to JSON
    fn serialize_config(&self) -> JsonValue {
        json!({
            "chart_type": match self.chart_config.chart_type {
                ChartType::Line => "line",
                ChartType::Bar => "bar",
                ChartType::Area => "area",
                ChartType::Pie => "pie",
                ChartType::Donut => "donut",
                ChartType::Scatter => "scatter",
                ChartType::Heatmap => "heatmap",
                ChartType::Radar => "radar",
            },
            "series": self.chart_config.series.iter().map(|s| json!({
                "name": s.name,
                "data": s.data.iter().map(|p| json!({
                    "x": p.x,
                    "y": p.y,
                    "metadata": p.metadata,
                })).collect::<Vec<_>>(),
                "color": s.color,
                "visible": s.visible,
            })).collect::<Vec<_>>(),
            "x_axis": {
                "label": self.chart_config.x_axis.label,
                "type": self.chart_config.x_axis.axis_type,
                "show_grid": self.chart_config.x_axis.show_grid,
                "min": self.chart_config.x_axis.min,
                "max": self.chart_config.x_axis.max,
                "show_axis": self.chart_config.x_axis.show_axis,
            },
            "y_axis": {
                "label": self.chart_config.y_axis.label,
                "type": self.chart_config.y_axis.axis_type,
                "show_grid": self.chart_config.y_axis.show_grid,
                "min": self.chart_config.y_axis.min,
                "max": self.chart_config.y_axis.max,
                "show_axis": self.chart_config.y_axis.show_axis,
            },
            "legend": self.chart_config.legend,
            "tooltip": self.chart_config.tooltip,
            "zoom_enabled": self.chart_config.zoom_enabled,
            "pan_enabled": self.chart_config.pan_enabled,
            "animations_enabled": self.chart_config.animations_enabled,
            "stacked": self.chart_config.stacked,
            "color_palette": self.chart_config.color_palette,
        })
    }
}

#[async_trait]
impl Widget for ChartWidget {
    fn config(&self) -> &WidgetConfig {
        &self.config
    }

    fn data(&self) -> Option<&WidgetData> {
        self.data.as_ref()
    }

    async fn fetch_data(&mut self) -> WidgetResult<WidgetData> {
        let chart_data = self.serialize_config();
        let data = WidgetData::new(chart_data);
        self.data = Some(data.clone());
        Ok(data)
    }

    fn validate(&self) -> WidgetResult<()> {
        if self.chart_config.series.is_empty() {
            return Err(WidgetError::invalid_config("At least one data series is required"));
        }

        for series in &self.chart_config.series {
            if series.name.is_empty() {
                return Err(WidgetError::invalid_config("Series name cannot be empty"));
            }
            if series.data.is_empty() {
                return Err(WidgetError::invalid_config(
                    format!("Series '{}' has no data points", series.name)
                ));
            }
        }

        Ok(())
    }

    async fn handle_interaction(&mut self, event: InteractionEvent) -> WidgetResult<()> {
        match event.event_type.as_str() {
            "zoom" => {
                // Handle zoom interaction
                Ok(())
            }
            "pan" => {
                // Handle pan interaction
                Ok(())
            }
            "toggle_series" => {
                // Toggle series visibility
                if let Some(series_name) = event.data["series"].as_str() {
                    if let Some(series) = self.chart_config.series.iter_mut()
                        .find(|s| s.name == series_name) {
                        series.visible = !series.visible;
                    }
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn export(&self, format: ExportFormat) -> WidgetResult<Vec<u8>> {
        match format {
            ExportFormat::Json => {
                let json_data = self.serialize_config();
                serde_json::to_vec_pretty(&json_data)
                    .map_err(|e| WidgetError::render_error(format!("JSON export failed: {}", e)))
            }
            ExportFormat::Csv => {
                let mut csv = String::new();
                csv.push_str("Series,X,Y\n");

                for series in &self.chart_config.series {
                    for point in &series.data {
                        csv.push_str(&format!("{},{},{}\n",
                            series.name,
                            point.x,
                            point.y
                        ));
                    }
                }

                Ok(csv.into_bytes())
            }
            _ => Err(WidgetError::render_error(format!("Export format {:?} not supported", format))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chart_type_parsing() {
        assert_eq!(ChartType::from_str("line").unwrap(), ChartType::Line);
        assert_eq!(ChartType::from_str("BAR").unwrap(), ChartType::Bar);
        assert!(ChartType::from_str("invalid").is_err());
    }

    #[test]
    fn test_data_series_builder() {
        let data = vec![
            DataPoint::new(json!("2024-01"), 100.0),
            DataPoint::new(json!("2024-02"), 150.0),
        ];

        let series = DataSeries::new("Sales".to_string(), data)
            .with_color("#1976d2".to_string());

        assert_eq!(series.name, "Sales");
        assert_eq!(series.data.len(), 2);
        assert_eq!(series.color, Some("#1976d2".to_string()));
    }

    #[test]
    fn test_chart_config() {
        let mut config = ChartConfig::new(ChartType::Line);
        let series = DataSeries::new("Test".to_string(), vec![
            DataPoint::new(json!(1), 10.0),
        ]);

        config.add_series(series);
        assert_eq!(config.total_data_points(), 1);
    }
}
