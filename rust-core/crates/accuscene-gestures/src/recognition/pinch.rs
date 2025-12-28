use crate::config::PinchConfig;
use crate::events::{GestureEvent, GesturePhase, TouchPoint};
use crate::error::GestureResult;
use crate::state::{GestureStateMachine, VelocityCalculator};
use super::GestureRecognizer;

#[derive(Debug)]
pub struct PinchRecognizer {
    config: PinchConfig,
    initial_distance: Option<f64>,
    previous_distance: Option<f64>,
    current_scale: f64,
    total_scale: f64,
    velocity_calc: VelocityCalculator,
    is_active: bool,
    touch1: Option<TouchPoint>,
    touch2: Option<TouchPoint>,
}

impl PinchRecognizer {
    pub fn new(config: PinchConfig) -> Self {
        Self {
            config,
            initial_distance: None,
            previous_distance: None,
            current_scale: 1.0,
            total_scale: 1.0,
            velocity_calc: VelocityCalculator::default(),
            is_active: false,
            touch1: None,
            touch2: None,
        }
    }

    fn calculate_center(p1: &TouchPoint, p2: &TouchPoint) -> TouchPoint {
        TouchPoint::new(
            0, // ID doesn't matter for center point
            (p1.x + p2.x) / 2.0,
            (p1.y + p2.y) / 2.0,
        )
    }

    fn calculate_distance(p1: &TouchPoint, p2: &TouchPoint) -> f64 {
        p1.distance_to(p2)
    }

    fn calculate_scale(&self, current_distance: f64) -> f64 {
        if let Some(initial) = self.initial_distance {
            if initial > 0.0 {
                let scale = (current_distance / initial) * self.config.sensitivity;
                scale.clamp(self.config.min_scale, self.config.max_scale)
            } else {
                1.0
            }
        } else {
            1.0
        }
    }

    fn calculate_velocity(&self) -> f64 {
        self.velocity_calc.get_speed()
    }
}

impl GestureRecognizer for PinchRecognizer {
    fn process(
        &mut self,
        touches: &[TouchPoint],
        state_machine: &mut GestureStateMachine,
    ) -> GestureResult<Option<GestureEvent>> {
        match touches.len() {
            2 => {
                let touch1 = touches[0];
                let touch2 = touches[1];
                let distance = Self::calculate_distance(&touch1, &touch2);
                let center = Self::calculate_center(&touch1, &touch2);

                if !self.is_active {
                    // Start pinch gesture
                    self.initial_distance = Some(distance);
                    self.previous_distance = Some(distance);
                    self.current_scale = 1.0;
                    self.total_scale = 1.0;
                    self.is_active = true;
                    self.touch1 = Some(touch1);
                    self.touch2 = Some(touch2);
                    self.velocity_calc.reset();
                    self.velocity_calc.add_sample(distance, 0.0, touch1.timestamp);

                    state_machine.update_state("pinch", GesturePhase::Began)?;

                    return Ok(Some(GestureEvent::PinchStart {
                        center,
                        initial_distance: distance,
                        touch1,
                        touch2,
                    }));
                } else {
                    // Continue pinch gesture
                    self.previous_distance = Some(distance);
                    self.current_scale = self.calculate_scale(distance);
                    self.total_scale = self.current_scale;
                    self.touch1 = Some(touch1);
                    self.touch2 = Some(touch2);
                    self.velocity_calc.add_sample(distance, 0.0, touch1.timestamp);

                    state_machine.update_state("pinch", GesturePhase::Changed)?;

                    // Check if scale change is significant enough
                    if (self.current_scale - 1.0).abs() >= self.config.min_scale_delta {
                        let velocity = self.calculate_velocity();

                        return Ok(Some(GestureEvent::PinchMove {
                            center,
                            scale: self.current_scale,
                            distance,
                            velocity,
                            touch1,
                            touch2,
                        }));
                    }
                }

                Ok(None)
            }
            0 | 1 => {
                // End pinch gesture
                if self.is_active {
                    let center = if let (Some(t1), Some(t2)) = (self.touch1, self.touch2) {
                        Self::calculate_center(&t1, &t2)
                    } else {
                        TouchPoint::new(0, 0.0, 0.0)
                    };

                    state_machine.update_state("pinch", GesturePhase::Ended)?;

                    let event = GestureEvent::PinchEnd {
                        center,
                        final_scale: self.current_scale,
                        total_scale_change: self.total_scale - 1.0,
                    };

                    self.reset();
                    return Ok(Some(event));
                }

                Ok(None)
            }
            _ => {
                // More than 2 touches - cancel pinch
                if self.is_active {
                    self.reset();
                    state_machine.update_state("pinch", GesturePhase::Cancelled)?;
                }
                Ok(None)
            }
        }
    }

    fn reset(&mut self) {
        self.initial_distance = None;
        self.previous_distance = None;
        self.current_scale = 1.0;
        self.total_scale = 1.0;
        self.velocity_calc.reset();
        self.is_active = false;
        self.touch1 = None;
        self.touch2 = None;
    }

    fn name(&self) -> &str {
        "pinch"
    }

    fn can_handle_touch_count(&self, count: usize) -> bool {
        count >= 2 && count <= 2
    }
}
