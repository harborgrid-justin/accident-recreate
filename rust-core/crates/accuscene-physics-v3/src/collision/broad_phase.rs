//! Broad phase collision detection.
//!
//! Efficiently identifies potentially colliding pairs using:
//! - Sweep and Prune (Sort and Sweep)
//! - Spatial Hashing
//! - Bounding Volume Hierarchies

use std::collections::{HashMap, HashSet};

use nalgebra::Vector3;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use super::{CollisionPair, AABB};
use crate::config::BroadPhaseMethod;

/// Broad phase collision detector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadPhase {
    /// Detection method.
    method: BroadPhaseMethod,

    /// Sweep and prune state.
    sweep_and_prune: SweepAndPrune,

    /// Spatial hash state.
    spatial_hash: SpatialHash,
}

impl BroadPhase {
    /// Creates a new broad phase detector.
    pub fn new(method: BroadPhaseMethod) -> Self {
        Self {
            method,
            sweep_and_prune: SweepAndPrune::new(),
            spatial_hash: SpatialHash::new(2.0), // 2m cell size
        }
    }

    /// Detects potentially colliding pairs.
    pub fn detect_pairs(
        &mut self,
        bodies: &[(usize, AABB)],
    ) -> Vec<CollisionPair> {
        match self.method {
            BroadPhaseMethod::SweepAndPrune => {
                self.sweep_and_prune.detect_pairs(bodies)
            }
            BroadPhaseMethod::SpatialHash => {
                self.spatial_hash.detect_pairs(bodies)
            }
            BroadPhaseMethod::BVH => {
                // Simplified: fall back to sweep and prune
                self.sweep_and_prune.detect_pairs(bodies)
            }
        }
    }

    /// Updates configuration.
    pub fn set_method(&mut self, method: BroadPhaseMethod) {
        self.method = method;
    }
}

/// Sweep and Prune algorithm.
///
/// Sorts AABBs along an axis and detects overlaps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SweepAndPrune {
    /// Sort axis (0=x, 1=y, 2=z).
    sort_axis: usize,
}

impl SweepAndPrune {
    /// Creates a new sweep and prune detector.
    pub fn new() -> Self {
        Self { sort_axis: 0 }
    }

    /// Detects potentially colliding pairs.
    pub fn detect_pairs(&mut self, bodies: &[(usize, AABB)]) -> Vec<CollisionPair> {
        if bodies.len() < 2 {
            return Vec::new();
        }

        // Choose best axis (with maximum variance)
        self.choose_best_axis(bodies);

        // Create endpoints
        let mut endpoints: Vec<Endpoint> = Vec::with_capacity(bodies.len() * 2);
        for (id, aabb) in bodies {
            endpoints.push(Endpoint {
                value: self.get_min(aabb),
                body_id: *id,
                is_min: true,
            });
            endpoints.push(Endpoint {
                value: self.get_max(aabb),
                body_id: *id,
                is_min: false,
            });
        }

        // Sort endpoints
        endpoints.sort_by(|a, b| a.value.partial_cmp(&b.value).unwrap());

        // Sweep and detect overlaps
        let mut pairs = Vec::new();
        let mut active: HashSet<usize> = HashSet::new();

        for endpoint in endpoints {
            if endpoint.is_min {
                // Check against all active AABBs
                for &active_id in &active {
                    if active_id != endpoint.body_id {
                        pairs.push(CollisionPair::new(endpoint.body_id, active_id));
                    }
                }
                active.insert(endpoint.body_id);
            } else {
                active.remove(&endpoint.body_id);
            }
        }

        // Filter by full AABB overlap
        let aabb_map: HashMap<usize, AABB> = bodies.iter().copied().collect();
        pairs.retain(|pair| {
            if let (Some(aabb_a), Some(aabb_b)) = (aabb_map.get(&pair.body_a), aabb_map.get(&pair.body_b)) {
                aabb_a.overlaps(aabb_b)
            } else {
                false
            }
        });

        pairs
    }

    /// Chooses the best sorting axis based on variance.
    fn choose_best_axis(&mut self, bodies: &[(usize, AABB)]) {
        let mut variance = [0.0; 3];

        for axis in 0..3 {
            let centers: Vec<f64> = bodies.iter().map(|(_, aabb)| aabb.center()[axis]).collect();
            let mean = centers.iter().sum::<f64>() / centers.len() as f64;
            variance[axis] = centers.iter().map(|c| (c - mean).powi(2)).sum::<f64>() / centers.len() as f64;
        }

        self.sort_axis = if variance[0] >= variance[1] && variance[0] >= variance[2] {
            0
        } else if variance[1] >= variance[2] {
            1
        } else {
            2
        };
    }

