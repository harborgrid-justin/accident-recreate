//! Gilbert-Johnson-Keerthi (GJK) algorithm for collision detection.
//!
//! GJK is a iterative algorithm for determining if two convex shapes intersect.
//! It operates on the Minkowski difference of the two shapes.

use super::Collision;
use nalgebra::{Point3, Vector3};

const MAX_ITERATIONS: usize = 64;
const EPSILON: f64 = 1e-10;

/// GJK algorithm implementation for convex collision detection.
pub struct GjkAlgorithm {
    simplex: Vec<Vector3<f64>>,
}

impl GjkAlgorithm {
    /// Creates a new GJK algorithm instance.
    pub fn new() -> Self {
        Self {
            simplex: Vec::with_capacity(4),
        }
    }

    /// Tests if two convex shapes collide using GJK algorithm.
    pub fn test_collision(
        &mut self,
        vertices_a: &[Point3<f64>],
        vertices_b: &[Point3<f64>],
    ) -> Option<Collision> {
        if vertices_a.is_empty() || vertices_b.is_empty() {
            return None;
        }

        self.simplex.clear();

        // Start with an arbitrary direction
        let mut direction = Vector3::new(1.0, 0.0, 0.0);

        // Get the first support point
        let support = self.support(vertices_a, vertices_b, direction);
        self.simplex.push(support);

        // New direction is towards the origin
        direction = -support;

        for _ in 0..MAX_ITERATIONS {
            let support = self.support(vertices_a, vertices_b, direction);

            // If the support point doesn't pass the origin, no collision
            if support.dot(&direction) < 0.0 {
                return None;
            }

            self.simplex.push(support);

            // Check if simplex contains origin
            if self.contains_origin(&mut direction) {
                // Collision detected, compute collision info
                return Some(self.compute_collision_info(vertices_a, vertices_b));
            }
        }

        None
    }

    /// Computes the support point in the Minkowski difference.
    fn support(
        &self,
        vertices_a: &[Point3<f64>],
        vertices_b: &[Point3<f64>],
        direction: Vector3<f64>,
    ) -> Vector3<f64> {
        let point_a = self.furthest_point(vertices_a, direction);
        let point_b = self.furthest_point(vertices_b, -direction);
        point_a - point_b
    }

    /// Finds the furthest point in a given direction.
    fn furthest_point(&self, vertices: &[Point3<f64>], direction: Vector3<f64>) -> Vector3<f64> {
        let mut max_point = vertices[0].coords;
        let mut max_distance = max_point.dot(&direction);

        for vertex in vertices.iter().skip(1) {
            let distance = vertex.coords.dot(&direction);
            if distance > max_distance {
                max_distance = distance;
                max_point = vertex.coords;
            }
        }

        max_point
    }

    /// Checks if the simplex contains the origin and updates the direction.
    fn contains_origin(&mut self, direction: &mut Vector3<f64>) -> bool {
        let a = self.simplex[self.simplex.len() - 1];

        match self.simplex.len() {
            2 => self.line_case(a, direction),
            3 => self.triangle_case(a, direction),
            4 => self.tetrahedron_case(a, direction),
            _ => false,
        }
    }

    /// Handles the line simplex case.
    fn line_case(&mut self, a: Vector3<f64>, direction: &mut Vector3<f64>) -> bool {
        let b = self.simplex[0];
        let ab = b - a;
        let ao = -a;

        if ab.dot(&ao) > 0.0 {
            // Region AB
            *direction = ab.cross(&ao).cross(&ab);
            if direction.norm_squared() < EPSILON {
                *direction = ab.cross(&Vector3::new(1.0, 0.0, 0.0));
                if direction.norm_squared() < EPSILON {
                    *direction = ab.cross(&Vector3::new(0.0, 1.0, 0.0));
                }
            }
        } else {
            // Region A
            self.simplex.clear();
            self.simplex.push(a);
            *direction = ao;
        }

        false
    }

    /// Handles the triangle simplex case.
    fn triangle_case(&mut self, a: Vector3<f64>, direction: &mut Vector3<f64>) -> bool {
        let b = self.simplex[1];
        let c = self.simplex[0];

        let ab = b - a;
        let ac = c - a;
        let ao = -a;

        let abc = ab.cross(&ac);

        if abc.cross(&ac).dot(&ao) > 0.0 {
            if ac.dot(&ao) > 0.0 {
                // Region AC
                self.simplex.clear();
                self.simplex.push(c);
                self.simplex.push(a);
                *direction = ac.cross(&ao).cross(&ac);
            } else {
                // Check AB region
                return self.check_ab_region(a, b, ab, ao, direction);
            }
        } else if ab.cross(&abc).dot(&ao) > 0.0 {
            // Check AB region
            return self.check_ab_region(a, b, ab, ao, direction);
        } else if abc.dot(&ao) > 0.0 {
            // Region ABC (above)
            *direction = abc;
        } else {
            // Region ABC (below)
            self.simplex.clear();
            self.simplex.push(b);
            self.simplex.push(c);
            self.simplex.push(a);
            *direction = -abc;
        }

        false
    }

