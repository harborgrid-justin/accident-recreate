use crate::config::RotateConfig;
use crate::events::{GestureEvent, GesturePhase, TouchPoint};
use crate::error::GestureResult;
use crate::state::{GestureStateMachine, VelocityCalculator};
use super::GestureRecognizer;
use std::f64::consts::PI;

#[derive(Debug)]
pub struct RotateRecognizer {
    config: RotateConfig,
    initial_angle: Option<f64>,
    previous_angle: Option<f64>,
    current_rotation: f64,
    total_rotation: f64,
    velocity_calc: VelocityCalculator,
    is_active: bool,
    touch1: Option<TouchPoint>,
    touch2: Option<TouchPoint>,
}

impl RotateRecognizer {
    pub fn new(config: RotateConfig) -> Self {
        Self {
            config,
            initial_angle: None,
            previous_angle: None,
            current_rotation: 0.0,
            total_rotation: 0.0,
            velocity_calc: VelocityCalculator::default(),
            is_active: false,
            touch1: None,
            touch2: None,
        }
    }

    fn calculate_center(p1: &TouchPoint, p2: &TouchPoint) -> TouchPoint {
        TouchPoint::new(
            0,
            (p1.x + p2.x) / 2.0,
            (p1.y + p2.y) / 2.0,
        )
    }

    fn calculate_angle(p1: &TouchPoint, p2: &TouchPoint) -> f64 {
        let dx = p2.x - p1.x;
        let dy = p2.y - p1.y;
        dy.atan2(dx)
    }

    fn normalize_angle(angle: f64) -> f64 {
        let mut normalized = angle;
        while normalized > PI {
            normalized -= 2.0 * PI;
        }
        while normalized < -PI {
            normalized += 2.0 * PI;
        }
        normalized
    }

    fn calculate_delta_angle(&self, current_angle: f64) -> f64 {
        if let Some(prev) = self.previous_angle {
            Self::normalize_angle(current_angle - prev)
        } else {
            0.0
        }
    }

    fn degrees(radians: f64) -> f64 {
        radians * 180.0 / PI
    }

    fn radians(degrees: f64) -> f64 {
        degrees * PI / 180.0
    }
}

impl GestureRecognizer for RotateRecognizer {
    fn process(
        &mut self,
        touches: &[TouchPoint],
        state_machine: &mut GestureStateMachine,
    ) -> GestureResult<Option<GestureEvent>> {
        match touches.len() {
            2 => {
                let touch1 = touches[0];
                let touch2 = touches[1];
                let angle = Self::calculate_angle(&touch1, &touch2);
                let center = Self::calculate_center(&touch1, &touch2);

                if !self.is_active {
                    // Start rotation gesture
                    self.initial_angle = Some(angle);
                    self.previous_angle = Some(angle);
                    self.current_rotation = 0.0;
                    self.total_rotation = 0.0;
                    self.is_active = true;
                    self.touch1 = Some(touch1);
                    self.touch2 = Some(touch2);
                    self.velocity_calc.reset();
                    self.velocity_calc.add_sample(angle, 0.0, touch1.timestamp);

                    state_machine.update_state("rotate", GesturePhase::Began)?;

                    return Ok(Some(GestureEvent::RotateStart {
                        center,
                        initial_angle: Self::degrees(angle),
                        touch1,
                        touch2,
                    }));
                } else {
                    // Continue rotation gesture
                    let delta = self.calculate_delta_angle(angle);
                    self.current_rotation = delta;
                    self.total_rotation += delta;
                    self.previous_angle = Some(angle);
                    self.touch1 = Some(touch1);
                    self.touch2 = Some(touch2);
                    self.velocity_calc.add_sample(angle, 0.0, touch1.timestamp);

                    state_machine.update_state("rotate", GesturePhase::Changed)?;

                    // Check if rotation is significant enough
                    if Self::degrees(delta.abs()) >= self.config.min_angle_delta {
                        let angular_velocity = self.velocity_calc.get_speed();

                        return Ok(Some(GestureEvent::RotateMove {
                            center,
                            angle: Self::degrees(angle),
                            delta_angle: Self::degrees(delta) * self.config.sensitivity,
                            angular_velocity,
                            touch1,
                            touch2,
                        }));
                    }
                }

                Ok(None)
            }
            0 | 1 => {
                // End rotation gesture
                if self.is_active {
                    let center = if let (Some(t1), Some(t2)) = (self.touch1, self.touch2) {
                        Self::calculate_center(&t1, &t2)
                    } else {
                        TouchPoint::new(0, 0.0, 0.0)
                    };

                    let final_angle = if let Some(prev) = self.previous_angle {
                        Self::degrees(prev)
                    } else {
                        0.0
                    };

                    state_machine.update_state("rotate", GesturePhase::Ended)?;

                    let event = GestureEvent::RotateEnd {
                        center,
                        final_angle,
                        total_rotation: Self::degrees(self.total_rotation),
                    };

                    self.reset();
                    return Ok(Some(event));
                }

                Ok(None)
            }
            _ => {
                // More than 2 touches - cancel rotation
                if self.is_active {
                    self.reset();
                    state_machine.update_state("rotate", GesturePhase::Cancelled)?;
                }
                Ok(None)
            }
        }
    }

    fn reset(&mut self) {
        self.initial_angle = None;
        self.previous_angle = None;
        self.current_rotation = 0.0;
        self.total_rotation = 0.0;
        self.velocity_calc.reset();
        self.is_active = false;
        self.touch1 = None;
        self.touch2 = None;
    }

    fn name(&self) -> &str {
        "rotate"
    }

    fn can_handle_touch_count(&self, count: usize) -> bool {
        count >= 2 && count <= 2
    }
}
