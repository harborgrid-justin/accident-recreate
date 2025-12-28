//! Separating Axis Theorem (SAT) for collision detection.
//!
//! SAT is used for detecting collisions between convex polyhedra by testing
//! for separation along potential separating axes.

use super::Collision;
use nalgebra::{Point3, Vector3};

/// SAT algorithm implementation for convex polyhedra collision detection.
pub struct SatAlgorithm;

impl SatAlgorithm {
    /// Creates a new SAT algorithm instance.
    pub fn new() -> Self {
        Self
    }

    /// Tests if two convex polyhedra collide using SAT.
    pub fn test_collision(
        &self,
        vertices_a: &[Point3<f64>],
        vertices_b: &[Point3<f64>],
        faces_a: &[(usize, usize, usize)],
        faces_b: &[(usize, usize, usize)],
    ) -> Option<Collision> {
        let mut min_penetration = f64::INFINITY;
        let mut collision_normal = Vector3::zeros();
        let mut collision_point = Point3::origin();

        // Test face normals of A
        for face in faces_a {
            let normal = self.compute_face_normal(vertices_a, *face);
            if let Some((penetration, point)) = self.test_axis(&normal, vertices_a, vertices_b) {
                if penetration < min_penetration {
                    min_penetration = penetration;
                    collision_normal = normal;
                    collision_point = point;
                }
            } else {
                return None; // Separating axis found
            }
        }

        // Test face normals of B
        for face in faces_b {
            let normal = self.compute_face_normal(vertices_b, *face);
            if let Some((penetration, point)) = self.test_axis(&normal, vertices_a, vertices_b) {
                if penetration < min_penetration {
                    min_penetration = penetration;
                    collision_normal = -normal;
                    collision_point = point;
                }
            } else {
                return None; // Separating axis found
            }
        }

        // Test edge cross products
        for edge_a in self.get_edges(vertices_a, faces_a) {
            for edge_b in self.get_edges(vertices_b, faces_b) {
                let axis = edge_a.cross(&edge_b);
                let axis_length = axis.norm();

                if axis_length < 1e-6 {
                    continue; // Edges are parallel
                }

                let normalized_axis = axis / axis_length;
                if let Some((penetration, point)) =
                    self.test_axis(&normalized_axis, vertices_a, vertices_b)
                {
                    if penetration < min_penetration {
                        min_penetration = penetration;
                        collision_normal = normalized_axis;
                        collision_point = point;
                    }
                } else {
                    return None; // Separating axis found
                }
            }
        }

        // Collision detected
        Some(Collision {
            object_a: 0,
            object_b: 0,
            point: collision_point,
            normal: collision_normal.normalize(),
            penetration: min_penetration,
            time_of_impact: 0.0,
            relative_velocity: Vector3::zeros(),
        })
    }

    /// Tests a potential separating axis.
    fn test_axis(
        &self,
        axis: &Vector3<f64>,
        vertices_a: &[Point3<f64>],
        vertices_b: &[Point3<f64>],
    ) -> Option<(f64, Point3<f64>)> {
        let (min_a, max_a) = self.project_vertices(vertices_a, axis);
        let (min_b, max_b) = self.project_vertices(vertices_b, axis);

        // Check for overlap
        if max_a < min_b || max_b < min_a {
            return None; // Separating axis found
        }

        // Calculate penetration depth
        let penetration = if max_a > max_b {
            max_b - min_a
        } else {
            max_a - min_b
        };

        // Find contact point (simplified - midpoint of overlap)
        let contact_projection = (min_a.max(min_b) + max_a.min(max_b)) / 2.0;
        let contact_point = Point3::from(axis * contact_projection);

        Some((penetration, contact_point))
    }

    /// Projects vertices onto an axis and returns min/max projections.
    fn project_vertices(&self, vertices: &[Point3<f64>], axis: &Vector3<f64>) -> (f64, f64) {
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;

        for vertex in vertices {
            let projection = vertex.coords.dot(axis);
            min = min.min(projection);
            max = max.max(projection);
        }

        (min, max)
    }

    /// Computes the normal of a face.
    fn compute_face_normal(
        &self,
        vertices: &[Point3<f64>],
        face: (usize, usize, usize),
    ) -> Vector3<f64> {
        let v0 = vertices[face.0].coords;
        let v1 = vertices[face.1].coords;
        let v2 = vertices[face.2].coords;

        let edge1 = v1 - v0;
        let edge2 = v2 - v0;

        edge1.cross(&edge2).normalize()
    }

    /// Gets all unique edges from faces.
    fn get_edges(
        &self,
        vertices: &[Point3<f64>],
        faces: &[(usize, usize, usize)],
    ) -> Vec<Vector3<f64>> {
        let mut edges = Vec::new();

        for face in faces {
            let v0 = vertices[face.0].coords;
            let v1 = vertices[face.1].coords;
            let v2 = vertices[face.2].coords;

            edges.push((v1 - v0).normalize());
            edges.push((v2 - v1).normalize());
            edges.push((v0 - v2).normalize());
        }

        edges
    }
}

impl Default for SatAlgorithm {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sat_collision() {
        let sat = SatAlgorithm::new();

        // Two overlapping triangular prisms
        let vertices_a = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.5, 1.0, 0.0),
            Point3::new(0.0, 0.0, 1.0),
            Point3::new(1.0, 0.0, 1.0),
            Point3::new(0.5, 1.0, 1.0),
        ];

        let faces_a = vec![
            (0, 1, 2), // Bottom
            (3, 5, 4), // Top
            (0, 3, 4), // Side 1
            (0, 4, 1), // Side 1 continued
            (1, 4, 5), // Side 2
            (1, 5, 2), // Side 2 continued
            (2, 5, 3), // Side 3
            (2, 3, 0), // Side 3 continued
        ];

        let vertices_b = vec![
            Point3::new(0.3, 0.3, 0.3),
            Point3::new(1.3, 0.3, 0.3),
            Point3::new(0.8, 1.3, 0.3),
            Point3::new(0.3, 0.3, 1.3),
            Point3::new(1.3, 0.3, 1.3),
            Point3::new(0.8, 1.3, 1.3),
        ];

        let faces_b = faces_a.clone();

        let result = sat.test_collision(&vertices_a, &vertices_b, &faces_a, &faces_b);
        assert!(result.is_some());
    }

    #[test]
    fn test_sat_no_collision() {
        let sat = SatAlgorithm::new();

        let vertices_a = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.5, 1.0, 0.0),
        ];

        let faces_a = vec![(0, 1, 2)];

        let vertices_b = vec![
            Point3::new(5.0, 5.0, 5.0),
            Point3::new(6.0, 5.0, 5.0),
            Point3::new(5.5, 6.0, 5.0),
        ];

        let faces_b = vec![(0, 1, 2)];

        let result = sat.test_collision(&vertices_a, &vertices_b, &faces_a, &faces_b);
        assert!(result.is_none());
    }
}
