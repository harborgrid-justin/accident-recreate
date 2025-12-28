//! Main physics simulation engine with timestep control.

use crate::collision::{CollisionDetector, CollisionResolver};
use crate::dynamics::VehicleDynamics;
use crate::friction::FrictionModel;
use crate::simulation::{RigidBody, SimulationRecording, SimulationSnapshot, SimulationState};
use nalgebra::{Matrix3, Vector3};
use parking_lot::RwLock;
use std::sync::Arc;
use tracing::{debug, info};

/// Configuration for the physics engine.
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Internal simulation timestep (s)
    pub timestep: f64,
    /// Maximum substeps per frame
    pub max_substeps: usize,
    /// Gravity vector (m/s²)
    pub gravity: Vector3<f64>,
    /// Global friction model
    pub friction_model: FrictionModel,
    /// Enable collision detection
    pub collision_detection: bool,
    /// Enable recording
    pub recording: bool,
    /// Spatial hash cell size for collision detection
    pub spatial_cell_size: f64,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            timestep: 0.001,         // 1ms (1000Hz)
            max_substeps: 10,
            gravity: Vector3::new(0.0, 0.0, -9.81),
            friction_model: FrictionModel::default(),
            collision_detection: true,
            recording: false,
            spatial_cell_size: 10.0, // 10 meter cells
        }
    }
}

/// Main physics simulation engine.
pub struct PhysicsEngine {
    /// Engine configuration
    config: EngineConfig,
    /// Current simulation state
    state: Arc<RwLock<SimulationState>>,
    /// Collision detector
    collision_detector: CollisionDetector,
    /// Collision resolver
    collision_resolver: CollisionResolver,
    /// Vehicle dynamics models
    vehicle_dynamics: std::collections::HashMap<u64, VehicleDynamics>,
    /// Simulation recording
    recording: Option<SimulationRecording>,
}

impl PhysicsEngine {
    /// Creates a new physics engine.
    pub fn new(config: EngineConfig) -> Self {
        let state = SimulationState::new(config.timestep);

        Self {
            collision_detector: CollisionDetector::new(config.spatial_cell_size),
            collision_resolver: CollisionResolver::new(),
            config,
            state: Arc::new(RwLock::new(state)),
            vehicle_dynamics: std::collections::HashMap::new(),
            recording: None,
        }
    }

    /// Adds a rigid body to the simulation.
    pub fn add_body(&mut self, body: RigidBody) {
        let id = body.id;
        self.state.write().add_body(body);

        // Update collision detector
        if let Some(body) = self.state.read().get_body(id) {
            self.collision_detector.update_aabb(id, body.aabb);
            self.collision_detector
                .update_vertices(id, body.vertices.clone());
        }
    }

    /// Adds a vehicle with dynamics model.
    pub fn add_vehicle(&mut self, body: RigidBody, dynamics: VehicleDynamics) {
        let id = body.id;
        self.vehicle_dynamics.insert(id, dynamics);
        self.add_body(body);
    }

    /// Removes a body from the simulation.
    pub fn remove_body(&mut self, id: u64) {
        self.state.write().remove_body(id);
        self.collision_detector.remove_object(id);
        self.vehicle_dynamics.remove(&id);
    }

    /// Steps the simulation forward by a given time.
    pub fn step(&mut self, dt: f64) {
        let num_substeps = ((dt / self.config.timestep).ceil() as usize).min(self.config.max_substeps);
        let substep_dt = dt / num_substeps as f64;

        for _ in 0..num_substeps {
            self.substep(substep_dt);
        }

        // Record snapshot if recording is enabled
        if self.config.recording && self.recording.is_some() {
            let snapshot = SimulationSnapshot::from_state(&self.state.read());
            self.recording.as_mut().unwrap().add_snapshot(snapshot);
        }
    }

