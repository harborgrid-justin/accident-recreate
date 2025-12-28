//! Accident scene representation
//!
//! This module defines the structure for representing complete
//! accident scenes with environmental conditions and involved entities.

use crate::error::{AccuSceneError, Result};
use crate::traits::{Identifiable, MemoryFootprint, Serializable, Timestamped, Validatable};
use crate::types::vector::Vector2D;
use crate::types::vehicle::Vehicle;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Weather conditions at time of accident
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeatherCondition {
    /// Clear skies
    Clear,
    /// Partly cloudy
    PartlyCloudy,
    /// Overcast/cloudy
    Cloudy,
    /// Light rain
    LightRain,
    /// Heavy rain
    HeavyRain,
    /// Fog/mist
    Fog,
    /// Snow
    Snow,
    /// Ice/freezing conditions
    Ice,
    /// High winds
    Windy,
    /// Unknown conditions
    Unknown,
}

impl WeatherCondition {
    /// Get typical visibility distance in meters
    pub fn visibility_distance(&self) -> f64 {
        match self {
            Self::Clear => 10000.0,
            Self::PartlyCloudy => 10000.0,
            Self::Cloudy => 8000.0,
            Self::LightRain => 5000.0,
            Self::HeavyRain => 1000.0,
            Self::Fog => 500.0,
            Self::Snow => 2000.0,
            Self::Ice => 5000.0,
            Self::Windy => 8000.0,
            Self::Unknown => 5000.0,
        }
    }

    /// Get road friction multiplier
    pub fn friction_multiplier(&self) -> f64 {
        match self {
            Self::Clear => 1.0,
            Self::PartlyCloudy => 1.0,
            Self::Cloudy => 1.0,
            Self::LightRain => 0.8,
            Self::HeavyRain => 0.6,
            Self::Fog => 0.9,
            Self::Snow => 0.4,
            Self::Ice => 0.2,
            Self::Windy => 0.95,
            Self::Unknown => 0.7,
        }
    }
}

/// Road surface conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoadCondition {
    /// Dry pavement
    Dry,
    /// Wet pavement
    Wet,
    /// Icy surface
    Icy,
    /// Snow covered
    Snowy,
    /// Gravel or unpaved
    Gravel,
    /// Dirt road
    Dirt,
    /// Under construction
    Construction,
    /// Damaged/potholed
    Damaged,
    /// Unknown condition
    Unknown,
}

impl RoadCondition {
    /// Get base friction coefficient
    pub fn friction_coefficient(&self) -> f64 {
        match self {
            Self::Dry => 0.8,
            Self::Wet => 0.6,
            Self::Icy => 0.2,
            Self::Snowy => 0.4,
            Self::Gravel => 0.7,
            Self::Dirt => 0.65,
            Self::Construction => 0.5,
            Self::Damaged => 0.7,
            Self::Unknown => 0.6,
        }
    }
}

/// Lighting conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LightingCondition {
    /// Bright daylight
    Daylight,
    /// Dawn or dusk
    Twilight,
    /// Dark with street lights
    DarkLit,
    /// Dark without street lights
    DarkUnlit,
    /// Unknown
    Unknown,
}

/// Traffic control present at scene
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrafficControl {
    /// No traffic control
    None,
    /// Stop sign
    StopSign,
    /// Yield sign
    YieldSign,
    /// Traffic signal (lights)
    TrafficSignal,
    /// Roundabout
    Roundabout,
    /// Pedestrian crossing
    PedestrianCrossing,
    /// Railroad crossing
    RailroadCrossing,
    /// Other control
    Other,
}

