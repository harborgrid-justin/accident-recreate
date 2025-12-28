//! Vehicle types and physics properties
//!
//! This module defines vehicle structures with complete physics
//! properties for accurate accident reconstruction.

use crate::error::{AccuSceneError, Result};
use crate::traits::{Identifiable, MemoryFootprint, Serializable, Timestamped, Validatable};
use crate::types::vector::Vector2D;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Vehicle category classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VehicleCategory {
    /// Passenger car
    Car,
    /// Sport utility vehicle
    SUV,
    /// Pickup truck
    Truck,
    /// Motorcycle or scooter
    Motorcycle,
    /// Commercial van
    Van,
    /// Heavy commercial vehicle
    Commercial,
    /// Bus
    Bus,
    /// Bicycle
    Bicycle,
    /// Pedestrian (treated as mobile object)
    Pedestrian,
    /// Other/unknown category
    Other,
}

impl VehicleCategory {
    /// Get typical mass range for category (min, max) in kg
    pub fn typical_mass_range(&self) -> (f64, f64) {
        match self {
            Self::Car => (1000.0, 2000.0),
            Self::SUV => (1800.0, 3000.0),
            Self::Truck => (1500.0, 3500.0),
            Self::Motorcycle => (150.0, 400.0),
            Self::Van => (1500.0, 2500.0),
            Self::Commercial => (3500.0, 12000.0),
            Self::Bus => (8000.0, 18000.0),
            Self::Bicycle => (10.0, 25.0),
            Self::Pedestrian => (50.0, 120.0),
            Self::Other => (500.0, 5000.0),
        }
    }

    /// Get typical dimensions (length, width, height) in meters
    pub fn typical_dimensions(&self) -> (f64, f64, f64) {
        match self {
            Self::Car => (4.5, 1.8, 1.5),
            Self::SUV => (4.8, 2.0, 1.8),
            Self::Truck => (5.5, 2.0, 1.8),
            Self::Motorcycle => (2.2, 0.8, 1.2),
            Self::Van => (5.0, 2.0, 2.0),
            Self::Commercial => (8.0, 2.5, 3.0),
            Self::Bus => (12.0, 2.5, 3.5),
            Self::Bicycle => (1.8, 0.6, 1.1),
            Self::Pedestrian => (0.6, 0.5, 1.7),
            Self::Other => (4.0, 1.8, 1.5),
        }
    }
}

/// Vehicle metadata and description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleMetadata {
    /// Vehicle make (manufacturer)
    pub make: String,
    /// Vehicle model
    pub model: String,
    /// Model year
    pub year: Option<u16>,
    /// Vehicle color
    pub color: Option<String>,
    /// License plate number
    pub license_plate: Option<String>,
    /// VIN (Vehicle Identification Number)
    pub vin: Option<String>,
    /// Additional notes
    pub notes: Option<String>,
}

impl Default for VehicleMetadata {
    fn default() -> Self {
        Self {
            make: "Unknown".to_string(),
            model: "Unknown".to_string(),
            year: None,
            color: None,
            license_plate: None,
            vin: None,
            notes: None,
        }
    }
}

