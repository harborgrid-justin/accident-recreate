use crate::charts::{AxisConfig, ChartData, ChartType, SeriesData, SeriesPoint};
use crate::data::{HistogramBin, StatisticalSummary};
use crate::data::aggregation::{calculate_summary, create_histogram};
use crate::error::{Result, VisualizationError};
use serde::{Deserialize, Serialize};
use statrs::distribution::{ContinuousCDF, Normal};

/// Distribution chart generator
#[derive(Debug, Clone)]
pub struct DistributionChart {
    data: Vec<f64>,
    title: String,
    bin_count: usize,
}

impl DistributionChart {
    pub fn new(title: String, data: Vec<f64>) -> Self {
        Self {
            data,
            title,
            bin_count: 20,
        }
    }

    pub fn with_bin_count(mut self, bin_count: usize) -> Self {
        self.bin_count = bin_count;
        self
    }

    pub fn generate_histogram(&self) -> Result<ChartData> {
        if self.data.is_empty() {
            return Err(VisualizationError::EmptyDataset);
        }

        let histogram = create_histogram(&self.data, self.bin_count)?;

        let mut chart = ChartData::new(self.title.clone());

        chart.x_axis = AxisConfig {
            label: "Value".to_string(),
            min: Some(histogram.first().unwrap().start),
            max: Some(histogram.last().unwrap().end),
            grid: true,
            tick_format: None,
        };

        chart.y_axis = AxisConfig {
            label: "Frequency".to_string(),
            min: Some(0.0),
            max: None,
            grid: true,
            tick_format: None,
        };

        let points: Vec<SeriesPoint> = histogram
            .iter()
            .map(|bin| {
                let x = (bin.start + bin.end) / 2.0;
                SeriesPoint::new(x, bin.count as f64)
                    .with_label(format!("{:.2} - {:.2}", bin.start, bin.end))
            })
            .collect();

        chart.add_series(SeriesData::new(
            "Histogram".to_string(),
            points,
            ChartType::Bar,
        ));

        Ok(chart)
    }

    pub fn generate_density(&self) -> Result<ChartData> {
        if self.data.is_empty() {
            return Err(VisualizationError::EmptyDataset);
        }

        let summary = calculate_summary(&self.data)?;
        let kde_points = estimate_kernel_density(&self.data, 100)?;

        let mut chart = ChartData::new(format!("{} - Density", self.title));

        chart.x_axis = AxisConfig {
            label: "Value".to_string(),
            min: Some(summary.min),
            max: Some(summary.max),
            grid: true,
            tick_format: None,
        };

        chart.y_axis = AxisConfig {
            label: "Density".to_string(),
            min: Some(0.0),
            max: None,
            grid: true,
            tick_format: None,
        };

        let points: Vec<SeriesPoint> = kde_points
            .iter()
            .map(|&(x, y)| SeriesPoint::new(x, y))
            .collect();

        chart.add_series(SeriesData::new(
            "Density".to_string(),
            points,
            ChartType::Area,
        ));

        Ok(chart)
    }
}

/// Box plot data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoxPlotData {
    pub min: f64,
    pub q1: f64,
    pub median: f64,
    pub q3: f64,
    pub max: f64,
    pub outliers: Vec<f64>,
}

impl BoxPlotData {
    pub fn from_data(data: &[f64]) -> Result<Self> {
        let summary = calculate_summary(data)?;

        // Calculate IQR and outliers
        let iqr = summary.quartiles.q3 - summary.quartiles.q1;
        let lower_fence = summary.quartiles.q1 - 1.5 * iqr;
        let upper_fence = summary.quartiles.q3 + 1.5 * iqr;

        let outliers: Vec<f64> = data
            .iter()
            .copied()
            .filter(|&x| x < lower_fence || x > upper_fence)
            .collect();

        // Find min/max excluding outliers
        let mut non_outliers: Vec<f64> = data
            .iter()
            .copied()
            .filter(|&x| x >= lower_fence && x <= upper_fence)
            .collect();
        non_outliers.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let min = non_outliers.first().copied().unwrap_or(summary.min);
        let max = non_outliers.last().copied().unwrap_or(summary.max);

        Ok(BoxPlotData {
            min,
            q1: summary.quartiles.q1,
            median: summary.quartiles.q2,
            q3: summary.quartiles.q3,
            max,
            outliers,
        })
    }
}

