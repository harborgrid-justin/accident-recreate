//! Simulation state management.

use crate::collision::{Aabb, Collision};
use crate::dynamics::VehicleState;
use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Rigid body in the simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RigidBody {
    /// Unique identifier
    pub id: u64,
    /// Current state
    pub state: VehicleState,
    /// Mass (kg)
    pub mass: f64,
    /// Bounding box
    pub aabb: Aabb,
    /// Vertices for collision detection
    pub vertices: Vec<Point3<f64>>,
    /// Is this body static (immovable)?
    pub is_static: bool,
    /// User-defined metadata
    pub metadata: HashMap<String, String>,
}

impl RigidBody {
    /// Creates a new rigid body.
    pub fn new(id: u64, mass: f64) -> Self {
        Self {
            id,
            state: VehicleState::default(),
            mass,
            aabb: Aabb::new(Point3::origin(), Point3::new(1.0, 1.0, 1.0)),
            vertices: Vec::new(),
            is_static: false,
            metadata: HashMap::new(),
        }
    }

    /// Sets the position of the rigid body.
    pub fn with_position(mut self, position: Point3<f64>) -> Self {
        self.state.position = position;
        self
    }

    /// Sets the velocity of the rigid body.
    pub fn with_velocity(mut self, velocity: Vector3<f64>) -> Self {
        self.state.velocity = velocity;
        self
    }

    /// Marks the body as static (immovable).
    pub fn as_static(mut self) -> Self {
        self.is_static = true;
        self.mass = f64::INFINITY;
        self
    }

    /// Sets the vertices for collision detection.
    pub fn with_vertices(mut self, vertices: Vec<Point3<f64>>) -> Self {
        self.vertices = vertices;
        self.update_aabb();
        self
    }

    /// Updates the AABB from vertices.
    pub fn update_aabb(&mut self) {
        if self.vertices.is_empty() {
            return;
        }

        let mut min = self.vertices[0];
        let mut max = self.vertices[0];

        for vertex in &self.vertices {
            min.x = min.x.min(vertex.x);
            min.y = min.y.min(vertex.y);
            min.z = min.z.min(vertex.z);
            max.x = max.x.max(vertex.x);
            max.y = max.y.max(vertex.y);
            max.z = max.z.max(vertex.z);
        }

        self.aabb = Aabb::new(min, max);
    }

    /// Adds metadata to the body.
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Complete simulation state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationState {
    /// Current simulation time (s)
    pub time: f64,
    /// Timestep size (s)
    pub dt: f64,
    /// All rigid bodies in the simulation
    pub bodies: HashMap<u64, RigidBody>,
    /// Recent collisions
    pub collisions: Vec<Collision>,
    /// Is the simulation paused?
    pub paused: bool,
    /// Simulation metadata
    pub metadata: HashMap<String, String>,
}

impl SimulationState {
    /// Creates a new simulation state.
    pub fn new(dt: f64) -> Self {
        Self {
            time: 0.0,
            dt,
            bodies: HashMap::new(),
            collisions: Vec::new(),
            paused: false,
            metadata: HashMap::new(),
        }
    }

    /// Adds a rigid body to the simulation.
    pub fn add_body(&mut self, body: RigidBody) {
        self.bodies.insert(body.id, body);
    }

    /// Removes a rigid body from the simulation.
    pub fn remove_body(&mut self, id: u64) -> Option<RigidBody> {
        self.bodies.remove(&id)
    }

    /// Gets a rigid body by ID.
    pub fn get_body(&self, id: u64) -> Option<&RigidBody> {
        self.bodies.get(&id)
    }

    /// Gets a mutable rigid body by ID.
    pub fn get_body_mut(&mut self, id: u64) -> Option<&mut RigidBody> {
        self.bodies.get_mut(&id)
    }

    /// Advances simulation time.
    pub fn advance_time(&mut self) {
        if !self.paused {
            self.time += self.dt;
        }
    }

    /// Resets the simulation.
    pub fn reset(&mut self) {
        self.time = 0.0;
        self.collisions.clear();
        self.paused = false;
    }

    /// Records a collision.
    pub fn record_collision(&mut self, collision: Collision) {
        self.collisions.push(collision);
    }

    /// Clears collision history.
    pub fn clear_collisions(&mut self) {
        self.collisions.clear();
    }

    /// Returns the number of active bodies.
    pub fn num_bodies(&self) -> usize {
        self.bodies.len()
    }

    /// Returns the number of collisions recorded.
    pub fn num_collisions(&self) -> usize {
        self.collisions.len()
    }

    /// Pauses the simulation.
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Resumes the simulation.
    pub fn resume(&mut self) {
        self.paused = false;
    }

