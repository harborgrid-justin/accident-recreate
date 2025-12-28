use crate::config::GeneralConfig;
use crate::events::{GestureEvent, GesturePhase, TouchPoint};
use crate::error::{GestureError, GestureResult};
use crate::state::GestureStateMachine;
use super::GestureRecognizer;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Custom gesture pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomGesturePattern {
    pub name: String,
    pub min_points: usize,
    pub max_points: usize,
    pub min_duration_ms: i64,
    pub max_duration_ms: i64,
    pub pattern_matcher: PatternMatcher,
    pub metadata: serde_json::Value,
}

/// Pattern matching strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PatternMatcher {
    /// Match based on point sequence
    PointSequence {
        points: Vec<Point2D>,
        tolerance: f64,
    },
    /// Match based on drawing shape
    ShapeRecognition {
        shape_type: ShapeType,
        tolerance: f64,
    },
    /// Match based on touch count and timing
    TouchPattern {
        touch_counts: Vec<usize>,
        timing_windows_ms: Vec<i64>,
    },
    /// Custom function-based matching
    Custom {
        algorithm: String,
        parameters: HashMap<String, f64>,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

impl Point2D {
    pub fn from_touch(touch: &TouchPoint) -> Self {
        Self {
            x: touch.x,
            y: touch.y,
        }
    }

    pub fn distance_to(&self, other: &Point2D) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ShapeType {
    Circle,
    Triangle,
    Square,
    Rectangle,
    Star,
    Heart,
    Line,
    Zigzag,
    Custom,
}

/// Custom gesture recognizer
#[derive(Debug)]
pub struct CustomGestureRecognizer {
    patterns: HashMap<String, CustomGesturePattern>,
    touch_trail: Vec<TouchPoint>,
    start_time: Option<i64>,
    is_active: bool,
    max_trail_points: usize,
}

impl CustomGestureRecognizer {
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
            touch_trail: Vec::new(),
            start_time: None,
            is_active: false,
            max_trail_points: 1000,
        }
    }

    pub fn register_pattern(&mut self, pattern: CustomGesturePattern) {
        self.patterns.insert(pattern.name.clone(), pattern);
    }

    pub fn unregister_pattern(&mut self, name: &str) {
        self.patterns.remove(name);
    }

    fn match_pattern(&self, pattern: &CustomGesturePattern) -> Option<f64> {
        // Check point count
        if self.touch_trail.len() < pattern.min_points
            || self.touch_trail.len() > pattern.max_points
        {
            return None;
        }

        // Check duration
        if let Some(start) = self.start_time {
            let duration = chrono::Utc::now().timestamp_millis() - start;
            if duration < pattern.min_duration_ms || duration > pattern.max_duration_ms {
                return None;
            }
        }

        // Match based on pattern type
        match &pattern.pattern_matcher {
            PatternMatcher::PointSequence { points, tolerance } => {
                self.match_point_sequence(points, *tolerance)
            }
            PatternMatcher::ShapeRecognition { shape_type, tolerance } => {
                self.match_shape(shape_type, *tolerance)
            }
            PatternMatcher::TouchPattern {
                touch_counts,
                timing_windows_ms,
            } => self.match_touch_pattern(touch_counts, timing_windows_ms),
            PatternMatcher::Custom { algorithm, parameters } => {
                self.match_custom(algorithm, parameters)
            }
        }
    }

    fn match_point_sequence(&self, pattern_points: &[Point2D], tolerance: f64) -> Option<f64> {
        if self.touch_trail.len() < pattern_points.len() {
            return None;
        }

        // Normalize both trails to same scale
        let trail_points: Vec<Point2D> = self
            .touch_trail
            .iter()
            .map(Point2D::from_touch)
            .collect();

        let normalized_trail = Self::normalize_points(&trail_points);
        let normalized_pattern = Self::normalize_points(pattern_points);

        // Calculate matching score using Dynamic Time Warping
        let score = Self::dtw_distance(&normalized_trail, &normalized_pattern);

        if score <= tolerance {
            Some(1.0 - (score / tolerance))
        } else {
            None
        }
    }

    fn match_shape(&self, shape_type: &ShapeType, tolerance: f64) -> Option<f64> {
        let trail_points: Vec<Point2D> = self
            .touch_trail
            .iter()
            .map(Point2D::from_touch)
            .collect();

        match shape_type {
            ShapeType::Circle => self.match_circle(&trail_points, tolerance),
            ShapeType::Triangle => self.match_triangle(&trail_points, tolerance),
            ShapeType::Square | ShapeType::Rectangle => {
                self.match_rectangle(&trail_points, tolerance)
            }
            ShapeType::Line => self.match_line(&trail_points, tolerance),
            _ => None, // Other shapes not implemented
        }
    }

    fn match_touch_pattern(
        &self,
        _touch_counts: &[usize],
        _timing_windows_ms: &[i64],
    ) -> Option<f64> {
        // Simplified implementation
        Some(0.8)
    }

    fn match_custom(
        &self,
        _algorithm: &str,
        _parameters: &HashMap<String, f64>,
    ) -> Option<f64> {
        // Placeholder for custom matching algorithms
        None
    }

    // Helper methods for shape matching
    fn match_circle(&self, points: &[Point2D], tolerance: f64) -> Option<f64> {
        if points.len() < 10 {
            return None;
        }

        // Calculate centroid
        let (cx, cy) = Self::calculate_centroid(points);

        // Calculate average radius and variance
        let radii: Vec<f64> = points
            .iter()
            .map(|p| ((p.x - cx).powi(2) + (p.y - cy).powi(2)).sqrt())
            .collect();

        let avg_radius = radii.iter().sum::<f64>() / radii.len() as f64;
        let variance = radii
            .iter()
            .map(|r| (r - avg_radius).powi(2))
            .sum::<f64>()
            / radii.len() as f64;

        let std_dev = variance.sqrt();
        let circularity = 1.0 - (std_dev / avg_radius);

        if circularity >= 1.0 - tolerance {
            Some(circularity)
        } else {
            None
        }
    }

    fn match_triangle(&self, points: &[Point2D], tolerance: f64) -> Option<f64> {
        // Simplified triangle detection
        if points.len() < 4 {
            return None;
        }

        // Find corners (points with high curvature)
        let corners = Self::find_corners(points, 3);

        if corners.len() == 3 {
            Some(1.0 - tolerance)
        } else {
            None
        }
    }

    fn match_rectangle(&self, points: &[Point2D], tolerance: f64) -> Option<f64> {
        if points.len() < 4 {
            return None;
        }

        let corners = Self::find_corners(points, 4);

        if corners.len() == 4 {
            Some(1.0 - tolerance)
        } else {
            None
        }
    }

    fn match_line(&self, points: &[Point2D], tolerance: f64) -> Option<f64> {
        if points.len() < 2 {
            return None;
        }

        // Calculate linearity using least squares
        let linearity = Self::calculate_linearity(points);

        if linearity >= 1.0 - tolerance {
            Some(linearity)
        } else {
            None
        }
    }

    // Utility methods
    fn normalize_points(points: &[Point2D]) -> Vec<Point2D> {
        if points.is_empty() {
            return Vec::new();
        }

        let (cx, cy) = Self::calculate_centroid(points);
        let scale = Self::calculate_scale(points, cx, cy);

        points
            .iter()
            .map(|p| Point2D {
                x: (p.x - cx) / scale,
                y: (p.y - cy) / scale,
            })
            .collect()
    }

    fn calculate_centroid(points: &[Point2D]) -> (f64, f64) {
        let sum_x: f64 = points.iter().map(|p| p.x).sum();
        let sum_y: f64 = points.iter().map(|p| p.y).sum();
        let n = points.len() as f64;
        (sum_x / n, sum_y / n)
    }

    fn calculate_scale(points: &[Point2D], cx: f64, cy: f64) -> f64 {
        let max_dist = points
            .iter()
            .map(|p| ((p.x - cx).powi(2) + (p.y - cy).powi(2)).sqrt())
            .fold(0.0f64, f64::max);

        if max_dist > 0.0 {
            max_dist
        } else {
            1.0
        }
    }

    fn dtw_distance(a: &[Point2D], b: &[Point2D]) -> f64 {
        let n = a.len();
        let m = b.len();

        if n == 0 || m == 0 {
            return f64::INFINITY;
        }

        let mut dtw = vec![vec![f64::INFINITY; m + 1]; n + 1];
        dtw[0][0] = 0.0;

        for i in 1..=n {
            for j in 1..=m {
                let cost = a[i - 1].distance_to(&b[j - 1]);
                dtw[i][j] = cost + dtw[i - 1][j].min(dtw[i][j - 1]).min(dtw[i - 1][j - 1]);
            }
        }

        dtw[n][m]
    }

    fn find_corners(points: &[Point2D], expected: usize) -> Vec<usize> {
        // Simplified corner detection
        let mut corners = Vec::new();
        let window_size = points.len() / (expected * 2).max(1);

        for i in window_size..(points.len() - window_size) {
            let curvature = Self::calculate_curvature(points, i, window_size);
            if curvature > 0.5 {
                corners.push(i);
            }
        }

        corners
    }

    fn calculate_curvature(points: &[Point2D], index: usize, window: usize) -> f64 {
        if index < window || index + window >= points.len() {
            return 0.0;
        }

        let p1 = &points[index - window];
        let p2 = &points[index];
        let p3 = &points[index + window];

        let v1x = p2.x - p1.x;
        let v1y = p2.y - p1.y;
        let v2x = p3.x - p2.x;
        let v2y = p3.y - p2.y;

        let dot = v1x * v2x + v1y * v2y;
        let mag1 = (v1x * v1x + v1y * v1y).sqrt();
        let mag2 = (v2x * v2x + v2y * v2y).sqrt();

        if mag1 == 0.0 || mag2 == 0.0 {
            return 0.0;
        }

        let cos_angle = dot / (mag1 * mag2);
        1.0 - cos_angle
    }

    fn calculate_linearity(points: &[Point2D]) -> f64 {
        if points.len() < 2 {
            return 0.0;
        }

        // Calculate best-fit line using least squares
        let n = points.len() as f64;
        let sum_x: f64 = points.iter().map(|p| p.x).sum();
        let sum_y: f64 = points.iter().map(|p| p.y).sum();
        let sum_xx: f64 = points.iter().map(|p| p.x * p.x).sum();
        let sum_xy: f64 = points.iter().map(|p| p.x * p.y).sum();

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_xx - sum_x * sum_x);
        let intercept = (sum_y - slope * sum_x) / n;

        // Calculate R-squared
        let mean_y = sum_y / n;
        let ss_tot: f64 = points.iter().map(|p| (p.y - mean_y).powi(2)).sum();
        let ss_res: f64 = points
            .iter()
            .map(|p| (p.y - (slope * p.x + intercept)).powi(2))
            .sum();

        if ss_tot == 0.0 {
            return 0.0;
        }

        1.0 - (ss_res / ss_tot)
    }
}

