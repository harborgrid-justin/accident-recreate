//! Physics constraint solvers.
//!
//! Implements iterative solvers for:
//! - Contact constraints
//! - Joint constraints
//! - Velocity and position corrections

pub mod pgs;
pub mod sequential_impulse;

pub use pgs::*;
pub use sequential_impulse::*;

use serde::{Deserialize, Serialize};

use crate::config::SolverConfig;
use crate::error::PhysicsResult;
use crate::rigid_body::{constraints::ContactConstraint, RigidBody};

/// Physics solver for constraint resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsSolver {
    /// Solver configuration.
    pub config: SolverConfig,

    /// Sequential impulse solver.
    pub si_solver: SequentialImpulseSolver,

    /// PGS solver.
    pub pgs_solver: PGSSolver,
}

impl PhysicsSolver {
    /// Creates a new physics solver.
    pub fn new(config: SolverConfig) -> Self {
        Self {
            si_solver: SequentialImpulseSolver::new(config.clone()),
            pgs_solver: PGSSolver::new(config.clone()),
            config,
        }
    }

    /// Solves all constraints for one iteration.
    pub fn solve_constraints(
        &mut self,
        bodies: &mut [RigidBody],
        contacts: &mut [ContactConstraint],
        dt: f64,
    ) -> PhysicsResult<SolverStats> {
        // Use sequential impulse solver
        self.si_solver.solve(bodies, contacts, dt)
    }

    /// Applies position correction (Baumgarte stabilization).
    pub fn apply_position_correction(
        &self,
        bodies: &mut [RigidBody],
        contacts: &[ContactConstraint],
    ) -> PhysicsResult<()> {
        for contact in contacts {
            if contact.penetration <= self.config.contact_slop {
                continue;
            }

            let body_a_idx = contact.body_a;
            let body_b_idx = contact.body_b;

            if body_a_idx == body_b_idx || body_a_idx >= bodies.len() || body_b_idx >= bodies.len() {
                continue;
            }

            // Compute correction magnitude
            let correction = (contact.penetration - self.config.contact_slop)
                * self.config.baumgarte_factor;

            let correction_vector = contact.normal * correction;

            // Get inverse masses
            let (inv_mass_a, inv_mass_b, is_static_a, is_static_b) = {
                let body_a = &bodies[body_a_idx];
                let body_b = &bodies[body_b_idx];
                (
                    body_a.mass_props.inverse_mass,
                    body_b.mass_props.inverse_mass,
                    body_a.is_static,
                    body_b.is_static,
                )
            };

            let inv_mass_sum = inv_mass_a + inv_mass_b;

            if inv_mass_sum > 1e-10 {
                if !is_static_a {
                    bodies[body_a_idx].position -= correction_vector * (inv_mass_a / inv_mass_sum);
                }

                if !is_static_b {
                    bodies[body_b_idx].position += correction_vector * (inv_mass_b / inv_mass_sum);
                }
            }
        }

        Ok(())
    }

    /// Updates solver configuration.
    pub fn update_config(&mut self, config: SolverConfig) {
        self.config = config.clone();
        self.si_solver.config = config.clone();
        self.pgs_solver.config = config;
    }
}

/// Statistics from solver execution.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SolverStats {
    /// Number of iterations performed.
    pub iterations: usize,

    /// Final residual error.
    pub residual: f64,

    /// Did the solver converge?
    pub converged: bool,

    /// Number of active constraints.
    pub num_constraints: usize,

    /// Solve time (seconds).
    pub solve_time: f64,
}

impl SolverStats {
    /// Creates default solver stats.
    pub fn default() -> Self {
        Self {
            iterations: 0,
            residual: 0.0,
            converged: false,
            num_constraints: 0,
            solve_time: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solver_creation() {
        let config = SolverConfig::default();
        let solver = PhysicsSolver::new(config);

        assert!(solver.config.max_iterations > 0);
    }
}
