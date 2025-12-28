//! Trajectory prediction and analysis.

use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};

/// A single point along a trajectory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectoryPoint {
    /// Time since trajectory start (s)
    pub time: f64,
    /// Position in world space
    pub position: Point3<f64>,
    /// Velocity at this point (m/s)
    pub velocity: Vector3<f64>,
    /// Acceleration at this point (m/s²)
    pub acceleration: Vector3<f64>,
}

/// Complete trajectory of an object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trajectory {
    /// Points along the trajectory
    pub points: Vec<TrajectoryPoint>,
    /// Total distance traveled (m)
    pub total_distance: f64,
    /// Total duration (s)
    pub duration: f64,
}

impl Trajectory {
    /// Creates a new empty trajectory.
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            total_distance: 0.0,
            duration: 0.0,
        }
    }

    /// Adds a point to the trajectory.
    pub fn add_point(&mut self, point: TrajectoryPoint) {
        if let Some(last) = self.points.last() {
            let distance = (point.position - last.position).norm();
            self.total_distance += distance;
        }

        self.duration = point.time;
        self.points.push(point);
    }

    /// Gets the position at a specific time using linear interpolation.
    pub fn position_at_time(&self, time: f64) -> Option<Point3<f64>> {
        if self.points.is_empty() {
            return None;
        }

        if time <= self.points[0].time {
            return Some(self.points[0].position);
        }

        if time >= self.points.last().unwrap().time {
            return Some(self.points.last().unwrap().position);
        }

        // Find the two points to interpolate between
        for i in 0..self.points.len() - 1 {
            if time >= self.points[i].time && time <= self.points[i + 1].time {
                let t = (time - self.points[i].time)
                    / (self.points[i + 1].time - self.points[i].time);
                let pos = self.points[i].position.coords * (1.0 - t)
                    + self.points[i + 1].position.coords * t;
                return Some(Point3::from(pos));
            }
        }

        None
    }

    /// Gets the velocity at a specific time using linear interpolation.
    pub fn velocity_at_time(&self, time: f64) -> Option<Vector3<f64>> {
        if self.points.is_empty() {
            return None;
        }

        if time <= self.points[0].time {
            return Some(self.points[0].velocity);
        }

        if time >= self.points.last().unwrap().time {
            return Some(self.points.last().unwrap().velocity);
        }

        for i in 0..self.points.len() - 1 {
            if time >= self.points[i].time && time <= self.points[i + 1].time {
                let t = (time - self.points[i].time)
                    / (self.points[i + 1].time - self.points[i].time);
                let vel = self.points[i].velocity * (1.0 - t) + self.points[i + 1].velocity * t;
                return Some(vel);
            }
        }

        None
    }

    /// Calculates the average velocity over the trajectory.
    pub fn average_velocity(&self) -> Vector3<f64> {
        if self.points.len() < 2 {
            return Vector3::zeros();
        }

        let displacement =
            self.points.last().unwrap().position - self.points.first().unwrap().position;
        displacement.coords / self.duration
    }

    /// Calculates the maximum speed reached.
    pub fn max_speed(&self) -> f64 {
        self.points
            .iter()
            .map(|p| p.velocity.norm())
            .fold(0.0, f64::max)
    }

    /// Calculates the maximum acceleration magnitude.
    pub fn max_acceleration(&self) -> f64 {
        self.points
            .iter()
            .map(|p| p.acceleration.norm())
            .fold(0.0, f64::max)
    }
}

impl Default for Trajectory {
    fn default() -> Self {
        Self::new()
    }
}

/// Trajectory predictor for forward simulation.
pub struct TrajectoryPredictor {
    /// Time step for prediction (s)
    dt: f64,
}

impl TrajectoryPredictor {
    /// Creates a new trajectory predictor.
    pub fn new(dt: f64) -> Self {
        Self { dt }
    }

    /// Predicts trajectory under constant acceleration.
    pub fn predict_constant_acceleration(
        &self,
        initial_position: Point3<f64>,
        initial_velocity: Vector3<f64>,
        acceleration: Vector3<f64>,
        duration: f64,
    ) -> Trajectory {
        let mut trajectory = Trajectory::new();
        let num_steps = (duration / self.dt).ceil() as usize;

        for i in 0..=num_steps {
            let t = (i as f64) * self.dt;
            let t = t.min(duration);

            // Kinematic equations:
            // x = x₀ + v₀t + ½at²
            // v = v₀ + at
            let position = initial_position
                + initial_velocity * t
                + acceleration * (0.5 * t * t);
            let velocity = initial_velocity + acceleration * t;

            trajectory.add_point(TrajectoryPoint {
                time: t,
                position,
                velocity,
                acceleration,
            });

            if t >= duration {
                break;
            }
        }

        trajectory
    }

