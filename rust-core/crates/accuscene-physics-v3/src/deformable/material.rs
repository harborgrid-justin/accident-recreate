//! Material property definitions for deformable bodies.
//!
//! Provides material models for:
//! - Steel (structural components)
//! - Aluminum (lightweight structures)
//! - Plastic (bumpers, trim)
//! - Rubber (tires, seals)
//! - Composite materials

use serde::{Deserialize, Serialize};

/// Material model for FEM simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialModel {
    /// Name of the material.
    pub name: String,

    /// Density (kg/m³).
    pub density: f64,

    /// Young's modulus (Pa) - stiffness.
    pub youngs_modulus: f64,

    /// Poisson's ratio (dimensionless).
    pub poisson_ratio: f64,

    /// Yield strength (Pa) - onset of plastic deformation.
    pub yield_strength: f64,

    /// Ultimate tensile strength (Pa) - fracture point.
    pub ultimate_strength: f64,

    /// Lamé's first parameter λ.
    pub lame_lambda: f64,

    /// Lamé's second parameter μ (shear modulus).
    pub lame_mu: f64,

    /// Bulk modulus K.
    pub bulk_modulus: f64,
}

impl MaterialModel {
    /// Creates a new material model with computed Lamé parameters.
    pub fn new(
        name: String,
        density: f64,
        youngs_modulus: f64,
        poisson_ratio: f64,
        yield_strength: f64,
        ultimate_strength: f64,
    ) -> Self {
        // Compute Lamé parameters from E and ν
        // λ = E * ν / ((1 + ν) * (1 - 2ν))
        // μ = E / (2 * (1 + ν))  (shear modulus G)

        let lame_lambda = youngs_modulus * poisson_ratio
            / ((1.0 + poisson_ratio) * (1.0 - 2.0 * poisson_ratio));

        let lame_mu = youngs_modulus / (2.0 * (1.0 + poisson_ratio));

        // Bulk modulus: K = E / (3 * (1 - 2ν))
        let bulk_modulus = youngs_modulus / (3.0 * (1.0 - 2.0 * poisson_ratio));

        Self {
            name,
            density,
            youngs_modulus,
            poisson_ratio,
            yield_strength,
            ultimate_strength,
            lame_lambda,
            lame_mu,
            bulk_modulus,
        }
    }

    /// Structural steel (typical automotive steel).
    pub fn steel() -> Self {
        Self::new(
            "Steel".to_string(),
            7850.0,              // kg/m³
            200e9,               // Pa (200 GPa)
            0.30,                // dimensionless
            250e6,               // Pa (250 MPa)
            400e6,               // Pa (400 MPa)
        )
    }

    /// High-strength steel (for safety structures).
    pub fn high_strength_steel() -> Self {
        Self::new(
            "High-Strength Steel".to_string(),
            7850.0,
            210e9,
            0.30,
            550e6,   // Higher yield strength
            700e6,
        )
    }

    /// Aluminum alloy (6061-T6).
    pub fn aluminum() -> Self {
        Self::new(
            "Aluminum 6061-T6".to_string(),
            2700.0,
            69e9,
            0.33,
            95e6,    // Lower yield than steel
            110e6,
        )
    }

    /// ABS plastic (bumpers, interior components).
    pub fn abs_plastic() -> Self {
        Self::new(
            "ABS Plastic".to_string(),
            1050.0,
            2.3e9,
            0.35,
            40e6,
            45e6,
        )
    }

    /// Polycarbonate (windshields, windows).
    pub fn polycarbonate() -> Self {
        Self::new(
            "Polycarbonate".to_string(),
            1200.0,
            2.4e9,
            0.37,
            60e6,
            70e6,
        )
    }

    /// Rubber (tires, seals).
    pub fn rubber() -> Self {
        Self::new(
            "Rubber".to_string(),
            1200.0,
            0.05e9,  // Very low Young's modulus (soft)
            0.49,     // Nearly incompressible
            15e6,
            20e6,
        )
    }

    /// Carbon fiber composite.
    pub fn carbon_fiber() -> Self {
        Self::new(
            "Carbon Fiber Composite".to_string(),
            1600.0,
            150e9,   // High stiffness
            0.30,
            600e6,   // High strength
            1200e6,
        )
    }

