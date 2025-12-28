use crate::charts::{AxisConfig, ChartData, ChartType, SeriesData, SeriesPoint};
use crate::data::aggregation::calculate_correlation;
use crate::data::CorrelationEntry;
use crate::error::{Result, VisualizationError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Correlation matrix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationMatrix {
    pub variables: Vec<String>,
    pub matrix: Vec<Vec<f64>>,
}

impl CorrelationMatrix {
    pub fn from_data(data: &HashMap<String, Vec<f64>>) -> Result<Self> {
        if data.is_empty() {
            return Err(VisualizationError::EmptyDataset);
        }

        let variables: Vec<String> = data.keys().cloned().collect();
        let n = variables.len();
        let mut matrix = vec![vec![0.0; n]; n];

        for (i, var1) in variables.iter().enumerate() {
            for (j, var2) in variables.iter().enumerate() {
                if i == j {
                    matrix[i][j] = 1.0;
                } else {
                    let data1 = &data[var1];
                    let data2 = &data[var2];
                    matrix[i][j] = calculate_correlation(data1, data2)?;
                }
            }
        }

        Ok(CorrelationMatrix { variables, matrix })
    }

    pub fn to_heatmap(&self) -> ChartData {
        let mut chart = ChartData::new("Correlation Matrix".to_string());

        chart.x_axis = AxisConfig {
            label: String::new(),
            min: Some(0.0),
            max: Some(self.variables.len() as f64),
            grid: false,
            tick_format: Some("categorical".to_string()),
        };

        chart.y_axis = AxisConfig {
            label: String::new(),
            min: Some(0.0),
            max: Some(self.variables.len() as f64),
            grid: false,
            tick_format: Some("categorical".to_string()),
        };

        let mut points = Vec::new();

        for (i, row) in self.matrix.iter().enumerate() {
            for (j, &value) in row.iter().enumerate() {
                let mut point = SeriesPoint::new(j as f64, i as f64);
                point.metadata = Some(serde_json::json!({
                    "value": value,
                    "x_label": self.variables[j],
                    "y_label": self.variables[i],
                }));
                points.push(point);
            }
        }

        chart.add_series(SeriesData::new(
            "Correlation".to_string(),
            points,
            ChartType::Heatmap,
        ));

        chart
    }

    pub fn get_correlations(&self, threshold: f64) -> Vec<CorrelationEntry> {
        let mut correlations = Vec::new();

        for (i, var1) in self.variables.iter().enumerate() {
            for (j, var2) in self.variables.iter().enumerate() {
                if i < j && self.matrix[i][j].abs() >= threshold {
                    correlations.push(CorrelationEntry {
                        variable1: var1.clone(),
                        variable2: var2.clone(),
                        correlation: self.matrix[i][j],
                        p_value: None,
                    });
                }
            }
        }

        correlations.sort_by(|a, b| {
            b.correlation
                .abs()
                .partial_cmp(&a.correlation.abs())
                .unwrap()
        });

        correlations
    }
}

/// Scatter plot matrix
#[derive(Debug, Clone)]
pub struct ScatterPlotMatrix {
    pub variables: Vec<String>,
    pub data: HashMap<String, Vec<f64>>,
}

impl ScatterPlotMatrix {
    pub fn new(data: HashMap<String, Vec<f64>>) -> Self {
        let variables: Vec<String> = data.keys().cloned().collect();
        Self { variables, data }
    }

    pub fn generate_plot(&self, var1: &str, var2: &str) -> Result<ChartData> {
        let data1 = self
            .data
            .get(var1)
            .ok_or_else(|| VisualizationError::InvalidData(format!("Variable {} not found", var1)))?;
        let data2 = self
            .data
            .get(var2)
            .ok_or_else(|| VisualizationError::InvalidData(format!("Variable {} not found", var2)))?;

        if data1.len() != data2.len() {
            return Err(VisualizationError::DimensionMismatch {
                expected: data1.len(),
                actual: data2.len(),
            });
        }

        let mut chart = ChartData::new(format!("{} vs {}", var1, var2));

        chart.x_axis = AxisConfig {
            label: var1.to_string(),
            min: None,
            max: None,
            grid: true,
            tick_format: None,
        };

        chart.y_axis = AxisConfig {
            label: var2.to_string(),
            min: None,
            max: None,
            grid: true,
            tick_format: None,
        };

        let points: Vec<SeriesPoint> = data1
            .iter()
            .zip(data2)
            .map(|(&x, &y)| SeriesPoint::new(x, y))
            .collect();

        chart.add_series(SeriesData::new(
            "Data".to_string(),
            points,
            ChartType::Scatter,
        ));

        // Add regression line
        let regression = calculate_linear_regression(data1, data2)?;
        let min_x = data1.iter().copied().fold(f64::INFINITY, f64::min);
        let max_x = data1.iter().copied().fold(f64::NEG_INFINITY, f64::max);

        let regression_points = vec![
            SeriesPoint::new(min_x, regression.predict(min_x)),
            SeriesPoint::new(max_x, regression.predict(max_x)),
        ];

        chart.add_series(
            SeriesData::new("Regression".to_string(), regression_points, ChartType::Line)
                .with_color("#FF6B6B".to_string()),
        );

        Ok(chart)
    }
}

/// Linear regression model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearRegression {
    pub slope: f64,
    pub intercept: f64,
    pub r_squared: f64,
}

