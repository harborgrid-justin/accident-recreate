use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Touch point representation
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TouchPoint {
    pub id: u32,
    pub x: f64,
    pub y: f64,
    pub timestamp: i64,
    pub pressure: f64,
    pub radius_x: f64,
    pub radius_y: f64,
}

impl TouchPoint {
    pub fn new(id: u32, x: f64, y: f64) -> Self {
        Self {
            id,
            x,
            y,
            timestamp: Utc::now().timestamp_millis(),
            pressure: 1.0,
            radius_x: 1.0,
            radius_y: 1.0,
        }
    }

    pub fn distance_to(&self, other: &TouchPoint) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    pub fn angle_to(&self, other: &TouchPoint) -> f64 {
        (other.y - self.y).atan2(other.x - self.x)
    }
}

/// Gesture event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GestureEvent {
    /// Tap gestures
    Tap {
        point: TouchPoint,
        count: u8,
        timestamp: i64,
    },
    DoubleTap {
        point: TouchPoint,
        timestamp: i64,
    },
    TripleTap {
        point: TouchPoint,
        timestamp: i64,
    },

    /// Swipe gestures
    Swipe {
        start: TouchPoint,
        end: TouchPoint,
        direction: SwipeDirection,
        velocity: f64,
        distance: f64,
        duration_ms: i64,
    },

    /// Pinch gestures
    PinchStart {
        center: TouchPoint,
        initial_distance: f64,
        touch1: TouchPoint,
        touch2: TouchPoint,
    },
    PinchMove {
        center: TouchPoint,
        scale: f64,
        distance: f64,
        velocity: f64,
        touch1: TouchPoint,
        touch2: TouchPoint,
    },
    PinchEnd {
        center: TouchPoint,
        final_scale: f64,
        total_scale_change: f64,
    },

    /// Rotation gestures
    RotateStart {
        center: TouchPoint,
        initial_angle: f64,
        touch1: TouchPoint,
        touch2: TouchPoint,
    },
    RotateMove {
        center: TouchPoint,
        angle: f64,
        delta_angle: f64,
        angular_velocity: f64,
        touch1: TouchPoint,
        touch2: TouchPoint,
    },
    RotateEnd {
        center: TouchPoint,
        final_angle: f64,
        total_rotation: f64,
    },

    /// Pan/Drag gestures
    PanStart {
        point: TouchPoint,
    },
    PanMove {
        point: TouchPoint,
        delta_x: f64,
        delta_y: f64,
        velocity_x: f64,
        velocity_y: f64,
        total_delta_x: f64,
        total_delta_y: f64,
    },
    PanEnd {
        point: TouchPoint,
        total_delta_x: f64,
        total_delta_y: f64,
        final_velocity_x: f64,
        final_velocity_y: f64,
    },

    /// Long press gestures
    LongPressStart {
        point: TouchPoint,
    },
    LongPress {
        point: TouchPoint,
        duration_ms: i64,
    },
    LongPressEnd {
        point: TouchPoint,
        total_duration_ms: i64,
    },

    /// Custom gestures
    CustomGesture {
        name: String,
        points: Vec<TouchPoint>,
        confidence: f64,
        metadata: serde_json::Value,
    },

    /// Touch events
    TouchStart {
        touches: Vec<TouchPoint>,
    },
    TouchMove {
        touches: Vec<TouchPoint>,
    },
    TouchEnd {
        touches: Vec<TouchPoint>,
    },
    TouchCancel {
        touches: Vec<TouchPoint>,
    },
}

/// Swipe direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SwipeDirection {
    Up,
    Down,
    Left,
    Right,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}

impl SwipeDirection {
    pub fn from_angle(angle: f64) -> Self {
        let angle_deg = angle.to_degrees();
        let normalized = ((angle_deg + 360.0) % 360.0) as i32;

        match normalized {
            337..=360 | 0..=22 => SwipeDirection::Right,
            23..=67 => SwipeDirection::UpRight,
            68..=112 => SwipeDirection::Up,
            113..=157 => SwipeDirection::UpLeft,
            158..=202 => SwipeDirection::Left,
            203..=247 => SwipeDirection::DownLeft,
            248..=292 => SwipeDirection::Down,
            293..=336 => SwipeDirection::DownRight,
            _ => SwipeDirection::Right,
        }
    }

    pub fn is_horizontal(&self) -> bool {
        matches!(self, SwipeDirection::Left | SwipeDirection::Right)
    }

    pub fn is_vertical(&self) -> bool {
        matches!(self, SwipeDirection::Up | SwipeDirection::Down)
    }

    pub fn is_diagonal(&self) -> bool {
        matches!(
            self,
            SwipeDirection::UpLeft
                | SwipeDirection::UpRight
                | SwipeDirection::DownLeft
                | SwipeDirection::DownRight
        )
    }
}

/// Gesture phase
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GesturePhase {
    Began,
    Changed,
    Ended,
    Cancelled,
    Failed,
}

/// Gesture priority for conflict resolution
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum GesturePriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}
