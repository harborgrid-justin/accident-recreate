use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::events::{TouchPoint, GesturePhase, GesturePriority};
use crate::error::{GestureError, GestureResult};

/// Gesture state machine
#[derive(Debug, Clone)]
pub struct GestureStateMachine {
    states: HashMap<String, GestureState>,
    active_gestures: Vec<String>,
    touch_history: Vec<TouchHistory>,
    max_history_size: usize,
}

impl GestureStateMachine {
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
            active_gestures: Vec::new(),
            touch_history: Vec::new(),
            max_history_size: 100,
        }
    }

    pub fn register_gesture(&mut self, name: String, priority: GesturePriority) {
        let state = GestureState::new(name.clone(), priority);
        self.states.insert(name, state);
    }

    pub fn update_state(
        &mut self,
        gesture_name: &str,
        phase: GesturePhase,
    ) -> GestureResult<()> {
        let state = self.states.get_mut(gesture_name).ok_or_else(|| {
            GestureError::RecognitionFailed(format!("Gesture '{}' not registered", gesture_name))
        })?;

        state.transition(phase)?;

        match phase {
            GesturePhase::Began => {
                if !self.active_gestures.contains(&gesture_name.to_string()) {
                    self.active_gestures.push(gesture_name.to_string());
                }
            }
            GesturePhase::Ended | GesturePhase::Cancelled | GesturePhase::Failed => {
                self.active_gestures.retain(|g| g != gesture_name);
            }
            _ => {}
        }

        Ok(())
    }

    pub fn is_active(&self, gesture_name: &str) -> bool {
        self.active_gestures.contains(&gesture_name.to_string())
    }

    pub fn get_state(&self, gesture_name: &str) -> Option<&GestureState> {
        self.states.get(gesture_name)
    }

    pub fn add_touch_point(&mut self, point: TouchPoint) {
        let history = TouchHistory {
            point,
            timestamp: point.timestamp,
        };
        self.touch_history.push(history);

        // Limit history size
        if self.touch_history.len() > self.max_history_size {
            self.touch_history.remove(0);
        }
    }

    pub fn get_touch_history(&self, since_ms: i64) -> Vec<&TouchHistory> {
        let now = chrono::Utc::now().timestamp_millis();
        self.touch_history
            .iter()
            .filter(|h| now - h.timestamp <= since_ms)
            .collect()
    }

    pub fn clear_touch_history(&mut self) {
        self.touch_history.clear();
    }

    pub fn get_active_gestures(&self) -> &[String] {
        &self.active_gestures
    }

    pub fn has_conflicts(&self) -> bool {
        self.active_gestures.len() > 1
    }

    pub fn resolve_conflicts(&mut self) -> Option<String> {
        if self.active_gestures.len() <= 1 {
            return None;
        }

        // Find highest priority gesture
        let mut highest_priority = GesturePriority::Low;
        let mut winner = None;

        for gesture_name in &self.active_gestures {
            if let Some(state) = self.states.get(gesture_name) {
                if state.priority > highest_priority {
                    highest_priority = state.priority;
                    winner = Some(gesture_name.clone());
                }
            }
        }

        // Cancel lower priority gestures
        if let Some(ref winner_name) = winner {
            let gestures_to_cancel: Vec<String> = self
                .active_gestures
                .iter()
                .filter(|g| g != &winner_name)
                .cloned()
                .collect();

            for gesture_name in gestures_to_cancel {
                let _ = self.update_state(&gesture_name, GesturePhase::Cancelled);
            }
        }

        winner
    }

    pub fn reset(&mut self) {
        for state in self.states.values_mut() {
            state.reset();
        }
        self.active_gestures.clear();
        self.touch_history.clear();
    }
}

impl Default for GestureStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

/// Individual gesture state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GestureState {
    pub name: String,
    pub phase: GesturePhase,
    pub priority: GesturePriority,
    pub start_time: Option<i64>,
    pub last_update: Option<i64>,
    pub transition_count: u32,
}

