use crate::config::PanConfig;
use crate::events::{GestureEvent, GesturePhase, TouchPoint};
use crate::error::GestureResult;
use crate::state::{GestureStateMachine, VelocityCalculator};
use super::GestureRecognizer;

#[derive(Debug)]
pub struct PanRecognizer {
    config: PanConfig,
    start_point: Option<TouchPoint>,
    previous_point: Option<TouchPoint>,
    total_delta_x: f64,
    total_delta_y: f64,
    velocity_calc: VelocityCalculator,
    is_active: bool,
    has_started: bool,
}

impl PanRecognizer {
    pub fn new(config: PanConfig) -> Self {
        Self {
            config,
            start_point: None,
            previous_point: None,
            total_delta_x: 0.0,
            total_delta_y: 0.0,
            velocity_calc: VelocityCalculator::default(),
            is_active: false,
            has_started: false,
        }
    }

    fn calculate_delta(&self, current: &TouchPoint) -> (f64, f64) {
        if let Some(prev) = self.previous_point {
            let mut dx = current.x - prev.x;
            let mut dy = current.y - prev.y;

            // Apply directional constraints
            if !self.config.enable_horizontal {
                dx = 0.0;
            }
            if !self.config.enable_vertical {
                dy = 0.0;
            }

            (dx, dy)
        } else {
            (0.0, 0.0)
        }
    }

    fn should_start_pan(&self, current: &TouchPoint) -> bool {
        if let Some(start) = self.start_point {
            let distance = start.distance_to(current);
            distance >= self.config.min_distance
        } else {
            false
        }
    }

    fn apply_momentum(&mut self, velocity_x: f64, velocity_y: f64) -> (f64, f64) {
        if self.config.enable_momentum {
            (
                velocity_x * self.config.momentum_decay,
                velocity_y * self.config.momentum_decay,
            )
        } else {
            (velocity_x, velocity_y)
        }
    }
}

impl GestureRecognizer for PanRecognizer {
    fn process(
        &mut self,
        touches: &[TouchPoint],
        state_machine: &mut GestureStateMachine,
    ) -> GestureResult<Option<GestureEvent>> {
        let touch_count = touches.len();

        // Check if touch count is within valid range
        if touch_count > self.config.max_touches || touch_count < self.config.min_touches {
            if self.is_active {
                self.reset();
                state_machine.update_state("pan", GesturePhase::Cancelled)?;
            }
            return Ok(None);
        }

        match touch_count {
            0 => {
                // Touch ended
                if self.is_active && self.has_started {
                    let (velocity_x, velocity_y) = self.velocity_calc.calculate_velocity();
                    let (final_vx, final_vy) = self.apply_momentum(velocity_x, velocity_y);

                    state_machine.update_state("pan", GesturePhase::Ended)?;

                    let event = GestureEvent::PanEnd {
                        point: self.previous_point.unwrap_or_else(|| TouchPoint::new(0, 0.0, 0.0)),
                        total_delta_x: self.total_delta_x,
                        total_delta_y: self.total_delta_y,
                        final_velocity_x: final_vx,
                        final_velocity_y: final_vy,
                    };

                    self.reset();
                    return Ok(Some(event));
                }

                self.reset();
                Ok(None)
            }
            _ => {
                // Use first touch for panning (or average of all touches)
                let point = if self.config.max_touches == 1 {
                    touches[0]
                } else {
                    // Calculate average position for multi-touch pan
                    let avg_x = touches.iter().map(|t| t.x).sum::<f64>() / touches.len() as f64;
                    let avg_y = touches.iter().map(|t| t.y).sum::<f64>() / touches.len() as f64;
                    TouchPoint::new(touches[0].id, avg_x, avg_y)
                };

                if !self.is_active {
                    // Start tracking
                    self.start_point = Some(point);
                    self.previous_point = Some(point);
                    self.is_active = true;
                    self.has_started = false;
                    self.velocity_calc.reset();
                    self.velocity_calc.add_sample(point.x, point.y, point.timestamp);
                    Ok(None)
                } else if !self.has_started {
                    // Check if we should start the pan gesture
                    if self.should_start_pan(&point) {
                        self.has_started = true;
                        state_machine.update_state("pan", GesturePhase::Began)?;

                        return Ok(Some(GestureEvent::PanStart { point }));
                    }

                    self.previous_point = Some(point);
                    self.velocity_calc.add_sample(point.x, point.y, point.timestamp);
                    Ok(None)
                } else {
                    // Continue panning
                    let (delta_x, delta_y) = self.calculate_delta(&point);
                    self.total_delta_x += delta_x;
                    self.total_delta_y += delta_y;
                    self.previous_point = Some(point);
                    self.velocity_calc.add_sample(point.x, point.y, point.timestamp);

                    let (velocity_x, velocity_y) = self.velocity_calc.calculate_velocity();

                    state_machine.update_state("pan", GesturePhase::Changed)?;

                    Ok(Some(GestureEvent::PanMove {
                        point,
                        delta_x,
                        delta_y,
                        velocity_x,
                        velocity_y,
                        total_delta_x: self.total_delta_x,
                        total_delta_y: self.total_delta_y,
                    }))
                }
            }
        }
    }

    fn reset(&mut self) {
        self.start_point = None;
        self.previous_point = None;
        self.total_delta_x = 0.0;
        self.total_delta_y = 0.0;
        self.velocity_calc.reset();
        self.is_active = false;
        self.has_started = false;
    }

    fn name(&self) -> &str {
        "pan"
    }

    fn can_handle_touch_count(&self, count: usize) -> bool {
        count >= self.config.min_touches && count <= self.config.max_touches
    }
}
