//! AccuScene Enterprise Gesture Recognition System
//!
//! A comprehensive gesture recognition library for mobile and touch interfaces.
//! Supports tap, swipe, pinch, rotate, pan, long press, and custom gestures.
//!
//! # Features
//!
//! - Multi-touch gesture recognition
//! - Gesture conflict resolution
//! - Configurable sensitivity and thresholds
//! - Custom gesture pattern matching
//! - State machine for gesture lifecycle management
//! - Velocity and momentum calculations
//!
//! # Example
//!
//! ```rust
//! use accuscene_gestures::{GestureConfig, GestureRecognitionEngine, TouchPoint};
//!
//! let config = GestureConfig::default();
//! let mut engine = GestureRecognitionEngine::new(config);
//!
//! let touch = TouchPoint::new(0, 100.0, 200.0);
//! let events = engine.process_touch_input(&[touch]).unwrap();
//!
//! for event in events {
//!     println!("Gesture detected: {:?}", event);
//! }
//! ```

pub mod config;
pub mod error;
pub mod events;
pub mod recognition;
pub mod state;

// Re-export main types
pub use config::{
    GestureConfig, TapConfig, SwipeConfig, PinchConfig, RotateConfig, PanConfig,
    LongPressConfig, GeneralConfig,
};
pub use error::{GestureError, GestureResult};
pub use events::{
    GestureEvent, TouchPoint, SwipeDirection, GesturePhase, GesturePriority,
};
pub use recognition::{
    GestureRecognizer, GestureRecognitionEngine,
    custom::{CustomGesturePattern, PatternMatcher, ShapeType, Point2D},
};
pub use state::{GestureStateMachine, GestureState, VelocityCalculator};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_touch_point_creation() {
        let point = TouchPoint::new(1, 100.0, 200.0);
        assert_eq!(point.id, 1);
        assert_eq!(point.x, 100.0);
        assert_eq!(point.y, 200.0);
    }

    #[test]
    fn test_touch_point_distance() {
        let p1 = TouchPoint::new(1, 0.0, 0.0);
        let p2 = TouchPoint::new(2, 3.0, 4.0);
        let distance = p1.distance_to(&p2);
        assert_eq!(distance, 5.0);
    }

    #[test]
    fn test_swipe_direction() {
        let angle = 0.0; // 0 degrees = Right
        let direction = SwipeDirection::from_angle(angle);
        assert_eq!(direction, SwipeDirection::Right);

        let angle = std::f64::consts::PI / 2.0; // 90 degrees = Up
        let direction = SwipeDirection::from_angle(angle);
        assert_eq!(direction, SwipeDirection::Up);
    }

    #[test]
    fn test_gesture_config_default() {
        let config = GestureConfig::default();
        assert!(config.tap.enable_double_tap);
        assert!(config.swipe.enable_diagonal);
        assert!(config.general.enable_conflict_resolution);
    }

    #[test]
    fn test_gesture_state_machine() {
        let mut state_machine = GestureStateMachine::new();
        state_machine.register_gesture("test".to_string(), GesturePriority::Normal);

        assert!(!state_machine.is_active("test"));

        state_machine
            .update_state("test", GesturePhase::Began)
            .unwrap();
        assert!(state_machine.is_active("test"));

        state_machine
            .update_state("test", GesturePhase::Ended)
            .unwrap();
        assert!(!state_machine.is_active("test"));
    }

    #[test]
    fn test_velocity_calculator() {
        let mut calc = VelocityCalculator::new(5);
        calc.add_sample(0.0, 0.0, 0);
        calc.add_sample(100.0, 0.0, 1000);

        let (vx, vy) = calc.calculate_velocity();
        assert_eq!(vx, 100.0);
        assert_eq!(vy, 0.0);
    }

    #[test]
    fn test_gesture_recognition_engine() {
        let config = GestureConfig::default();
        let mut engine = GestureRecognitionEngine::new(config);

        let touch = TouchPoint::new(0, 100.0, 200.0);
        let result = engine.process_touch_input(&[touch]);
        assert!(result.is_ok());
    }
}