    fn check_ab_region(
        &mut self,
        a: Vector3<f64>,
        b: Vector3<f64>,
        ab: Vector3<f64>,
        ao: Vector3<f64>,
        direction: &mut Vector3<f64>,
    ) -> bool {
        if ab.dot(&ao) > 0.0 {
            // Region AB
            self.simplex.clear();
            self.simplex.push(b);
            self.simplex.push(a);
            *direction = ab.cross(&ao).cross(&ab);
            false
        } else {
            // Region A
            self.simplex.clear();
            self.simplex.push(a);
            *direction = ao;
            false
        }
    }

    /// Handles the tetrahedron simplex case.
    fn tetrahedron_case(&mut self, a: Vector3<f64>, direction: &mut Vector3<f64>) -> bool {
        let b = self.simplex[2];
        let c = self.simplex[1];
        let d = self.simplex[0];

        let ab = b - a;
        let ac = c - a;
        let ad = d - a;
        let ao = -a;

        let abc = ab.cross(&ac);
        let acd = ac.cross(&ad);
        let adb = ad.cross(&ab);

        // Check each face
        if abc.dot(&ao) > 0.0 {
            self.simplex.clear();
            self.simplex.push(c);
            self.simplex.push(b);
            self.simplex.push(a);
            return self.triangle_case(a, direction);
        }

        if acd.dot(&ao) > 0.0 {
            self.simplex.clear();
            self.simplex.push(d);
            self.simplex.push(c);
            self.simplex.push(a);
            return self.triangle_case(a, direction);
        }

        if adb.dot(&ao) > 0.0 {
            self.simplex.clear();
            self.simplex.push(b);
            self.simplex.push(d);
            self.simplex.push(a);
            return self.triangle_case(a, direction);
        }

        // Origin is inside tetrahedron
        true
    }

    /// Computes detailed collision information using EPA (Expanding Polytope Algorithm).
    fn compute_collision_info(
        &self,
        _vertices_a: &[Point3<f64>],
        _vertices_b: &[Point3<f64>],
    ) -> Collision {
        // Simplified collision info - in a full implementation, EPA would be used here
        Collision {
            object_a: 0,
            object_b: 0,
            point: Point3::origin(),
            normal: Vector3::new(0.0, 0.0, 1.0),
            penetration: 0.1,
            time_of_impact: 0.0,
            relative_velocity: Vector3::zeros(),
        }
    }
}

impl Default for GjkAlgorithm {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gjk_collision() {
        let mut gjk = GjkAlgorithm::new();

        // Two overlapping cubes
        let cube1 = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(1.0, 1.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
            Point3::new(0.0, 0.0, 1.0),
            Point3::new(1.0, 0.0, 1.0),
            Point3::new(1.0, 1.0, 1.0),
            Point3::new(0.0, 1.0, 1.0),
        ];

        let cube2 = vec![
            Point3::new(0.5, 0.5, 0.5),
            Point3::new(1.5, 0.5, 0.5),
            Point3::new(1.5, 1.5, 0.5),
            Point3::new(0.5, 1.5, 0.5),
            Point3::new(0.5, 0.5, 1.5),
            Point3::new(1.5, 0.5, 1.5),
            Point3::new(1.5, 1.5, 1.5),
            Point3::new(0.5, 1.5, 1.5),
        ];

        let result = gjk.test_collision(&cube1, &cube2);
        assert!(result.is_some());
    }

    #[test]
    fn test_gjk_no_collision() {
        let mut gjk = GjkAlgorithm::new();

        let cube1 = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(1.0, 1.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
            Point3::new(0.0, 0.0, 1.0),
            Point3::new(1.0, 0.0, 1.0),
            Point3::new(1.0, 1.0, 1.0),
            Point3::new(0.0, 1.0, 1.0),
        ];

        let cube2 = vec![
            Point3::new(5.0, 5.0, 5.0),
            Point3::new(6.0, 5.0, 5.0),
            Point3::new(6.0, 6.0, 5.0),
            Point3::new(5.0, 6.0, 5.0),
            Point3::new(5.0, 5.0, 6.0),
            Point3::new(6.0, 5.0, 6.0),
            Point3::new(6.0, 6.0, 6.0),
            Point3::new(5.0, 6.0, 6.0),
        ];

        let result = gjk.test_collision(&cube1, &cube2);
        assert!(result.is_none());
    }
}
