//! Sequential Impulse (SI) solver.
//!
//! Implements Erin Catto's Sequential Impulse method used in Box2D and similar engines.
//! This is an iterative method that solves one constraint at a time in sequence.

use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::config::SolverConfig;
use crate::error::PhysicsResult;
use crate::rigid_body::{constraints::ContactConstraint, RigidBody};
use crate::solver::SolverStats;

/// Sequential impulse constraint solver.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequentialImpulseSolver {
    /// Solver configuration.
    pub config: SolverConfig,

    /// Warm start factor (0-1).
    pub warm_start_factor: f64,
}

impl SequentialImpulseSolver {
    /// Creates a new sequential impulse solver.
    pub fn new(config: SolverConfig) -> Self {
        Self {
            config,
            warm_start_factor: 0.8,
        }
    }

    /// Solves all constraints for the specified number of iterations.
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

        // Warm start (apply cached impulses)
        if self.config.warm_starting {
            self.warm_start(bodies, contacts);
        }

        // Iterative velocity solver
        let mut converged = false;
        let mut final_residual = 0.0;
        let mut iterations = 0;

        for iter in 0..self.config.velocity_iterations {
            iterations = iter + 1;

            let residual = self.solve_velocity_iteration(bodies, contacts, dt)?;
            final_residual = residual;

            if residual < self.config.tolerance {
                converged = true;
                break;
            }
        }

        // Position correction iterations
        for _ in 0..self.config.position_iterations {
            self.solve_position_iteration(bodies, contacts)?;
        }

        let solve_time = start_time.elapsed().as_secs_f64();

