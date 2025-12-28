use crate::config::InterpolationMethod;
use crate::data::DataPoint;
use crate::error::{Result, VisualizationError};

/// Interpolate data points using the specified method
pub fn interpolate(
    data: &[DataPoint],
    point_count: usize,
    method: InterpolationMethod,
) -> Result<Vec<DataPoint>> {
    if data.len() < 2 {
        return Err(VisualizationError::InsufficientData {
            expected: 2,
            actual: data.len(),
        });
    }

    match method {
        InterpolationMethod::Linear => linear_interpolate(data, point_count),
        InterpolationMethod::Cubic => cubic_interpolate(data, point_count),
        InterpolationMethod::Spline => spline_interpolate(data, point_count),
        InterpolationMethod::Step => step_interpolate(data, point_count),
        InterpolationMethod::Basis => basis_interpolate(data, point_count),
    }
}

/// Linear interpolation between points
pub fn linear_interpolate(data: &[DataPoint], point_count: usize) -> Result<Vec<DataPoint>> {
    if data.len() < 2 {
        return Err(VisualizationError::InsufficientData {
            expected: 2,
            actual: data.len(),
        });
    }

    let min_x = data.first().unwrap().x;
    let max_x = data.last().unwrap().x;
    let step = (max_x - min_x) / (point_count - 1) as f64;

    let mut result = Vec::with_capacity(point_count);

    for i in 0..point_count {
        let x = min_x + i as f64 * step;
        let y = linear_interpolate_at(data, x)?;
        result.push(DataPoint::new(x, y));
    }

    Ok(result)
}

/// Interpolate y value at a specific x using linear interpolation
fn linear_interpolate_at(data: &[DataPoint], x: f64) -> Result<f64> {
    // Find the two points that bracket x
    let mut i = 0;
    while i < data.len() - 1 && data[i + 1].x < x {
        i += 1;
    }

    if i >= data.len() - 1 {
        return Ok(data[data.len() - 1].y);
    }

    let p0 = data[i];
    let p1 = data[i + 1];

    if p1.x == p0.x {
        return Ok(p0.y);
    }

    let t = (x - p0.x) / (p1.x - p0.x);
    Ok(p0.y + t * (p1.y - p0.y))
}

/// Cubic Hermite interpolation
pub fn cubic_interpolate(data: &[DataPoint], point_count: usize) -> Result<Vec<DataPoint>> {
    if data.len() < 2 {
        return Err(VisualizationError::InsufficientData {
            expected: 2,
            actual: data.len(),
        });
    }

    // Calculate tangents at each point
    let tangents = calculate_tangents(data);

    let min_x = data.first().unwrap().x;
    let max_x = data.last().unwrap().x;
    let step = (max_x - min_x) / (point_count - 1) as f64;

    let mut result = Vec::with_capacity(point_count);

    for i in 0..point_count {
        let x = min_x + i as f64 * step;
        let y = cubic_interpolate_at(data, &tangents, x)?;
        result.push(DataPoint::new(x, y));
    }

    Ok(result)
}

/// Calculate tangents for cubic interpolation using finite differences
fn calculate_tangents(data: &[DataPoint]) -> Vec<f64> {
    let n = data.len();
    let mut tangents = vec![0.0; n];

    // First point
    tangents[0] = (data[1].y - data[0].y) / (data[1].x - data[0].x);

    // Interior points - use central difference
    for i in 1..n - 1 {
        let dx1 = data[i].x - data[i - 1].x;
        let dx2 = data[i + 1].x - data[i].x;
        let dy1 = data[i].y - data[i - 1].y;
        let dy2 = data[i + 1].y - data[i].y;

        tangents[i] = (dy1 / dx1 + dy2 / dx2) / 2.0;
    }

    // Last point
    tangents[n - 1] = (data[n - 1].y - data[n - 2].y) / (data[n - 1].x - data[n - 2].x);

    tangents
}

