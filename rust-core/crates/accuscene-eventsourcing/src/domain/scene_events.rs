//! Scene-related events for 3D accident reconstruction.

use crate::event::Event;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Scene created event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SceneCreated {
    /// Scene identifier.
    pub scene_id: String,

    /// Associated case identifier.
    pub case_id: String,

    /// Scene name.
    pub name: String,

    /// Scene description.
    pub description: Option<String>,

    /// Location information.
    pub location: Option<Location>,

    /// Created by.
    pub created_by: String,

    /// Timestamp when created.
    pub created_at: DateTime<Utc>,

    /// Scene metadata.
    pub metadata: HashMap<String, String>,
}

impl Event for SceneCreated {
    fn event_type(&self) -> &'static str {
        "SceneCreated"
    }

    fn aggregate_id(&self) -> &str {
        &self.scene_id
    }

    fn aggregate_type(&self) -> &'static str {
        "Scene"
    }
}

/// Vehicle placed in scene event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VehiclePlaced {
    /// Scene identifier.
    pub scene_id: String,

    /// Vehicle identifier.
    pub vehicle_id: String,

    /// Vehicle make and model.
    pub make_model: String,

    /// Vehicle year.
    pub year: Option<u16>,

    /// Vehicle color.
    pub color: Option<String>,

    /// Initial position in scene.
    pub position: Position3D,

    /// Initial rotation.
    pub rotation: Rotation3D,

    /// Vehicle dimensions.
    pub dimensions: Option<Dimensions3D>,

    /// Placed by.
    pub placed_by: String,

    /// Timestamp when placed.
    pub placed_at: DateTime<Utc>,
}

impl Event for VehiclePlaced {
    fn event_type(&self) -> &'static str {
        "VehiclePlaced"
    }

    fn aggregate_id(&self) -> &str {
        &self.scene_id
    }

    fn aggregate_type(&self) -> &'static str {
        "Scene"
    }
}

/// Vehicle moved event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VehicleMoved {
    /// Scene identifier.
    pub scene_id: String,

    /// Vehicle identifier.
    pub vehicle_id: String,

    /// Previous position.
    pub old_position: Position3D,

    /// New position.
    pub new_position: Position3D,

    /// Previous rotation.
    pub old_rotation: Rotation3D,

    /// New rotation.
    pub new_rotation: Rotation3D,

    /// Moved by.
    pub moved_by: String,

    /// Timestamp when moved.
    pub moved_at: DateTime<Utc>,
}

impl Event for VehicleMoved {
    fn event_type(&self) -> &'static str {
        "VehicleMoved"
    }

    fn aggregate_id(&self) -> &str {
        &self.scene_id
    }

    fn aggregate_type(&self) -> &'static str {
        "Scene"
    }
}

/// Vehicle removed event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VehicleRemoved {
    /// Scene identifier.
    pub scene_id: String,

    /// Vehicle identifier.
    pub vehicle_id: String,

    /// Removed by.
    pub removed_by: String,

    /// Timestamp when removed.
    pub removed_at: DateTime<Utc>,
}

impl Event for VehicleRemoved {
    fn event_type(&self) -> &'static str {
        "VehicleRemoved"
    }

    fn aggregate_id(&self) -> &str {
        &self.scene_id
    }

    fn aggregate_type(&self) -> &'static str {
        "Scene"
    }
}

/// Object placed in scene event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ObjectPlaced {
    /// Scene identifier.
    pub scene_id: String,

    /// Object identifier.
    pub object_id: String,

    /// Object type.
    pub object_type: ObjectType,

    /// Object description.
    pub description: Option<String>,

    /// Position in scene.
    pub position: Position3D,

    /// Rotation.
    pub rotation: Rotation3D,

    /// Scale.
    pub scale: Option<Scale3D>,

    /// Placed by.
    pub placed_by: String,

    /// Timestamp when placed.
    pub placed_at: DateTime<Utc>,
}

impl Event for ObjectPlaced {
    fn event_type(&self) -> &'static str {
        "ObjectPlaced"
    }

    fn aggregate_id(&self) -> &str {
        &self.scene_id
    }

    fn aggregate_type(&self) -> &'static str {
        "Scene"
    }
}

/// Trajectory created event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrajectoryCreated {
    /// Scene identifier.
    pub scene_id: String,

    /// Trajectory identifier.
    pub trajectory_id: String,

    /// Vehicle or object identifier.
    pub entity_id: String,

    /// Waypoints defining the trajectory.
    pub waypoints: Vec<Waypoint>,

    /// Created by.
    pub created_by: String,

    /// Timestamp when created.
    pub created_at: DateTime<Utc>,
}

impl Event for TrajectoryCreated {
    fn event_type(&self) -> &'static str {
        "TrajectoryCreated"
    }

    fn aggregate_id(&self) -> &str {
        &self.scene_id
    }

    fn aggregate_type(&self) -> &'static str {
        "Scene"
    }
}

/// Trajectory updated event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrajectoryUpdated {
    /// Scene identifier.
    pub scene_id: String,

    /// Trajectory identifier.
    pub trajectory_id: String,

    /// Updated waypoints.
    pub waypoints: Vec<Waypoint>,

    /// Updated by.
    pub updated_by: String,

    /// Timestamp when updated.
    pub updated_at: DateTime<Utc>,
}

