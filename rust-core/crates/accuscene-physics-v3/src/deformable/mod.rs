//! Deformable body simulation.
//!
//! This module implements soft body and deformable object simulation using:
//! - Finite Element Method (FEM) for structural analysis
//! - Plasticity models for permanent deformation
//! - Material property databases

pub mod fem;
pub mod material;

pub use fem::*;
pub use material::*;

use nalgebra::Vector3;
use serde::{Deserialize, Serialize};

/// Deformable body representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeformableBody {
    /// Unique identifier.
    pub id: usize,

    /// Mesh nodes (positions).
    pub nodes: Vec<Vector3<f64>>,

    /// Node velocities.
    pub velocities: Vec<Vector3<f64>>,

    /// Nodal masses.
    pub masses: Vec<f64>,

    /// Tetrahedral elements (indices into nodes).
    pub elements: Vec<[usize; 4]>,

    /// Material properties.
    pub material: MaterialModel,

    /// Rest configuration (for computing strain).
    pub rest_nodes: Vec<Vector3<f64>>,

    /// Plastic deformation state.
    pub plastic_strain: Vec<f64>,

    /// Is the body static?
    pub is_static: bool,
}

impl DeformableBody {
    /// Creates a new deformable body.
    pub fn new(
        id: usize,
        nodes: Vec<Vector3<f64>>,
        elements: Vec<[usize; 4]>,
        material: MaterialModel,
        density: f64,
    ) -> Self {
        let num_nodes = nodes.len();

        // Compute nodal masses (distribute element mass to nodes)
        let mut masses = vec![0.0; num_nodes];
        for element in &elements {
            let volume = Self::compute_element_volume(&nodes, element);
            let element_mass = volume * density;
            let node_mass = element_mass / 4.0; // Distribute to 4 nodes

            for &node_idx in element {
                masses[node_idx] += node_mass;
            }
        }

        Self {
            id,
            rest_nodes: nodes.clone(),
            nodes,
            velocities: vec![Vector3::zeros(); num_nodes],
            masses,
            elements,
            material,
            plastic_strain: vec![0.0; num_nodes],
            is_static: false,
        }
    }

    /// Computes volume of a tetrahedral element.
    fn compute_element_volume(nodes: &[Vector3<f64>], element: &[usize; 4]) -> f64 {
        let p0 = nodes[element[0]];
        let p1 = nodes[element[1]];
        let p2 = nodes[element[2]];
        let p3 = nodes[element[3]];

        let v1 = p1 - p0;
        let v2 = p2 - p0;
        let v3 = p3 - p0;

        (v1.cross(&v2).dot(&v3) / 6.0).abs()
    }

    /// Creates a box-shaped deformable body with subdivision.
    pub fn create_box(
        id: usize,
        center: Vector3<f64>,
        size: Vector3<f64>,
        subdivisions: [usize; 3],
        material: MaterialModel,
        density: f64,
    ) -> Self {
        let mut nodes = Vec::new();
        let mut elements = Vec::new();

        let half_size = size * 0.5;
        let dx = size.x / subdivisions[0] as f64;
        let dy = size.y / subdivisions[1] as f64;
        let dz = size.z / subdivisions[2] as f64;

        // Create grid of nodes
        for i in 0..=subdivisions[0] {
            for j in 0..=subdivisions[1] {
                for k in 0..=subdivisions[2] {
                    let x = -half_size.x + i as f64 * dx;
                    let y = -half_size.y + j as f64 * dy;
                    let z = -half_size.z + k as f64 * dz;
                    nodes.push(center + Vector3::new(x, y, z));
                }
            }
        }

        // Create tetrahedral elements from grid
        let ny = subdivisions[1] + 1;
        let nz = subdivisions[2] + 1;

        for i in 0..subdivisions[0] {
            for j in 0..subdivisions[1] {
                for k in 0..subdivisions[2] {
                    let idx = |di: usize, dj: usize, dk: usize| -> usize {
                        (i + di) * ny * nz + (j + dj) * nz + (k + dk)
                    };

                    // Split cube into 5 tetrahedra
                    elements.push([idx(0, 0, 0), idx(1, 0, 0), idx(1, 1, 0), idx(1, 0, 1)]);
                    elements.push([idx(0, 0, 0), idx(1, 1, 0), idx(0, 1, 0), idx(0, 1, 1)]);
                    elements.push([idx(0, 0, 0), idx(1, 0, 1), idx(0, 1, 1), idx(0, 0, 1)]);
                    elements.push([idx(1, 1, 1), idx(1, 0, 1), idx(1, 1, 0), idx(0, 1, 1)]);
                    elements.push([idx(0, 0, 0), idx(1, 1, 0), idx(1, 0, 1), idx(0, 1, 1)]);
                }
            }
        }

        Self::new(id, nodes, elements, material, density)
    }

    /// Computes total kinetic energy.
    pub fn kinetic_energy(&self) -> f64 {
        self.velocities
            .iter()
            .zip(&self.masses)
            .map(|(v, &m)| 0.5 * m * v.norm_squared())
            .sum()
    }

    /// Computes center of mass.
    pub fn center_of_mass(&self) -> Vector3<f64> {
        let total_mass: f64 = self.masses.iter().sum();
        if total_mass < 1e-10 {
            return Vector3::zeros();
        }

        let weighted_sum: Vector3<f64> = self
            .nodes
            .iter()
            .zip(&self.masses)
            .map(|(pos, &mass)| pos * mass)
            .sum();

        weighted_sum / total_mass
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_deformable_body_creation() {
        let material = MaterialModel::steel();
        let body = DeformableBody::create_box(
            0,
            Vector3::zeros(),
            Vector3::new(1.0, 1.0, 1.0),
            [2, 2, 2],
            material,
            7850.0,
        );

        assert!(!body.nodes.is_empty());
        assert!(!body.elements.is_empty());
        assert_eq!(body.nodes.len(), body.velocities.len());
    }

    #[test]
    fn test_element_volume() {
        let nodes = vec![
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
        ];

        let element = [0, 1, 2, 3];
        let volume = DeformableBody::compute_element_volume(&nodes, &element);

        // Volume of unit tetrahedron is 1/6
        assert_relative_eq!(volume, 1.0 / 6.0, epsilon = 1e-6);
    }
}
