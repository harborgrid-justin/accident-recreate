//! Utility functions for AccuScene Core
//!
//! This module provides common utility functions used throughout
//! the AccuScene platform.

use crate::error::{AccuSceneError, Result};
use uuid::Uuid;

/// Generate a new UUID v4
pub fn generate_id() -> String {
    Uuid::new_v4().to_string()
}

/// Generate a short ID (first 8 characters of UUID)
pub fn generate_short_id() -> String {
    let uuid = Uuid::new_v4().to_string();
    uuid.chars().take(8).collect()
}

/// Validate a UUID string
pub fn is_valid_uuid(id: &str) -> bool {
    Uuid::parse_str(id).is_ok()
}

/// Clamp a value between min and max
pub fn clamp<T: PartialOrd>(value: T, min: T, max: T) -> T {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

/// Linear interpolation between two values
pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

/// Normalize a value from one range to another
pub fn normalize(value: f64, from_min: f64, from_max: f64, to_min: f64, to_max: f64) -> f64 {
    let normalized = (value - from_min) / (from_max - from_min);
    lerp(to_min, to_max, normalized)
}

/// Check if two floating point numbers are approximately equal
pub fn approx_equal(a: f64, b: f64, epsilon: f64) -> bool {
    (a - b).abs() < epsilon
}

/// Convert degrees to radians
pub fn deg_to_rad(degrees: f64) -> f64 {
    degrees * std::f64::consts::PI / 180.0
}

/// Convert radians to degrees
pub fn rad_to_deg(radians: f64) -> f64 {
    radians * 180.0 / std::f64::consts::PI
}

/// Calculate distance between two 2D points
pub fn distance_2d(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    (dx * dx + dy * dy).sqrt()
}

/// Calculate distance between two 3D points
pub fn distance_3d(x1: f64, y1: f64, z1: f64, x2: f64, y2: f64, z2: f64) -> f64 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let dz = z2 - z1;
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Format a timestamp as ISO 8601 string
pub fn format_timestamp(timestamp: &chrono::DateTime<chrono::Utc>) -> String {
    timestamp.to_rfc3339()
}

/// Parse an ISO 8601 timestamp string
pub fn parse_timestamp(s: &str) -> Result<chrono::DateTime<chrono::Utc>> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .map_err(|e| AccuSceneError::ValidationError {
            message: format!("Invalid timestamp format: {}", e),
            field: Some("timestamp".to_string()),
        })
}

/// Round a float to a specified number of decimal places
pub fn round_to(value: f64, decimals: u32) -> f64 {
    let multiplier = 10_f64.powi(decimals as i32);
    (value * multiplier).round() / multiplier
}

/// Convert meters per second to kilometers per hour
pub fn ms_to_kmh(ms: f64) -> f64 {
    ms * 3.6
}

/// Convert kilometers per hour to meters per second
pub fn kmh_to_ms(kmh: f64) -> f64 {
    kmh / 3.6
}

/// Convert meters per second to miles per hour
pub fn ms_to_mph(ms: f64) -> f64 {
    ms * 2.236936
}

/// Convert miles per hour to meters per second
pub fn mph_to_ms(mph: f64) -> f64 {
    mph / 2.236936
}

/// Calculate kinetic energy (in Joules)
/// KE = 0.5 * m * v²
pub fn kinetic_energy(mass_kg: f64, velocity_ms: f64) -> f64 {
    0.5 * mass_kg * velocity_ms * velocity_ms
}

/// Calculate momentum (in kg⋅m/s)
/// p = m * v
pub fn momentum(mass_kg: f64, velocity_ms: f64) -> f64 {
    mass_kg * velocity_ms
}

/// Calculate force from mass and acceleration (Newton's second law)
/// F = m * a
pub fn force(mass_kg: f64, acceleration_ms2: f64) -> f64 {
    mass_kg * acceleration_ms2
}

