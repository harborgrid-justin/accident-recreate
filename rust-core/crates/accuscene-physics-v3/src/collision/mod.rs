//! Collision detection and response system.
//!
//! This module provides:
//! - Broad phase collision detection (AABB, spatial hashing)
//! - Narrow phase collision detection (GJK/EPA)
//! - Contact manifold generation
//! - Collision response (impulse-based)

pub mod broad_phase;
pub mod narrow_phase;
pub mod response;

pub use broad_phase::*;
pub use narrow_phase::*;
pub use response::*;

use nalgebra::Vector3;
use serde::{Deserialize, Serialize};

/// Axis-Aligned Bounding Box.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AABB {
    /// Minimum corner.
    pub min: Vector3<f64>,

    /// Maximum corner.
    pub max: Vector3<f64>,
}

impl AABB {
    /// Creates a new AABB.
    pub fn new(min: Vector3<f64>, max: Vector3<f64>) -> Self {
        Self { min, max }
    }

    /// Creates an AABB from a center point and half extents.
    pub fn from_center_half_extents(center: Vector3<f64>, half_extents: Vector3<f64>) -> Self {
        Self {
            min: center - half_extents,
            max: center + half_extents,
        }
    }

    /// Computes the center of the AABB.
    pub fn center(&self) -> Vector3<f64> {
        (self.min + self.max) * 0.5
    }

    /// Computes the half extents.
    pub fn half_extents(&self) -> Vector3<f64> {
        (self.max - self.min) * 0.5
    }

    /// Computes the surface area.
    pub fn surface_area(&self) -> f64 {
        let extents = self.max - self.min;
        2.0 * (extents.x * extents.y + extents.y * extents.z + extents.z * extents.x)
    }

    /// Checks if this AABB overlaps another.
    pub fn overlaps(&self, other: &AABB) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }

    /// Checks if a point is inside the AABB.
    pub fn contains_point(&self, point: Vector3<f64>) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
            && point.z >= self.min.z
            && point.z <= self.max.z
    }

    /// Expands the AABB by a margin.
    pub fn expand(&self, margin: f64) -> AABB {
        let expansion = Vector3::new(margin, margin, margin);
        AABB {
            min: self.min - expansion,
            max: self.max + expansion,
        }
    }

    /// Merges this AABB with another.
    pub fn merge(&self, other: &AABB) -> AABB {
        AABB {
            min: Vector3::new(
                self.min.x.min(other.min.x),
                self.min.y.min(other.min.y),
                self.min.z.min(other.min.z),
            ),
            max: Vector3::new(
                self.max.x.max(other.max.x),
                self.max.y.max(other.max.y),
                self.max.z.max(other.max.z),
            ),
        }
    }

    /// Computes the closest point on the AABB to a given point.
    pub fn closest_point(&self, point: Vector3<f64>) -> Vector3<f64> {
        Vector3::new(
            point.x.max(self.min.x).min(self.max.x),
            point.y.max(self.min.y).min(self.max.y),
            point.z.max(self.min.z).min(self.max.z),
        )
    }
}

/// Collision shape types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollisionShape {
    /// Sphere defined by radius.
    Sphere { radius: f64 },

    /// Box defined by half extents.
    Box { half_extents: Vector3<f64> },

    /// Capsule (cylinder with hemispherical caps).
    Capsule { radius: f64, half_height: f64 },

    /// Convex hull defined by vertices.
    ConvexHull { vertices: Vec<Vector3<f64>> },

    /// Triangle mesh (for static geometry).
    TriangleMesh { vertices: Vec<Vector3<f64>>, indices: Vec<[usize; 3]> },
}

