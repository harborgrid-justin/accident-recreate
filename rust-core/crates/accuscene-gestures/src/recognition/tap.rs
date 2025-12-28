use crate::config::TapConfig;
use crate::events::{GestureEvent, GesturePhase, TouchPoint};
use crate::error::{GestureError, GestureResult};
use crate::state::GestureStateMachine;
use super::GestureRecognizer;
use chrono::Utc;

#[derive(Debug)]
pub struct TapRecognizer {
    config: TapConfig,
    start_point: Option<TouchPoint>,
    tap_count: u8,
    last_tap_time: Option<i64>,
    last_tap_point: Option<TouchPoint>,
    is_down: bool,
}

impl TapRecognizer {
    pub fn new(config: TapConfig) -> Self {
        Self {
            config,
            start_point: None,
            tap_count: 0,
            last_tap_time: None,
            last_tap_point: None,
            is_down: false,
        }
    }

    fn is_within_tap_area(&self, point: &TouchPoint, reference: &TouchPoint) -> bool {
        point.distance_to(reference) <= self.config.max_movement
    }

    fn is_within_multi_tap_window(&self, current_time: i64) -> bool {
        if let Some(last_time) = self.last_tap_time {
            current_time - last_time <= self.config.multi_tap_delay_ms
        } else {
            true
        }
    }

    fn is_within_multi_tap_area(&self, point: &TouchPoint) -> bool {
        if let Some(ref last_point) = self.last_tap_point {
            point.distance_to(last_point) <= self.config.multi_tap_distance
        } else {
            true
        }
    }

    fn handle_touch_start(
        &mut self,
        point: TouchPoint,
        state_machine: &mut GestureStateMachine,
    ) -> GestureResult<Option<GestureEvent>> {
        self.start_point = Some(point);
        self.is_down = true;
        state_machine.update_state("tap", GesturePhase::Began)?;
        Ok(None)
    }

    fn handle_touch_end(
        &mut self,
        point: TouchPoint,
        state_machine: &mut GestureStateMachine,
    ) -> GestureResult<Option<GestureEvent>> {
        if !self.is_down {
            return Ok(None);
        }

        let start = match self.start_point {
            Some(p) => p,
            None => return Ok(None),
        };

        let now = Utc::now().timestamp_millis();
        let duration = now - start.timestamp;

        // Check if movement and duration are within tap thresholds
        if !self.is_within_tap_area(&point, &start) {
            self.reset();
            state_machine.update_state("tap", GesturePhase::Failed)?;
            return Ok(None);
        }

        if duration > self.config.max_duration_ms {
            self.reset();
            state_machine.update_state("tap", GesturePhase::Failed)?;
            return Ok(None);
        }

        // Check for multi-tap
        let is_multi_tap = self.is_within_multi_tap_window(now)
            && self.is_within_multi_tap_area(&point);

        if is_multi_tap {
            self.tap_count += 1;
        } else {
            self.tap_count = 1;
        }

        self.last_tap_time = Some(now);
        self.last_tap_point = Some(point);
        self.is_down = false;

        // Generate appropriate event
        let event = match self.tap_count {
            1 => {
                state_machine.update_state("tap", GesturePhase::Ended)?;
                Some(GestureEvent::Tap {
                    point,
                    count: 1,
                    timestamp: now,
                })
            }
            2 if self.config.enable_double_tap => {
                state_machine.update_state("tap", GesturePhase::Ended)?;
                Some(GestureEvent::DoubleTap {
                    point,
                    timestamp: now,
                })
            }
            3 if self.config.enable_triple_tap => {
                state_machine.update_state("tap", GesturePhase::Ended)?;
                self.tap_count = 0; // Reset after triple tap
                Some(GestureEvent::TripleTap {
                    point,
                    timestamp: now,
                })
            }
            _ => {
                state_machine.update_state("tap", GesturePhase::Ended)?;
                Some(GestureEvent::Tap {
                    point,
                    count: self.tap_count,
                    timestamp: now,
                })
            }
        };

        Ok(event)
    }
}

impl GestureRecognizer for TapRecognizer {
    fn process(
        &mut self,
        touches: &[TouchPoint],
        state_machine: &mut GestureStateMachine,
    ) -> GestureResult<Option<GestureEvent>> {
        match touches.len() {
            0 => {
                // Touch ended
                if let Some(start) = self.start_point {
                    self.handle_touch_end(start, state_machine)
                } else {
                    Ok(None)
                }
            }
            1 => {
                let point = touches[0];
                if !self.is_down {
                    // Touch started
                    self.handle_touch_start(point, state_machine)
                } else {
                    // Touch moved - check if still within tap area
                    if let Some(start) = self.start_point {
                        if !self.is_within_tap_area(&point, &start) {
                            self.reset();
                            state_machine.update_state("tap", GesturePhase::Failed)?;
                        }
                    }
                    Ok(None)
                }
            }
            _ => {
                // Multiple touches - not a tap
                if self.is_down {
                    self.reset();
                    state_machine.update_state("tap", GesturePhase::Failed)?;
                }
                Ok(None)
            }
        }
    }

    fn reset(&mut self) {
        self.start_point = None;
        self.is_down = false;
        // Don't reset tap_count and last_tap_* to allow multi-tap detection
    }

    fn name(&self) -> &str {
        "tap"
    }

    fn can_handle_touch_count(&self, count: usize) -> bool {
        count <= 1
    }
}
