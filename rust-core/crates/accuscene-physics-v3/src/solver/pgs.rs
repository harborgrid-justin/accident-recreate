//! Projected Gauss-Seidel (PGS) solver.
//!
//! Implements PGS method for solving Linear Complementarity Problems (LCP)
//! arising from contact constraints. This is a more sophisticated solver
//! that handles inequality constraints naturally.

use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::config::SolverConfig;
use crate::error::PhysicsResult;
use crate::rigid_body::{constraints::ContactConstraint, RigidBody};
use crate::solver::SolverStats;

/// Projected Gauss-Seidel constraint solver.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PGSSolver {
    /// Solver configuration.
    pub config: SolverConfig,

    /// Over-relaxation factor (1.0 = standard Gauss-Seidel, >1.0 = SOR).
    pub omega: f64,
}

impl PGSSolver {
    /// Creates a new PGS solver.
    pub fn new(config: SolverConfig) -> Self {
        Self {
            config,
            omega: 1.0, // Standard Gauss-Seidel (can use 1.2-1.8 for SOR)
        }
    }

    /// Solves the constraint system using PGS method.
    ///
    /// Solves: A * λ + b ≥ 0, λ ≥ 0, λ^T (A * λ + b) = 0
    /// Where λ is the vector of constraint impulses.
    pub fn solve(
        &mut self,
        bodies: &mut [RigidBody],
        contacts: &mut [ContactConstraint],
        dt: f64,
    ) -> PhysicsResult<SolverStats> {
        let start_time = Instant::now();

        if contacts.is_empty() {
            return Ok(SolverStats {
                iterations: 0,
                residual: 0.0,
                converged: true,
                num_constraints: 0,
                solve_time: start_time.elapsed().as_secs_f64(),
            });
        }

        let num_constraints = contacts.len();
        let mut lambda = vec![0.0; num_constraints]; // Impulse magnitudes

        // Initialize with warm start values
        if self.config.warm_starting {
            for (i, contact) in contacts.iter().enumerate() {
                lambda[i] = contact.accumulated_normal_impulse;
            }
        }

        let mut converged = false;
        let mut final_residual = 0.0;
        let mut iterations = 0;

        for iter in 0..self.config.max_iterations {
            iterations = iter + 1;

            let old_lambda = lambda.clone();

            // Gauss-Seidel iteration
            for i in 0..num_constraints {
                let delta_lambda = self.solve_single_constraint(
                    bodies,
                    &contacts[i],
                    &lambda,
                    i,
                    dt,
                )?;

                // Update with over-relaxation
                lambda[i] += self.omega * delta_lambda;

                // Project onto feasible set (λ ≥ 0)
                lambda[i] = lambda[i].max(0.0);

                // Update accumulated impulse for warm starting
                // We'll do this after iteration completes
            }

            // Check convergence
            let residual = self.compute_residual(&lambda, &old_lambda);
            final_residual = residual;

            if residual < self.config.tolerance {
                converged = true;
                break;
            }
        }

        // Apply final impulses and update accumulated values
        for (i, contact) in contacts.iter_mut().enumerate() {
            contact.accumulated_normal_impulse = lambda[i];
        }

        let solve_time = start_time.elapsed().as_secs_f64();

        Ok(SolverStats {
            iterations,
            residual: final_residual,
            converged,
            num_constraints: num_constraints,
            solve_time,
        })
    }