/// Calculate deceleration rate from initial velocity, final velocity, and distance
pub fn deceleration_rate(
    initial_velocity_ms: f64,
    final_velocity_ms: f64,
    distance_m: f64,
) -> Result<f64> {
    if distance_m <= 0.0 {
        return Err(AccuSceneError::math("Distance must be positive"));
    }

    // Using: v² = u² + 2as, solving for a: a = (v² - u²) / (2s)
    let v_squared = final_velocity_ms * final_velocity_ms;
    let u_squared = initial_velocity_ms * initial_velocity_ms;
    Ok((v_squared - u_squared) / (2.0 * distance_m))
}

/// Calculate stopping distance given initial velocity and deceleration
pub fn stopping_distance(initial_velocity_ms: f64, deceleration_ms2: f64) -> Result<f64> {
    if deceleration_ms2 >= 0.0 {
        return Err(AccuSceneError::math(
            "Deceleration must be negative for stopping",
        ));
    }

    // Using: v² = u² + 2as, where v = 0, solving for s: s = -u² / (2a)
    Ok(-(initial_velocity_ms * initial_velocity_ms) / (2.0 * deceleration_ms2))
}

/// Sanitize a string for use as a filename
pub fn sanitize_filename(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect()
}

/// Calculate percentage
pub fn percentage(part: f64, whole: f64) -> Result<f64> {
    if whole == 0.0 {
        return Err(AccuSceneError::math("Cannot calculate percentage of zero"));
    }
    Ok((part / whole) * 100.0)
}

/// Calculate angle between two 2D vectors
pub fn angle_between_2d(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    let dot = x1 * x2 + y1 * y2;
    let mag1 = (x1 * x1 + y1 * y1).sqrt();
    let mag2 = (x2 * x2 + y2 * y2).sqrt();

    if mag1 == 0.0 || mag2 == 0.0 {
        return 0.0;
    }

    (dot / (mag1 * mag2)).acos()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_id() {
        let id = generate_id();
        assert_eq!(id.len(), 36); // UUID format: 8-4-4-4-12
        assert!(is_valid_uuid(&id));
    }

    #[test]
    fn test_clamp() {
        assert_eq!(clamp(5, 0, 10), 5);
        assert_eq!(clamp(-5, 0, 10), 0);
        assert_eq!(clamp(15, 0, 10), 10);
    }

    #[test]
    fn test_lerp() {
        assert_eq!(lerp(0.0, 10.0, 0.0), 0.0);
        assert_eq!(lerp(0.0, 10.0, 1.0), 10.0);
        assert_eq!(lerp(0.0, 10.0, 0.5), 5.0);
    }

    #[test]
    fn test_approx_equal() {
        assert!(approx_equal(1.0, 1.0000001, 0.00001));
        assert!(!approx_equal(1.0, 1.1, 0.00001));
    }

    #[test]
    fn test_angle_conversion() {
        let rad = deg_to_rad(180.0);
        assert!(approx_equal(rad, std::f64::consts::PI, 0.00001));

        let deg = rad_to_deg(std::f64::consts::PI);
        assert!(approx_equal(deg, 180.0, 0.00001));
    }

    #[test]
    fn test_distance() {
        let dist = distance_2d(0.0, 0.0, 3.0, 4.0);
        assert_eq!(dist, 5.0);

        let dist3d = distance_3d(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        assert!(approx_equal(dist3d, 1.732050808, 0.00001));
    }

    #[test]
    fn test_velocity_conversion() {
        let kmh = ms_to_kmh(10.0);
        assert_eq!(kmh, 36.0);

        let ms = kmh_to_ms(36.0);
        assert_eq!(ms, 10.0);
    }

    #[test]
    fn test_physics_calculations() {
        let ke = kinetic_energy(1000.0, 20.0);
        assert_eq!(ke, 200000.0);

        let p = momentum(1000.0, 20.0);
        assert_eq!(p, 20000.0);
    }

    #[test]
    fn test_sanitize_filename() {
        let name = sanitize_filename("test/file:name?.txt");
        assert_eq!(name, "test_file_name_.txt");
    }

    #[test]
    fn test_round_to() {
        assert_eq!(round_to(3.14159, 2), 3.14);
        assert_eq!(round_to(3.14159, 4), 3.1416);
    }
}
