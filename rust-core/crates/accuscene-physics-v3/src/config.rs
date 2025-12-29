//! Configuration structures for the physics engine.
//!
//! This module defines all configuration parameters for simulation, including
//! time stepping, solver tolerances, material properties, and numerical methods.

use serde::{Deserialize, Serialize};

/// Global physics engine configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsConfig {
    /// Time step configuration.
    pub time_step: TimeStepConfig,

    /// Solver configuration.
    pub solver: SolverConfig,

    /// Collision detection configuration.
    pub collision: CollisionConfig,

    /// Deformable body configuration.
    pub deformable: DeformableConfig,

    /// Vehicle dynamics configuration.
    pub vehicle: VehicleConfig,

    /// Parallel execution configuration.
    pub parallel: ParallelConfig,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            time_step: TimeStepConfig::default(),
            solver: SolverConfig::default(),
            collision: CollisionConfig::default(),
            deformable: DeformableConfig::default(),
            vehicle: VehicleConfig::default(),
            parallel: ParallelConfig::default(),
        }
    }
}

/// Time integration configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeStepConfig {
    /// Fixed time step in seconds (default: 1/120 = 0.00833s).
    pub dt: f64,

    /// Maximum substeps per frame.
    pub max_substeps: usize,

    /// Integration method.
    pub method: IntegrationMethod,

    /// Enable adaptive time stepping.
    pub adaptive: bool,

    /// Minimum time step for adaptive stepping.
    pub min_dt: f64,

    /// Maximum time step for adaptive stepping.
    pub max_dt: f64,
}

impl Default for TimeStepConfig {
    fn default() -> Self {
        Self {
            dt: 1.0 / 120.0,
            max_substeps: 10,
            method: IntegrationMethod::SemiImplicitEuler,
            adaptive: false,
            min_dt: 1.0 / 1000.0,
            max_dt: 1.0 / 60.0,
        }
    }
}

/// Numerical integration methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntegrationMethod {
    /// Semi-implicit Euler (symplectic).
    SemiImplicitEuler,

    /// Velocity Verlet (2nd order).
    VelocityVerlet,

    /// Runge-Kutta 4th order.
    RK4,
}

/// Physics solver configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolverConfig {
    /// Maximum solver iterations.
    pub max_iterations: usize,

    /// Convergence tolerance.
    pub tolerance: f64,

    /// Position correction iterations.
    pub position_iterations: usize,

    /// Velocity correction iterations.
    pub velocity_iterations: usize,

    /// Baumgarte stabilization parameter (0.0 - 1.0).
    pub baumgarte_factor: f64,

    /// Contact slop (penetration tolerance) in meters.
    pub contact_slop: f64,

    /// Enable warm starting for iterative solver.
    pub warm_starting: bool,
}

impl Default for SolverConfig {
    fn default() -> Self {
        Self {
            max_iterations: 20,
            tolerance: 1e-4,
            position_iterations: 3,
            velocity_iterations: 8,
            baumgarte_factor: 0.2,
            contact_slop: 0.001, // 1mm
            warm_starting: true,
        }
    }
}

/// Collision detection configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollisionConfig {
    /// Broad phase method.
    pub broad_phase: BroadPhaseMethod,

    /// Enable continuous collision detection (CCD).
    pub enable_ccd: bool,

    /// CCD motion threshold (fraction of object size).
    pub ccd_threshold: f64,

    /// Contact generation tolerance.
    pub contact_tolerance: f64,

    /// Maximum contacts per collision pair.
    pub max_contacts: usize,

    /// Default restitution coefficient.
    pub default_restitution: f64,

    /// Default friction coefficient.
    pub default_friction: f64,
}

impl Default for CollisionConfig {
    fn default() -> Self {
        Self {
            broad_phase: BroadPhaseMethod::SweepAndPrune,
            enable_ccd: true,
            ccd_threshold: 0.5,
            contact_tolerance: 0.0001, // 0.1mm
            max_contacts: 4,
            default_restitution: 0.3,
            default_friction: 0.7,
        }
    }
}

/// Broad phase collision detection methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BroadPhaseMethod {
    /// Sweep and prune algorithm.
    SweepAndPrune,

    /// Spatial hashing.
    SpatialHash,

    /// Bounding volume hierarchy.
    BVH,
}