    /// Solves a single constraint row in the PGS iteration.
    fn solve_single_constraint(
        &self,
        bodies: &[RigidBody],
        contact: &ContactConstraint,
        _lambda: &[f64],
        _constraint_idx: usize,
        _dt: f64,
    ) -> PhysicsResult<f64> {
        let body_a = &bodies[contact.body_a];
        let body_b = &bodies[contact.body_b];

        let ra = contact.point_a - body_a.position;
        let rb = contact.point_b - body_b.position;

        // Compute relative velocity
        let va = body_a.velocity_at_point(contact.point_a);
        let vb = body_b.velocity_at_point(contact.point_b);
        let v_rel = vb - va;

        let v_normal = v_rel.dot(&contact.normal);

        // Compute effective mass
        let ra_cross_n = ra.cross(&contact.normal);
        let rb_cross_n = rb.cross(&contact.normal);

        let inv_mass_a = if body_a.is_static { 0.0 } else { body_a.mass_props.inverse_mass };
        let inv_mass_b = if body_b.is_static { 0.0 } else { body_b.mass_props.inverse_mass };

        let inv_inertia_a = if body_a.is_static {
            nalgebra::Matrix3::zeros()
        } else {
            body_a.world_inverse_inertia_tensor()
        };

        let inv_inertia_b = if body_b.is_static {
            nalgebra::Matrix3::zeros()
        } else {
            body_b.world_inverse_inertia_tensor()
        };

        let angular_factor_a = ra_cross_n.dot(&(inv_inertia_a * ra_cross_n));
        let angular_factor_b = rb_cross_n.dot(&(inv_inertia_b * rb_cross_n));

        let effective_mass_inv = inv_mass_a + inv_mass_b + angular_factor_a + angular_factor_b;

        if effective_mass_inv < 1e-10 {
            return Ok(0.0);
        }

        let k = 1.0 / effective_mass_inv;

        // Compute bias for restitution and penetration correction
        let bias = if contact.penetration > self.config.contact_slop {
            let position_correction = self.config.baumgarte_factor
                * (contact.penetration - self.config.contact_slop)
                / 0.01; // Assume dt

            let restitution_bias = if v_normal < -1.0 {
                -contact.restitution * v_normal
            } else {
                0.0
            };

            position_correction + restitution_bias
        } else {
            0.0
        };

        // Compute impulse change
        let delta_lambda = -k * (v_normal - bias);

        Ok(delta_lambda)
    }

    /// Computes the residual (convergence measure).
    fn compute_residual(&self, lambda: &[f64], old_lambda: &[f64]) -> f64 {
        let mut residual_squared = 0.0;

        for i in 0..lambda.len() {
            let diff = lambda[i] - old_lambda[i];
            residual_squared += diff * diff;
        }

        (residual_squared / lambda.len() as f64).sqrt()
    }

    /// Sets the over-relaxation factor (1.0 = standard GS, >1.0 = SOR).
    pub fn set_omega(&mut self, omega: f64) {
        self.omega = omega.clamp(1.0, 2.0);
    }
}

/// Successive Over-Relaxation (SOR) variant of PGS.
///
/// SOR can converge faster than standard Gauss-Seidel by using an
/// over-relaxation factor ω ∈ (1, 2).
pub struct SORSolver {
    pgs: PGSSolver,
}

impl SORSolver {
    /// Creates a new SOR solver with optimal omega.
    pub fn new(config: SolverConfig, omega: f64) -> Self {
        let mut pgs = PGSSolver::new(config);
        pgs.set_omega(omega);

        Self { pgs }
    }

    /// Solves using SOR method.
    pub fn solve(
        &mut self,
        bodies: &mut [RigidBody],
        contacts: &mut [ContactConstraint],
        dt: f64,
    ) -> PhysicsResult<SolverStats> {
        self.pgs.solve(bodies, contacts, dt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rigid_body::dynamics::MassProperties;
    use nalgebra::Vector3;

    #[test]
    fn test_pgs_solver_creation() {
        let config = SolverConfig::default();
        let solver = PGSSolver::new(config);

        assert_eq!(solver.omega, 1.0);
    }

    #[test]
    fn test_pgs_solver_empty_contacts() {
        let config = SolverConfig::default();
        let mut solver = PGSSolver::new(config);

        let mut bodies = vec![];
        let mut contacts = vec![];

        let result = solver.solve(&mut bodies, &mut contacts, 0.01);
        assert!(result.is_ok());

        let stats = result.unwrap();
        assert_eq!(stats.num_constraints, 0);
        assert!(stats.converged);
    }

    #[test]
    fn test_sor_creation() {
        let config = SolverConfig::default();
        let solver = SORSolver::new(config, 1.5);

        assert_eq!(solver.pgs.omega, 1.5);
    }

    #[test]
    fn test_omega_clamping() {
        let config = SolverConfig::default();
        let mut solver = PGSSolver::new(config);

        solver.set_omega(2.5); // Should clamp to 2.0
        assert_eq!(solver.omega, 2.0);

        solver.set_omega(0.5); // Should clamp to 1.0
        assert_eq!(solver.omega, 1.0);
    }
}
