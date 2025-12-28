//! AccuScene Analytics Engine v0.2.0
//!
//! A comprehensive analytics engine for real-time accident reconstruction data analysis.
//!
//! # Features
//!
//! - **Metrics Framework**: Counters, gauges, histograms, and time series
//! - **Aggregations**: Temporal, spatial, and multi-dimensional aggregations
//! - **Statistics**: Descriptive stats, regression, correlation, distribution fitting
//! - **Windowing**: Sliding, tumbling, session, and hopping windows
//! - **Anomaly Detection**: Z-score, IQR, moving average, isolation forest, density-based
//! - **Forecasting**: Moving average, exponential smoothing, Holt's, AR, seasonal decomposition
//! - **Reporting**: Fluent report builder with JSON, CSV, HTML export
//! - **Domain Analytics**: Collision, vehicle, case, and performance analytics
//!
//! # Example
//!
//! ```no_run
//! use accuscene_analytics::{AnalyticsEngine, AnalyticsConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = AnalyticsConfig::default();
//!     let engine = AnalyticsEngine::new(config);
//!
//!     engine.start().await?;
//!
//!     // Use the engine...
//!
//!     engine.stop().await?;
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod aggregation;
pub mod anomaly;
pub mod config;
pub mod domain;
pub mod engine;
pub mod error;
pub mod forecasting;
pub mod metrics;
pub mod query;
pub mod reporting;
pub mod statistics;
pub mod storage;
pub mod windowing;

// Re-export main types
pub use aggregation::{
    AggregationOp, AggregationResult, DimensionalAggregator, Dimension, DimensionValue,
    SpatialAggregator, SpatialGrid, SpatialPoint, TemporalAggregator, TemporalBucket,
};

pub use anomaly::{
    Anomaly, DensityDetector, IQRDetector, IsolationForestDetector, MovingAverageDetector,
    ZScoreDetector,
};

pub use config::{AggregationConfig, AnalyticsConfig, StorageConfig, TemporalInterval};

pub use domain::{
    CaseAnalytics, CollisionAnalytics, PerformanceAnalytics, VehicleAnalytics,
};

pub use engine::{AnalyticsEngine, AnalyticsEvent, EventProcessor, MetricsRegistry};

pub use error::{AnalyticsError, Result};

pub use forecasting::{
    ARForecaster, ExponentialSmoothingForecaster, Forecast, HoltForecaster,
    MovingAverageForecaster, SeasonalForecaster, SeasonalMethod,
};

pub use metrics::{
    Counter, Gauge, Histogram, Metric, MetricMetadata, RateCounter, TimeSeries, TimeSeriesPoint,
};

pub use query::{
    Aggregation, Filter, FilterOperator, FilterValue, OrderBy, OrderDirection, Query,
    QueryExecutor, QueryMetadata, QueryResult, ResultRow, TimeRange,
};

pub use reporting::{
    CsvExporter, HtmlExporter, JsonExporter, Report, ReportBuilder, ReportExporter, ReportFormat,
    ReportMetadata, ReportSection,
};

pub use statistics::{
    AutocorrelationAnalyzer, CorrelationAnalyzer, CorrelationType, DescriptiveStats,
    DistributionFitter, DistributionType, LinearRegression, NormalDistribution,
    PolynomialRegression, RegressionResult, Statistics,
};

pub use storage::{AnalyticsStorage, StorageEntry, StorageMetadata, TimeSeriesStorage};

pub use windowing::{
    HoppingWindow, Session, SessionWindow, SlidingWindow, TumblingWindow,
};

/// Version of the analytics engine
pub const VERSION: &str = "0.2.0";

/// Get the analytics engine version
pub fn version() -> &'static str {
    VERSION
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(version(), "0.2.0");
    }
}