/// Complete accident scene representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccidentScene {
    /// Unique identifier
    pub id: String,

    /// Human-readable name/title
    pub name: String,

    /// Scene description
    pub description: Option<String>,

    /// Location (latitude, longitude)
    pub location: Option<(f64, f64)>,

    /// Address or location description
    pub address: Option<String>,

    /// Date and time of accident
    pub accident_time: DateTime<Utc>,

    /// Weather conditions
    pub weather: WeatherCondition,

    /// Road surface condition
    pub road_condition: RoadCondition,

    /// Lighting condition
    pub lighting: LightingCondition,

    /// Traffic control present
    pub traffic_control: TrafficControl,

    /// Temperature in Celsius
    pub temperature_c: Option<f64>,

    /// Speed limit at location (km/h)
    pub speed_limit_kmh: Option<f64>,

    /// Road gradient (slope) in percentage
    pub road_gradient: f64,

    /// Involved vehicles
    pub vehicles: Vec<Vehicle>,

    /// Scene dimensions (width, height) in meters
    pub scene_bounds: (f64, f64),

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl AccidentScene {
    /// Create a new accident scene
    pub fn new(name: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description: None,
            location: None,
            address: None,
            accident_time: now,
            weather: WeatherCondition::Unknown,
            road_condition: RoadCondition::Unknown,
            lighting: LightingCondition::Unknown,
            traffic_control: TrafficControl::None,
            temperature_c: None,
            speed_limit_kmh: None,
            road_gradient: 0.0,
            vehicles: Vec::new(),
            scene_bounds: (100.0, 100.0),
            created_at: now,
            updated_at: now,
        }
    }

    /// Add a vehicle to the scene
    pub fn add_vehicle(&mut self, vehicle: Vehicle) -> Result<()> {
        vehicle.validate()?;
        self.vehicles.push(vehicle);
        self.touch();
        Ok(())
    }

    /// Remove a vehicle by ID
    pub fn remove_vehicle(&mut self, vehicle_id: &str) -> Result<()> {
        let initial_len = self.vehicles.len();
        self.vehicles.retain(|v| v.id != vehicle_id);

        if self.vehicles.len() == initial_len {
            return Err(AccuSceneError::not_found("Vehicle", vehicle_id));
        }

        self.touch();
        Ok(())
    }

    /// Get a vehicle by ID
    pub fn get_vehicle(&self, vehicle_id: &str) -> Option<&Vehicle> {
        self.vehicles.iter().find(|v| v.id == vehicle_id)
    }

    /// Get a mutable reference to a vehicle by ID
    pub fn get_vehicle_mut(&mut self, vehicle_id: &str) -> Option<&mut Vehicle> {
        self.vehicles.iter_mut().find(|v| v.id == vehicle_id)
    }

    /// Get number of vehicles in scene
    pub fn vehicle_count(&self) -> usize {
        self.vehicles.len()
    }

    /// Calculate effective friction coefficient based on conditions
    pub fn effective_friction(&self) -> f64 {
        let base = self.road_condition.friction_coefficient();
        let weather_factor = self.weather.friction_multiplier();
        base * weather_factor
    }

    /// Get all stationary vehicles
    pub fn stationary_vehicles(&self) -> Vec<&Vehicle> {
        self.vehicles
            .iter()
            .filter(|v| v.is_stationary(0.1))
            .collect()
    }

    /// Get all moving vehicles
    pub fn moving_vehicles(&self) -> Vec<&Vehicle> {
        self.vehicles
            .iter()
            .filter(|v| !v.is_stationary(0.1))
            .collect()
    }

    /// Calculate total kinetic energy in the scene
    pub fn total_kinetic_energy(&self) -> f64 {
        self.vehicles.iter().map(|v| v.kinetic_energy()).sum()
    }

    /// Find vehicles near a point
    pub fn vehicles_near(&self, point: Vector2D, radius: f64) -> Vec<&Vehicle> {
        self.vehicles
            .iter()
            .filter(|v| v.position.distance(&point) <= radius)
            .collect()
    }

    /// Check if scene is within bounds
    pub fn is_within_bounds(&self, point: Vector2D) -> bool {
        point.x >= 0.0
            && point.x <= self.scene_bounds.0
            && point.y >= 0.0
            && point.y <= self.scene_bounds.1
    }

    /// Update all vehicle positions by time step
    pub fn step_simulation(&mut self, dt: f64) -> Result<()> {
        for vehicle in &mut self.vehicles {
            vehicle.update_position(dt);
        }
        self.touch();
        Ok(())
    }

    /// Get scene summary statistics
    pub fn statistics(&self) -> SceneStatistics {
        let total_mass: f64 = self.vehicles.iter().map(|v| v.mass_kg).sum();
        let avg_speed: f64 = if self.vehicles.is_empty() {
            0.0
        } else {
            self.vehicles.iter().map(|v| v.speed()).sum::<f64>() / self.vehicles.len() as f64
        };

        SceneStatistics {
            vehicle_count: self.vehicles.len(),
            total_mass_kg: total_mass,
            total_kinetic_energy_j: self.total_kinetic_energy(),
            average_speed_ms: avg_speed,
            stationary_count: self.stationary_vehicles().len(),
            moving_count: self.moving_vehicles().len(),
        }
    }
}

