//! Narrow phase collision detection.
//!
//! Implements precise collision detection algorithms:
//! - GJK (Gilbert-Johnson-Keerthi) for intersection testing
//! - EPA (Expanding Polytope Algorithm) for penetration depth
//! - Contact manifold generation

use nalgebra::Vector3;
use serde::{Deserialize, Serialize};

use super::{CollisionShape, ContactPoint};
use crate::error::{PhysicsError, PhysicsResult};

/// GJK/EPA narrow phase detector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrowPhase {
    /// Maximum GJK iterations.
    max_gjk_iterations: usize,

    /// Maximum EPA iterations.
    max_epa_iterations: usize,

    /// Convergence tolerance.
    tolerance: f64,
}

impl NarrowPhase {
    /// Creates a new narrow phase detector.
    pub fn new() -> Self {
        Self {
            max_gjk_iterations: 64,
            max_epa_iterations: 32,
            tolerance: 1e-6,
        }
    }

    /// Detects collision between two shapes using GJK.
    ///
    /// Returns true if shapes are intersecting.
    pub fn gjk_intersect(
        &self,
        shape_a: &CollisionShape,
        pos_a: Vector3<f64>,
        shape_b: &CollisionShape,
        pos_b: Vector3<f64>,
    ) -> bool {
        let mut simplex = Simplex::new();

        // Initial direction
        let mut direction = pos_b - pos_a;
        if direction.norm_squared() < 1e-10 {
            direction = Vector3::x();
        }

        // Initial support point
        let support = self.support(shape_a, pos_a, shape_b, pos_b, direction);
        simplex.push(support);

        direction = -support;

        for _ in 0..self.max_gjk_iterations {
            let support = self.support(shape_a, pos_a, shape_b, pos_b, direction);

            if support.dot(&direction) <= 0.0 {
                return false; // No intersection
            }

            simplex.push(support);

            if simplex.contains_origin(&mut direction) {
                return true; // Intersection found
            }
        }

        false
    }

    /// Computes contact information using EPA (Expanding Polytope Algorithm).
    pub fn epa_contact(
        &self,
        shape_a: &CollisionShape,
        pos_a: Vector3<f64>,
        shape_b: &CollisionShape,
        pos_b: Vector3<f64>,
    ) -> PhysicsResult<ContactPoint> {
        // First run GJK to get initial simplex
        let mut simplex = Simplex::new();
        let mut direction = pos_b - pos_a;
        if direction.norm_squared() < 1e-10 {
            direction = Vector3::x();
        }

        let support = self.support(shape_a, pos_a, shape_b, pos_b, direction);
        simplex.push(support);
        direction = -support;

        for _ in 0..self.max_gjk_iterations {
            let support = self.support(shape_a, pos_a, shape_b, pos_b, direction);
            if support.dot(&direction) <= 0.0 {
                return Err(PhysicsError::CollisionDetectionError(
                    "No intersection found".to_string(),
                ));
            }

            simplex.push(support);

            if simplex.contains_origin(&mut direction) {
                break;
            }
        }

        // Run EPA to find penetration depth and normal
        let (penetration, normal) = self.epa(&simplex, shape_a, pos_a, shape_b, pos_b)?;

        // Contact point (approximate as midpoint on surface)
        let contact_point = pos_a + normal * (penetration * 0.5);

        Ok(ContactPoint {
            point: contact_point,
            normal,
            penetration,
            feature_id: 0,
        })
    }

