//! Collision detection and response module.
//!
//! Provides comprehensive collision detection using multiple algorithms:
//! - Broad-phase: AABB and spatial hashing
//! - Narrow-phase: GJK and SAT algorithms
//! - Collision response: Impulse-based resolution

pub mod detector;
pub mod gjk;
pub mod resolver;
pub mod sat;

pub use detector::{BroadPhaseDetector, CollisionDetector, NarrowPhaseDetector};
pub use gjk::GjkAlgorithm;
pub use resolver::{CollisionResolver, ImpulseResponse};
pub use sat::SatAlgorithm;

use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};

/// Represents a collision between two objects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collision {
    /// ID of the first object involved in the collision
    pub object_a: u64,
    /// ID of the second object involved in the collision
    pub object_b: u64,
    /// Point of collision in world space
    pub point: Point3<f64>,
    /// Normal vector at the collision point (from A to B)
    pub normal: Vector3<f64>,
    /// Penetration depth
    pub penetration: f64,
    /// Time of impact (0.0 = start of timestep, 1.0 = end of timestep)
    pub time_of_impact: f64,
    /// Relative velocity at collision point
    pub relative_velocity: Vector3<f64>,
}

/// Axis-Aligned Bounding Box for broad-phase collision detection.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Aabb {
    pub min: Point3<f64>,
    pub max: Point3<f64>,
}

impl Aabb {
    /// Creates a new AABB from min and max points.
    pub fn new(min: Point3<f64>, max: Point3<f64>) -> Self {
        Self { min, max }
    }

    /// Creates an AABB from a center point and half-extents.
    pub fn from_center_and_extents(center: Point3<f64>, extents: Vector3<f64>) -> Self {
        Self {
            min: center - extents,
            max: center + extents,
        }
    }

    /// Checks if this AABB intersects with another.
    pub fn intersects(&self, other: &Aabb) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }

    /// Returns the center of the AABB.
    pub fn center(&self) -> Point3<f64> {
        Point3::from((self.min.coords + self.max.coords) / 2.0)
    }

    /// Returns the extents (half-size) of the AABB.
    pub fn extents(&self) -> Vector3<f64> {
        (self.max.coords - self.min.coords) / 2.0
    }

    /// Expands the AABB to include a point.
    pub fn expand_to_include(&mut self, point: Point3<f64>) {
        self.min.x = self.min.x.min(point.x);
        self.min.y = self.min.y.min(point.y);
        self.min.z = self.min.z.min(point.z);
        self.max.x = self.max.x.max(point.x);
        self.max.y = self.max.y.max(point.y);
        self.max.z = self.max.z.max(point.z);
    }

    /// Merges with another AABB.
    pub fn merge(&self, other: &Aabb) -> Aabb {
        Aabb {
            min: Point3::new(
                self.min.x.min(other.min.x),
                self.min.y.min(other.min.y),
                self.min.z.min(other.min.z),
            ),
            max: Point3::new(
                self.max.x.max(other.max.x),
                self.max.y.max(other.max.y),
                self.max.z.max(other.max.z),
            ),
        }
    }

    /// Returns the surface area of the AABB.
    pub fn surface_area(&self) -> f64 {
        let d = self.max.coords - self.min.coords;
        2.0 * (d.x * d.y + d.y * d.z + d.z * d.x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aabb_intersection() {
        let aabb1 = Aabb::new(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 1.0, 1.0));
        let aabb2 = Aabb::new(Point3::new(0.5, 0.5, 0.5), Point3::new(1.5, 1.5, 1.5));
        let aabb3 = Aabb::new(Point3::new(2.0, 2.0, 2.0), Point3::new(3.0, 3.0, 3.0));

        assert!(aabb1.intersects(&aabb2));
        assert!(!aabb1.intersects(&aabb3));
    }

    #[test]
    fn test_aabb_center_extents() {
        let aabb = Aabb::new(Point3::new(-1.0, -2.0, -3.0), Point3::new(1.0, 2.0, 3.0));
        let center = aabb.center();
        let extents = aabb.extents();

        assert_eq!(center, Point3::new(0.0, 0.0, 0.0));
        assert_eq!(extents, Vector3::new(1.0, 2.0, 3.0));
    }
}
