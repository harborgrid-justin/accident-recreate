//! AccuScene Visualization Library
//!
//! Advanced data visualization and chart generation for AccuScene Enterprise.
//!
//! This library provides:
//! - Data aggregation and statistical analysis
//! - Intelligent data sampling for large datasets
//! - Multiple interpolation methods
//! - Time series analysis and visualization
//! - Distribution and correlation analysis
//! - Chart data generation for frontend rendering
//! - Export capabilities (SVG, PNG, JSON, CSV)

pub mod charts;
pub mod config;
pub mod data;
pub mod error;
pub mod export;

pub use charts::{
    ChartData, ChartType, SeriesData, SeriesPoint, AxisConfig,
    correlation::{CorrelationMatrix, LinearRegression, ScatterPlotMatrix},
    distribution::{BoxPlotData, DistributionChart, QQPlotData},
    timeseries::{TimeSeriesAnalysis, TimeSeriesChart, TimeSeriesStats},
};

pub use config::{
    AggregationConfig, AggregationMethod, AnimationConfig, ChartConfig, ColorScheme,
    FontConfig, InterpolationConfig, InterpolationMethod, Margin, SamplingConfig,
    SamplingStrategy,
};

pub use data::{
    DataPoint, DataSeries, LineStyle, MultiDimPoint, TimeSeriesPoint, StatisticalSummary,
    Quartiles, HistogramBin, CorrelationEntry,
    aggregation::{
        aggregate, bucket_aggregate, calculate_correlation, calculate_summary,
        create_histogram, exponential_moving_average, moving_average, rolling_aggregate,
    },
    interpolation::{
        basis_interpolate, cubic_interpolate, interpolate, linear_interpolate,
        spline_interpolate, step_interpolate,
    },
    sampling::{
        adaptive_sample, lttb_downsample, minmax_sample, random_sample, sample_data,
        systematic_sample,
    },
};

pub use error::{Result, VisualizationError};

pub use export::{
    export_chart, to_base64, to_data_url, ExportConfig, ExportFormat, ExportResult,
};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the visualization library
pub fn init() {
    tracing::debug!("AccuScene Visualization Library v{} initialized", VERSION);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_init() {
        init();
    }
}
