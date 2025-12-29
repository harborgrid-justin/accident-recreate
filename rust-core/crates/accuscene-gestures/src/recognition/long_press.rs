use crate::config::LongPressConfig;
use crate::events::{GestureEvent, GesturePhase, TouchPoint};
use crate::error::GestureResult;
use crate::state::GestureStateMachine;
use super::GestureRecognizer;
use chrono::Utc;

#[derive(Debug)]
pub struct LongPressRecognizer {
    config: LongPressConfig,
    start_point: Option<TouchPoint>,
    start_time: Option<i64>,
    is_active: bool,
    has_triggered: bool,
}

impl LongPressRecognizer {
    pub fn new(config: LongPressConfig) -> Self {
        Self {
            config,
            start_point: None,
            start_time: None,
            is_active: false,
            has_triggered: false,
        }
    }

    fn is_within_movement_threshold(&self, current: &TouchPoint) -> bool {
        if let Some(start) = self.start_point {
            start.distance_to(current) <= self.config.max_movement
        } else {
            true
        }
    }

    fn get_duration_ms(&self) -> i64 {
        if let Some(start) = self.start_time {
            Utc::now().timestamp_millis() - start
        } else {
            0
        }
    }

    fn has_reached_threshold(&self) -> bool {
        self.get_duration_ms() >= self.config.min_duration_ms
    }
}

impl GestureRecognizer for LongPressRecognizer {
    fn process(
        &mut self,
        touches: &[TouchPoint],
        state_machine: &mut GestureStateMachine,
    ) -> GestureResult<Option<GestureEvent>> {
        let touch_count = touches.len();

        // Check if we have the required number of touches
        if touch_count != self.config.required_touches {
            if self.is_active {
                self.reset();
                state_machine.update_state("long_press", GesturePhase::Cancelled)?;
            }
            return Ok(None);
        }

        match touch_count {
            0 => {
                // Touch ended
                if self.is_active {
                    let duration = self.get_duration_ms();
                    let point = self.start_point.unwrap_or_else(|| TouchPoint::new(0, 0.0, 0.0));

                    if self.has_triggered {
                        state_machine.update_state("long_press", GesturePhase::Ended)?;

                        let event = GestureEvent::LongPressEnd {
                            point,
                            total_duration_ms: duration,
                        };

                        self.reset();
                        return Ok(Some(event));
                    } else {
                        state_machine.update_state("long_press", GesturePhase::Failed)?;
                    }
                }

                self.reset();
                Ok(None)
            }
            _ => {
                // Calculate center point if multiple touches
                let point = if touches.len() == 1 {
                    touches[0]
                } else {
                    let avg_x = touches.iter().map(|t| t.x).sum::<f64>() / touches.len() as f64;
                    let avg_y = touches.iter().map(|t| t.y).sum::<f64>() / touches.len() as f64;
                    TouchPoint::new(touches[0].id, avg_x, avg_y)
                };

                if !self.is_active {
                    // Start tracking
                    self.start_point = Some(point);
                    self.start_time = Some(Utc::now().timestamp_millis());
                    self.is_active = true;
                    self.has_triggered = false;
                    Ok(None)
                } else {
                    // Check if movement is within threshold
                    if !self.is_within_movement_threshold(&point) {
                        self.reset();
                        state_machine.update_state("long_press", GesturePhase::Failed)?;
                        return Ok(None);
                    }

                    let duration = self.get_duration_ms();

                    if !self.has_triggered && self.has_reached_threshold() {
                        // First trigger
                        self.has_triggered = true;
                        state_machine.update_state("long_press", GesturePhase::Began)?;

                        return Ok(Some(GestureEvent::LongPressStart { point }));
                    } else if self.has_triggered {
                        // Continue long press
                        state_machine.update_state("long_press", GesturePhase::Changed)?;

                        return Ok(Some(GestureEvent::LongPress {
                            point,
                            duration_ms: duration,
                        }));
                    }

                    Ok(None)
                }
            }
        }
    }

    fn reset(&mut self) {
        self.start_point = None;
        self.start_time = None;
        self.is_active = false;
        self.has_triggered = false;
    }

    fn name(&self) -> &str {
        "long_press"
    }

    fn can_handle_touch_count(&self, count: usize) -> bool {
        count == self.config.required_touches || (self.is_active && count == 0)
    }
}
