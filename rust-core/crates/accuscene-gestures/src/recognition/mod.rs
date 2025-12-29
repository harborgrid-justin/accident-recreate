pub mod tap;
pub mod swipe;
pub mod pinch;
pub mod rotate;
pub mod pan;
pub mod long_press;
pub mod custom;

use crate::config::GestureConfig;
use crate::events::{GestureEvent, TouchPoint};
use crate::error::GestureResult;
use crate::state::GestureStateMachine;

/// Trait for gesture recognizers
pub trait GestureRecognizer: Send + Sync {
    /// Process touch input and potentially recognize a gesture
    fn process(
        &mut self,
        touches: &[TouchPoint],
        state_machine: &mut GestureStateMachine,
    ) -> GestureResult<Option<GestureEvent>>;

    /// Reset the recognizer state
    fn reset(&mut self);

    /// Get the recognizer name
    fn name(&self) -> &str;

    /// Check if this recognizer can handle the current touch count
    fn can_handle_touch_count(&self, count: usize) -> bool;
}

/// Main gesture recognition engine
pub struct GestureRecognitionEngine {
    config: GestureConfig,
    state_machine: GestureStateMachine,
    recognizers: Vec<Box<dyn GestureRecognizer>>,
}

impl GestureRecognitionEngine {
    pub fn new(config: GestureConfig) -> Self {
        let mut state_machine = GestureStateMachine::new();
        let mut recognizers: Vec<Box<dyn GestureRecognizer>> = Vec::new();

        // Register all recognizers
        let tap_recognizer = tap::TapRecognizer::new(config.tap.clone());
        state_machine.register_gesture("tap".to_string(), config.tap.priority);
        recognizers.push(Box::new(tap_recognizer));

        let swipe_recognizer = swipe::SwipeRecognizer::new(config.swipe.clone());
        state_machine.register_gesture("swipe".to_string(), config.swipe.priority);
        recognizers.push(Box::new(swipe_recognizer));

        let pinch_recognizer = pinch::PinchRecognizer::new(config.pinch.clone());
        state_machine.register_gesture("pinch".to_string(), config.pinch.priority);
        recognizers.push(Box::new(pinch_recognizer));

        let rotate_recognizer = rotate::RotateRecognizer::new(config.rotate.clone());
        state_machine.register_gesture("rotate".to_string(), config.rotate.priority);
        recognizers.push(Box::new(rotate_recognizer));

        let pan_recognizer = pan::PanRecognizer::new(config.pan.clone());
        state_machine.register_gesture("pan".to_string(), config.pan.priority);
        recognizers.push(Box::new(pan_recognizer));

        let long_press_recognizer = long_press::LongPressRecognizer::new(config.long_press.clone());
        state_machine.register_gesture("long_press".to_string(), config.long_press.priority);
        recognizers.push(Box::new(long_press_recognizer));

        Self {
            config,
            state_machine,
            recognizers,
        }
    }

    pub fn process_touch_input(&mut self, touches: &[TouchPoint]) -> GestureResult<Vec<GestureEvent>> {
        let mut events = Vec::new();

        // Track touches
        for touch in touches {
            self.state_machine.add_touch_point(*touch);
        }

        // Process with each recognizer
        for recognizer in &mut self.recognizers {
            if recognizer.can_handle_touch_count(touches.len()) {
                if let Some(event) = recognizer.process(touches, &mut self.state_machine)? {
                    events.push(event);
                }
            }
        }

        // Resolve conflicts if enabled
        if self.config.general.enable_conflict_resolution && self.state_machine.has_conflicts() {
            if let Some(winner) = self.state_machine.resolve_conflicts() {
                // Filter events to only include the winning gesture
                events.retain(|event| self.event_matches_gesture(event, &winner));
            }
        }

        Ok(events)
    }

    fn event_matches_gesture(&self, event: &GestureEvent, gesture_name: &str) -> bool {
        match (gesture_name, event) {
            ("tap", GestureEvent::Tap { .. }) => true,
            ("tap", GestureEvent::DoubleTap { .. }) => true,
            ("tap", GestureEvent::TripleTap { .. }) => true,
            ("swipe", GestureEvent::Swipe { .. }) => true,
            ("pinch", GestureEvent::PinchStart { .. }) => true,
            ("pinch", GestureEvent::PinchMove { .. }) => true,
            ("pinch", GestureEvent::PinchEnd { .. }) => true,
            ("rotate", GestureEvent::RotateStart { .. }) => true,
            ("rotate", GestureEvent::RotateMove { .. }) => true,
            ("rotate", GestureEvent::RotateEnd { .. }) => true,
            ("pan", GestureEvent::PanStart { .. }) => true,
            ("pan", GestureEvent::PanMove { .. }) => true,
            ("pan", GestureEvent::PanEnd { .. }) => true,
            ("long_press", GestureEvent::LongPressStart { .. }) => true,
            ("long_press", GestureEvent::LongPress { .. }) => true,
            ("long_press", GestureEvent::LongPressEnd { .. }) => true,
            _ => false,
        }
    }

    pub fn reset(&mut self) {
        self.state_machine.reset();
        for recognizer in &mut self.recognizers {
            recognizer.reset();
        }
    }

    pub fn get_state_machine(&self) -> &GestureStateMachine {
        &self.state_machine
    }

    pub fn update_config(&mut self, config: GestureConfig) {
        self.config = config;
        // Recreate recognizers with new config
        *self = Self::new(self.config.clone());
    }
}
