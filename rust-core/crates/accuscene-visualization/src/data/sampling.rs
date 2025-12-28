use crate::config::SamplingStrategy;
use crate::data::DataPoint;
use crate::error::{Result, VisualizationError};
use rand::seq::SliceRandom;
use rand::thread_rng;

/// Sample data points using the specified strategy
pub fn sample_data(
    data: &[DataPoint],
    max_points: usize,
    strategy: SamplingStrategy,
) -> Result<Vec<DataPoint>> {
    if data.is_empty() {
        return Err(VisualizationError::EmptyDataset);
    }

    if data.len() <= max_points {
        return Ok(data.to_vec());
    }

    match strategy {
        SamplingStrategy::LTTB => lttb_downsample(data, max_points),
        SamplingStrategy::Random => random_sample(data, max_points),
        SamplingStrategy::Systematic => systematic_sample(data, max_points),
        SamplingStrategy::MinMax => minmax_sample(data, max_points),
    }
}

/// Largest Triangle Three Buckets (LTTB) downsampling algorithm
/// Preserves visual characteristics while reducing data points
pub fn lttb_downsample(data: &[DataPoint], threshold: usize) -> Result<Vec<DataPoint>> {
    if threshold >= data.len() || threshold < 3 {
        return Ok(data.to_vec());
    }

    let mut sampled = Vec::with_capacity(threshold);

    // Always include first point
    sampled.push(data[0]);

    let bucket_size = (data.len() - 2) as f64 / (threshold - 2) as f64;

    let mut a = 0;

    for i in 0..(threshold - 2) {
        // Calculate point average for next bucket
        let avg_range_start = ((i + 1) as f64 * bucket_size).floor() as usize + 1;
        let avg_range_end = ((i + 2) as f64 * bucket_size).floor() as usize + 1;
        let avg_range_end = avg_range_end.min(data.len());

        let avg_x: f64 = data[avg_range_start..avg_range_end]
            .iter()
            .map(|p| p.x)
            .sum::<f64>()
            / (avg_range_end - avg_range_start) as f64;
        let avg_y: f64 = data[avg_range_start..avg_range_end]
            .iter()
            .map(|p| p.y)
            .sum::<f64>()
            / (avg_range_end - avg_range_start) as f64;

        // Get the range for this bucket
        let range_start = (i as f64 * bucket_size).floor() as usize + 1;
        let range_end = ((i + 1) as f64 * bucket_size).floor() as usize + 1;

        // Point a
        let point_a = data[a];

        let mut max_area = -1.0;
        let mut max_area_point = range_start;

        for (idx, point) in data[range_start..range_end].iter().enumerate() {
            // Calculate triangle area over three buckets
            let area = ((point_a.x - avg_x) * (point.y - point_a.y)
                - (point_a.x - point.x) * (avg_y - point_a.y))
                .abs()
                * 0.5;

            if area > max_area {
                max_area = area;
                max_area_point = range_start + idx;
            }
        }

        sampled.push(data[max_area_point]);
        a = max_area_point;
    }

    // Always include last point
    sampled.push(data[data.len() - 1]);

    Ok(sampled)
}

/// Random sampling
pub fn random_sample(data: &[DataPoint], sample_size: usize) -> Result<Vec<DataPoint>> {
    if sample_size >= data.len() {
        return Ok(data.to_vec());
    }

    let mut rng = thread_rng();
    let mut indices: Vec<usize> = (0..data.len()).collect();
    indices.shuffle(&mut rng);

    let mut sampled: Vec<DataPoint> = indices[..sample_size]
        .iter()
        .map(|&i| data[i])
        .collect();

    // Sort by x coordinate to maintain order
    sampled.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());

    Ok(sampled)
}

/// Systematic sampling (every nth point)
pub fn systematic_sample(data: &[DataPoint], sample_size: usize) -> Result<Vec<DataPoint>> {
    if sample_size >= data.len() {
        return Ok(data.to_vec());
    }

    let step = data.len() as f64 / sample_size as f64;
    let sampled: Vec<DataPoint> = (0..sample_size)
        .map(|i| {
            let idx = (i as f64 * step).floor() as usize;
            data[idx.min(data.len() - 1)]
        })
        .collect();

    Ok(sampled)
}

