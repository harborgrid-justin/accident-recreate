//! Metrics widget implementation
//!
//! Displays key performance indicators and metrics

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};

use super::{Widget, WidgetConfig, WidgetData, InteractionEvent};
use crate::error::{WidgetError, WidgetResult};

/// Metric display format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricFormat {
    /// Raw number
    Number,
    /// Percentage (0-100)
    Percentage,
    /// Currency
    Currency,
    /// Duration (seconds)
    Duration,
    /// Bytes (data size)
    Bytes,
}

/// Metric trend direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrendDirection {
    /// Increasing
    Up,
    /// Decreasing
    Down,
    /// Stable
    Stable,
}

/// Metric value with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricValue {
    /// Current value
    pub value: f64,

    /// Previous value for comparison
    pub previous_value: Option<f64>,

    /// Format type
    pub format: MetricFormat,

    /// Trend direction
    pub trend: Option<TrendDirection>,

    /// Change percentage
    pub change_percent: Option<f64>,

    /// Label for the metric
    pub label: String,

    /// Unit of measurement
    pub unit: Option<String>,

    /// Target value (for goal tracking)
    pub target: Option<f64>,

    /// Color indicator (hex)
    pub color: Option<String>,
}

impl MetricValue {
    /// Create a new metric value
    pub fn new(label: String, value: f64, format: MetricFormat) -> Self {
        Self {
            value,
            previous_value: None,
            format,
            trend: None,
            change_percent: None,
            label,
            unit: None,
            target: None,
            color: None,
        }
    }

    /// Calculate trend and change percentage
    pub fn calculate_trend(&mut self) {
        if let Some(prev) = self.previous_value {
            if prev == 0.0 {
                self.trend = if self.value > 0.0 {
                    Some(TrendDirection::Up)
                } else if self.value < 0.0 {
                    Some(TrendDirection::Down)
                } else {
                    Some(TrendDirection::Stable)
                };
                return;
            }

            let change = ((self.value - prev) / prev.abs()) * 100.0;
            self.change_percent = Some(change);

            self.trend = if change > 0.1 {
                Some(TrendDirection::Up)
            } else if change < -0.1 {
                Some(TrendDirection::Down)
            } else {
                Some(TrendDirection::Stable)
            };
        }
    }

    /// Format value as string
    pub fn format_value(&self) -> String {
        match self.format {
            MetricFormat::Number => {
                if self.value.abs() >= 1_000_000.0 {
                    format!("{:.2}M", self.value / 1_000_000.0)
                } else if self.value.abs() >= 1_000.0 {
                    format!("{:.2}K", self.value / 1_000.0)
                } else {
                    format!("{:.2}", self.value)
                }
            }
            MetricFormat::Percentage => format!("{:.1}%", self.value),
            MetricFormat::Currency => format!("${:.2}", self.value),
            MetricFormat::Duration => {
                if self.value >= 3600.0 {
                    format!("{:.1}h", self.value / 3600.0)
                } else if self.value >= 60.0 {
                    format!("{:.1}m", self.value / 60.0)
                } else {
                    format!("{:.0}s", self.value)
                }
            }
            MetricFormat::Bytes => {
                if self.value >= 1_073_741_824.0 {
                    format!("{:.2} GB", self.value / 1_073_741_824.0)
                } else if self.value >= 1_048_576.0 {
                    format!("{:.2} MB", self.value / 1_048_576.0)
                } else if self.value >= 1024.0 {
                    format!("{:.2} KB", self.value / 1024.0)
                } else {
                    format!("{:.0} B", self.value)
                }
            }
        }
    }

    /// Check if target is met
    pub fn is_target_met(&self) -> Option<bool> {
        self.target.map(|t| self.value >= t)
    }
}

/// Metrics widget configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Metric definitions
    pub metrics: Vec<MetricValue>,

    /// Layout mode
    pub layout: MetricsLayout,

    /// Show trend indicators
    pub show_trend: bool,

    /// Show sparklines
    pub show_sparkline: bool,

    /// Comparison period label
    pub comparison_period: Option<String>,
}

/// Metrics layout mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricsLayout {
    /// Single metric (large display)
    Single,
    /// Grid of metrics
    Grid,
    /// Horizontal list
    Horizontal,
    /// Vertical list
    Vertical,
}

/// Metrics widget implementation
pub struct MetricsWidget {
    config: WidgetConfig,
    data: Option<WidgetData>,
    metrics_config: MetricsConfig,
}

impl MetricsWidget {
    /// Create a new metrics widget
    pub fn new(config: WidgetConfig, metrics_config: MetricsConfig) -> Self {
        Self {
            config,
            data: None,
            metrics_config,
        }
    }

    /// Update metric values
    pub fn update_metrics(&mut self, metrics: Vec<MetricValue>) {
        self.metrics_config.metrics = metrics;
    }

    /// Get metric by label
    pub fn get_metric(&self, label: &str) -> Option<&MetricValue> {
        self.metrics_config.metrics.iter().find(|m| m.label == label)
    }

