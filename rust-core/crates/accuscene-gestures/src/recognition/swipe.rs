use crate::config::SwipeConfig;
use crate::events::{GestureEvent, GesturePhase, SwipeDirection, TouchPoint};
use crate::error::GestureResult;
use crate::state::{GestureStateMachine, VelocityCalculator};
use super::GestureRecognizer;
use chrono::Utc;

#[derive(Debug)]
pub struct SwipeRecognizer {
    config: SwipeConfig,
    start_point: Option<TouchPoint>,
    current_point: Option<TouchPoint>,
    velocity_calc: VelocityCalculator,
    is_tracking: bool,
}

impl SwipeRecognizer {
    pub fn new(config: SwipeConfig) -> Self {
        Self {
            config,
            start_point: None,
            current_point: None,
            velocity_calc: VelocityCalculator::default(),
            is_tracking: false,
        }
    }

    fn calculate_direction(&self, start: &TouchPoint, end: &TouchPoint) -> SwipeDirection {
        let dx = end.x - start.x;
        let dy = end.y - start.y;
        let angle = dy.atan2(dx);
        SwipeDirection::from_angle(angle)
    }

    fn is_valid_swipe(&self, start: &TouchPoint, end: &TouchPoint) -> bool {
        let distance = start.distance_to(end);
        let duration = end.timestamp - start.timestamp;

        if distance < self.config.min_distance {
            return false;
        }

        if duration > self.config.max_duration_ms {
            return false;
        }

        let velocity = self.velocity_calc.get_speed();
        if velocity < self.config.min_velocity {
            return false;
        }

        // Check direction consistency if diagonal swipes are disabled
        if !self.config.enable_diagonal {
            let direction = self.calculate_direction(start, end);
            if direction.is_diagonal() {
                return false;
            }
        }

        true
    }
}

impl GestureRecognizer for SwipeRecognizer {
    fn process(
        &mut self,
        touches: &[TouchPoint],
        state_machine: &mut GestureStateMachine,
    ) -> GestureResult<Option<GestureEvent>> {
        match touches.len() {
            0 => {
                // Touch ended - check if it's a valid swipe
                if !self.is_tracking {
                    return Ok(None);
                }

                if let (Some(start), Some(end)) = (self.start_point, self.current_point) {
                    if self.is_valid_swipe(&start, &end) {
                        let direction = self.calculate_direction(&start, &end);
                        let distance = start.distance_to(&end);
                        let duration = end.timestamp - start.timestamp;
                        let velocity = self.velocity_calc.get_speed();

                        state_machine.update_state("swipe", GesturePhase::Ended)?;

                        let event = GestureEvent::Swipe {
                            start,
                            end,
                            direction,
                            velocity,
                            distance,
                            duration_ms: duration,
                        };

                        self.reset();
                        return Ok(Some(event));
                    }
                }

                self.reset();
                state_machine.update_state("swipe", GesturePhase::Failed)?;
                Ok(None)
            }
            1 => {
                let point = touches[0];

                if !self.is_tracking {
                    // Start tracking
                    self.start_point = Some(point);
                    self.current_point = Some(point);
                    self.is_tracking = true;
                    self.velocity_calc.reset();
                    self.velocity_calc.add_sample(point.x, point.y, point.timestamp);
                    state_machine.update_state("swipe", GesturePhase::Began)?;
                } else {
                    // Update tracking
                    self.current_point = Some(point);
                    self.velocity_calc.add_sample(point.x, point.y, point.timestamp);

                    if let Some(start) = self.start_point {
                        let distance = start.distance_to(&point);
                        if distance >= self.config.min_distance {
                            state_machine.update_state("swipe", GesturePhase::Changed)?;
                        }
                    }
                }

                Ok(None)
            }
            _ => {
                // Multiple touches - not a swipe
                if self.is_tracking {
                    self.reset();
                    state_machine.update_state("swipe", GesturePhase::Failed)?;
                }
                Ok(None)
            }
        }
    }

    fn reset(&mut self) {
        self.start_point = None;
        self.current_point = None;
        self.velocity_calc.reset();
        self.is_tracking = false;
    }

    fn name(&self) -> &str {
        "swipe"
    }

    fn can_handle_touch_count(&self, count: usize) -> bool {
        count <= 1
    }
}
