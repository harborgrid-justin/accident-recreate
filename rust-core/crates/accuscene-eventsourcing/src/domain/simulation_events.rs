//! Simulation-related events for physics-based accident reconstruction.

use crate::event::Event;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Simulation started event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SimulationStarted {
    /// Simulation identifier.
    pub simulation_id: String,

    /// Associated scene identifier.
    pub scene_id: String,

    /// Simulation type.
    pub simulation_type: SimulationType,

    /// Physics parameters.
    pub physics_params: PhysicsParameters,

    /// Started by.
    pub started_by: String,

    /// Timestamp when started.
    pub started_at: DateTime<Utc>,

    /// Simulation configuration.
    pub config: HashMap<String, String>,
}

impl Event for SimulationStarted {
    fn event_type(&self) -> &'static str {
        "SimulationStarted"
    }

    fn aggregate_id(&self) -> &str {
        &self.simulation_id
    }

    fn aggregate_type(&self) -> &'static str {
        "Simulation"
    }
}

/// Simulation progressed event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SimulationProgressed {
    /// Simulation identifier.
    pub simulation_id: String,

    /// Current simulation time.
    pub current_time: f64,

    /// Total simulation duration.
    pub total_duration: f64,

    /// Progress percentage (0-100).
    pub progress_percent: f32,

    /// Number of physics steps completed.
    pub steps_completed: u64,

    /// Timestamp when progressed.
    pub progressed_at: DateTime<Utc>,
}

impl Event for SimulationProgressed {
    fn event_type(&self) -> &'static str {
        "SimulationProgressed"
    }

    fn aggregate_id(&self) -> &str {
        &self.simulation_id
    }

    fn aggregate_type(&self) -> &'static str {
        "Simulation"
    }
}

/// Collision detected event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CollisionDetected {
    /// Simulation identifier.
    pub simulation_id: String,

    /// Collision identifier.
    pub collision_id: String,

    /// First entity involved.
    pub entity1_id: String,

    /// Second entity involved.
    pub entity2_id: String,

    /// Collision point.
    pub collision_point: CollisionPoint,

    /// Impact force magnitude.
    pub impact_force: f64,

    /// Collision normal vector.
    pub collision_normal: Vector3D,

    /// Simulation time when collision occurred.
    pub collision_time: f64,

    /// Detected at timestamp.
    pub detected_at: DateTime<Utc>,

    /// Collision metadata.
    pub metadata: HashMap<String, String>,
}

impl Event for CollisionDetected {
    fn event_type(&self) -> &'static str {
        "CollisionDetected"
    }

    fn aggregate_id(&self) -> &str {
        &self.simulation_id
    }

    fn aggregate_type(&self) -> &'static str {
        "Simulation"
    }
}

/// Simulation paused event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SimulationPaused {
    /// Simulation identifier.
    pub simulation_id: String,

    /// Current simulation time.
    pub current_time: f64,

    /// Paused by.
    pub paused_by: String,

    /// Timestamp when paused.
    pub paused_at: DateTime<Utc>,

    /// Reason for pausing.
    pub reason: Option<String>,
}

impl Event for SimulationPaused {
    fn event_type(&self) -> &'static str {
        "SimulationPaused"
    }

    fn aggregate_id(&self) -> &str {
        &self.simulation_id
    }

    fn aggregate_type(&self) -> &'static str {
        "Simulation"
    }
}

/// Simulation resumed event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SimulationResumed {
    /// Simulation identifier.
    pub simulation_id: String,

    /// Current simulation time.
    pub current_time: f64,

    /// Resumed by.
    pub resumed_by: String,

    /// Timestamp when resumed.
    pub resumed_at: DateTime<Utc>,
}

impl Event for SimulationResumed {
    fn event_type(&self) -> &'static str {
        "SimulationResumed"
    }

    fn aggregate_id(&self) -> &str {
        &self.simulation_id
    }

    fn aggregate_type(&self) -> &'static str {
        "Simulation"
    }
}

/// Simulation completed event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SimulationCompleted {
    /// Simulation identifier.
    pub simulation_id: String,

    /// Final simulation time.
    pub final_time: f64,

    /// Total steps executed.
    pub total_steps: u64,

    /// Execution time in milliseconds.
    pub execution_time_ms: u64,

    /// Number of collisions detected.
    pub collision_count: usize,

    /// Simulation results summary.
    pub results: SimulationResults,

    /// Completed at timestamp.
    pub completed_at: DateTime<Utc>,
}

impl Event for SimulationCompleted {
    fn event_type(&self) -> &'static str {
        "SimulationCompleted"
    }

    fn aggregate_id(&self) -> &str {
        &self.simulation_id
    }

    fn aggregate_type(&self) -> &'static str {
        "Simulation"
    }
}

/// Simulation failed event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SimulationFailed {
    /// Simulation identifier.
    pub simulation_id: String,

    /// Error message.
    pub error: String,

    /// Error code.
    pub error_code: Option<String>,

    /// Current simulation time when failed.
    pub failed_at_time: f64,

    /// Failed at timestamp.
    pub failed_at: DateTime<Utc>,

    /// Stack trace or additional debug info.
    pub debug_info: Option<String>,
}