impl CollisionShape {
    /// Computes the AABB for this shape at a given position and orientation.
    pub fn compute_aabb(&self, position: Vector3<f64>, _orientation: &nalgebra::UnitQuaternion<f64>) -> AABB {
        match self {
            CollisionShape::Sphere { radius } => {
                let r = Vector3::new(*radius, *radius, *radius);
                AABB::from_center_half_extents(position, r)
            }
            CollisionShape::Box { half_extents } => {
                // Simplified: use conservative AABB (proper would rotate corners)
                let max_extent = half_extents.max();
                let r = Vector3::new(max_extent, max_extent, max_extent);
                AABB::from_center_half_extents(position, r)
            }
            CollisionShape::Capsule { radius, half_height } => {
                let r = Vector3::new(*radius, *radius, half_height + radius);
                AABB::from_center_half_extents(position, r)
            }
            CollisionShape::ConvexHull { vertices } => {
                if vertices.is_empty() {
                    return AABB::from_center_half_extents(position, Vector3::new(0.1, 0.1, 0.1));
                }
                let mut min = vertices[0];
                let mut max = vertices[0];
                for v in vertices {
                    min = Vector3::new(min.x.min(v.x), min.y.min(v.y), min.z.min(v.z));
                    max = Vector3::new(max.x.max(v.x), max.y.max(v.y), max.z.max(v.z));
                }
                AABB::new(min + position, max + position)
            }
            CollisionShape::TriangleMesh { vertices, .. } => {
                if vertices.is_empty() {
                    return AABB::from_center_half_extents(position, Vector3::new(0.1, 0.1, 0.1));
                }
                let mut min = vertices[0];
                let mut max = vertices[0];
                for v in vertices {
                    min = Vector3::new(min.x.min(v.x), min.y.min(v.y), min.z.min(v.z));
                    max = Vector3::new(max.x.max(v.x), max.y.max(v.y), max.z.max(v.z));
                }
                AABB::new(min + position, max + position)
            }
        }
    }

    /// Returns support point in a given direction (for GJK).
    pub fn support(&self, direction: Vector3<f64>) -> Vector3<f64> {
        match self {
            CollisionShape::Sphere { radius } => {
                let normalized = if direction.norm() > 1e-6 {
                    direction.normalize()
                } else {
                    Vector3::x()
                };
                normalized * *radius
            }
            CollisionShape::Box { half_extents } => {
                Vector3::new(
                    if direction.x >= 0.0 { half_extents.x } else { -half_extents.x },
                    if direction.y >= 0.0 { half_extents.y } else { -half_extents.y },
                    if direction.z >= 0.0 { half_extents.z } else { -half_extents.z },
                )
            }
            CollisionShape::ConvexHull { vertices } => {
                vertices
                    .iter()
                    .max_by(|a, b| {
                        a.dot(&direction)
                            .partial_cmp(&b.dot(&direction))
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .copied()
                    .unwrap_or_else(Vector3::zeros)
            }
            _ => Vector3::zeros(), // Simplified for other shapes
        }
    }
}

/// Contact point information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactPoint {
    /// Contact point in world space.
    pub point: Vector3<f64>,

    /// Contact normal (from body A to body B).
    pub normal: Vector3<f64>,

    /// Penetration depth (positive = overlapping).
    pub penetration: f64,

    /// Feature ID for contact tracking.
    pub feature_id: u32,
}

/// Collision pair for broad phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CollisionPair {
    /// First body ID (always smaller).
    pub body_a: usize,

    /// Second body ID (always larger).
    pub body_b: usize,
}

impl CollisionPair {
    /// Creates a new collision pair (automatically orders IDs).
    pub fn new(id_a: usize, id_b: usize) -> Self {
        if id_a < id_b {
            Self { body_a: id_a, body_b: id_b }
        } else {
            Self { body_a: id_b, body_b: id_a }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_aabb_creation() {
        let aabb = AABB::from_center_half_extents(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
        );
        assert_relative_eq!(aabb.min.x, -1.0);
        assert_relative_eq!(aabb.max.x, 1.0);
    }

    #[test]
    fn test_aabb_overlap() {
        let aabb1 = AABB::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(2.0, 2.0, 2.0));
        let aabb2 = AABB::new(Vector3::new(1.0, 1.0, 1.0), Vector3::new(3.0, 3.0, 3.0));
        assert!(aabb1.overlaps(&aabb2));

        let aabb3 = AABB::new(Vector3::new(5.0, 5.0, 5.0), Vector3::new(7.0, 7.0, 7.0));
        assert!(!aabb1.overlaps(&aabb3));
    }

    #[test]
    fn test_collision_pair_ordering() {
        let pair1 = CollisionPair::new(5, 2);
        assert_eq!(pair1.body_a, 2);
        assert_eq!(pair1.body_b, 5);

        let pair2 = CollisionPair::new(3, 7);
        assert_eq!(pair2.body_a, 3);
        assert_eq!(pair2.body_b, 7);
    }
}