    /// EPA algorithm implementation.
    fn epa(
        &self,
        initial_simplex: &Simplex,
        shape_a: &CollisionShape,
        pos_a: Vector3<f64>,
        shape_b: &CollisionShape,
        pos_b: Vector3<f64>,
    ) -> PhysicsResult<(f64, Vector3<f64>)> {
        // Build initial polytope from simplex
        let mut polytope = vec![
            initial_simplex.points[0],
            initial_simplex.points[1],
            initial_simplex.points[2],
            initial_simplex.points[3],
        ];

        // Triangle faces (indices into polytope)
        let mut faces = vec![
            (0, 1, 2),
            (0, 2, 3),
            (0, 3, 1),
            (1, 3, 2),
        ];

        for _ in 0..self.max_epa_iterations {
            // Find closest face to origin
            let (closest_idx, closest_dist, closest_normal) = self.find_closest_face(&polytope, &faces)?;

            // Get support point in direction of normal
            let support = self.support(shape_a, pos_a, shape_b, pos_b, closest_normal);

            let distance = support.dot(&closest_normal);

            // Check convergence
            if distance - closest_dist < self.tolerance {
                return Ok((closest_dist, closest_normal));
            }

            // Expand polytope
            polytope.push(support);
            let new_idx = polytope.len() - 1;

            // Remove faces visible from new point and add new faces
            let mut new_faces = Vec::new();
            faces.retain(|&(i0, i1, i2)| {
                let v0 = polytope[i0];
                let v1 = polytope[i1];
                let v2 = polytope[i2];
                let normal = (v1 - v0).cross(&(v2 - v0)).normalize();

                if (support - v0).dot(&normal) > 0.0 {
                    // Face is visible, add edges to new faces
                    new_faces.push((i0, i1, new_idx));
                    new_faces.push((i1, i2, new_idx));
                    new_faces.push((i2, i0, new_idx));
                    false
                } else {
                    true
                }
            });

            faces.extend(new_faces);
        }

        Err(PhysicsError::CollisionDetectionError(
            "EPA failed to converge".to_string(),
        ))
    }

    /// Finds the closest face to the origin.
    fn find_closest_face(
        &self,
        polytope: &[Vector3<f64>],
        faces: &[(usize, usize, usize)],
    ) -> PhysicsResult<(usize, f64, Vector3<f64>)> {
        let mut min_dist = f64::INFINITY;
        let mut min_idx = 0;
        let mut min_normal = Vector3::zeros();

        for (idx, &(i0, i1, i2)) in faces.iter().enumerate() {
            let v0 = polytope[i0];
            let v1 = polytope[i1];
            let v2 = polytope[i2];

            let normal = (v1 - v0).cross(&(v2 - v0));
            let norm = normal.norm();

            if norm < 1e-10 {
                continue;
            }

            let normal = normal / norm;
            let dist = v0.dot(&normal).abs();

            if dist < min_dist {
                min_dist = dist;
                min_idx = idx;
                min_normal = normal;
            }
        }

        if min_dist == f64::INFINITY {
            return Err(PhysicsError::CollisionDetectionError(
                "No valid face found".to_string(),
            ));
        }

        Ok((min_idx, min_dist, min_normal))
    }

    /// Computes support point for Minkowski difference.
    ///
    /// support(A - B, d) = support(A, d) - support(B, -d)
    fn support(
        &self,
        shape_a: &CollisionShape,
        pos_a: Vector3<f64>,
        shape_b: &CollisionShape,
        pos_b: Vector3<f64>,
        direction: Vector3<f64>,
    ) -> Vector3<f64> {
        let support_a = pos_a + shape_a.support(direction);
        let support_b = pos_b + shape_b.support(-direction);
        support_a - support_b
    }
}

impl Default for NarrowPhase {
    fn default() -> Self {
        Self::new()
    }
}

/// Simplex for GJK algorithm (up to 4 points).
#[derive(Debug, Clone)]
struct Simplex {
    points: [Vector3<f64>; 4],
    size: usize,
}

impl Simplex {
    fn new() -> Self {
        Self {
            points: [Vector3::zeros(); 4],
            size: 0,
        }
    }

    fn push(&mut self, point: Vector3<f64>) {
        self.points[self.size] = point;
        self.size += 1;
    }