/// Complete vehicle representation with physics properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vehicle {
    /// Unique identifier
    pub id: String,

    /// Vehicle category
    pub category: VehicleCategory,

    /// Vehicle metadata
    pub metadata: VehicleMetadata,

    /// Mass in kilograms
    pub mass_kg: f64,

    /// Dimensions: length in meters
    pub length_m: f64,

    /// Dimensions: width in meters
    pub width_m: f64,

    /// Dimensions: height in meters
    pub height_m: f64,

    /// Position in 2D space (meters)
    pub position: Vector2D,

    /// Velocity vector (m/s)
    pub velocity: Vector2D,

    /// Acceleration vector (m/s²)
    pub acceleration: Vector2D,

    /// Rotation angle in radians
    pub rotation: f64,

    /// Angular velocity in radians/second
    pub angular_velocity: f64,

    /// Coefficient of friction
    pub friction_coefficient: f64,

    /// Coefficient of restitution (bounciness)
    pub restitution_coefficient: f64,

    /// Damage level (0.0 = none, 1.0 = total)
    pub damage_level: f64,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl Vehicle {
    /// Create a new vehicle with default physics properties
    pub fn new(category: VehicleCategory) -> Self {
        let (min_mass, max_mass) = category.typical_mass_range();
        let typical_mass = (min_mass + max_mass) / 2.0;
        let (length, width, height) = category.typical_dimensions();

        let now = Utc::now();

        Self {
            id: Uuid::new_v4().to_string(),
            category,
            metadata: VehicleMetadata::default(),
            mass_kg: typical_mass,
            length_m: length,
            width_m: width,
            height_m: height,
            position: Vector2D::zero(),
            velocity: Vector2D::zero(),
            acceleration: Vector2D::zero(),
            rotation: 0.0,
            angular_velocity: 0.0,
            friction_coefficient: 0.7,
            restitution_coefficient: 0.3,
            damage_level: 0.0,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a new vehicle with custom metadata
    pub fn with_metadata(category: VehicleCategory, metadata: VehicleMetadata) -> Self {
        let mut vehicle = Self::new(category);
        vehicle.metadata = metadata;
        vehicle
    }

    /// Calculate kinetic energy (Joules)
    pub fn kinetic_energy(&self) -> f64 {
        0.5 * self.mass_kg * self.velocity.magnitude_squared()
    }

    /// Calculate momentum (kg⋅m/s)
    pub fn momentum(&self) -> Vector2D {
        self.velocity * self.mass_kg
    }

    /// Calculate rotational kinetic energy
    pub fn rotational_energy(&self) -> f64 {
        // Using simplified moment of inertia for a rectangular box
        let moment = (self.mass_kg / 12.0)
            * (self.length_m * self.length_m + self.width_m * self.width_m);
        0.5 * moment * self.angular_velocity * self.angular_velocity
    }

    /// Get current speed in m/s
    pub fn speed(&self) -> f64 {
        self.velocity.magnitude()
    }

    /// Get current speed in km/h
    pub fn speed_kmh(&self) -> f64 {
        self.speed() * 3.6
    }

    /// Get current speed in mph
    pub fn speed_mph(&self) -> f64 {
        self.speed() * 2.236936
    }

    /// Set velocity from speed and direction angle
    pub fn set_velocity_polar(&mut self, speed_ms: f64, angle_rad: f64) {
        self.velocity = Vector2D::from_polar(speed_ms, angle_rad);
        self.touch();
    }

    /// Apply force to the vehicle
    pub fn apply_force(&mut self, force: Vector2D, dt: f64) {
        // F = ma, so a = F/m
        let acceleration = force / self.mass_kg;
        self.acceleration += acceleration;
        self.velocity += acceleration * dt;
        self.touch();
    }

    /// Apply braking force
    pub fn apply_braking(&mut self, brake_force: f64, dt: f64) {
        if self.velocity.magnitude() < 0.01 {
            self.velocity = Vector2D::zero();
            return;
        }

        let brake_direction = self.velocity.normalize_or_zero() * -1.0;
        let brake_vector = brake_direction * brake_force;
        self.apply_force(brake_vector, dt);
        self.touch();
    }

    /// Update position based on current velocity
    pub fn update_position(&mut self, dt: f64) {
        self.position += self.velocity * dt;
        self.rotation += self.angular_velocity * dt;
        self.touch();
    }

    /// Get bounding box corners in world space
    pub fn bounding_box(&self) -> [Vector2D; 4] {
        let half_length = self.length_m / 2.0;
        let half_width = self.width_m / 2.0;

        let corners = [
            Vector2D::new(half_length, half_width),
            Vector2D::new(-half_length, half_width),
            Vector2D::new(-half_length, -half_width),
            Vector2D::new(half_length, -half_width),
        ];

        corners.map(|c| {
            let rotated = c.rotate(self.rotation);
            self.position + rotated
        })
    }

    /// Check if point is inside vehicle bounds
    pub fn contains_point(&self, point: Vector2D) -> bool {
        let local = point - self.position;
        let rotated = local.rotate(-self.rotation);

        let half_length = self.length_m / 2.0;
        let half_width = self.width_m / 2.0;

        rotated.x.abs() <= half_length && rotated.y.abs() <= half_width
    }

    /// Calculate stopping distance at current speed
    pub fn stopping_distance(&self, deceleration_ms2: f64) -> Result<f64> {
        if deceleration_ms2 <= 0.0 {
            return Err(AccuSceneError::physics(
                "Deceleration must be positive for stopping distance calculation",
            ));
        }

        let speed = self.speed();
        Ok((speed * speed) / (2.0 * deceleration_ms2))
    }

    /// Set damage level
    pub fn set_damage(&mut self, level: f64) {
        self.damage_level = level.clamp(0.0, 1.0);
        self.touch();
    }

    /// Check if vehicle is stationary
    pub fn is_stationary(&self, threshold: f64) -> bool {
        self.velocity.magnitude() < threshold && self.angular_velocity.abs() < threshold
    }
}

impl Identifiable for Vehicle {
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

impl Timestamped for Vehicle {
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

impl Validatable for Vehicle {
    fn validate(&self) -> Result<()> {
        if self.mass_kg <= 0.0 {
            return Err(AccuSceneError::validation_field(
                "Mass must be positive",
                "mass_kg",
            ));
        }

        if self.length_m <= 0.0 || self.width_m <= 0.0 || self.height_m <= 0.0 {
            return Err(AccuSceneError::validation_field(
                "Dimensions must be positive",
                "dimensions",
            ));
        }

        if self.friction_coefficient < 0.0 || self.friction_coefficient > 1.0 {
            return Err(AccuSceneError::validation_field(
                "Friction coefficient must be between 0 and 1",
                "friction_coefficient",
            ));
        }

        if self.restitution_coefficient < 0.0 || self.restitution_coefficient > 1.0 {
            return Err(AccuSceneError::validation_field(
                "Restitution coefficient must be between 0 and 1",
                "restitution_coefficient",
            ));
        }

        if self.damage_level < 0.0 || self.damage_level > 1.0 {
            return Err(AccuSceneError::validation_field(
                "Damage level must be between 0 and 1",
                "damage_level",
            ));
        }

        self.position.validate()?;
        self.velocity.validate()?;
        self.acceleration.validate()?;

        Ok(())
    }
}

impl Serializable for Vehicle {}

impl MemoryFootprint for Vehicle {
    fn memory_footprint(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.metadata.make.capacity()
            + self.metadata.model.capacity()
            + self
                .metadata
                .color
                .as_ref()
                .map(|s| s.capacity())
                .unwrap_or(0)
            + self
                .metadata
                .license_plate
                .as_ref()
                .map(|s| s.capacity())
                .unwrap_or(0)
            + self.metadata.vin.as_ref().map(|s| s.capacity()).unwrap_or(0)
            + self
                .metadata
                .notes
                .as_ref()
                .map(|s| s.capacity())
                .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vehicle_creation() {
        let vehicle = Vehicle::new(VehicleCategory::Car);
        assert!(vehicle.validate().is_ok());
        assert_eq!(vehicle.category, VehicleCategory::Car);
    }

    #[test]
    fn test_vehicle_physics() {
        let mut vehicle = Vehicle::new(VehicleCategory::Car);
        vehicle.velocity = Vector2D::new(20.0, 0.0); // 20 m/s

        let ke = vehicle.kinetic_energy();
        assert!(ke > 0.0);

        let momentum = vehicle.momentum();
        assert_eq!(momentum.x, vehicle.mass_kg * 20.0);
    }

    #[test]
    fn test_vehicle_braking() {
        let mut vehicle = Vehicle::new(VehicleCategory::Car);
        vehicle.velocity = Vector2D::new(20.0, 0.0);

        let initial_speed = vehicle.speed();
        vehicle.apply_braking(5000.0, 0.1);

        assert!(vehicle.speed() < initial_speed);
    }

    #[test]
    fn test_vehicle_bounding_box() {
        let vehicle = Vehicle::new(VehicleCategory::Car);
        let bbox = vehicle.bounding_box();
        assert_eq!(bbox.len(), 4);
    }

    #[test]
    fn test_stopping_distance() {
        let mut vehicle = Vehicle::new(VehicleCategory::Car);
        vehicle.velocity = Vector2D::new(20.0, 0.0); // 20 m/s (~72 km/h)

        let distance = vehicle.stopping_distance(5.0).unwrap();
        assert!(distance > 0.0);
        // v² = u² + 2as, so s = v²/2a = 400/10 = 40m
        assert!((distance - 40.0).abs() < 0.01);
    }
}
