use crate::charts::ChartData;
use crate::config::ChartConfig;
use crate::error::{Result, VisualizationError};
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};

/// Export format options
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    SVG,
    PNG,
    JSON,
    CSV,
}

/// Export configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    pub format: ExportFormat,
    pub width: u32,
    pub height: u32,
    pub dpi: u32,
    pub background_color: String,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            format: ExportFormat::SVG,
            width: 1920,
            height: 1080,
            dpi: 96,
            background_color: "#FFFFFF".to_string(),
        }
    }
}

/// Export result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    pub format: ExportFormat,
    pub data: String,
    pub mime_type: String,
}

/// Export chart data to various formats
pub fn export_chart(
    chart: &ChartData,
    config: &ExportConfig,
) -> Result<ExportResult> {
    match config.format {
        ExportFormat::SVG => export_svg(chart, config),
        ExportFormat::PNG => export_png(chart, config),
        ExportFormat::JSON => export_json(chart),
        ExportFormat::CSV => export_csv(chart),
    }
}

/// Export chart as SVG
fn export_svg(chart: &ChartData, config: &ExportConfig) -> Result<ExportResult> {
    let svg = generate_svg(chart, config)?;

    Ok(ExportResult {
        format: ExportFormat::SVG,
        data: svg,
        mime_type: "image/svg+xml".to_string(),
    })
}

/// Generate SVG string from chart data
fn generate_svg(chart: &ChartData, config: &ExportConfig) -> Result<String> {
    let width = config.width;
    let height = config.height;

    let mut svg = String::new();

    // SVG header
    svg.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">"#,
        width, height, width, height
    ));

    // Background
    svg.push_str(&format!(
        r#"<rect width="100%" height="100%" fill="{}" />"#,
        config.background_color
    ));

    // Title
    svg.push_str(&format!(
        r#"<text x="{}" y="30" font-size="24" font-weight="bold" text-anchor="middle">{}</text>"#,
        width / 2,
        chart.title
    ));

    // Define margins
    let margin_top = 60;
    let margin_right = 40;
    let margin_bottom = 60;
    let margin_left = 80;

    let plot_width = width - margin_left - margin_right;
    let plot_height = height - margin_top - margin_bottom;

    // Create plot group
    svg.push_str(&format!(
        r#"<g transform="translate({}, {})">"#,
        margin_left, margin_top
    ));

    // Draw axes
    svg.push_str(&format!(
        r#"<line x1="0" y1="{}" x2="{}" y2="{}" stroke="#333" stroke-width="2" />"#,
        plot_height, plot_width, plot_height
    ));
    svg.push_str(&format!(
        r#"<line x1="0" y1="0" x2="0" y2="{}" stroke="#333" stroke-width="2" />"#,
        plot_height
    ));

    // Draw grid
    if chart.x_axis.grid {
        for i in 0..=10 {
            let x = (plot_width as f64 * i as f64 / 10.0) as u32;
            svg.push_str(&format!(
                r#"<line x1="{}" y1="0" x2="{}" y2="{}" stroke="#E5E7EB" stroke-width="1" opacity="0.5" />"#,
                x, x, plot_height
            ));
        }
    }

    if chart.y_axis.grid {
        for i in 0..=10 {
            let y = (plot_height as f64 * i as f64 / 10.0) as u32;
            svg.push_str(&format!(
                r#"<line x1="0" y1="{}" x2="{}" y2="{}" stroke="#E5E7EB" stroke-width="1" opacity="0.5" />"#,
                y, plot_width, y
            ));
        }
    }

    // Draw series
    let colors = vec!["#3B82F6", "#10B981", "#F59E0B", "#EF4444", "#8B5CF6"];

    for (idx, series) in chart.series.iter().enumerate() {
        let color = series
            .color
            .as_ref()
            .unwrap_or(&colors[idx % colors.len()]);

        if !series.data.is_empty() {
            // Find data bounds
            let x_min = series.data.iter().map(|p| p.x).fold(f64::INFINITY, f64::min);
            let x_max = series
                .data
                .iter()
                .map(|p| p.x)
                .fold(f64::NEG_INFINITY, f64::max);
            let y_min = series.data.iter().map(|p| p.y).fold(f64::INFINITY, f64::min);
            let y_max = series
                .data
                .iter()
                .map(|p| p.y)
                .fold(f64::NEG_INFINITY, f64::max);

            let scale_x = |x: f64| -> u32 {
                ((x - x_min) / (x_max - x_min) * plot_width as f64) as u32
            };
            let scale_y = |y: f64| -> u32 {
                (plot_height as f64 - (y - y_min) / (y_max - y_min) * plot_height as f64) as u32
            };

            match series.chart_type {
                crate::charts::ChartType::Line | crate::charts::ChartType::Area => {
                    let mut path = format!("M {} {}", scale_x(series.data[0].x), scale_y(series.data[0].y));

                    for point in &series.data[1..] {
                        path.push_str(&format!(" L {} {}", scale_x(point.x), scale_y(point.y)));
                    }

                    if series.chart_type == crate::charts::ChartType::Area {
                        path.push_str(&format!(" L {} {}", scale_x(series.data.last().unwrap().x), plot_height));
                        path.push_str(&format!(" L {} {} Z", scale_x(series.data[0].x), plot_height));
                        svg.push_str(&format!(
                            r#"<path d="{}" fill="{}" fill-opacity="0.3" />"#,
                            path, color
                        ));
                    }

                    svg.push_str(&format!(
                        r#"<path d="{}" stroke="{}" stroke-width="2" fill="none" />"#,
                        path.split(" Z").next().unwrap_or(&path),
                        color
                    ));
                }
                crate::charts::ChartType::Bar => {
                    let bar_width = (plot_width as f64 / series.data.len() as f64 * 0.8) as u32;
                    for point in &series.data {
                        let x = scale_x(point.x);
                        let y = scale_y(point.y);
                        let height = plot_height - y;

                        svg.push_str(&format!(
                            r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}" />"#,
                            x.saturating_sub(bar_width / 2),
                            y,
                            bar_width,
                            height,
                            color
                        ));
                    }
                }
                crate::charts::ChartType::Scatter => {
                    for point in &series.data {
                        let x = scale_x(point.x);
                        let y = scale_y(point.y);

                        svg.push_str(&format!(
                            r#"<circle cx="{}" cy="{}" r="4" fill="{}" />"#,
                            x, y, color
                        ));
                    }
                }
                _ => {}
            }
        }
    }

    // Axis labels
    svg.push_str(&format!(
        r#"<text x="{}" y="{}" text-anchor="middle" font-size="14">{}</text>"#,
        plot_width / 2,
        plot_height + 40,
        chart.x_axis.label
    ));

    svg.push_str(&format!(
        r#"<text x="-{}" y="-50" transform="rotate(-90)" text-anchor="middle" font-size="14">{}</text>"#,
        plot_height / 2,
        chart.y_axis.label
    ));

    svg.push_str("</g>");
    svg.push_str("</svg>");

    Ok(svg)
}

