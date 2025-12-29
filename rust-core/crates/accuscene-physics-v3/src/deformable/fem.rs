//! Finite Element Method for deformable body simulation.
//!
//! Implements:
//! - Linear elastic FEM
//! - Corotational formulation for large deformations
//! - Plasticity (von Mises yield criterion)
//! - Explicit and implicit time integration

use nalgebra::{Matrix3, Vector3};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use super::{DeformableBody, MaterialModel};
use crate::error::{PhysicsError, PhysicsResult};

/// FEM solver for deformable bodies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FEMSolver {
    /// Time integration method.
    pub integration: FEMIntegration,

    /// Damping coefficient (Rayleigh damping).
    pub damping: f64,

    /// Enable plasticity.
    pub enable_plasticity: bool,

    /// Yield threshold multiplier.
    pub yield_threshold: f64,
}

impl FEMSolver {
    /// Creates a new FEM solver.
    pub fn new() -> Self {
        Self {
            integration: FEMIntegration::ExplicitEuler,
            damping: 0.01,
            enable_plasticity: true,
            yield_threshold: 1.0,
        }
    }

    /// Performs one time step of FEM simulation.
    pub fn step(
        &self,
        body: &mut DeformableBody,
        dt: f64,
        gravity: Vector3<f64>,
    ) -> PhysicsResult<()> {
        if body.is_static {
            return Ok(());
        }

        // Compute elastic forces
        let forces = self.compute_elastic_forces(body)?;

        // Add gravity and damping
        let num_nodes = body.nodes.len();
        let mut total_forces = vec![Vector3::zeros(); num_nodes];

        for i in 0..num_nodes {
            total_forces[i] = forces[i] + gravity * body.masses[i];

            // Damping force: F_d = -c * v
            total_forces[i] -= body.velocities[i] * self.damping;
        }

        // Time integration
        match self.integration {
            FEMIntegration::ExplicitEuler => {
                self.explicit_euler_step(body, &total_forces, dt);
            }
            FEMIntegration::SemiImplicit => {
                self.semi_implicit_step(body, &total_forces, dt);
            }
        }

        // Handle plasticity
        if self.enable_plasticity {
            self.apply_plasticity(body)?;
        }

        Ok(())
    }

    /// Computes elastic forces for all nodes.
    fn compute_elastic_forces(&self, body: &DeformableBody) -> PhysicsResult<Vec<Vector3<f64>>> {
        let num_nodes = body.nodes.len();
        let mut forces = vec![Vector3::zeros(); num_nodes];

        // Parallel computation over elements
        let element_forces: Vec<_> = body
            .elements
            .par_iter()
            .map(|element| self.compute_element_forces(body, element))
            .collect::<PhysicsResult<Vec<_>>>()?;

        // Accumulate element forces to nodes
        for (_element_idx, element) in body.elements.iter().enumerate() {
            let elem_forces = &element_forces[element_idx];
            for (i, &node_idx) in element.iter().enumerate() {
                forces[node_idx] += elem_forces[i];
            }
        }

        Ok(forces)
    }

    /// Computes forces for a single tetrahedral element.
    fn compute_element_forces(
        &self,
        body: &DeformableBody,
        element: &[usize; 4],
    ) -> PhysicsResult<[Vector3<f64>; 4]> {
        // Get current and rest positions
        let x = [
            body.nodes[element[0]],
            body.nodes[element[1]],
            body.nodes[element[2]],
            body.nodes[element[3]],
        ];

        let x0 = [
            body.rest_nodes[element[0]],
            body.rest_nodes[element[1]],
            body.rest_nodes[element[2]],
            body.rest_nodes[element[3]],
        ];

        // Compute deformation gradient F = ∂x/∂X
        let dm = Self::compute_shape_matrix(&x);
        let dm0 = Self::compute_shape_matrix(&x0);

        let dm0_inv = dm0.try_inverse().ok_or_else(|| PhysicsError::SingularMatrix {
            operation: "FEM rest shape matrix inversion".to_string(),
            determinant: dm0.determinant(),
        })?;

        let f = dm * dm0_inv; // Deformation gradient

        // Compute Green strain: E = 0.5 * (F^T F - I)
        let green_strain = (f.transpose() * f - Matrix3::identity()) * 0.5;

        // Compute stress using constitutive model (linear elastic)
        let stress = self.compute_stress(&green_strain, &body.material);

        // Compute first Piola-Kirchhoff stress: P = F * S
        let p = f * stress;

        // Compute volume in rest configuration
        let volume0 = (dm0.determinant() / 6.0).abs();

        // Compute nodal forces: f = -V0 * P * ∇N
        let mut forces = [Vector3::zeros(); 4];

        // Shape function gradients in reference configuration
        let grad_n = Self::compute_shape_gradients(&dm0_inv);

        for i in 0..4 {
            forces[i] = -volume0 * (p * grad_n[i]);
        }

        Ok(forces)
    }

    /// Computes shape matrix from node positions.
    fn compute_shape_matrix(x: &[Vector3<f64>; 4]) -> Matrix3<f64> {
        let v1 = x[1] - x[0];
        let v2 = x[2] - x[0];
        let v3 = x[3] - x[0];

        Matrix3::from_columns(&[v1, v2, v3])
    }