/// Cubic Hermite interpolation at a specific x
fn cubic_interpolate_at(data: &[DataPoint], tangents: &[f64], x: f64) -> Result<f64> {
    // Find segment
    let mut i = 0;
    while i < data.len() - 1 && data[i + 1].x < x {
        i += 1;
    }

    if i >= data.len() - 1 {
        return Ok(data[data.len() - 1].y);
    }

    let p0 = data[i];
    let p1 = data[i + 1];
    let m0 = tangents[i];
    let m1 = tangents[i + 1];

    let dx = p1.x - p0.x;
    if dx == 0.0 {
        return Ok(p0.y);
    }

    let t = (x - p0.x) / dx;
    let t2 = t * t;
    let t3 = t2 * t;

    // Hermite basis functions
    let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
    let h10 = t3 - 2.0 * t2 + t;
    let h01 = -2.0 * t3 + 3.0 * t2;
    let h11 = t3 - t2;

    Ok(h00 * p0.y + h10 * dx * m0 + h01 * p1.y + h11 * dx * m1)
}

/// Natural cubic spline interpolation
pub fn spline_interpolate(data: &[DataPoint], point_count: usize) -> Result<Vec<DataPoint>> {
    if data.len() < 3 {
        return Err(VisualizationError::InsufficientData {
            expected: 3,
            actual: data.len(),
        });
    }

    let coefficients = calculate_spline_coefficients(data)?;

    let min_x = data.first().unwrap().x;
    let max_x = data.last().unwrap().x;
    let step = (max_x - min_x) / (point_count - 1) as f64;

    let mut result = Vec::with_capacity(point_count);

    for i in 0..point_count {
        let x = min_x + i as f64 * step;
        let y = evaluate_spline(data, &coefficients, x)?;
        result.push(DataPoint::new(x, y));
    }

    Ok(result)
}

/// Calculate natural cubic spline coefficients
fn calculate_spline_coefficients(data: &[DataPoint]) -> Result<Vec<[f64; 4]>> {
    let n = data.len() - 1;
    let mut a = vec![0.0; n + 1];
    let mut b = vec![0.0; n];
    let mut c = vec![0.0; n + 1];
    let mut d = vec![0.0; n];

    for i in 0..=n {
        a[i] = data[i].y;
    }

    let mut h = vec![0.0; n];
    for i in 0..n {
        h[i] = data[i + 1].x - data[i].x;
    }

    let mut alpha = vec![0.0; n];
    for i in 1..n {
        alpha[i] = 3.0 / h[i] * (a[i + 1] - a[i]) - 3.0 / h[i - 1] * (a[i] - a[i - 1]);
    }

    let mut l = vec![0.0; n + 1];
    let mut mu = vec![0.0; n + 1];
    let mut z = vec![0.0; n + 1];

    l[0] = 1.0;
    mu[0] = 0.0;
    z[0] = 0.0;

    for i in 1..n {
        l[i] = 2.0 * (data[i + 1].x - data[i - 1].x) - h[i - 1] * mu[i - 1];
        mu[i] = h[i] / l[i];
        z[i] = (alpha[i] - h[i - 1] * z[i - 1]) / l[i];
    }

    l[n] = 1.0;
    z[n] = 0.0;
    c[n] = 0.0;

    for j in (0..n).rev() {
        c[j] = z[j] - mu[j] * c[j + 1];
        b[j] = (a[j + 1] - a[j]) / h[j] - h[j] * (c[j + 1] + 2.0 * c[j]) / 3.0;
        d[j] = (c[j + 1] - c[j]) / (3.0 * h[j]);
    }

    Ok((0..n)
        .map(|i| [a[i], b[i], c[i], d[i]])
        .collect())
}

/// Evaluate cubic spline at a specific x
fn evaluate_spline(data: &[DataPoint], coefficients: &[[f64; 4]], x: f64) -> Result<f64> {
    let mut i = 0;
    while i < data.len() - 1 && data[i + 1].x < x {
        i += 1;
    }

    if i >= coefficients.len() {
        i = coefficients.len() - 1;
    }

    let dx = x - data[i].x;
    let [a, b, c, d] = coefficients[i];

    Ok(a + b * dx + c * dx * dx + d * dx * dx * dx)
}