impl Event for TrajectoryUpdated {
    fn event_type(&self) -> &'static str {
        "TrajectoryUpdated"
    }

    fn aggregate_id(&self) -> &str {
        &self.scene_id
    }

    fn aggregate_type(&self) -> &'static str {
        "Scene"
    }
}

/// Measurement added event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MeasurementAdded {
    /// Scene identifier.
    pub scene_id: String,

    /// Measurement identifier.
    pub measurement_id: String,

    /// Measurement type.
    pub measurement_type: MeasurementType,

    /// Start point.
    pub start_point: Position3D,

    /// End point.
    pub end_point: Position3D,

    /// Measured value.
    pub value: f64,

    /// Unit of measurement.
    pub unit: String,

    /// Added by.
    pub added_by: String,

    /// Timestamp when added.
    pub added_at: DateTime<Utc>,
}

impl Event for MeasurementAdded {
    fn event_type(&self) -> &'static str {
        "MeasurementAdded"
    }

    fn aggregate_id(&self) -> &str {
        &self.scene_id
    }

    fn aggregate_type(&self) -> &'static str {
        "Scene"
    }
}

/// Scene snapshot saved event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SceneSnapshotSaved {
    /// Scene identifier.
    pub scene_id: String,

    /// Snapshot identifier.
    pub snapshot_id: String,

    /// Snapshot name.
    pub name: String,

    /// Snapshot description.
    pub description: Option<String>,

    /// Camera position and settings.
    pub camera_state: Option<CameraState>,

    /// Saved by.
    pub saved_by: String,

    /// Timestamp when saved.
    pub saved_at: DateTime<Utc>,
}

impl Event for SceneSnapshotSaved {
    fn event_type(&self) -> &'static str {
        "SceneSnapshotSaved"
    }

    fn aggregate_id(&self) -> &str {
        &self.scene_id
    }

    fn aggregate_type(&self) -> &'static str {
        "Scene"
    }
}

/// 3D position.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Position3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// 3D rotation (Euler angles).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Rotation3D {
    pub pitch: f64,
    pub yaw: f64,
    pub roll: f64,
}

/// 3D dimensions.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Dimensions3D {
    pub length: f64,
    pub width: f64,
    pub height: f64,
}

/// 3D scale.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Scale3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// Location information.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Location {
    /// Street address.
    pub address: Option<String>,

    /// City.
    pub city: Option<String>,

    /// State or province.
    pub state: Option<String>,

    /// Postal code.
    pub postal_code: Option<String>,

    /// Country.
    pub country: Option<String>,

    /// Latitude.
    pub latitude: Option<f64>,

    /// Longitude.
    pub longitude: Option<f64>,
}

/// Waypoint for trajectory.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Waypoint {
    /// Position.
    pub position: Position3D,

    /// Rotation at this point.
    pub rotation: Option<Rotation3D>,

    /// Timestamp or time offset.
    pub time: f64,

    /// Velocity at this point.
    pub velocity: Option<f64>,
}

/// Object type enumeration.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ObjectType {
    /// Traffic sign.
    TrafficSign,

    /// Traffic light.
    TrafficLight,

    /// Road marking.
    RoadMarking,

    /// Debris.
    Debris,

    /// Pedestrian.
    Pedestrian,

    /// Barrier.
    Barrier,

    /// Tree or vegetation.
    Vegetation,

    /// Building.
    Building,

    /// Custom object.
    Custom,
}

/// Measurement type enumeration.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MeasurementType {
    /// Distance measurement.
    Distance,

    /// Height measurement.
    Height,

    /// Angle measurement.
    Angle,

    /// Area measurement.
    Area,

    /// Volume measurement.
    Volume,
}

/// Camera state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CameraState {
    /// Camera position.
    pub position: Position3D,

    /// Look-at target.
    pub target: Position3D,

    /// Field of view.
    pub fov: f64,

    /// Near clip plane.
    pub near: f64,

    /// Far clip plane.
    pub far: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scene_created_event() {
        let event = SceneCreated {
            scene_id: "scene-123".to_string(),
            case_id: "case-456".to_string(),
            name: "Intersection Scene".to_string(),
            description: Some("4-way intersection".to_string()),
            location: None,
            created_by: "user-789".to_string(),
            created_at: Utc::now(),
            metadata: HashMap::new(),
        };

        assert_eq!(event.event_type(), "SceneCreated");
        assert_eq!(event.aggregate_id(), "scene-123");
    }

    #[test]
    fn test_vehicle_placed_event() {
        let event = VehiclePlaced {
            scene_id: "scene-123".to_string(),
            vehicle_id: "vehicle-1".to_string(),
            make_model: "Toyota Camry".to_string(),
            year: Some(2020),
            color: Some("Silver".to_string()),
            position: Position3D {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            rotation: Rotation3D {
                pitch: 0.0,
                yaw: 0.0,
                roll: 0.0,
            },
            dimensions: None,
            placed_by: "user-789".to_string(),
            placed_at: Utc::now(),
        };

        assert_eq!(event.event_type(), "VehiclePlaced");
    }
}