/// Export chart as PNG (base64 encoded)
fn export_png(_chart: &ChartData, _config: &ExportConfig) -> Result<ExportResult> {
    // Note: PNG export requires image rendering capabilities
    // In a full implementation, this would use resvg/tiny-skia to render SVG to PNG
    // For now, return an error or placeholder

    #[cfg(feature = "svg-export")]
    {
        // Convert SVG to PNG using resvg
        // Implementation would go here
        Err(VisualizationError::ExportError(
            "PNG export not yet implemented".to_string(),
        ))
    }

    #[cfg(not(feature = "svg-export"))]
    {
        Err(VisualizationError::ExportError(
            "PNG export requires 'svg-export' feature".to_string(),
        ))
    }
}

/// Export chart data as JSON
fn export_json(chart: &ChartData) -> Result<ExportResult> {
    let json = serde_json::to_string_pretty(chart)?;

    Ok(ExportResult {
        format: ExportFormat::JSON,
        data: json,
        mime_type: "application/json".to_string(),
    })
}

/// Export chart data as CSV
fn export_csv(chart: &ChartData) -> Result<ExportResult> {
    let mut csv = String::new();

    // Header
    csv.push_str("series,x,y,label\n");

    // Data rows
    for series in &chart.series {
        for point in &series.data {
            csv.push_str(&format!(
                "{},{},{},{}\n",
                series.name,
                point.x,
                point.y,
                point.label.as_deref().unwrap_or("")
            ));
        }
    }

    Ok(ExportResult {
        format: ExportFormat::CSV,
        data: csv,
        mime_type: "text/csv".to_string(),
    })
}

/// Convert export result to base64
pub fn to_base64(result: &ExportResult) -> String {
    general_purpose::STANDARD.encode(result.data.as_bytes())
}

/// Create data URL from export result
pub fn to_data_url(result: &ExportResult) -> String {
    format!(
        "data:{};base64,{}",
        result.mime_type,
        to_base64(result)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charts::{ChartData, SeriesData, SeriesPoint, ChartType};

    fn create_test_chart() -> ChartData {
        let mut chart = ChartData::new("Test Chart".to_string());
        let points = vec![
            SeriesPoint::new(0.0, 0.0),
            SeriesPoint::new(1.0, 1.0),
            SeriesPoint::new(2.0, 4.0),
        ];
        chart.add_series(SeriesData::new("Test".to_string(), points, ChartType::Line));
        chart
    }

    #[test]
    fn test_export_json() {
        let chart = create_test_chart();
        let result = export_json(&chart);
        assert!(result.is_ok());
    }

    #[test]
    fn test_export_csv() {
        let chart = create_test_chart();
        let result = export_csv(&chart);
        assert!(result.is_ok());
    }

    #[test]
    fn test_export_svg() {
        let chart = create_test_chart();
        let config = ExportConfig::default();
        let result = export_svg(&chart, &config);
        assert!(result.is_ok());
    }
}
