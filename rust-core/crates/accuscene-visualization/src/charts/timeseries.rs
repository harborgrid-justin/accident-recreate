use crate::charts::{AxisConfig, ChartData, ChartType, SeriesData, SeriesPoint};
use crate::data::{DataPoint, TimeSeriesPoint};
use crate::error::{Result, VisualizationError};
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};

/// Time series chart generator
#[derive(Debug, Clone)]
pub struct TimeSeriesChart {
    data: Vec<TimeSeriesData>,
    title: String,
}

impl TimeSeriesChart {
    pub fn new(title: String) -> Self {
        Self {
            data: Vec::new(),
            title,
        }
    }

    pub fn add_series(&mut self, name: String, points: Vec<TimeSeriesPoint>) {
        self.data.push(TimeSeriesData { name, points });
    }

    pub fn generate(&self) -> Result<ChartData> {
        if self.data.is_empty() {
            return Err(VisualizationError::EmptyDataset);
        }

        let mut chart = ChartData::new(self.title.clone());

        // Configure axes
        let (min_time, max_time) = self.get_time_range()?;
        chart.x_axis = AxisConfig {
            label: "Time".to_string(),
            min: Some(min_time as f64),
            max: Some(max_time as f64),
            grid: true,
            tick_format: Some("datetime".to_string()),
        };

        chart.y_axis = AxisConfig {
            label: "Value".to_string(),
            min: None,
            max: None,
            grid: true,
            tick_format: None,
        };

        // Add series
        for ts_data in &self.data {
            let points: Vec<SeriesPoint> = ts_data
                .points
                .iter()
                .map(|p| {
                    SeriesPoint::new(p.timestamp as f64, p.value).with_label(
                        format_timestamp(p.timestamp),
                    )
                })
                .collect();

            chart.add_series(SeriesData::new(
                ts_data.name.clone(),
                points,
                ChartType::Line,
            ));
        }

        Ok(chart)
    }

    fn get_time_range(&self) -> Result<(i64, i64)> {
        let mut min_time = i64::MAX;
        let mut max_time = i64::MIN;

        for series in &self.data {
            for point in &series.points {
                min_time = min_time.min(point.timestamp);
                max_time = max_time.max(point.timestamp);
            }
        }

        if min_time == i64::MAX {
            return Err(VisualizationError::EmptyDataset);
        }

        Ok((min_time, max_time))
    }
}

#[derive(Debug, Clone)]
struct TimeSeriesData {
    name: String,
    points: Vec<TimeSeriesPoint>,
}

/// Format timestamp for display
fn format_timestamp(timestamp: i64) -> String {
    if let Some(dt) = NaiveDateTime::from_timestamp_opt(timestamp, 0) {
        DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string()
    } else {
        timestamp.to_string()
    }
}

/// Time series analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesAnalysis {
    pub trend: Vec<DataPoint>,
    pub seasonal: Vec<DataPoint>,
    pub residual: Vec<DataPoint>,
    pub statistics: TimeSeriesStats,
}

/// Time series statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesStats {
    pub mean: f64,
    pub std_dev: f64,
    pub autocorrelation: Vec<f64>,
    pub trend_slope: f64,
}

/// Decompose time series into trend, seasonal, and residual components
pub fn decompose_time_series(data: &[TimeSeriesPoint]) -> Result<TimeSeriesAnalysis> {
    if data.len() < 4 {
        return Err(VisualizationError::InsufficientData {
            expected: 4,
            actual: data.len(),
        });
    }

    let values: Vec<f64> = data.iter().map(|p| p.value).collect();

    // Calculate trend using moving average
    let trend = calculate_trend(&values)?;

    // Detrend data
    let detrended: Vec<f64> = values
        .iter()
        .zip(&trend)
        .map(|(v, t)| v - t)
        .collect();

    // Extract seasonal component (simplified)
    let seasonal = calculate_seasonal(&detrended)?;

    // Calculate residual
    let residual: Vec<f64> = detrended
        .iter()
        .zip(&seasonal)
        .map(|(d, s)| d - s)
        .collect();

    // Convert to DataPoints
    let trend_points: Vec<DataPoint> = data
        .iter()
        .zip(&trend)
        .map(|(p, &t)| DataPoint::new(p.timestamp as f64, t))
        .collect();

    let seasonal_points: Vec<DataPoint> = data
        .iter()
        .zip(&seasonal)
        .map(|(p, &s)| DataPoint::new(p.timestamp as f64, s))
        .collect();

    let residual_points: Vec<DataPoint> = data
        .iter()
        .zip(&residual)
        .map(|(p, &r)| DataPoint::new(p.timestamp as f64, r))
        .collect();

    // Calculate statistics
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>()
        / values.len() as f64;
    let std_dev = variance.sqrt();

    let autocorr = calculate_autocorrelation(&values, 10);
    let trend_slope = calculate_trend_slope(&trend_points);

    Ok(TimeSeriesAnalysis {
        trend: trend_points,
        seasonal: seasonal_points,
        residual: residual_points,
        statistics: TimeSeriesStats {
            mean,
            std_dev,
            autocorrelation: autocorr,
            trend_slope,
        },
    })
}

/// Calculate trend using simple moving average
fn calculate_trend(values: &[f64]) -> Result<Vec<f64>> {
    let window_size = (values.len() / 4).max(3);
    let mut trend = Vec::with_capacity(values.len());

    for i in 0..values.len() {
        let start = i.saturating_sub(window_size / 2);
        let end = (i + window_size / 2).min(values.len());
        let window_mean = values[start..end].iter().sum::<f64>()
            / (end - start) as f64;
        trend.push(window_mean);
    }

    Ok(trend)
}

/// Calculate seasonal component (simplified)
fn calculate_seasonal(detrended: &[f64]) -> Result<Vec<f64>> {
    // For simplicity, return zeros
    // In a real implementation, this would detect and extract periodicity
    Ok(vec![0.0; detrended.len()])
}

/// Calculate autocorrelation at different lags
fn calculate_autocorrelation(values: &[f64], max_lag: usize) -> Vec<f64> {
    let n = values.len();
    let mean = values.iter().sum::<f64>() / n as f64;
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>();

    let mut autocorr = Vec::with_capacity(max_lag);

    for lag in 1..=max_lag {
        if lag >= n {
            autocorr.push(0.0);
            continue;
        }

        let mut sum = 0.0;
        for i in 0..n - lag {
            sum += (values[i] - mean) * (values[i + lag] - mean);
        }

        autocorr.push(sum / variance);
    }

    autocorr
}

/// Calculate trend slope using linear regression
fn calculate_trend_slope(trend: &[DataPoint]) -> f64 {
    let n = trend.len() as f64;
    let sum_x: f64 = trend.iter().map(|p| p.x).sum();
    let sum_y: f64 = trend.iter().map(|p| p.y).sum();
    let sum_xy: f64 = trend.iter().map(|p| p.x * p.y).sum();
    let sum_x2: f64 = trend.iter().map(|p| p.x * p.x).sum();

    (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_time_series() -> Vec<TimeSeriesPoint> {
        (0..100)
            .map(|i| {
                let value = (i as f64 * 0.1).sin() + i as f64 * 0.01;
                TimeSeriesPoint::new(i * 3600, value)
            })
            .collect()
    }

    #[test]
    fn test_time_series_chart() {
        let mut chart = TimeSeriesChart::new("Test Chart".to_string());
        chart.add_series("Series 1".to_string(), create_test_time_series());
        let result = chart.generate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_decompose_time_series() {
        let data = create_test_time_series();
        let result = decompose_time_series(&data);
        assert!(result.is_ok());
    }
}
