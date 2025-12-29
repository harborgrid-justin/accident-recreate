//! Deformation energy calculations for crush analysis.
//!
//! Essential for accident reconstruction to estimate:
//! - Energy absorbed by vehicle deformation
//! - Crush depth and patterns
//! - Impact severity

use nalgebra::{Matrix3, Vector3};
use serde::{Deserialize, Serialize};

use crate::deformable::{DeformableBody, MaterialModel};

/// Deformation energy analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeformationEnergy {
    /// Total elastic energy stored (J).
    pub elastic_energy: f64,

    /// Total plastic deformation energy dissipated (J).
    pub plastic_energy: f64,

    /// Total deformation energy (J).
    pub total_energy: f64,

    /// Maximum plastic strain.
    pub max_plastic_strain: f64,

    /// Average plastic strain.
    pub average_plastic_strain: f64,

    /// Volume of plastically deformed material (m³).
    pub deformed_volume: f64,
}

impl DeformationEnergy {
    /// Computes deformation energy from a deformable body.
    pub fn from_body(body: &DeformableBody) -> Self {
        let elastic_energy = compute_elastic_energy(body);
        let plastic_energy = compute_plastic_energy(body);
        let total_energy = elastic_energy + plastic_energy;

        let max_plastic_strain = body
            .plastic_strain
            .iter()
            .copied()
            .fold(0.0, f64::max);

        let average_plastic_strain = if !body.plastic_strain.is_empty() {
            body.plastic_strain.iter().sum::<f64>() / body.plastic_strain.len() as f64
        } else {
            0.0
        };

        let deformed_volume = estimate_deformed_volume(body);

        Self {
            elastic_energy,
            plastic_energy,
            total_energy,
            max_plastic_strain,
            average_plastic_strain,
            deformed_volume,
        }
    }

    /// Estimates equivalent impact speed that would produce this deformation.
    ///
    /// Assumes all kinetic energy converts to deformation:
    /// 0.5 * m * v² = E_def
    pub fn equivalent_impact_speed(&self, mass: f64) -> f64 {
        if mass > 1e-6 {
            (2.0 * self.total_energy / mass).sqrt()
        } else {
            0.0
        }
    }

    /// Generates a crush analysis report.
    pub fn crush_report(&self) -> String {
        format!(
            "Crush Analysis:\n\
             Total Deformation Energy: {:.2} kJ\n\
             - Elastic (recoverable): {:.2} kJ\n\
             - Plastic (permanent): {:.2} kJ\n\
             Maximum Plastic Strain: {:.1}%\n\
             Average Plastic Strain: {:.1}%\n\
             Deformed Volume: {:.4} m³",
            self.total_energy / 1000.0,
            self.elastic_energy / 1000.0,
            self.plastic_energy / 1000.0,
            self.max_plastic_strain * 100.0,
            self.average_plastic_strain * 100.0,
            self.deformed_volume
        )
    }
}

/// Computes elastic energy stored in a deformable body.
///
/// U_elastic = 0.5 * ∫ σ:ε dV
/// For linear elasticity: U = 0.5 * k * x²
pub fn compute_elastic_energy(body: &DeformableBody) -> f64 {
    let mut total_energy = 0.0;

    for element in &body.elements {
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

        // Compute deformation gradient
        let dm = compute_shape_matrix(&x);
        let dm0 = compute_shape_matrix(&x0);

        if let Some(dm0_inv) = dm0.try_inverse() {
            let f = dm * dm0_inv;

            // Green strain: E = 0.5 * (F^T F - I)
            let green_strain = (f.transpose() * f - Matrix3::identity()) * 0.5;

            // Compute stress (linear elastic)
            let stress = compute_stress(&green_strain, &body.material);

            // Strain energy density: W = 0.5 * σ:ε
            let energy_density = 0.5 * tensor_double_dot(&stress, &green_strain);

            // Element volume
            let volume = (dm0.determinant() / 6.0).abs();

            total_energy += energy_density * volume;
        }
    }

    total_energy
}

/// Computes plastic deformation energy dissipated.
///
/// E_plastic = ∫ σ_y * ε_p dV
/// Where σ_y is yield stress and ε_p is plastic strain.
pub fn compute_plastic_energy(body: &DeformableBody) -> f64 {
    let yield_strength = body.material.yield_strength;

    body.plastic_strain
        .iter()
        .zip(&body.masses)
        .map(|(&strain, &mass)| {
            // Energy density = σ_y * ε_p
            let energy_density = yield_strength * strain;

            // Convert mass to volume: V = m / ρ
            let volume = mass / body.material.density;

            energy_density * volume
        })
        .sum()
}

/// Computes total deformation energy (elastic + plastic).
pub fn compute_deformation_energy(body: &DeformableBody) -> f64 {
    compute_elastic_energy(body) + compute_plastic_energy(body)
}