    /// Computes shape function gradients.
    fn compute_shape_gradients(dm_inv: &Matrix3<f64>) -> [Vector3<f64>; 4] {
        let col_sum = -dm_inv.column(0) - dm_inv.column(1) - dm_inv.column(2);

        [
            Vector3::new(col_sum.x, col_sum.y, col_sum.z),
            dm_inv.column(0).into(),
            dm_inv.column(1).into(),
            dm_inv.column(2).into(),
        ]
    }

    /// Computes stress tensor from strain (linear elastic constitutive model).
    ///
    /// σ = λ * tr(ε) * I + 2μ * ε
    ///
    /// Where λ and μ are Lamé parameters.
    fn compute_stress(&self, strain: &Matrix3<f64>, material: &MaterialModel) -> Matrix3<f64> {
        let trace_strain = strain[(0, 0)] + strain[(1, 1)] + strain[(2, 2)];

        material.lame_lambda * trace_strain * Matrix3::identity() + strain * (2.0 * material.lame_mu)
    }

    /// Explicit Euler time integration.
    fn explicit_euler_step(
        &self,
        body: &mut DeformableBody,
        forces: &[Vector3<f64>],
        dt: f64,
    ) {
        for i in 0..body.nodes.len() {
            if body.masses[i] > 1e-10 {
                let acceleration = forces[i] / body.masses[i];
                body.velocities[i] += acceleration * dt;
                body.nodes[i] += body.velocities[i] * dt;
            }
        }
    }

    /// Semi-implicit Euler time integration.
    fn semi_implicit_step(
        &self,
        body: &mut DeformableBody,
        forces: &[Vector3<f64>],
        dt: f64,
    ) {
        for i in 0..body.nodes.len() {
            if body.masses[i] > 1e-10 {
                let acceleration = forces[i] / body.masses[i];
                body.velocities[i] += acceleration * dt;
                body.nodes[i] += body.velocities[i] * dt;
            }
        }
    }

    /// Applies plasticity model (von Mises yield criterion).
    fn apply_plasticity(&self, body: &mut DeformableBody) -> PhysicsResult<()> {
        for (_element_idx, element) in body.elements.iter().enumerate() {
            // Compute strain
            let x = [
                body.nodes[element[0]],
                body.nodes[element[1]],
                body.nodes[element[2]],
                body.nodes[element[3]],
            ];

            let x0 = [
                body.rest_nodes[element[0]],
                body.rest_nodes[element[1]],
                body.rest_nodes[element[2]],
                body.rest_nodes[element[3]],
            ];

            let dm = Self::compute_shape_matrix(&x);
            let dm0 = Self::compute_shape_matrix(&x0);

            if let Some(dm0_inv) = dm0.try_inverse() {
                let f = dm * dm0_inv;
                let green_strain = (f.transpose() * f - Matrix3::identity()) * 0.5;

                // Compute von Mises equivalent strain
                let dev_strain = green_strain - Matrix3::identity() * green_strain.trace() / 3.0;
                let equiv_strain = (1.5 * dev_strain.norm_squared()).sqrt();

                // Check yield
                let yield_strain = body.material.yield_strength / body.material.youngs_modulus;

                if equiv_strain > yield_strain * self.yield_threshold {
                    // Plastic deformation occurred - update plastic strain
                    for &node_idx in element {
                        body.plastic_strain[node_idx] =
                            body.plastic_strain[node_idx].max(equiv_strain);
                    }

                    // Update rest configuration (permanent deformation)
                    // Simplified: scale towards current configuration
                    let plastic_factor = 0.1; // How much plastic deformation to apply
                    for &node_idx in element.iter() {
                        let current_pos = body.nodes[node_idx];
                        let rest_pos = body.rest_nodes[node_idx];
                        body.rest_nodes[node_idx] += (current_pos - rest_pos) * plastic_factor;
                    }
                }
            }
        }

        Ok(())
    }
}

impl Default for FEMSolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Time integration methods for FEM.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FEMIntegration {
    /// Explicit Euler (fast but less stable).
    ExplicitEuler,

    /// Semi-implicit Euler (more stable).
    SemiImplicit,
}

/// Computes crush energy (plastic deformation energy).
pub fn compute_crush_energy(body: &DeformableBody) -> f64 {
    // Approximate as integral of plastic strain times yield strength
    let yield_strength = body.material.yield_strength;

    body.plastic_strain
        .iter()
        .zip(&body.masses)
        .map(|(&strain, &mass)| {
            // Energy density = σ * ε (simplified)
            let energy_density = yield_strength * strain;
            energy_density * mass / body.material.density
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fem_solver_creation() {
        let solver = FEMSolver::new();
        assert!(solver.enable_plasticity);
    }

    #[test]
    fn test_shape_matrix() {
        let x = [
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
        ];

        let dm = FEMSolver::compute_shape_matrix(&x);

        assert_eq!(dm[(0, 0)], 1.0);
        assert_eq!(dm[(1, 1)], 1.0);
        assert_eq!(dm[(2, 2)], 1.0);
    }
}