    /// Parse metrics from data
    fn parse_metrics_from_data(&self, data: &JsonValue) -> WidgetResult<Vec<MetricValue>> {
        let metrics_array = data.as_array()
            .ok_or_else(|| WidgetError::data_error("Expected array of metrics"))?;

        let mut metrics = Vec::new();
        for metric_data in metrics_array {
            let label = metric_data["label"]
                .as_str()
                .ok_or_else(|| WidgetError::missing_field("label"))?
                .to_string();

            let value = metric_data["value"]
                .as_f64()
                .ok_or_else(|| WidgetError::missing_field("value"))?;

            let format_str = metric_data["format"]
                .as_str()
                .unwrap_or("number");

            let format = match format_str {
                "percentage" => MetricFormat::Percentage,
                "currency" => MetricFormat::Currency,
                "duration" => MetricFormat::Duration,
                "bytes" => MetricFormat::Bytes,
                _ => MetricFormat::Number,
            };

            let mut metric = MetricValue::new(label, value, format);

            if let Some(prev) = metric_data["previous_value"].as_f64() {
                metric.previous_value = Some(prev);
                metric.calculate_trend();
            }

            if let Some(target) = metric_data["target"].as_f64() {
                metric.target = Some(target);
            }

            if let Some(unit) = metric_data["unit"].as_str() {
                metric.unit = Some(unit.to_string());
            }

            if let Some(color) = metric_data["color"].as_str() {
                metric.color = Some(color.to_string());
            }

            metrics.push(metric);
        }

        Ok(metrics)
    }
}

#[async_trait]
impl Widget for MetricsWidget {
    fn config(&self) -> &WidgetConfig {
        &self.config
    }

    fn data(&self) -> Option<&WidgetData> {
        self.data.as_ref()
    }

    async fn fetch_data(&mut self) -> WidgetResult<WidgetData> {
        // In a real implementation, this would fetch from the data source
        // For now, we'll convert the current metrics to data

        let metrics_json: Vec<JsonValue> = self.metrics_config.metrics
            .iter()
            .map(|m| json!({
                "label": m.label,
                "value": m.value,
                "formatted_value": m.format_value(),
                "previous_value": m.previous_value,
                "format": match m.format {
                    MetricFormat::Number => "number",
                    MetricFormat::Percentage => "percentage",
                    MetricFormat::Currency => "currency",
                    MetricFormat::Duration => "duration",
                    MetricFormat::Bytes => "bytes",
                },
                "trend": m.trend.as_ref().map(|t| match t {
                    TrendDirection::Up => "up",
                    TrendDirection::Down => "down",
                    TrendDirection::Stable => "stable",
                }),
                "change_percent": m.change_percent,
                "target": m.target,
                "target_met": m.is_target_met(),
                "unit": m.unit,
                "color": m.color,
            }))
            .collect();

        let data = WidgetData::new(json!({
            "metrics": metrics_json,
            "layout": match self.metrics_config.layout {
                MetricsLayout::Single => "single",
                MetricsLayout::Grid => "grid",
                MetricsLayout::Horizontal => "horizontal",
                MetricsLayout::Vertical => "vertical",
            },
            "show_trend": self.metrics_config.show_trend,
            "show_sparkline": self.metrics_config.show_sparkline,
            "comparison_period": self.metrics_config.comparison_period,
        }));

        self.data = Some(data.clone());
        Ok(data)
    }

    fn validate(&self) -> WidgetResult<()> {
        if self.metrics_config.metrics.is_empty() {
            return Err(WidgetError::invalid_config("At least one metric is required"));
        }

        for metric in &self.metrics_config.metrics {
            if metric.label.is_empty() {
                return Err(WidgetError::invalid_config("Metric label cannot be empty"));
            }
        }

        Ok(())
    }

    async fn handle_interaction(&mut self, event: InteractionEvent) -> WidgetResult<()> {
        match event.event_type.as_str() {
            "refresh" => {
                self.fetch_data().await?;
                Ok(())
            }
            "drill_down" => {
                // Handle drill-down interaction
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::{WidgetMetadata, WidgetType, DisplayOptions, InteractionOptions};

    #[test]
    fn test_metric_value_formatting() {
        let mut metric = MetricValue::new("Test".to_string(), 1234.56, MetricFormat::Number);
        assert_eq!(metric.format_value(), "1.23K");

        metric.format = MetricFormat::Currency;
        assert_eq!(metric.format_value(), "$1234.56");

        metric.format = MetricFormat::Percentage;
        metric.value = 75.5;
        assert_eq!(metric.format_value(), "75.5%");
    }

    #[test]
    fn test_trend_calculation() {
        let mut metric = MetricValue::new("Test".to_string(), 150.0, MetricFormat::Number);
        metric.previous_value = Some(100.0);
        metric.calculate_trend();

        assert_eq!(metric.trend, Some(TrendDirection::Up));
        assert_eq!(metric.change_percent, Some(50.0));
    }

    #[test]
    fn test_target_check() {
        let mut metric = MetricValue::new("Test".to_string(), 80.0, MetricFormat::Number);
        metric.target = Some(100.0);
        assert_eq!(metric.is_target_met(), Some(false));

        metric.value = 110.0;
        assert_eq!(metric.is_target_met(), Some(true));
    }
}