    /// Toggles pause state.
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }
}

impl Default for SimulationState {
    fn default() -> Self {
        Self::new(0.001) // 1ms default timestep (1000Hz)
    }
}

/// Snapshot of simulation state for recording/playback.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationSnapshot {
    /// Time of snapshot
    pub time: f64,
    /// Body states at this time
    pub body_states: HashMap<u64, VehicleState>,
    /// Collisions at this timestep
    pub collisions: Vec<Collision>,
}

impl SimulationSnapshot {
    /// Creates a snapshot from current simulation state.
    pub fn from_state(state: &SimulationState) -> Self {
        let body_states = state
            .bodies
            .iter()
            .map(|(id, body)| (*id, body.state.clone()))
            .collect();

        Self {
            time: state.time,
            body_states,
            collisions: state.collisions.clone(),
        }
    }

    /// Applies this snapshot to a simulation state.
    pub fn apply_to_state(&self, state: &mut SimulationState) {
        state.time = self.time;
        for (id, body_state) in &self.body_states {
            if let Some(body) = state.bodies.get_mut(id) {
                body.state = body_state.clone();
            }
        }
        state.collisions = self.collisions.clone();
    }
}

/// Recording of a simulation run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationRecording {
    /// Snapshots over time
    pub snapshots: Vec<SimulationSnapshot>,
    /// Recording metadata
    pub metadata: HashMap<String, String>,
}

impl SimulationRecording {
    /// Creates a new recording.
    pub fn new() -> Self {
        Self {
            snapshots: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Adds a snapshot to the recording.
    pub fn add_snapshot(&mut self, snapshot: SimulationSnapshot) {
        self.snapshots.push(snapshot);
    }

    /// Returns the duration of the recording.
    pub fn duration(&self) -> f64 {
        if let (Some(first), Some(last)) = (self.snapshots.first(), self.snapshots.last()) {
            last.time - first.time
        } else {
            0.0
        }
    }

    /// Returns the number of snapshots.
    pub fn num_snapshots(&self) -> usize {
        self.snapshots.len()
    }

    /// Gets a snapshot at a specific time (or closest).
    pub fn snapshot_at_time(&self, time: f64) -> Option<&SimulationSnapshot> {
        if self.snapshots.is_empty() {
            return None;
        }

        // Binary search for closest time
        let mut closest_idx = 0;
        let mut min_diff = (self.snapshots[0].time - time).abs();

        for (i, snapshot) in self.snapshots.iter().enumerate() {
            let diff = (snapshot.time - time).abs();
            if diff < min_diff {
                min_diff = diff;
                closest_idx = i;
            }
        }

        Some(&self.snapshots[closest_idx])
    }
}

impl Default for SimulationRecording {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rigid_body_creation() {
        let body = RigidBody::new(1, 1500.0)
            .with_position(Point3::new(10.0, 0.0, 0.0))
            .with_velocity(Vector3::new(5.0, 0.0, 0.0));

        assert_eq!(body.id, 1);
        assert_eq!(body.mass, 1500.0);
        assert_eq!(body.state.position.x, 10.0);
        assert_eq!(body.state.velocity.x, 5.0);
    }

    #[test]
    fn test_simulation_state() {
        let mut state = SimulationState::new(0.01);

        let body = RigidBody::new(1, 1000.0);
        state.add_body(body);

        assert_eq!(state.num_bodies(), 1);
        assert!(state.get_body(1).is_some());

        state.remove_body(1);
        assert_eq!(state.num_bodies(), 0);
    }

    #[test]
    fn test_simulation_time() {
        let mut state = SimulationState::new(0.01);

        assert_eq!(state.time, 0.0);

        state.advance_time();
        assert!((state.time - 0.01).abs() < 1e-10);

        state.pause();
        state.advance_time();
        assert!((state.time - 0.01).abs() < 1e-10); // Still 0.01, didn't advance

        state.resume();
        state.advance_time();
        assert!((state.time - 0.02).abs() < 1e-10);
    }

    #[test]
    fn test_simulation_recording() {
        let mut recording = SimulationRecording::new();

        let snapshot1 = SimulationSnapshot {
            time: 0.0,
            body_states: HashMap::new(),
            collisions: Vec::new(),
        };

        let snapshot2 = SimulationSnapshot {
            time: 1.0,
            body_states: HashMap::new(),
            collisions: Vec::new(),
        };

        recording.add_snapshot(snapshot1);
        recording.add_snapshot(snapshot2);

        assert_eq!(recording.num_snapshots(), 2);
        assert_eq!(recording.duration(), 1.0);

        let snapshot = recording.snapshot_at_time(0.6);
        assert!(snapshot.is_some());
    }
}