    /// Predicts trajectory with drag force (air resistance).
    pub fn predict_with_drag(
        &self,
        initial_position: Point3<f64>,
        initial_velocity: Vector3<f64>,
        mass: f64,
        drag_coefficient: f64,
        frontal_area: f64,
        duration: f64,
    ) -> Trajectory {
        let mut trajectory = Trajectory::new();
        let num_steps = (duration / self.dt).ceil() as usize;

        let mut position = initial_position;
        let mut velocity = initial_velocity;
        let air_density = 1.225; // kg/m³

        for i in 0..=num_steps {
            let t = (i as f64) * self.dt;
            let t = t.min(duration);

            // Drag force: F = -0.5 * ρ * v² * Cd * A * v̂
            let speed = velocity.norm();
            let drag_force = if speed > 0.01 {
                let drag_magnitude =
                    0.5 * air_density * speed * speed * drag_coefficient * frontal_area;
                -velocity.normalize() * drag_magnitude
            } else {
                Vector3::zeros()
            };

            // Gravity
            let gravity = Vector3::new(0.0, 0.0, -9.81) * mass;

            // Total acceleration
            let acceleration = (drag_force + gravity) / mass;

            trajectory.add_point(TrajectoryPoint {
                time: t,
                position,
                velocity,
                acceleration,
            });

            // Euler integration
            velocity += acceleration * self.dt;
            position += velocity * self.dt;

            if t >= duration {
                break;
            }
        }

        trajectory
    }

    /// Predicts trajectory with friction (sliding on surface).
    pub fn predict_sliding(
        &self,
        initial_position: Point3<f64>,
        initial_velocity: Vector3<f64>,
        friction_coefficient: f64,
        _mass: f64,
    ) -> Trajectory {
        let mut trajectory = Trajectory::new();
        let mut position = initial_position;
        let mut velocity = initial_velocity;
        let mut time = 0.0;

        let gravity = 9.81;
        let friction_deceleration = friction_coefficient * gravity;

        loop {
            let speed = velocity.norm();
            if speed < 0.01 {
                // Stopped
                trajectory.add_point(TrajectoryPoint {
                    time,
                    position,
                    velocity: Vector3::zeros(),
                    acceleration: Vector3::zeros(),
                });
                break;
            }

            // Friction force opposes velocity
            let friction_force = -velocity.normalize() * friction_deceleration;

            trajectory.add_point(TrajectoryPoint {
                time,
                position,
                velocity,
                acceleration: friction_force,
            });

            // Update velocity and position
            velocity += friction_force * self.dt;
            position += velocity * self.dt;
            time += self.dt;

            if time > 100.0 {
                // Safety limit
                break;
            }
        }

        trajectory
    }

    /// Calculates the stopping distance for a vehicle with friction.
    pub fn stopping_distance(initial_speed: f64, friction_coefficient: f64) -> f64 {
        // Using energy conservation: ½mv² = μmgd
        // d = v² / (2μg)
        let gravity = 9.81;
        initial_speed * initial_speed / (2.0 * friction_coefficient * gravity)
    }

    /// Calculates the initial speed from skid mark length.
    pub fn speed_from_skid_length(skid_length: f64, friction_coefficient: f64) -> f64 {
        // Reverse of stopping distance formula
        let gravity = 9.81;
        (2.0 * friction_coefficient * gravity * skid_length).sqrt()
    }
}

impl Default for TrajectoryPredictor {
    fn default() -> Self {
        Self::new(0.01)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trajectory_creation() {
        let mut trajectory = Trajectory::new();

        trajectory.add_point(TrajectoryPoint {
            time: 0.0,
            position: Point3::origin(),
            velocity: Vector3::new(10.0, 0.0, 0.0),
            acceleration: Vector3::zeros(),
        });

        trajectory.add_point(TrajectoryPoint {
            time: 1.0,
            position: Point3::new(10.0, 0.0, 0.0),
            velocity: Vector3::new(10.0, 0.0, 0.0),
            acceleration: Vector3::zeros(),
        });

        assert_eq!(trajectory.points.len(), 2);
        assert_eq!(trajectory.duration, 1.0);
        assert!((trajectory.total_distance - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_constant_acceleration_prediction() {
        let predictor = TrajectoryPredictor::new(0.1);

        let trajectory = predictor.predict_constant_acceleration(
            Point3::origin(),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(2.0, 0.0, 0.0),
            5.0,
        );

        // After 5 seconds with 2 m/s² acceleration: x = ½at² = ½ * 2 * 25 = 25 m
        let final_pos = trajectory.points.last().unwrap().position;
        assert!((final_pos.x - 25.0).abs() < 0.1);
    }

    #[test]
    fn test_stopping_distance() {
        // At 100 km/h (27.78 m/s) with μ=0.7
        let speed = 27.78;
        let friction = 0.7;
        let distance = TrajectoryPredictor::stopping_distance(speed, friction);

        // Expected: v²/(2μg) = 771.73/(2*0.7*9.81) ≈ 56.2 m
        assert!((distance - 56.2).abs() < 1.0);
    }

    #[test]
    fn test_speed_from_skid() {
        let skid_length = 50.0; // 50 meters
        let friction = 0.7;
        let speed = TrajectoryPredictor::speed_from_skid_length(skid_length, friction);

        // Expected: sqrt(2μgd) = sqrt(2*0.7*9.81*50) ≈ 26.2 m/s
        assert!((speed - 26.2).abs() < 1.0);
    }
}
