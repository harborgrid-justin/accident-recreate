use serde::{Deserialize, Serialize};
use crate::events::GesturePriority;

/// Gesture system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GestureConfig {
    pub tap: TapConfig,
    pub swipe: SwipeConfig,
    pub pinch: PinchConfig,
    pub rotate: RotateConfig,
    pub pan: PanConfig,
    pub long_press: LongPressConfig,
    pub general: GeneralConfig,
}

impl Default for GestureConfig {
    fn default() -> Self {
        Self {
            tap: TapConfig::default(),
            swipe: SwipeConfig::default(),
            pinch: PinchConfig::default(),
            rotate: RotateConfig::default(),
            pan: PanConfig::default(),
            long_press: LongPressConfig::default(),
            general: GeneralConfig::default(),
        }
    }
}

/// Tap gesture configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TapConfig {
    /// Maximum time between tap down and up (ms)
    pub max_duration_ms: i64,
    /// Maximum movement allowed during tap (pixels)
    pub max_movement: f64,
    /// Maximum time between taps for multi-tap (ms)
    pub multi_tap_delay_ms: i64,
    /// Maximum distance between taps for multi-tap (pixels)
    pub multi_tap_distance: f64,
    /// Priority for conflict resolution
    pub priority: GesturePriority,
    /// Enable double tap recognition
    pub enable_double_tap: bool,
    /// Enable triple tap recognition
    pub enable_triple_tap: bool,
}

impl Default for TapConfig {
    fn default() -> Self {
        Self {
            max_duration_ms: 300,
            max_movement: 10.0,
            multi_tap_delay_ms: 300,
            multi_tap_distance: 50.0,
            priority: GesturePriority::High,
            enable_double_tap: true,
            enable_triple_tap: true,
        }
    }
}

/// Swipe gesture configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwipeConfig {
    /// Minimum distance for swipe recognition (pixels)
    pub min_distance: f64,
    /// Minimum velocity for swipe recognition (pixels/second)
    pub min_velocity: f64,
    /// Maximum duration for swipe (ms)
    pub max_duration_ms: i64,
    /// Direction tolerance (degrees)
    pub direction_tolerance: f64,
    /// Priority for conflict resolution
    pub priority: GesturePriority,
    /// Enable diagonal swipes
    pub enable_diagonal: bool,
}

impl Default for SwipeConfig {
    fn default() -> Self {
        Self {
            min_distance: 50.0,
            min_velocity: 100.0,
            max_duration_ms: 1000,
            direction_tolerance: 30.0,
            priority: GesturePriority::Normal,
            enable_diagonal: true,
        }
    }
}

/// Pinch gesture configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinchConfig {
    /// Minimum scale change to recognize pinch
    pub min_scale_delta: f64,
    /// Maximum scale factor
    pub max_scale: f64,
    /// Minimum scale factor
    pub min_scale: f64,
    /// Scale sensitivity
    pub sensitivity: f64,
    /// Priority for conflict resolution
    pub priority: GesturePriority,
    /// Enable simultaneous rotation
    pub allow_simultaneous_rotation: bool,
}

impl Default for PinchConfig {
    fn default() -> Self {
        Self {
            min_scale_delta: 0.01,
            max_scale: 10.0,
            min_scale: 0.1,
            sensitivity: 1.0,
            priority: GesturePriority::High,
            allow_simultaneous_rotation: true,
        }
    }
}

/// Rotate gesture configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotateConfig {
    /// Minimum angle change to recognize rotation (degrees)
    pub min_angle_delta: f64,
    /// Rotation sensitivity
    pub sensitivity: f64,
    /// Priority for conflict resolution
    pub priority: GesturePriority,
    /// Enable simultaneous pinch
    pub allow_simultaneous_pinch: bool,
}

impl Default for RotateConfig {
    fn default() -> Self {
        Self {
            min_angle_delta: 5.0,
            sensitivity: 1.0,
            priority: GesturePriority::Normal,
            allow_simultaneous_pinch: true,
        }
    }
}

/// Pan gesture configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanConfig {
    /// Minimum movement to start pan (pixels)
    pub min_distance: f64,
    /// Maximum fingers for pan
    pub max_touches: usize,
    /// Minimum fingers for pan
    pub min_touches: usize,
    /// Priority for conflict resolution
    pub priority: GesturePriority,
    /// Enable horizontal panning
    pub enable_horizontal: bool,
    /// Enable vertical panning
    pub enable_vertical: bool,
    /// Enable momentum/inertia
    pub enable_momentum: bool,
    /// Momentum decay factor
    pub momentum_decay: f64,
}

impl Default for PanConfig {
    fn default() -> Self {
        Self {
            min_distance: 10.0,
            max_touches: 1,
            min_touches: 1,
            priority: GesturePriority::Normal,
            enable_horizontal: true,
            enable_vertical: true,
            enable_momentum: true,
            momentum_decay: 0.95,
        }
    }
}

/// Long press gesture configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LongPressConfig {
    /// Minimum duration for long press (ms)
    pub min_duration_ms: i64,
    /// Maximum movement during long press (pixels)
    pub max_movement: f64,
    /// Priority for conflict resolution
    pub priority: GesturePriority,
    /// Number of touches required
    pub required_touches: usize,
}

impl Default for LongPressConfig {
    fn default() -> Self {
        Self {
            min_duration_ms: 500,
            max_movement: 10.0,
            priority: GesturePriority::Normal,
            required_touches: 1,
        }
    }
}

/// General gesture system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Enable gesture conflict resolution
    pub enable_conflict_resolution: bool,
    /// Maximum simultaneous gestures
    pub max_simultaneous_gestures: usize,
    /// Touch sample rate (Hz)
    pub sample_rate_hz: f64,
    /// Enable touch prediction
    pub enable_prediction: bool,
    /// Prediction lookahead (ms)
    pub prediction_lookahead_ms: i64,
    /// Enable haptic feedback
    pub enable_haptic_feedback: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            enable_conflict_resolution: true,
            max_simultaneous_gestures: 4,
            sample_rate_hz: 60.0,
            enable_prediction: true,
            prediction_lookahead_ms: 16,
            enable_haptic_feedback: true,
        }
    }
}