/// Scene statistics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneStatistics {
    /// Number of vehicles
    pub vehicle_count: usize,
    /// Total mass in kg
    pub total_mass_kg: f64,
    /// Total kinetic energy in Joules
    pub total_kinetic_energy_j: f64,
    /// Average speed in m/s
    pub average_speed_ms: f64,
    /// Number of stationary vehicles
    pub stationary_count: usize,
    /// Number of moving vehicles
    pub moving_count: usize,
}

/// Simplified accident type for quick reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Accident {
    /// Unique identifier
    pub id: String,
    /// Accident name/title
    pub name: String,
    /// Full accident scene
    pub scene: AccidentScene,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl Accident {
    /// Create a new accident
    pub fn new(name: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.clone(),
            scene: AccidentScene::new(name),
            created_at: now,
            updated_at: now,
        }
    }
}

impl Identifiable for AccidentScene {
    type Id = String;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn set_id(&mut self, id: Self::Id) {
        self.id = id;
        self.touch();
    }

    fn with_new_id(mut self) -> Self {
        self.id = Uuid::new_v4().to_string();
        self
    }
}

impl Timestamped for AccidentScene {
    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}

impl Validatable for AccidentScene {
    fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(AccuSceneError::validation_field(
                "Scene name cannot be empty",
                "name",
            ));
        }

        if self.scene_bounds.0 <= 0.0 || self.scene_bounds.1 <= 0.0 {
            return Err(AccuSceneError::validation_field(
                "Scene bounds must be positive",
                "scene_bounds",
            ));
        }

        if self.road_gradient.abs() > 30.0 {
            return Err(AccuSceneError::validation_field(
                "Road gradient must be between -30% and +30%",
                "road_gradient",
            ));
        }

        for vehicle in &self.vehicles {
            vehicle.validate()?;
        }

        Ok(())
    }
}

impl Serializable for AccidentScene {}
impl Serializable for Accident {}
impl Serializable for SceneStatistics {}

impl MemoryFootprint for AccidentScene {
    fn memory_footprint(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.name.capacity()
            + self.description.as_ref().map(|s| s.capacity()).unwrap_or(0)
            + self.address.as_ref().map(|s| s.capacity()).unwrap_or(0)
            + self.vehicles.iter().map(|v| v.memory_footprint()).sum::<usize>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::vehicle::VehicleCategory;

    #[test]
    fn test_accident_scene_creation() {
        let scene = AccidentScene::new("Test Accident".to_string());
        assert!(scene.validate().is_ok());
        assert_eq!(scene.vehicle_count(), 0);
    }

    #[test]
    fn test_add_remove_vehicle() {
        let mut scene = AccidentScene::new("Test".to_string());
        let vehicle = Vehicle::new(VehicleCategory::Car);
        let vehicle_id = vehicle.id.clone();

        scene.add_vehicle(vehicle).unwrap();
        assert_eq!(scene.vehicle_count(), 1);

        scene.remove_vehicle(&vehicle_id).unwrap();
        assert_eq!(scene.vehicle_count(), 0);
    }

    #[test]
    fn test_effective_friction() {
        let mut scene = AccidentScene::new("Test".to_string());
        scene.road_condition = RoadCondition::Wet;
        scene.weather = WeatherCondition::HeavyRain;

        let friction = scene.effective_friction();
        assert!(friction < 0.6);
    }

    #[test]
    fn test_scene_statistics() {
        let mut scene = AccidentScene::new("Test".to_string());
        let mut vehicle = Vehicle::new(VehicleCategory::Car);
        vehicle.velocity = Vector2D::new(20.0, 0.0);

        scene.add_vehicle(vehicle).unwrap();

        let stats = scene.statistics();
        assert_eq!(stats.vehicle_count, 1);
        assert!(stats.total_kinetic_energy_j > 0.0);
    }
}