/// Step interpolation (piecewise constant)
pub fn step_interpolate(data: &[DataPoint], point_count: usize) -> Result<Vec<DataPoint>> {
    if data.is_empty() {
        return Err(VisualizationError::EmptyDataset);
    }

    let min_x = data.first().unwrap().x;
    let max_x = data.last().unwrap().x;
    let step = (max_x - min_x) / (point_count - 1) as f64;

    let mut result = Vec::with_capacity(point_count);

    for i in 0..point_count {
        let x = min_x + i as f64 * step;

        // Find the point to the left
        let mut idx = 0;
        for (j, point) in data.iter().enumerate() {
            if point.x <= x {
                idx = j;
            } else {
                break;
            }
        }

        result.push(DataPoint::new(x, data[idx].y));
    }

    Ok(result)
}

/// B-spline basis interpolation
pub fn basis_interpolate(data: &[DataPoint], point_count: usize) -> Result<Vec<DataPoint>> {
    if data.len() < 4 {
        return Err(VisualizationError::InsufficientData {
            expected: 4,
            actual: data.len(),
        });
    }

    let min_x = data.first().unwrap().x;
    let max_x = data.last().unwrap().x;
    let step = (max_x - min_x) / (point_count - 1) as f64;

    let mut result = Vec::with_capacity(point_count);

    for i in 0..point_count {
        let x = min_x + i as f64 * step;
        let y = basis_interpolate_at(data, x)?;
        result.push(DataPoint::new(x, y));
    }

    Ok(result)
}

/// B-spline interpolation at a specific x
fn basis_interpolate_at(data: &[DataPoint], x: f64) -> Result<f64> {
    // Find the segment
    let mut i = 0;
    while i < data.len() - 1 && data[i + 1].x < x {
        i += 1;
    }

    if i >= data.len() - 3 {
        i = data.len() - 4;
    }
    if i < 0 {
        i = 0;
    }

    let p0 = if i > 0 { data[i - 1] } else { data[0] };
    let p1 = data[i];
    let p2 = data[i + 1];
    let p3 = if i + 2 < data.len() {
        data[i + 2]
    } else {
        data[data.len() - 1]
    };

    let t = if p2.x != p1.x {
        (x - p1.x) / (p2.x - p1.x)
    } else {
        0.0
    };

    // Cubic B-spline basis functions
    let b0 = (1.0 - t).powi(3) / 6.0;
    let b1 = (3.0 * t.powi(3) - 6.0 * t.powi(2) + 4.0) / 6.0;
    let b2 = (-3.0 * t.powi(3) + 3.0 * t.powi(2) + 3.0 * t + 1.0) / 6.0;
    let b3 = t.powi(3) / 6.0;

    Ok(b0 * p0.y + b1 * p1.y + b2 * p2.y + b3 * p3.y)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_data() -> Vec<DataPoint> {
        vec![
            DataPoint::new(0.0, 0.0),
            DataPoint::new(1.0, 1.0),
            DataPoint::new(2.0, 4.0),
            DataPoint::new(3.0, 9.0),
        ]
    }

    #[test]
    fn test_linear_interpolate() {
        let data = create_test_data();
        let result = linear_interpolate(&data, 10).unwrap();
        assert_eq!(result.len(), 10);
        assert_eq!(result[0].x, 0.0);
        assert_eq!(result[result.len() - 1].x, 3.0);
    }

    #[test]
    fn test_cubic_interpolate() {
        let data = create_test_data();
        let result = cubic_interpolate(&data, 20).unwrap();
        assert_eq!(result.len(), 20);
    }

    #[test]
    fn test_step_interpolate() {
        let data = create_test_data();
        let result = step_interpolate(&data, 15).unwrap();
        assert_eq!(result.len(), 15);
    }
}