    /// Glass fiber composite.
    pub fn glass_fiber() -> Self {
        Self::new(
            "Glass Fiber Composite".to_string(),
            1800.0,
            40e9,
            0.30,
            400e6,
            800e6,
        )
    }

    /// Soft foam (energy absorption).
    pub fn foam() -> Self {
        Self::new(
            "Foam".to_string(),
            50.0,     // Very low density
            0.01e9,   // Very soft
            0.10,
            0.5e6,
            1e6,
        )
    }

    /// Computes the speed of sound in this material.
    ///
    /// c = √(E / ρ) for longitudinal waves
    pub fn speed_of_sound(&self) -> f64 {
        (self.youngs_modulus / self.density).sqrt()
    }

    /// Computes the shear wave speed.
    ///
    /// c_s = √(μ / ρ)
    pub fn shear_wave_speed(&self) -> f64 {
        (self.lame_mu / self.density).sqrt()
    }

    /// Checks if the material can handle a given stress without yielding.
    pub fn is_elastic(&self, stress: f64) -> bool {
        stress < self.yield_strength
    }

    /// Checks if the material has fractured.
    pub fn is_fractured(&self, stress: f64) -> bool {
        stress > self.ultimate_strength
    }
}

/// Material failure modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailureMode {
    /// Elastic deformation (recoverable).
    Elastic,

    /// Plastic deformation (permanent).
    Plastic,

    /// Fracture (structural failure).
    Fractured,
}

impl MaterialModel {
    /// Determines failure mode based on stress level.
    pub fn failure_mode(&self, stress: f64) -> FailureMode {
        if stress > self.ultimate_strength {
            FailureMode::Fractured
        } else if stress > self.yield_strength {
            FailureMode::Plastic
        } else {
            FailureMode::Elastic
        }
    }
}

/// Database of common automotive materials.
pub struct MaterialDatabase;

impl MaterialDatabase {
    /// Returns all available material presets.
    pub fn all_materials() -> Vec<MaterialModel> {
        vec![
            MaterialModel::steel(),
            MaterialModel::high_strength_steel(),
            MaterialModel::aluminum(),
            MaterialModel::abs_plastic(),
            MaterialModel::polycarbonate(),
            MaterialModel::rubber(),
            MaterialModel::carbon_fiber(),
            MaterialModel::glass_fiber(),
            MaterialModel::foam(),
        ]
    }

    /// Finds a material by name (case-insensitive).
    pub fn find_by_name(name: &str) -> Option<MaterialModel> {
        Self::all_materials()
            .into_iter()
            .find(|m| m.name.to_lowercase() == name.to_lowercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_steel_material() {
        let steel = MaterialModel::steel();
        assert_eq!(steel.density, 7850.0);
        assert_eq!(steel.youngs_modulus, 200e9);
        assert!(steel.lame_lambda > 0.0);
        assert!(steel.lame_mu > 0.0);
    }

    #[test]
    fn test_lame_parameters() {
        let steel = MaterialModel::steel();

        // Verify relationship: E = μ(3λ + 2μ) / (λ + μ)
        let e_computed = steel.lame_mu * (3.0 * steel.lame_lambda + 2.0 * steel.lame_mu)
            / (steel.lame_lambda + steel.lame_mu);

        assert_relative_eq!(e_computed, steel.youngs_modulus, epsilon = 1e-3);
    }

    #[test]
    fn test_speed_of_sound() {
        let steel = MaterialModel::steel();
        let speed = steel.speed_of_sound();

        // Speed of sound in steel is approximately 5000 m/s
        assert!(speed > 4000.0 && speed < 6000.0);
    }

    #[test]
    fn test_failure_mode() {
        let steel = MaterialModel::steel();

        assert_eq!(steel.failure_mode(100e6), FailureMode::Elastic);
        assert_eq!(steel.failure_mode(300e6), FailureMode::Plastic);
        assert_eq!(steel.failure_mode(500e6), FailureMode::Fractured);
    }

    #[test]
    fn test_material_database() {
        let materials = MaterialDatabase::all_materials();
        assert!(materials.len() >= 9);

        let steel = MaterialDatabase::find_by_name("steel");
        assert!(steel.is_some());
        assert_eq!(steel.unwrap().name, "Steel");
    }
}