/// Deformable body simulation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeformableConfig {
    /// Enable deformable body simulation.
    pub enabled: bool,

    /// FEM solver iterations.
    pub fem_iterations: usize,

    /// FEM convergence tolerance.
    pub fem_tolerance: f64,

    /// Damping coefficient for deformable bodies.
    pub damping: f64,

    /// Plastic deformation threshold (yield stress).
    pub plastic_threshold: f64,

    /// Maximum plastic strain.
    pub max_plastic_strain: f64,
}

impl Default for DeformableConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            fem_iterations: 10,
            fem_tolerance: 1e-3,
            damping: 0.01,
            plastic_threshold: 250e6, // 250 MPa (steel yield strength)
            max_plastic_strain: 0.3,   // 30% strain
        }
    }
}

/// Vehicle dynamics configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleConfig {
    /// Enable advanced tire model (Pacejka Magic Formula).
    pub enable_magic_formula: bool,

    /// Tire relaxation length (meters).
    pub tire_relaxation_length: f64,

    /// Suspension stiffness (N/m).
    pub suspension_stiffness: f64,

    /// Suspension damping (N·s/m).
    pub suspension_damping: f64,

    /// Enable engine dynamics.
    pub enable_engine: bool,

    /// Gear ratios.
    pub gear_ratios: Vec<f64>,
}

impl Default for VehicleConfig {
    fn default() -> Self {
        Self {
            enable_magic_formula: true,
            tire_relaxation_length: 0.5,
            suspension_stiffness: 30000.0,
            suspension_damping: 3000.0,
            enable_engine: true,
            gear_ratios: vec![3.5, 2.1, 1.4, 1.0, 0.75], // Typical 5-speed ratios
        }
    }
}

/// Parallel execution configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelConfig {
    /// Enable parallel execution.
    pub enabled: bool,

    /// Number of worker threads (0 = auto-detect).
    pub num_threads: usize,

    /// Minimum batch size for parallel processing.
    pub min_batch_size: usize,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            num_threads: 0, // Auto-detect
            min_batch_size: 32,
        }
    }
}

/// Material properties for physical objects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialProperties {
    /// Density (kg/m³).
    pub density: f64,

    /// Young's modulus (Pa).
    pub youngs_modulus: f64,

    /// Poisson's ratio.
    pub poisson_ratio: f64,

    /// Yield strength (Pa).
    pub yield_strength: f64,

    /// Ultimate tensile strength (Pa).
    pub ultimate_strength: f64,

    /// Coefficient of restitution.
    pub restitution: f64,

    /// Coefficient of friction.
    pub friction: f64,
}

impl MaterialProperties {
    /// Steel material properties.
    pub fn steel() -> Self {
        Self {
            density: 7850.0,
            youngs_modulus: 200e9,
            poisson_ratio: 0.3,
            yield_strength: 250e6,
            ultimate_strength: 400e6,
            restitution: 0.2,
            friction: 0.8,
        }
    }

    /// Aluminum material properties.
    pub fn aluminum() -> Self {
        Self {
            density: 2700.0,
            youngs_modulus: 69e9,
            poisson_ratio: 0.33,
            yield_strength: 95e6,
            ultimate_strength: 110e6,
            restitution: 0.25,
            friction: 0.7,
        }
    }

    /// Plastic (ABS) material properties.
    pub fn plastic() -> Self {
        Self {
            density: 1050.0,
            youngs_modulus: 2.3e9,
            poisson_ratio: 0.35,
            yield_strength: 40e6,
            ultimate_strength: 45e6,
            restitution: 0.5,
            friction: 0.5,
        }
    }

    /// Rubber material properties.
    pub fn rubber() -> Self {
        Self {
            density: 1200.0,
            youngs_modulus: 0.05e9,
            poisson_ratio: 0.49,
            yield_strength: 15e6,
            ultimate_strength: 20e6,
            restitution: 0.8,
            friction: 1.2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = PhysicsConfig::default();
        assert!(config.time_step.dt > 0.0);
        assert!(config.solver.max_iterations > 0);
    }

    #[test]
    fn test_material_properties() {
        let steel = MaterialProperties::steel();
        assert_eq!(steel.density, 7850.0);
        assert!(steel.youngs_modulus > 0.0);
    }
}