/// Estimates volume of material that has undergone plastic deformation.
fn estimate_deformed_volume(body: &DeformableBody) -> f64 {
    let plastic_threshold = 0.01; // 1% plastic strain threshold

    body.plastic_strain
        .iter()
        .zip(&body.masses)
        .filter(|(&strain, _)| strain > plastic_threshold)
        .map(|(_, &mass)| mass / body.material.density)
        .sum()
}

/// Crush stiffness model for vehicle structures.
///
/// Force-deflection relationship: F = k * x^n
/// Where:
/// - k = stiffness coefficient
/// - x = crush depth
/// - n = exponent (typically 1.5-2.0 for vehicles)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrushStiffness {
    /// Stiffness coefficient (N/m^n).
    pub k: f64,

    /// Exponent.
    pub n: f64,
}

impl CrushStiffness {
    /// Typical passenger car front structure.
    pub fn passenger_car_front() -> Self {
        Self {
            k: 50e6,  // 50 MN/m^2
            n: 2.0,
        }
    }

    /// Typical passenger car side structure.
    pub fn passenger_car_side() -> Self {
        Self {
            k: 30e6,
            n: 1.8,
        }
    }

    /// Heavy truck structure.
    pub fn heavy_truck() -> Self {
        Self {
            k: 100e6,
            n: 2.0,
        }
    }

    /// Computes force at given crush depth.
    pub fn force_at_depth(&self, depth: f64) -> f64 {
        self.k * depth.powf(self.n)
    }

    /// Computes energy absorbed up to given crush depth.
    ///
    /// E = ∫ F dx = ∫ k*x^n dx = k/(n+1) * x^(n+1)
    pub fn energy_at_depth(&self, depth: f64) -> f64 {
        (self.k / (self.n + 1.0)) * depth.powf(self.n + 1.0)
    }

    /// Computes crush depth for given energy.
    ///
    /// x = (E * (n+1) / k)^(1/(n+1))
    pub fn depth_for_energy(&self, energy: f64) -> f64 {
        ((energy * (self.n + 1.0)) / self.k).powf(1.0 / (self.n + 1.0))
    }
}

/// Helper: compute shape matrix from tetrahedron vertices.
fn compute_shape_matrix(x: &[Vector3<f64>; 4]) -> Matrix3<f64> {
    let v1 = x[1] - x[0];
    let v2 = x[2] - x[0];
    let v3 = x[3] - x[0];

    Matrix3::from_columns(&[v1, v2, v3])
}

/// Helper: compute stress from strain (linear elastic).
fn compute_stress(strain: &Matrix3<f64>, material: &MaterialModel) -> Matrix3<f64> {
    let trace_strain = strain[(0, 0)] + strain[(1, 1)] + strain[(2, 2)];

    material.lame_lambda * trace_strain * Matrix3::identity() + strain * (2.0 * material.lame_mu)
}

/// Helper: tensor double dot product (σ:ε).
fn tensor_double_dot(a: &Matrix3<f64>, b: &Matrix3<f64>) -> f64 {
    let mut sum = 0.0;
    for i in 0..3 {
        for j in 0..3 {
            sum += a[(i, j)] * b[(i, j)];
        }
    }
    sum
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_crush_stiffness() {
        let crush = CrushStiffness::passenger_car_front();

        // Force at 0.1m crush
        let force = crush.force_at_depth(0.1);
        assert!(force > 0.0);

        // Energy at 0.1m crush
        let energy = crush.energy_at_depth(0.1);
        assert!(energy > 0.0);

        // Depth for given energy (should be invertible)
        let depth = crush.depth_for_energy(energy);
        assert_relative_eq!(depth, 0.1, epsilon = 1e-6);
    }

    #[test]
    fn test_crush_energy_scaling() {
        let crush = CrushStiffness::passenger_car_front();

        let energy_1 = crush.energy_at_depth(0.1);
        let energy_2 = crush.energy_at_depth(0.2);

        // Energy should scale as x^(n+1)
        // For n=2: E ~ x^3, so doubling x should give 8x energy
        assert!(energy_2 > energy_1 * 7.0 && energy_2 < energy_1 * 9.0);
    }

    #[test]
    fn test_deformation_energy_empty_body() {
        let material = MaterialModel::steel();
        let body = DeformableBody::new(0, vec![], vec![], material, 7850.0);

        let energy = compute_deformation_energy(&body);
        assert_eq!(energy, 0.0);
    }

    #[test]
    fn test_tensor_double_dot() {
        let a = Matrix3::identity();
        let b = Matrix3::identity();

        let result = tensor_double_dot(&a, &b);
        assert_eq!(result, 3.0); // Trace of identity
    }
}
