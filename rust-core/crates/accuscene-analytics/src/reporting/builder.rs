//! Fluent report builder

use super::{ReportFormat, ReportMetadata};
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A complete analytics report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub metadata: ReportMetadata,
    pub sections: Vec<ReportSection>,
}

impl Report {
    pub fn builder() -> ReportBuilder {
        ReportBuilder::new()
    }

    /// Add a section to the report
    pub fn add_section(&mut self, section: ReportSection) {
        self.sections.push(section);
    }

    /// Get a section by title
    pub fn get_section(&self, title: &str) -> Option<&ReportSection> {
        self.sections.iter().find(|s| s.title == title)
    }

    /// Get all section titles
    pub fn section_titles(&self) -> Vec<String> {
        self.sections.iter().map(|s| s.title.clone()).collect()
    }
}

/// A section within a report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSection {
    pub title: String,
    pub description: String,
    pub content: SectionContent,
    pub order: usize,
}

/// Content types for report sections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SectionContent {
    Text(String),
    Table(TableData),
    Chart(ChartData),
    Metrics(Vec<MetricData>),
    KeyValue(HashMap<String, String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableData {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

impl TableData {
    pub fn new(headers: Vec<String>) -> Self {
        Self {
            headers,
            rows: Vec::new(),
        }
    }

    pub fn add_row(&mut self, row: Vec<String>) {
        self.rows.push(row);
    }

    pub fn row_count(&self) -> usize {
        self.rows.len()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartData {
    pub chart_type: ChartType,
    pub title: String,
    pub x_label: String,
    pub y_label: String,
    pub series: Vec<ChartSeries>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChartType {
    Line,
    Bar,
    Pie,
    Scatter,
    Area,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartSeries {
    pub name: String,
    pub data: Vec<(f64, f64)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricData {
    pub name: String,
    pub value: f64,
    pub unit: String,
    pub change: Option<f64>,
    pub change_type: Option<ChangeType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    Increase,
    Decrease,
    Stable,
}

/// Fluent builder for creating reports
pub struct ReportBuilder {
    metadata: ReportMetadata,
    sections: Vec<ReportSection>,
}

impl ReportBuilder {
    pub fn new() -> Self {
        Self {
            metadata: ReportMetadata::default(),
            sections: Vec::new(),
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.metadata.title = title.into();
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.metadata.description = description.into();
        self
    }

    pub fn generated_by(mut self, generated_by: impl Into<String>) -> Self {
        self.metadata.generated_by = generated_by.into();
        self
    }

    pub fn tag(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.tags.insert(key.into(), value.into());
        self
    }

    pub fn section(mut self, section: ReportSection) -> Self {
        self.sections.push(section);
        self
    }

    pub fn text_section(
        mut self,
        title: impl Into<String>,
        text: impl Into<String>,
    ) -> Self {
        self.sections.push(ReportSection {
            title: title.into(),
            description: String::new(),
            content: SectionContent::Text(text.into()),
            order: self.sections.len(),
        });
        self
    }

    pub fn table_section(
        mut self,
        title: impl Into<String>,
        table: TableData,
    ) -> Self {
        self.sections.push(ReportSection {
            title: title.into(),
            description: String::new(),
            content: SectionContent::Table(table),
            order: self.sections.len(),
        });
        self
    }

    pub fn chart_section(
        mut self,
        title: impl Into<String>,
        chart: ChartData,
    ) -> Self {
        self.sections.push(ReportSection {
            title: title.into(),
            description: String::new(),
            content: SectionContent::Chart(chart),
            order: self.sections.len(),
        });
        self
    }

    pub fn metrics_section(
        mut self,
        title: impl Into<String>,
        metrics: Vec<MetricData>,
    ) -> Self {
        self.sections.push(ReportSection {
            title: title.into(),
            description: String::new(),
            content: SectionContent::Metrics(metrics),
            order: self.sections.len(),
        });
        self
    }

    pub fn build(self) -> Report {
        Report {
            metadata: self.metadata,
            sections: self.sections,
        }
    }
}

impl Default for ReportBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper for building tables
pub struct TableBuilder {
    data: TableData,
}

impl TableBuilder {
    pub fn new(headers: Vec<String>) -> Self {
        Self {
            data: TableData::new(headers),
        }
    }

    pub fn row(mut self, row: Vec<impl Into<String>>) -> Self {
        let row: Vec<String> = row.into_iter().map(|s| s.into()).collect();
        self.data.add_row(row);
        self
    }

    pub fn build(self) -> TableData {
        self.data
    }
}

/// Helper for building charts
pub struct ChartBuilder {
    chart: ChartData,
}

impl ChartBuilder {
    pub fn new(chart_type: ChartType, title: impl Into<String>) -> Self {
        Self {
            chart: ChartData {
                chart_type,
                title: title.into(),
                x_label: String::new(),
                y_label: String::new(),
                series: Vec::new(),
            },
        }
    }

    pub fn x_label(mut self, label: impl Into<String>) -> Self {
        self.chart.x_label = label.into();
        self
    }

    pub fn y_label(mut self, label: impl Into<String>) -> Self {
        self.chart.y_label = label.into();
        self
    }

    pub fn series(mut self, name: impl Into<String>, data: Vec<(f64, f64)>) -> Self {
        self.chart.series.push(ChartSeries {
            name: name.into(),
            data,
        });
        self
    }

    pub fn build(self) -> ChartData {
        self.chart
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_builder() {
        let report = ReportBuilder::new()
            .title("Test Report")
            .description("A test report")
            .text_section("Introduction", "This is the introduction")
            .build();

        assert_eq!(report.metadata.title, "Test Report");
        assert_eq!(report.sections.len(), 1);
    }

    #[test]
    fn test_table_builder() {
        let table = TableBuilder::new(vec!["Name".to_string(), "Value".to_string()])
            .row(vec!["Item 1", "100"])
            .row(vec!["Item 2", "200"])
            .build();

        assert_eq!(table.headers.len(), 2);
        assert_eq!(table.rows.len(), 2);
    }

    #[test]
    fn test_chart_builder() {
        let chart = ChartBuilder::new(ChartType::Line, "Test Chart")
            .x_label("Time")
            .y_label("Value")
            .series("Series 1", vec![(1.0, 10.0), (2.0, 20.0)])
            .build();

        assert_eq!(chart.chart_type, ChartType::Line);
        assert_eq!(chart.series.len(), 1);
    }
}