impl Event for SimulationFailed {
    fn event_type(&self) -> &'static str {
        "SimulationFailed"
    }

    fn aggregate_id(&self) -> &str {
        &self.simulation_id
    }

    fn aggregate_type(&self) -> &'static str {
        "Simulation"
    }
}

/// Simulation reset event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SimulationReset {
    /// Simulation identifier.
    pub simulation_id: String,

    /// Reset by.
    pub reset_by: String,

    /// Timestamp when reset.
    pub reset_at: DateTime<Utc>,

    /// Reason for reset.
    pub reason: Option<String>,
}

impl Event for SimulationReset {
    fn event_type(&self) -> &'static str {
        "SimulationReset"
    }

    fn aggregate_id(&self) -> &str {
        &self.simulation_id
    }

    fn aggregate_type(&self) -> &'static str {
        "Simulation"
    }
}

/// Entity state updated event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EntityStateUpdated {
    /// Simulation identifier.
    pub simulation_id: String,

    /// Entity identifier.
    pub entity_id: String,

    /// Simulation time.
    pub time: f64,

    /// Entity state.
    pub state: EntityState,

    /// Updated at timestamp.
    pub updated_at: DateTime<Utc>,
}

impl Event for EntityStateUpdated {
    fn event_type(&self) -> &'static str {
        "EntityStateUpdated"
    }

    fn aggregate_id(&self) -> &str {
        &self.simulation_id
    }

    fn aggregate_type(&self) -> &'static str {
        "Simulation"
    }
}

/// Simulation type enumeration.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SimulationType {
    /// Forward simulation from initial conditions.
    Forward,

    /// Backward simulation from final state.
    Backward,

    /// Monte Carlo simulation with variations.
    MonteCarlo,

    /// Real-time interactive simulation.
    Interactive,
}

/// Physics parameters.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PhysicsParameters {
    /// Gravity magnitude (m/s²).
    pub gravity: f64,

    /// Time step size (seconds).
    pub time_step: f64,

    /// Air density (kg/m³).
    pub air_density: f64,

    /// Road friction coefficient.
    pub road_friction: f64,

    /// Enable collision detection.
    pub collision_detection: bool,

    /// Collision detection threshold.
    pub collision_threshold: f64,
}

impl Default for PhysicsParameters {
    fn default() -> Self {
        Self {
            gravity: 9.81,
            time_step: 0.016,
            air_density: 1.225,
            road_friction: 0.7,
            collision_detection: true,
            collision_threshold: 0.01,
        }
    }
}

/// Collision point information.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CollisionPoint {
    /// 3D position of collision.
    pub position: Vector3D,

    /// Depth of penetration.
    pub penetration_depth: f64,

    /// Contact area.
    pub contact_area: Option<f64>,
}

/// 3D vector.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Vector3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// Simulation results summary.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SimulationResults {
    /// Maximum velocity reached.
    pub max_velocity: f64,

    /// Maximum acceleration.
    pub max_acceleration: f64,

    /// Total energy dissipated.
    pub energy_dissipated: f64,

    /// Average frame rate.
    pub avg_frame_rate: f64,

    /// Additional metrics.
    pub metrics: HashMap<String, f64>,
}

/// Entity state in simulation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EntityState {
    /// Position.
    pub position: Vector3D,

    /// Velocity.
    pub velocity: Vector3D,

    /// Acceleration.
    pub acceleration: Vector3D,

    /// Rotation (quaternion).
    pub rotation: Quaternion,

    /// Angular velocity.
    pub angular_velocity: Vector3D,

    /// Mass.
    pub mass: f64,
}

/// Quaternion for rotation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Quaternion {
    pub w: f64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulation_started_event() {
        let event = SimulationStarted {
            simulation_id: "sim-123".to_string(),
            scene_id: "scene-456".to_string(),
            simulation_type: SimulationType::Forward,
            physics_params: PhysicsParameters::default(),
            started_by: "user-789".to_string(),
            started_at: Utc::now(),
            config: HashMap::new(),
        };

        assert_eq!(event.event_type(), "SimulationStarted");
        assert_eq!(event.aggregate_id(), "sim-123");
    }

    #[test]
    fn test_collision_detected_event() {
        let event = CollisionDetected {
            simulation_id: "sim-123".to_string(),
            collision_id: "collision-1".to_string(),
            entity1_id: "vehicle-1".to_string(),
            entity2_id: "vehicle-2".to_string(),
            collision_point: CollisionPoint {
                position: Vector3D {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                penetration_depth: 0.1,
                contact_area: Some(0.5),
            },
            impact_force: 15000.0,
            collision_normal: Vector3D {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            collision_time: 2.5,
            detected_at: Utc::now(),
            metadata: HashMap::new(),
        };

        assert_eq!(event.event_type(), "CollisionDetected");
    }
}