    /// Checks if simplex contains origin and updates direction.
    fn contains_origin(&mut self, direction: &mut Vector3<f64>) -> bool {
        match self.size {
            2 => self.line_case(direction),
            3 => self.triangle_case(direction),
            4 => self.tetrahedron_case(direction),
            _ => false,
        }
    }

    fn line_case(&mut self, direction: &mut Vector3<f64>) -> bool {
        let a = self.points[1];
        let b = self.points[0];

        let ab = b - a;
        let ao = -a;

        if ab.dot(&ao) > 0.0 {
            *direction = ab.cross(&ao).cross(&ab);
        } else {
            self.points[0] = a;
            self.size = 1;
            *direction = ao;
        }

        false
    }

    fn triangle_case(&mut self, direction: &mut Vector3<f64>) -> bool {
        let a = self.points[2];
        let b = self.points[1];
        let c = self.points[0];

        let ab = b - a;
        let ac = c - a;
        let ao = -a;

        let abc = ab.cross(&ac);

        if abc.cross(&ac).dot(&ao) > 0.0 {
            if ac.dot(&ao) > 0.0 {
                self.points[1] = c;
                self.size = 2;
                *direction = ac.cross(&ao).cross(&ac);
            } else {
                self.points[0] = b;
                self.points[1] = a;
                self.size = 2;
                return self.line_case(direction);
            }
        } else if ab.cross(&abc).dot(&ao) > 0.0 {
            self.points[0] = b;
            self.points[1] = a;
            self.size = 2;
            return self.line_case(direction);
        } else if abc.dot(&ao) > 0.0 {
            *direction = abc;
        } else {
            self.points[0] = b;
            self.points[1] = c;
            self.points[2] = a;
            *direction = -abc;
        }

        false
    }

    fn tetrahedron_case(&mut self, direction: &mut Vector3<f64>) -> bool {
        let a = self.points[3];
        let b = self.points[2];
        let c = self.points[1];
        let d = self.points[0];

        let ab = b - a;
        let ac = c - a;
        let ad = d - a;
        let ao = -a;

        let abc = ab.cross(&ac);
        let acd = ac.cross(&ad);
        let adb = ad.cross(&ab);

        if abc.dot(&ao) > 0.0 {
            self.points[0] = c;
            self.points[1] = b;
            self.points[2] = a;
            self.size = 3;
            return self.triangle_case(direction);
        }

        if acd.dot(&ao) > 0.0 {
            self.points[0] = d;
            self.points[1] = c;
            self.points[2] = a;
            self.size = 3;
            return self.triangle_case(direction);
        }

        if adb.dot(&ao) > 0.0 {
            self.points[0] = b;
            self.points[1] = d;
            self.points[2] = a;
            self.size = 3;
            return self.triangle_case(direction);
        }

        true // Origin is inside tetrahedron
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gjk_sphere_sphere_intersection() {
        let narrow = NarrowPhase::new();

        let sphere_a = CollisionShape::Sphere { radius: 1.0 };
        let sphere_b = CollisionShape::Sphere { radius: 1.0 };

        let pos_a = Vector3::new(0.0, 0.0, 0.0);
        let pos_b = Vector3::new(1.5, 0.0, 0.0);

        let intersects = narrow.gjk_intersect(&sphere_a, pos_a, &sphere_b, pos_b);
        assert!(intersects);
    }

    #[test]
    fn test_gjk_sphere_sphere_no_intersection() {
        let narrow = NarrowPhase::new();

        let sphere_a = CollisionShape::Sphere { radius: 1.0 };
        let sphere_b = CollisionShape::Sphere { radius: 1.0 };

        let pos_a = Vector3::new(0.0, 0.0, 0.0);
        let pos_b = Vector3::new(5.0, 0.0, 0.0);

        let intersects = narrow.gjk_intersect(&sphere_a, pos_a, &sphere_b, pos_b);
        assert!(!intersects);
    }
}