/// Estimate probability density using kernel density estimation
fn estimate_kernel_density(data: &[f64], num_points: usize) -> Result<Vec<(f64, f64)>> {
    if data.is_empty() {
        return Err(VisualizationError::EmptyDataset);
    }

    let n = data.len() as f64;
    let min = data.iter().copied().fold(f64::INFINITY, f64::min);
    let max = data.iter().copied().fold(f64::NEG_INFINITY, f64::max);

    // Silverman's rule of thumb for bandwidth
    let std_dev = {
        let mean = data.iter().sum::<f64>() / n;
        let variance = data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
        variance.sqrt()
    };
    let bandwidth = 1.06 * std_dev * n.powf(-0.2);

    let step = (max - min) / (num_points - 1) as f64;
    let mut density = Vec::with_capacity(num_points);

    for i in 0..num_points {
        let x = min + i as f64 * step;
        let mut sum = 0.0;

        for &xi in data {
            let z = (x - xi) / bandwidth;
            // Gaussian kernel
            sum += (-0.5 * z * z).exp();
        }

        let density_value = sum / (n * bandwidth * (2.0 * std::f64::consts::PI).sqrt());
        density.push((x, density_value));
    }

    Ok(density)
}

/// Q-Q plot data for normality testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QQPlotData {
    pub theoretical: Vec<f64>,
    pub sample: Vec<f64>,
}

impl QQPlotData {
    pub fn from_data(data: &[f64]) -> Result<Self> {
        if data.is_empty() {
            return Err(VisualizationError::EmptyDataset);
        }

        let mut sorted = data.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let n = sorted.len();
        let mean = data.iter().sum::<f64>() / n as f64;
        let variance = data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n as f64;
        let std_dev = variance.sqrt();

        let normal = Normal::new(0.0, 1.0)
            .map_err(|e| VisualizationError::StatisticalError(e.to_string()))?;

        let mut theoretical = Vec::with_capacity(n);
        let mut sample = Vec::with_capacity(n);

        for (i, &value) in sorted.iter().enumerate() {
            // Calculate theoretical quantile
            let p = (i as f64 + 0.5) / n as f64;
            let z = normal.inverse_cdf(p);

            theoretical.push(z);
            sample.push((value - mean) / std_dev);
        }

        Ok(QQPlotData {
            theoretical,
            sample,
        })
    }

    pub fn to_chart_data(&self) -> ChartData {
        let mut chart = ChartData::new("Q-Q Plot".to_string());

        chart.x_axis = AxisConfig {
            label: "Theoretical Quantiles".to_string(),
            min: None,
            max: None,
            grid: true,
            tick_format: None,
        };

        chart.y_axis = AxisConfig {
            label: "Sample Quantiles".to_string(),
            min: None,
            max: None,
            grid: true,
            tick_format: None,
        };

        let points: Vec<SeriesPoint> = self
            .theoretical
            .iter()
            .zip(&self.sample)
            .map(|(&x, &y)| SeriesPoint::new(x, y))
            .collect();

        chart.add_series(SeriesData::new(
            "Q-Q".to_string(),
            points,
            ChartType::Scatter,
        ));

        // Add reference line
        if let (Some(&min_x), Some(&max_x)) =
            (self.theoretical.first(), self.theoretical.last())
        {
            let reference = vec![
                SeriesPoint::new(min_x, min_x),
                SeriesPoint::new(max_x, max_x),
            ];

            chart.add_series(
                SeriesData::new("Reference".to_string(), reference, ChartType::Line)
                    .with_color("#999999".to_string()),
            );
        }

        chart
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_data() -> Vec<f64> {
        (0..1000)
            .map(|i| (i as f64 * 0.01).sin() + rand::random::<f64>() * 0.1)
            .collect()
    }

    #[test]
    fn test_histogram_generation() {
        let data = create_test_data();
        let chart = DistributionChart::new("Test".to_string(), data);
        let result = chart.generate_histogram();
        assert!(result.is_ok());
    }

    #[test]
    fn test_box_plot_data() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 100.0]; // 100.0 is an outlier
        let box_plot = BoxPlotData::from_data(&data).unwrap();
        assert!(!box_plot.outliers.is_empty());
    }

    #[test]
    fn test_qq_plot() {
        let data = create_test_data();
        let qq = QQPlotData::from_data(&data);
        assert!(qq.is_ok());
    }
}