        Ok(SolverStats {
            iterations,
            residual: final_residual,
            converged,
            num_constraints: contacts.len(),
            solve_time,
        })
    }

    /// Warm start: apply cached impulses from previous frame.
    fn warm_start(&self, bodies: &mut [RigidBody], contacts: &[ContactConstraint]) {
        for contact in contacts {
            let body_a_idx = contact.body_a;
            let body_b_idx = contact.body_b;

            if body_a_idx == body_b_idx || body_a_idx >= bodies.len() || body_b_idx >= bodies.len() {
                continue;
            }

            // Split the slice to get mutable references to both bodies
            let (mut body_a, mut body_b) = if body_a_idx < body_b_idx {
                let (left, right) = bodies.split_at_mut(body_b_idx);
                (&mut left[body_a_idx], &mut right[0])
            } else {
                let (left, right) = bodies.split_at_mut(body_a_idx);
                (&mut right[0], &mut left[body_b_idx])
            };

            let ra = contact.point_a - body_a.position;
            let rb = contact.point_b - body_b.position;

            // Apply cached normal impulse
            let impulse = contact.normal * contact.accumulated_normal_impulse * self.warm_start_factor;

            if !body_a.is_static {
                body_a.linear_velocity -= impulse * body_a.mass_props.inverse_mass;
                let angular_impulse_a = ra.cross(&impulse);
                let inv_inertia_a = body_a.world_inverse_inertia_tensor();
                body_a.angular_velocity -= inv_inertia_a * angular_impulse_a;
            }

            if !body_b.is_static {
                body_b.linear_velocity += impulse * body_b.mass_props.inverse_mass;
                let angular_impulse_b = rb.cross(&impulse);
                let inv_inertia_b = body_b.world_inverse_inertia_tensor();
                body_b.angular_velocity += inv_inertia_b * angular_impulse_b;
            }

            // Apply cached tangent impulse
            let tangent_impulse = contact.accumulated_tangent_impulse * self.warm_start_factor;

            if tangent_impulse.norm_squared() > 1e-10 {
                if !body_a.is_static {
                    body_a.linear_velocity -= tangent_impulse * body_a.mass_props.inverse_mass;
                    let angular_impulse_a = ra.cross(&tangent_impulse);
                    let inv_inertia_a = body_a.world_inverse_inertia_tensor();
                    body_a.angular_velocity -= inv_inertia_a * angular_impulse_a;
                }

                if !body_b.is_static {
                    body_b.linear_velocity += tangent_impulse * body_b.mass_props.inverse_mass;
                    let angular_impulse_b = rb.cross(&tangent_impulse);
                    let inv_inertia_b = body_b.world_inverse_inertia_tensor();
                    body_b.angular_velocity += inv_inertia_b * angular_impulse_b;
                }
            }
        }
    }

    /// Solves one velocity iteration.
    fn solve_velocity_iteration(
        &self,
        bodies: &mut [RigidBody],
        contacts: &mut [ContactConstraint],
        dt: f64,
    ) -> PhysicsResult<f64> {
        let mut total_error = 0.0;

        for contact in contacts.iter_mut() {
            // Get mutable references to both bodies
            // We need to handle the borrow checker carefully here
            let body_a_idx = contact.body_a;
            let body_b_idx = contact.body_b;

            if body_a_idx == body_b_idx {
                continue; // Skip self-collision
            }

            // Split the slice to get mutable references to both bodies
            let (mut body_a, mut body_b) = if body_a_idx < body_b_idx {
                let (left, right) = bodies.split_at_mut(body_b_idx);
                (&mut left[body_a_idx], &mut right[0])
            } else {
                let (left, right) = bodies.split_at_mut(body_a_idx);
                (&mut right[0], &mut left[body_b_idx])
            };

            // Solve this contact
            contact.solve(
                &mut body_a,
                &mut body_b,
                dt,
                self.config.baumgarte_factor,
                self.config.contact_slop,
            )?;

            // Compute error for convergence check
            let _ra = contact.point_a - body_a.position;
            let _rb = contact.point_b - body_b.position;

            let va = body_a.velocity_at_point(contact.point_a);
            let vb = body_b.velocity_at_point(contact.point_b);
            let v_rel = vb - va;

            let error = v_rel.dot(&contact.normal);
            total_error += error * error;
        }

        Ok((total_error / contacts.len() as f64).sqrt())
    }

    /// Solves one position iteration (Baumgarte stabilization).
    fn solve_position_iteration(
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

            if body_a_idx == body_b_idx {
                continue;
            }

            // Get body references
            let (inv_mass_a, inv_mass_b) = {
                let body_a = &bodies[body_a_idx];
                let body_b = &bodies[body_b_idx];

                (
                    if body_a.is_static { 0.0 } else { body_a.mass_props.inverse_mass },
                    if body_b.is_static { 0.0 } else { body_b.mass_props.inverse_mass },
                )
            };

            let inv_mass_sum = inv_mass_a + inv_mass_b;

            if inv_mass_sum < 1e-10 {
                continue;
            }

            // Compute position correction
            let correction_magnitude = (contact.penetration - self.config.contact_slop)
                * self.config.baumgarte_factor;

            let correction = contact.normal * correction_magnitude;

            // Apply corrections
            if !bodies[body_a_idx].is_static {
                bodies[body_a_idx].position -= correction * (inv_mass_a / inv_mass_sum);
            }

            if !bodies[body_b_idx].is_static {
                bodies[body_b_idx].position += correction * (inv_mass_b / inv_mass_sum);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rigid_body::dynamics::MassProperties;
    use nalgebra::Vector3;

    #[test]
    fn test_si_solver_creation() {
        let config = SolverConfig::default();
        let solver = SequentialImpulseSolver::new(config);

        assert!(solver.warm_start_factor > 0.0);
    }

    #[test]
    fn test_si_solver_empty_contacts() {
        let config = SolverConfig::default();
        let mut solver = SequentialImpulseSolver::new(config);

        let mut bodies = vec![];
        let mut contacts = vec![];

        let result = solver.solve(&mut bodies, &mut contacts, 0.01);
        assert!(result.is_ok());

        let stats = result.unwrap();
        assert_eq!(stats.num_constraints, 0);
        assert!(stats.converged);
    }

    #[test]
    fn test_si_solver_single_contact() {
        let config = SolverConfig::default();
        let mut solver = SequentialImpulseSolver::new(config);

        let mass_props = MassProperties::from_sphere(1000.0, 1.0);
        let mut body_a = RigidBody::new(0, mass_props.clone());
        let mut body_b = RigidBody::new(1, mass_props);

        body_a.position = Vector3::new(0.0, 0.0, 1.0);
        body_b.position = Vector3::new(0.0, 0.0, -1.0);

        body_a.linear_velocity = Vector3::new(0.0, 0.0, -10.0);
        body_b.linear_velocity = Vector3::new(0.0, 0.0, 10.0);

        let mut contact = ContactConstraint::new(
            0,
            1,
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
            0.1,
            0.5,
            0.3,
        );

        let mut bodies = vec![body_a, body_b];
        let mut contacts = vec![contact];

        let result = solver.solve(&mut bodies, &mut contacts, 0.01);
        assert!(result.is_ok());

        let stats = result.unwrap();
        assert_eq!(stats.num_constraints, 1);
    }
}