    /// Performs a single simulation substep.
    fn substep(&mut self, dt: f64) {
        let mut state = self.state.write();

        if state.paused {
            return;
        }

        // Clear collisions from previous step
        state.clear_collisions();

        // Update AABBs in collision detector
        for (id, body) in &state.bodies {
            self.collision_detector.update_aabb(*id, body.aabb);
        }

        // Detect collisions
        if self.config.collision_detection {
            let collisions = self.collision_detector.detect_collisions();

            for collision in collisions {
                state.record_collision(collision.clone());

                // Resolve collision
                if let (Some(body_a), Some(body_b)) = (
                    state.bodies.get(&collision.object_a),
                    state.bodies.get(&collision.object_b),
                ) {
                    let impulse = self.collision_resolver.resolve_collision(
                        &collision,
                        body_a.mass,
                        body_b.mass,
                        body_a.state.velocity,
                        body_b.state.velocity,
                        body_a.state.angular_velocity,
                        body_b.state.angular_velocity,
                        Matrix3::identity() * 1000.0, // Simplified inertia
                        Matrix3::identity() * 1000.0,
                    );

                    // Apply impulses (if bodies are not static)
                    if !body_a.is_static {
                        if let Some(body_a) = state.bodies.get_mut(&collision.object_a) {
                            body_a.state.velocity += impulse.impulse_a / body_a.mass;
                        }
                    }

                    if !body_b.is_static {
                        if let Some(body_b) = state.bodies.get_mut(&collision.object_b) {
                            body_b.state.velocity += impulse.impulse_b / body_b.mass;
                        }
                    }
                }
            }
        }

        // Integrate physics for each body
        let body_ids: Vec<u64> = state.bodies.keys().copied().collect();

        for id in body_ids {
            if let Some(body) = state.bodies.get_mut(&id) {
                if body.is_static {
                    continue;
                }

                // Apply gravity
                let gravity_force = self.config.gravity * body.mass;

                // Apply friction (simplified ground contact model)
                let friction_force = if body.state.position.z <= 0.1 {
                    let normal_force = -self.config.gravity.z * body.mass;
                    let friction_magnitude = self
                        .config
                        .friction_model
                        .friction_force(normal_force, body.state.velocity.norm());

                    if body.state.velocity.norm() > 0.01 {
                        -body.state.velocity.normalize() * friction_magnitude
                    } else {
                        Vector3::zeros()
                    }
                } else {
                    Vector3::zeros()
                };

                // Total force
                let total_force = gravity_force + friction_force;

                // Use vehicle dynamics if available
                if let Some(dynamics) = self.vehicle_dynamics.get(&id) {
                    let forces = dynamics.compute_forces(&body.state, 0.0, 0.0, 0.0);
                    let torques = dynamics.compute_torques(&body.state, 0.0);

                    // Make a mutable copy of state
                    let mut vehicle_state = body.state.clone();
                    dynamics.integrate(&mut vehicle_state, forces + total_force, torques, dt);
                    body.state = vehicle_state;
                } else {
                    // Simple Euler integration
                    body.state.acceleration = total_force / body.mass;
                    body.state.velocity += body.state.acceleration * dt;
                    body.state.position += body.state.velocity * dt;

                    // Prevent falling through ground (simplified)
                    if body.state.position.z < 0.0 {
                        body.state.position.z = 0.0;
                        body.state.velocity.z = body.state.velocity.z.max(0.0);
                    }
                }

                // Update AABB
                body.update_aabb();
            }
        }

        // Advance time
        state.advance_time();

        debug!("Simulation step at t={:.3}s", state.time);
    }

    /// Starts recording the simulation.
    pub fn start_recording(&mut self) {
        self.recording = Some(SimulationRecording::new());
        self.config.recording = true;
        info!("Started simulation recording");
    }

    /// Stops recording and returns the recording.
    pub fn stop_recording(&mut self) -> Option<SimulationRecording> {
        self.config.recording = false;
        let recording = self.recording.take();
        info!("Stopped simulation recording");
        recording
    }

    /// Gets the current simulation time.
    pub fn time(&self) -> f64 {
        self.state.read().time
    }

    /// Gets a read-only reference to the simulation state.
    pub fn state(&self) -> Arc<RwLock<SimulationState>> {
        Arc::clone(&self.state)
    }

    /// Resets the simulation.
    pub fn reset(&mut self) {
        self.state.write().reset();
        self.recording = None;
        info!("Reset simulation");
    }

    /// Pauses the simulation.
    pub fn pause(&mut self) {
        self.state.write().pause();
    }

    /// Resumes the simulation.
    pub fn resume(&mut self) {
        self.state.write().resume();
    }
}

impl Default for PhysicsEngine {
    fn default() -> Self {
        Self::new(EngineConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::RigidBody;
    use nalgebra::Point3;

    #[test]
    fn test_engine_creation() {
        let engine = PhysicsEngine::default();
        assert_eq!(engine.time(), 0.0);
    }

    #[test]
    fn test_add_remove_body() {
        let mut engine = PhysicsEngine::default();

        let body = RigidBody::new(1, 1000.0);
        engine.add_body(body);

        assert_eq!(engine.state.read().num_bodies(), 1);

        engine.remove_body(1);
        assert_eq!(engine.state.read().num_bodies(), 0);
    }

    #[test]
    fn test_simulation_step() {
        let mut engine = PhysicsEngine::default();

        let body = RigidBody::new(1, 1000.0)
            .with_position(Point3::new(0.0, 0.0, 10.0))
            .with_velocity(Vector3::zeros());

        engine.add_body(body);

        // Step simulation
        engine.step(0.1);

        // Time should have advanced
        assert!(engine.time() > 0.0);

        // Body should have fallen due to gravity
        let state = engine.state.read();
        let body = state.get_body(1).unwrap();
        assert!(body.state.position.z < 10.0);
    }

    #[test]
    fn test_pause_resume() {
        let mut engine = PhysicsEngine::default();

        let body = RigidBody::new(1, 1000.0);
        engine.add_body(body);

        engine.step(0.1);
        let time1 = engine.time();

        engine.pause();
        engine.step(0.1);
        let time2 = engine.time();

        // Time should not advance while paused
        assert_eq!(time1, time2);

        engine.resume();
        engine.step(0.1);
        let time3 = engine.time();

        // Time should advance after resume
        assert!(time3 > time2);
    }
}