impl LinearRegression {
    pub fn predict(&self, x: f64) -> f64 {
        self.slope * x + self.intercept
    }
}

/// Calculate linear regression
pub fn calculate_linear_regression(x: &[f64], y: &[f64]) -> Result<LinearRegression> {
    if x.len() != y.len() {
        return Err(VisualizationError::DimensionMismatch {
            expected: x.len(),
            actual: y.len(),
        });
    }

    if x.is_empty() {
        return Err(VisualizationError::EmptyDataset);
    }

    let n = x.len() as f64;
    let sum_x: f64 = x.iter().sum();
    let sum_y: f64 = y.iter().sum();
    let sum_xy: f64 = x.iter().zip(y).map(|(xi, yi)| xi * yi).sum();
    let sum_x2: f64 = x.iter().map(|xi| xi * xi).sum();
    let sum_y2: f64 = y.iter().map(|yi| yi * yi).sum();

    let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x);
    let intercept = (sum_y - slope * sum_x) / n;

    // Calculate R²
    let mean_y = sum_y / n;
    let ss_tot: f64 = y.iter().map(|yi| (yi - mean_y).powi(2)).sum();
    let ss_res: f64 = x
        .iter()
        .zip(y)
        .map(|(xi, yi)| {
            let pred = slope * xi + intercept;
            (yi - pred).powi(2)
        })
        .sum();

    let r_squared = if ss_tot != 0.0 {
        1.0 - (ss_res / ss_tot)
    } else {
        0.0
    };

    Ok(LinearRegression {
        slope,
        intercept,
        r_squared,
    })
}

/// Polynomial regression
pub fn calculate_polynomial_regression(
    x: &[f64],
    y: &[f64],
    degree: usize,
) -> Result<Vec<f64>> {
    if x.len() != y.len() {
        return Err(VisualizationError::DimensionMismatch {
            expected: x.len(),
            actual: y.len(),
        });
    }

    if x.is_empty() {
        return Err(VisualizationError::EmptyDataset);
    }

    if degree == 0 {
        return Err(VisualizationError::InvalidParameter {
            parameter: "degree".to_string(),
            value: "0".to_string(),
        });
    }

    // Build Vandermonde matrix
    let n = x.len();
    let m = degree + 1;

    let mut matrix = vec![vec![0.0; m]; n];
    for i in 0..n {
        for j in 0..m {
            matrix[i][j] = x[i].powi(j as i32);
        }
    }

    // Solve normal equations using matrix algebra (simplified)
    // In production, use a proper linear algebra library
    // For now, return linear regression coefficients
    let linear = calculate_linear_regression(x, y)?;
    Ok(vec![linear.intercept, linear.slope])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_data() -> HashMap<String, Vec<f64>> {
        let mut data = HashMap::new();
        data.insert("x1".to_string(), (0..100).map(|i| i as f64).collect());
        data.insert(
            "x2".to_string(),
            (0..100).map(|i| i as f64 * 2.0).collect(),
        );
        data.insert(
            "x3".to_string(),
            (0..100).map(|i| (i as f64).sin()).collect(),
        );
        data
    }

    #[test]
    fn test_correlation_matrix() {
        let data = create_test_data();
        let matrix = CorrelationMatrix::from_data(&data).unwrap();
        assert_eq!(matrix.variables.len(), 3);
        assert_eq!(matrix.matrix.len(), 3);
    }

    #[test]
    fn test_linear_regression() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        let regression = calculate_linear_regression(&x, &y).unwrap();
        assert!((regression.slope - 2.0).abs() < 1e-10);
        assert!(regression.intercept.abs() < 1e-10);
    }

    #[test]
    fn test_scatter_plot_matrix() {
        let data = create_test_data();
        let spm = ScatterPlotMatrix::new(data);
        let chart = spm.generate_plot("x1", "x2");
        assert!(chart.is_ok());
    }
}
