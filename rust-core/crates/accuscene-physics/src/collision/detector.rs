//! Collision detection using broad-phase and narrow-phase algorithms.

use super::{Aabb, Collision, GjkAlgorithm, SatAlgorithm};
use nalgebra::Point3;
use std::collections::HashMap;

/// Broad-phase collision detection using spatial hashing and AABB tests.
pub struct BroadPhaseDetector {
    /// Spatial hash grid cell size
    cell_size: f64,
    /// Spatial hash map
    spatial_hash: HashMap<(i32, i32, i32), Vec<u64>>,
}

impl BroadPhaseDetector {
    /// Creates a new broad-phase detector with the specified cell size.
    pub fn new(cell_size: f64) -> Self {
        Self {
            cell_size,
            spatial_hash: HashMap::new(),
        }
    }

    /// Clears the spatial hash.
    pub fn clear(&mut self) {
        self.spatial_hash.clear();
    }

    /// Inserts an object with its AABB into the spatial hash.
    pub fn insert(&mut self, object_id: u64, aabb: &Aabb) {
        let cells = self.get_occupied_cells(aabb);
        for cell in cells {
            self.spatial_hash
                .entry(cell)
                .or_insert_with(Vec::new)
                .push(object_id);
        }
    }

    /// Returns potential collision pairs.
    pub fn find_potential_pairs(&self) -> Vec<(u64, u64)> {
        let mut pairs = Vec::new();
        let mut checked = std::collections::HashSet::new();

        for objects in self.spatial_hash.values() {
            for i in 0..objects.len() {
                for j in (i + 1)..objects.len() {
                    let pair = if objects[i] < objects[j] {
                        (objects[i], objects[j])
                    } else {
                        (objects[j], objects[i])
                    };

                    if checked.insert(pair) {
                        pairs.push(pair);
                    }
                }
            }
        }

        pairs
    }

    /// Returns all cells occupied by an AABB.
    fn get_occupied_cells(&self, aabb: &Aabb) -> Vec<(i32, i32, i32)> {
        let min_cell = (
            (aabb.min.x / self.cell_size).floor() as i32,
            (aabb.min.y / self.cell_size).floor() as i32,
            (aabb.min.z / self.cell_size).floor() as i32,
        );
        let max_cell = (
            (aabb.max.x / self.cell_size).floor() as i32,
            (aabb.max.y / self.cell_size).floor() as i32,
            (aabb.max.z / self.cell_size).floor() as i32,
        );

        let mut cells = Vec::new();
        for x in min_cell.0..=max_cell.0 {
            for y in min_cell.1..=max_cell.1 {
                for z in min_cell.2..=max_cell.2 {
                    cells.push((x, y, z));
                }
            }
        }
        cells
    }
}

/// Narrow-phase collision detection using GJK and SAT algorithms.
pub struct NarrowPhaseDetector {
    gjk: GjkAlgorithm,
    sat: SatAlgorithm,
}

impl NarrowPhaseDetector {
    /// Creates a new narrow-phase detector.
    pub fn new() -> Self {
        Self {
            gjk: GjkAlgorithm::new(),
            sat: SatAlgorithm::new(),
        }
    }

    /// Tests for collision between two convex objects using GJK.
    pub fn test_gjk(
        &mut self,
        vertices_a: &[Point3<f64>],
        vertices_b: &[Point3<f64>],
    ) -> Option<Collision> {
        self.gjk.test_collision(vertices_a, vertices_b)
    }

    /// Tests for collision between two convex objects using SAT.
    pub fn test_sat(
        &mut self,
        vertices_a: &[Point3<f64>],
        vertices_b: &[Point3<f64>],
        faces_a: &[(usize, usize, usize)],
        faces_b: &[(usize, usize, usize)],
    ) -> Option<Collision> {
        self.sat
            .test_collision(vertices_a, vertices_b, faces_a, faces_b)
    }
}

impl Default for NarrowPhaseDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Complete collision detection system.
pub struct CollisionDetector {
    broad_phase: BroadPhaseDetector,
    narrow_phase: NarrowPhaseDetector,
    /// AABB cache for objects
    aabb_cache: HashMap<u64, Aabb>,
    /// Vertex cache for objects
    vertex_cache: HashMap<u64, Vec<Point3<f64>>>,
}

impl CollisionDetector {
    /// Creates a new collision detector.
    pub fn new(cell_size: f64) -> Self {
        Self {
            broad_phase: BroadPhaseDetector::new(cell_size),
            narrow_phase: NarrowPhaseDetector::new(),
            aabb_cache: HashMap::new(),
            vertex_cache: HashMap::new(),
        }
    }

    /// Updates the AABB for an object.
    pub fn update_aabb(&mut self, object_id: u64, aabb: Aabb) {
        self.aabb_cache.insert(object_id, aabb);
    }

    /// Updates the vertices for an object.
    pub fn update_vertices(&mut self, object_id: u64, vertices: Vec<Point3<f64>>) {
        self.vertex_cache.insert(object_id, vertices);
    }

    /// Detects all collisions in the current frame.
    pub fn detect_collisions(&mut self) -> Vec<Collision> {
        // Clear and rebuild spatial hash
        self.broad_phase.clear();
        for (&object_id, aabb) in &self.aabb_cache {
            self.broad_phase.insert(object_id, aabb);
        }

        // Find potential pairs using broad-phase
        let potential_pairs = self.broad_phase.find_potential_pairs();

        // Perform narrow-phase detection on potential pairs
        let mut collisions = Vec::new();
        for (id_a, id_b) in potential_pairs {
            if let (Some(vertices_a), Some(vertices_b)) = (
                self.vertex_cache.get(&id_a),
                self.vertex_cache.get(&id_b),
            ) {
                if let Some(mut collision) = self.narrow_phase.test_gjk(vertices_a, vertices_b) {
                    collision.object_a = id_a;
                    collision.object_b = id_b;
                    collisions.push(collision);
                }
            }
        }

        collisions
    }

    /// Removes an object from the detector.
    pub fn remove_object(&mut self, object_id: u64) {
        self.aabb_cache.remove(&object_id);
        self.vertex_cache.remove(&object_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_broad_phase_spatial_hash() {
        let mut detector = BroadPhaseDetector::new(10.0);

        let aabb1 = Aabb::new(Point3::new(0.0, 0.0, 0.0), Point3::new(5.0, 5.0, 5.0));
        let aabb2 = Aabb::new(Point3::new(3.0, 3.0, 3.0), Point3::new(8.0, 8.0, 8.0));

        detector.insert(1, &aabb1);
        detector.insert(2, &aabb2);

        let pairs = detector.find_potential_pairs();
        assert!(pairs.contains(&(1, 2)));
    }

    #[test]
    fn test_collision_detector() {
        let mut detector = CollisionDetector::new(10.0);

        let aabb1 = Aabb::new(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 1.0, 1.0));
        detector.update_aabb(1, aabb1);

        let aabb2 = Aabb::new(Point3::new(0.5, 0.5, 0.5), Point3::new(1.5, 1.5, 1.5));
        detector.update_aabb(2, aabb2);

        // AABBs should be found in broad phase
        detector.broad_phase.clear();
        detector.broad_phase.insert(1, &aabb1);
        detector.broad_phase.insert(2, &aabb2);

        let pairs = detector.broad_phase.find_potential_pairs();
        assert_eq!(pairs.len(), 1);
    }
}
