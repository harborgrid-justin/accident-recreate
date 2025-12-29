use crate::config::AggregationMethod;
use crate::error::{Result, VisualizationError};
use crate::data::{DataPoint, HistogramBin, StatisticalSummary, Quartiles};
use statrs::statistics::{Statistics, OrderStatistics};

/// Aggregate data points using the specified method
pub fn aggregate(data: &[f64], method: AggregationMethod) -> Result<f64> {
    if data.is_empty() {
        return Err(VisualizationError::EmptyDataset);
    }

    match method {
        AggregationMethod::Mean => Ok(data.iter().copied().collect::<Vec<f64>>().mean()),
        AggregationMethod::Median => {
            let mut sorted = data.to_vec();
            Ok(sorted.as_mut_slice().median())
        }
        AggregationMethod::Sum => Ok(data.iter().sum()),
        AggregationMethod::Min => {
            data.iter()
                .copied()
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .ok_or(VisualizationError::EmptyDataset)
        }
        AggregationMethod::Max => {
            data.iter()
                .copied()
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .ok_or(VisualizationError::EmptyDataset)
        }
        AggregationMethod::Count => Ok(data.len() as f64),
        AggregationMethod::StdDev => Ok(data.iter().copied().collect::<Vec<f64>>().std_dev()),
        AggregationMethod::Variance => Ok(data.iter().copied().collect::<Vec<f64>>().variance()),
    }
}

/// Rolling window aggregation
pub fn rolling_aggregate(
    data: &[DataPoint],
    window_size: usize,
    method: AggregationMethod,
) -> Result<Vec<DataPoint>> {
    if data.is_empty() {
        return Err(VisualizationError::EmptyDataset);
    }

    if window_size == 0 {
        return Err(VisualizationError::InvalidParameter {
            parameter: "window_size".to_string(),
            value: "0".to_string(),
        });
    }

    let mut result = Vec::with_capacity(data.len());

    for i in 0..data.len() {
        let start = i.saturating_sub(window_size / 2);
        let end = (i + window_size / 2).min(data.len());
        let window: Vec<f64> = data[start..end].iter().map(|p| p.y).collect();

        let aggregated_value = aggregate(&window, method)?;
        result.push(DataPoint::new(data[i].x, aggregated_value));
    }

    Ok(result)
}

/// Group data into time buckets and aggregate
pub fn bucket_aggregate(
    data: &[DataPoint],
    bucket_count: usize,
    method: AggregationMethod,
) -> Result<Vec<DataPoint>> {
    if data.is_empty() {
        return Err(VisualizationError::EmptyDataset);
    }

    if bucket_count == 0 {
        return Err(VisualizationError::InvalidParameter {
            parameter: "bucket_count".to_string(),
            value: "0".to_string(),
        });
    }

    let min_x = data.iter().map(|p| p.x).fold(f64::INFINITY, f64::min);
    let max_x = data.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max);
    let bucket_size = (max_x - min_x) / bucket_count as f64;

    let mut buckets: Vec<Vec<f64>> = vec![Vec::new(); bucket_count];

    for point in data {
        let bucket_idx = ((point.x - min_x) / bucket_size)
            .floor()
            .min((bucket_count - 1) as f64) as usize;
        buckets[bucket_idx].push(point.y);
    }

    let mut result = Vec::with_capacity(bucket_count);
    for (i, bucket) in buckets.iter().enumerate() {
        if !bucket.is_empty() {
            let bucket_center = min_x + (i as f64 + 0.5) * bucket_size;
            let aggregated_value = aggregate(bucket, method)?;
            result.push(DataPoint::new(bucket_center, aggregated_value));
        }
    }

    Ok(result)
}