impl GestureState {
    pub fn new(name: String, priority: GesturePriority) -> Self {
        Self {
            name,
            phase: GesturePhase::Failed,
            priority,
            start_time: None,
            last_update: None,
            transition_count: 0,
        }
    }

    pub fn transition(&mut self, new_phase: GesturePhase) -> GestureResult<()> {
        // Validate state transition
        let valid = match (&self.phase, &new_phase) {
            (GesturePhase::Failed, GesturePhase::Began) => true,
            (GesturePhase::Began, GesturePhase::Changed) => true,
            (GesturePhase::Began, GesturePhase::Ended) => true,
            (GesturePhase::Began, GesturePhase::Cancelled) => true,
            (GesturePhase::Began, GesturePhase::Failed) => true,
            (GesturePhase::Changed, GesturePhase::Changed) => true,
            (GesturePhase::Changed, GesturePhase::Ended) => true,
            (GesturePhase::Changed, GesturePhase::Cancelled) => true,
            (GesturePhase::Changed, GesturePhase::Failed) => true,
            (GesturePhase::Ended, GesturePhase::Began) => true,
            (GesturePhase::Cancelled, GesturePhase::Began) => true,
            (GesturePhase::Failed, GesturePhase::Failed) => true,
            _ => false,
        };

        if !valid {
            return Err(GestureError::InvalidStateTransition {
                from: format!("{:?}", self.phase),
                to: format!("{:?}", new_phase),
            });
        }

        let now = chrono::Utc::now().timestamp_millis();

        if new_phase == GesturePhase::Began {
            self.start_time = Some(now);
        }

        self.phase = new_phase;
        self.last_update = Some(now);
        self.transition_count += 1;

        Ok(())
    }

    pub fn duration_ms(&self) -> Option<i64> {
        self.start_time.map(|start| {
            chrono::Utc::now().timestamp_millis() - start
        })
    }

    pub fn reset(&mut self) {
        self.phase = GesturePhase::Failed;
        self.start_time = None;
        self.last_update = None;
        self.transition_count = 0;
    }

    pub fn is_active(&self) -> bool {
        matches!(self.phase, GesturePhase::Began | GesturePhase::Changed)
    }
}

/// Touch history entry
#[derive(Debug, Clone)]
pub struct TouchHistory {
    pub point: TouchPoint,
    pub timestamp: i64,
}

/// Velocity calculator
#[derive(Debug, Clone)]
pub struct VelocityCalculator {
    samples: Vec<VelocitySample>,
    max_samples: usize,
}

impl VelocityCalculator {
    pub fn new(max_samples: usize) -> Self {
        Self {
            samples: Vec::new(),
            max_samples,
        }
    }

    pub fn add_sample(&mut self, x: f64, y: f64, timestamp: i64) {
        self.samples.push(VelocitySample { x, y, timestamp });
        if self.samples.len() > self.max_samples {
            self.samples.remove(0);
        }
    }

    pub fn calculate_velocity(&self) -> (f64, f64) {
        if self.samples.len() < 2 {
            return (0.0, 0.0);
        }

        let first = &self.samples[0];
        let last = &self.samples[self.samples.len() - 1];

        let dt = (last.timestamp - first.timestamp) as f64 / 1000.0; // Convert to seconds
        if dt == 0.0 {
            return (0.0, 0.0);
        }

        let dx = last.x - first.x;
        let dy = last.y - first.y;

        (dx / dt, dy / dt)
    }

    pub fn get_speed(&self) -> f64 {
        let (vx, vy) = self.calculate_velocity();
        (vx * vx + vy * vy).sqrt()
    }

    pub fn reset(&mut self) {
        self.samples.clear();
    }
}

impl Default for VelocityCalculator {
    fn default() -> Self {
        Self::new(5)
    }
}

#[derive(Debug, Clone)]
struct VelocitySample {
    x: f64,
    y: f64,
    timestamp: i64,
}