/// Min-Max sampling - preserves peaks and valleys
pub fn minmax_sample(data: &[DataPoint], sample_size: usize) -> Result<Vec<DataPoint>> {
    if sample_size >= data.len() {
        return Ok(data.to_vec());
    }

    let bucket_count = sample_size / 2;
    let bucket_size = data.len() / bucket_count;

    let mut sampled = Vec::with_capacity(sample_size);

    for i in 0..bucket_count {
        let start = i * bucket_size;
        let end = ((i + 1) * bucket_size).min(data.len());

        let bucket = &data[start..end];
        if bucket.is_empty() {
            continue;
        }

        // Find min and max in bucket
        let min_point = bucket
            .iter()
            .min_by(|a, b| a.y.partial_cmp(&b.y).unwrap())
            .unwrap();
        let max_point = bucket
            .iter()
            .max_by(|a, b| a.y.partial_cmp(&b.y).unwrap())
            .unwrap();

        // Add in order of appearance
        if min_point.x < max_point.x {
            sampled.push(*min_point);
            sampled.push(*max_point);
        } else {
            sampled.push(*max_point);
            sampled.push(*min_point);
        }
    }

    Ok(sampled)
}

/// Adaptive sampling based on local variation
pub fn adaptive_sample(data: &[DataPoint], max_points: usize) -> Result<Vec<DataPoint>> {
    if data.len() <= max_points {
        return Ok(data.to_vec());
    }

    let mut sampled = Vec::with_capacity(max_points);
    sampled.push(data[0]);

    let mut importance = calculate_importance(data);
    let threshold = find_threshold(&importance, max_points);

    for (i, point) in data.iter().enumerate().skip(1).take(data.len() - 2) {
        if importance[i] > threshold {
            sampled.push(*point);
        }
    }

    sampled.push(data[data.len() - 1]);

    // If we don't have enough points, add more based on importance
    if sampled.len() < max_points {
        let needed = max_points - sampled.len();
        let mut remaining: Vec<_> = data
            .iter()
            .enumerate()
            .filter(|(i, _)| !sampled.iter().any(|s| s.x == data[*i].x))
            .collect();

        remaining.sort_by(|a, b| {
            importance[b.0]
                .partial_cmp(&importance[a.0])
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        for (_, point) in remaining.iter().take(needed) {
            sampled.push(**point);
        }
    }

    sampled.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
    Ok(sampled)
}

/// Calculate importance score for each point based on local variation
fn calculate_importance(data: &[DataPoint]) -> Vec<f64> {
    let mut importance = vec![0.0; data.len()];

    for i in 1..data.len() - 1 {
        // Calculate angle between adjacent points
        let dx1 = data[i].x - data[i - 1].x;
        let dy1 = data[i].y - data[i - 1].y;
        let dx2 = data[i + 1].x - data[i].x;
        let dy2 = data[i + 1].y - data[i].y;

        let angle = ((dy2 / dx2) - (dy1 / dx1)).abs();
        importance[i] = angle;
    }

    importance[0] = f64::INFINITY;
    importance[data.len() - 1] = f64::INFINITY;

    importance
}

/// Find importance threshold to achieve target point count
fn find_threshold(importance: &[f64], target_count: usize) -> f64 {
    let mut sorted = importance.to_vec();
    sorted.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));

    if target_count >= sorted.len() {
        return 0.0;
    }

    sorted[target_count]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_data() -> Vec<DataPoint> {
        (0..100)
            .map(|i| DataPoint::new(i as f64, (i as f64 * 0.1).sin()))
            .collect()
    }

    #[test]
    fn test_lttb_downsample() {
        let data = create_test_data();
        let result = lttb_downsample(&data, 20).unwrap();
        assert_eq!(result.len(), 20);
        assert_eq!(result[0], data[0]);
        assert_eq!(result[result.len() - 1], data[data.len() - 1]);
    }

    #[test]
    fn test_systematic_sample() {
        let data = create_test_data();
        let result = systematic_sample(&data, 10).unwrap();
        assert_eq!(result.len(), 10);
    }

    #[test]
    fn test_minmax_sample() {
        let data = create_test_data();
        let result = minmax_sample(&data, 20).unwrap();
        assert_eq!(result.len(), 20);
    }
}