/// Create a histogram from data
pub fn create_histogram(data: &[f64], bin_count: usize) -> Result<Vec<HistogramBin>> {
    if data.is_empty() {
        return Err(VisualizationError::EmptyDataset);
    }

    if bin_count == 0 {
        return Err(VisualizationError::InvalidParameter {
            parameter: "bin_count".to_string(),
            value: "0".to_string(),
        });
    }

    let min = data.iter().copied().fold(f64::INFINITY, f64::min);
    let max = data.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let bin_width = (max - min) / bin_count as f64;

    let mut bins = vec![0usize; bin_count];

    for &value in data {
        let bin_idx = ((value - min) / bin_width)
            .floor()
            .min((bin_count - 1) as f64) as usize;
        bins[bin_idx] += 1;
    }

    let total_count = data.len() as f64;
    let histogram: Vec<HistogramBin> = bins
        .iter()
        .enumerate()
        .map(|(i, &count)| HistogramBin {
            start: min + i as f64 * bin_width,
            end: min + (i + 1) as f64 * bin_width,
            count,
            frequency: count as f64 / total_count,
        })
        .collect();

    Ok(histogram)
}

/// Calculate statistical summary
pub fn calculate_summary(data: &[f64]) -> Result<StatisticalSummary> {
    if data.is_empty() {
        return Err(VisualizationError::EmptyDataset);
    }

    let mut sorted = data.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let stats_data = sorted.as_slice();
    let count = data.len();
    let mean = data.iter().copied().collect::<Vec<f64>>().mean();
    let median = stats_data.median();
    let std_dev = data.iter().copied().collect::<Vec<f64>>().std_dev();
    let variance = data.iter().copied().collect::<Vec<f64>>().variance();
    let min = stats_data[0];
    let max = stats_data[count - 1];

    let q1 = stats_data.lower_quartile();
    let q2 = median;
    let q3 = stats_data.upper_quartile();

    Ok(StatisticalSummary {
        count,
        mean,
        median,
        std_dev,
        variance,
        min,
        max,
        quartiles: Quartiles { q1, q2, q3 },
    })
}

/// Calculate correlation coefficient between two datasets
pub fn calculate_correlation(x: &[f64], y: &[f64]) -> Result<f64> {
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
    let mean_x = x.iter().sum::<f64>() / n;
    let mean_y = y.iter().sum::<f64>() / n;

    let mut covariance = 0.0;
    let mut var_x = 0.0;
    let mut var_y = 0.0;

    for i in 0..x.len() {
        let dx = x[i] - mean_x;
        let dy = y[i] - mean_y;
        covariance += dx * dy;
        var_x += dx * dx;
        var_y += dy * dy;
    }

    if var_x == 0.0 || var_y == 0.0 {
        return Err(VisualizationError::StatisticalError(
            "Zero variance in one or both datasets".to_string(),
        ));
    }

    Ok(covariance / (var_x * var_y).sqrt())
}

/// Calculate moving average
pub fn moving_average(data: &[DataPoint], window_size: usize) -> Result<Vec<DataPoint>> {
    rolling_aggregate(data, window_size, AggregationMethod::Mean)
}

/// Calculate exponential moving average
pub fn exponential_moving_average(
    data: &[DataPoint],
    alpha: f64,
) -> Result<Vec<DataPoint>> {
    if data.is_empty() {
        return Err(VisualizationError::EmptyDataset);
    }

    if !(0.0..=1.0).contains(&alpha) {
        return Err(VisualizationError::InvalidParameter {
            parameter: "alpha".to_string(),
            value: alpha.to_string(),
        });
    }

    let mut result = Vec::with_capacity(data.len());
    let mut ema = data[0].y;

    for point in data {
        ema = alpha * point.y + (1.0 - alpha) * ema;
        result.push(DataPoint::new(point.x, ema));
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregate_mean() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = aggregate(&data, AggregationMethod::Mean).unwrap();
        assert!((result - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_aggregate_median() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = aggregate(&data, AggregationMethod::Median).unwrap();
        assert!((result - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_correlation() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        let result = calculate_correlation(&x, &y).unwrap();
        assert!((result - 1.0).abs() < 1e-10);
    }
}