impl GestureRecognizer for CustomGestureRecognizer {
    fn process(
        &mut self,
        touches: &[TouchPoint],
        state_machine: &mut GestureStateMachine,
    ) -> GestureResult<Option<GestureEvent>> {
        match touches.len() {
            0 => {
                // Touch ended - try to match patterns
                if self.is_active && !self.touch_trail.is_empty() {
                    let mut best_match: Option<(String, f64)> = None;

                    for (name, pattern) in &self.patterns {
                        if let Some(confidence) = self.match_pattern(pattern) {
                            if best_match.is_none() || confidence > best_match.as_ref().unwrap().1
                            {
                                best_match = Some((name.clone(), confidence));
                            }
                        }
                    }

                    if let Some((name, confidence)) = best_match {
                        let pattern = self.patterns.get(&name).unwrap();
                        let event = GestureEvent::CustomGesture {
                            name,
                            points: self.touch_trail.clone(),
                            confidence,
                            metadata: pattern.metadata.clone(),
                        };

                        self.reset();
                        return Ok(Some(event));
                    }
                }

                self.reset();
                Ok(None)
            }
            _ => {
                // Add touch points to trail
                for touch in touches {
                    if !self.is_active {
                        self.start_time = Some(chrono::Utc::now().timestamp_millis());
                        self.is_active = true;
                    }

                    self.touch_trail.push(*touch);

                    // Limit trail size
                    if self.touch_trail.len() > self.max_trail_points {
                        self.touch_trail.remove(0);
                    }
                }

                Ok(None)
            }
        }
    }

    fn reset(&mut self) {
        self.touch_trail.clear();
        self.start_time = None;
        self.is_active = false;
    }

    fn name(&self) -> &str {
        "custom"
    }

    fn can_handle_touch_count(&self, _count: usize) -> bool {
        true // Custom gestures can handle any touch count
    }
}

impl Default for CustomGestureRecognizer {
    fn default() -> Self {
        Self::new()
    }
}