    fn get_min(&self, aabb: &AABB) -> f64 {
        aabb.min[self.sort_axis]
    }

    fn get_max(&self, aabb: &AABB) -> f64 {
        aabb.max[self.sort_axis]
    }
}

impl Default for SweepAndPrune {
    fn default() -> Self {
        Self::new()
    }
}

/// Endpoint for sweep and prune.
#[derive(Debug, Clone)]
struct Endpoint {
    value: f64,
    body_id: usize,
    is_min: bool,
}

/// Spatial hashing for broad phase.
///
/// Divides space into grid cells and checks only nearby cells.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialHash {
    /// Cell size.
    cell_size: f64,

    /// Hash table: cell key -> body IDs.
    #[serde(skip)]
    cells: HashMap<CellKey, Vec<usize>>,
}

impl SpatialHash {
    /// Creates a new spatial hash with given cell size.
    pub fn new(cell_size: f64) -> Self {
        Self {
            cell_size,
            cells: HashMap::new(),
        }
    }

    /// Detects potentially colliding pairs.
    pub fn detect_pairs(&mut self, bodies: &[(usize, AABB)]) -> Vec<CollisionPair> {
        // Clear and rebuild hash
        self.cells.clear();

        // Insert bodies into cells
        for (id, aabb) in bodies {
            let cells = self.get_cells_for_aabb(aabb);
            for cell in cells {
                self.cells.entry(cell).or_insert_with(Vec::new).push(*id);
            }
        }

        // Find pairs
        let mut pairs = HashSet::new();

        for cell_bodies in self.cells.values() {
            for i in 0..cell_bodies.len() {
                for j in (i + 1)..cell_bodies.len() {
                    pairs.insert(CollisionPair::new(cell_bodies[i], cell_bodies[j]));
                }
            }
        }

        // Filter by actual AABB overlap
        let aabb_map: HashMap<usize, AABB> = bodies.iter().copied().collect();
        pairs
            .into_iter()
            .filter(|pair| {
                if let (Some(aabb_a), Some(aabb_b)) = (aabb_map.get(&pair.body_a), aabb_map.get(&pair.body_b)) {
                    aabb_a.overlaps(aabb_b)
                } else {
                    false
                }
            })
            .collect()
    }

    /// Gets all cells overlapping an AABB.
    fn get_cells_for_aabb(&self, aabb: &AABB) -> Vec<CellKey> {
        let min_cell = self.world_to_cell(aabb.min);
        let max_cell = self.world_to_cell(aabb.max);

        let mut cells = Vec::new();
        for x in min_cell.x..=max_cell.x {
            for y in min_cell.y..=max_cell.y {
                for z in min_cell.z..=max_cell.z {
                    cells.push(CellKey { x, y, z });
                }
            }
        }
        cells
    }

    /// Converts world position to cell coordinates.
    fn world_to_cell(&self, pos: Vector3<f64>) -> CellKey {
        CellKey {
            x: (pos.x / self.cell_size).floor() as i32,
            y: (pos.y / self.cell_size).floor() as i32,
            z: (pos.z / self.cell_size).floor() as i32,
        }
    }
}

/// Cell key for spatial hash.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct CellKey {
    x: i32,
    y: i32,
    z: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sweep_and_prune() {
        let mut sap = SweepAndPrune::new();

        let bodies = vec![
            (0, AABB::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 1.0, 1.0))),
            (1, AABB::new(Vector3::new(0.5, 0.5, 0.5), Vector3::new(1.5, 1.5, 1.5))),
            (2, AABB::new(Vector3::new(5.0, 5.0, 5.0), Vector3::new(6.0, 6.0, 6.0))),
        ];

        let pairs = sap.detect_pairs(&bodies);

        // Bodies 0 and 1 should overlap, but not with 2
        assert_eq!(pairs.len(), 1);
        assert!(pairs.contains(&CollisionPair::new(0, 1)));
    }

    #[test]
    fn test_spatial_hash() {
        let mut hash = SpatialHash::new(2.0);

        let bodies = vec![
            (0, AABB::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 1.0, 1.0))),
            (1, AABB::new(Vector3::new(0.5, 0.5, 0.5), Vector3::new(1.5, 1.5, 1.5))),
            (2, AABB::new(Vector3::new(10.0, 10.0, 10.0), Vector3::new(11.0, 11.0, 11.0))),
        ];

        let pairs = hash.detect_pairs(&bodies);

        assert_eq!(pairs.len(), 1);
        assert!(pairs.contains(&CollisionPair::new(0, 1)));
    }
}
