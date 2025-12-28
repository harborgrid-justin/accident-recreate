//! Parallel simulation using rayon for complex scenes.

use crate::collision::Collision;
use crate::dynamics::VehicleState;
use crate::simulation::RigidBody;
use nalgebra::Vector3;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};

/// Parallel collision detection result.
pub struct ParallelCollisionResult {
    /// Detected collisions
    pub collisions: Vec<Collision>,
    /// Time taken (ms)
    pub duration_ms: u128,
}

/// Parallel physics calculator.
pub struct ParallelPhysics;

impl ParallelPhysics {
    /// Computes forces for multiple bodies in parallel.
    pub fn compute_forces_parallel(
        bodies: &[RigidBody],
        gravity: Vector3<f64>,
        friction_coefficient: f64,
    ) -> Vec<Vector3<f64>> {
        bodies
            .par_iter()
            .map(|body| {
                if body.is_static {
                    return Vector3::zeros();
                }

                // Gravity force
                let gravity_force = gravity * body.mass;

                // Friction force (simplified ground contact)
                let friction_force = if body.state.position.z <= 0.1 {
                    let normal_force = -gravity.z * body.mass;
                    let friction_magnitude = friction_coefficient * normal_force;

                    if body.state.velocity.norm() > 0.01 {
                        -body.state.velocity.normalize() * friction_magnitude
                    } else {
                        Vector3::zeros()
                    }
                } else {
                    Vector3::zeros()
                };

                gravity_force + friction_force
            })
            .collect()
    }

    /// Integrates multiple bodies in parallel using Euler method.
    pub fn integrate_parallel(
        bodies: &mut [RigidBody],
        forces: &[Vector3<f64>],
        dt: f64,
    ) {
        bodies
            .par_iter_mut()
            .zip(forces.par_iter())
            .for_each(|(body, force)| {
                if body.is_static {
                    return;
                }

                // Euler integration
                body.state.acceleration = *force / body.mass;
                body.state.velocity += body.state.acceleration * dt;
                body.state.position += body.state.velocity * dt;

                // Ground collision
                if body.state.position.z < 0.0 {
                    body.state.position.z = 0.0;
                    body.state.velocity.z = body.state.velocity.z.max(0.0);
                }

                body.update_aabb();
            });
    }

    /// Performs broad-phase collision detection in parallel.
    pub fn broad_phase_parallel(
        bodies: &[RigidBody],
    ) -> Vec<(u64, u64)> {
        let potential_pairs = Arc::new(Mutex::new(Vec::new()));

        bodies.par_iter().enumerate().for_each(|(i, body_a)| {
            for (j, body_b) in bodies.iter().enumerate().skip(i + 1) {
                if body_a.aabb.intersects(&body_b.aabb) {
                    let pair = if body_a.id < body_b.id {
                        (body_a.id, body_b.id)
                    } else {
                        (body_b.id, body_a.id)
                    };

                    potential_pairs.lock().unwrap().push(pair);
                }
            }
        });

        Arc::try_unwrap(potential_pairs).unwrap().into_inner().unwrap()
    }

    /// Computes energy for multiple bodies in parallel.
    pub fn compute_energies_parallel(bodies: &[RigidBody]) -> Vec<f64> {
        bodies
            .par_iter()
            .map(|body| {
                // Kinetic energy: KE = ½mv²
                let velocity_squared = body.state.velocity.norm_squared();
                0.5 * body.mass * velocity_squared
            })
            .collect()
    }

    /// Computes momentum for multiple bodies in parallel.
    pub fn compute_momenta_parallel(bodies: &[RigidBody]) -> Vec<Vector3<f64>> {
        bodies
            .par_iter()
            .map(|body| body.mass * body.state.velocity)
            .collect()
    }

    /// Updates AABBs for multiple bodies in parallel.
    pub fn update_aabbs_parallel(bodies: &mut [RigidBody]) {
        bodies.par_iter_mut().for_each(|body| {
            body.update_aabb();
        });
    }

    /// Runs a batch of simulations in parallel (Monte Carlo style).
    pub fn run_batch_simulations<F>(
        num_simulations: usize,
        simulation_fn: F,
    ) -> Vec<SimulationResult>
    where
        F: Fn(usize) -> SimulationResult + Send + Sync,
    {
        (0..num_simulations)
            .into_par_iter()
            .map(simulation_fn)
            .collect()
    }

    /// Computes collision matrix for all body pairs in parallel.
    pub fn collision_matrix_parallel(bodies: &[RigidBody]) -> Vec<Vec<bool>> {
        let n = bodies.len();
        let matrix = Arc::new(Mutex::new(vec![vec![false; n]; n]));

        bodies.par_iter().enumerate().for_each(|(i, body_a)| {
            for (j, body_b) in bodies.iter().enumerate().skip(i + 1) {
                if body_a.aabb.intersects(&body_b.aabb) {
                    let mut m = matrix.lock().unwrap();
                    m[i][j] = true;
                    m[j][i] = true;
                }
            }
        });

        Arc::try_unwrap(matrix).unwrap().into_inner().unwrap()
    }
}

/// Result of a simulation run.
#[derive(Debug, Clone)]
pub struct SimulationResult {
    /// Simulation ID
    pub id: usize,
    /// Final states of all bodies
    pub final_states: Vec<VehicleState>,
    /// Total energy
    pub total_energy: f64,
    /// Number of collisions
    pub num_collisions: usize,
    /// Simulation duration
    pub duration: f64,
    /// Success flag
    pub success: bool,
}

/// Parallel parameter sweep for sensitivity analysis.
pub struct ParameterSweep;

impl ParameterSweep {
    /// Sweeps a parameter range in parallel.
    pub fn sweep_parameter<F, T>(
        parameter_values: Vec<T>,
        simulation_fn: F,
    ) -> Vec<(T, SimulationResult)>
    where
        F: Fn(&T) -> SimulationResult + Send + Sync,
        T: Send + Sync,
    {
        parameter_values
            .into_par_iter()
            .map(|param| {
                let result = simulation_fn(&param);
                (param, result)
            })
            .collect()
    }

    /// 2D parameter sweep (grid search).
    pub fn sweep_2d<F, T1, T2>(
        param1_values: Vec<T1>,
        param2_values: Vec<T2>,
        simulation_fn: F,
    ) -> Vec<((T1, T2), SimulationResult)>
    where
        F: Fn(&T1, &T2) -> SimulationResult + Send + Sync,
        T1: Send + Sync + Clone,
        T2: Send + Sync + Clone,
    {
        let param_pairs: Vec<(T1, T2)> = param1_values
            .iter()
            .flat_map(|p1| {
                param2_values
                    .iter()
                    .map(move |p2| (p1.clone(), p2.clone()))
            })
            .collect();

        param_pairs
            .into_par_iter()
            .map(|(p1, p2)| {
                let result = simulation_fn(&p1, &p2);
                ((p1, p2), result)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::Point3;

    #[test]
    fn test_parallel_force_computation() {
        let bodies = vec![
            RigidBody::new(1, 1000.0).with_position(Point3::new(0.0, 0.0, 0.0)),
            RigidBody::new(2, 1500.0).with_position(Point3::new(10.0, 0.0, 0.0)),
            RigidBody::new(3, 2000.0).with_position(Point3::new(20.0, 0.0, 0.0)),
        ];

        let gravity = Vector3::new(0.0, 0.0, -9.81);
        let forces = ParallelPhysics::compute_forces_parallel(&bodies, gravity, 0.7);

        assert_eq!(forces.len(), 3);

        // Gravity forces should be proportional to mass
        assert!(forces[0].z < 0.0);
        assert!(forces[1].z < forces[0].z); // More massive, stronger force
    }

    #[test]
    fn test_parallel_energy_computation() {
        let bodies = vec![
            RigidBody::new(1, 1000.0).with_velocity(Vector3::new(10.0, 0.0, 0.0)),
            RigidBody::new(2, 1500.0).with_velocity(Vector3::new(20.0, 0.0, 0.0)),
        ];

        let energies = ParallelPhysics::compute_energies_parallel(&bodies);

        // KE1 = ½ * 1000 * 10² = 50,000
        // KE2 = ½ * 1500 * 20² = 300,000
        assert!((energies[0] - 50000.0).abs() < 1.0);
        assert!((energies[1] - 300000.0).abs() < 1.0);
    }

    #[test]
    fn test_parallel_momentum_computation() {
        let bodies = vec![
            RigidBody::new(1, 1000.0).with_velocity(Vector3::new(10.0, 0.0, 0.0)),
            RigidBody::new(2, 1500.0).with_velocity(Vector3::new(5.0, 0.0, 0.0)),
        ];

        let momenta = ParallelPhysics::compute_momenta_parallel(&bodies);

        // p1 = 1000 * 10 = 10,000
        // p2 = 1500 * 5 = 7,500
        assert_eq!(momenta[0].x, 10000.0);
        assert_eq!(momenta[1].x, 7500.0);
    }

    #[test]
    fn test_batch_simulation() {
        let results = ParallelPhysics::run_batch_simulations(5, |id| SimulationResult {
            id,
            final_states: vec![],
            total_energy: 100.0 * id as f64,
            num_collisions: id,
            duration: 1.0,
            success: true,
        });

        assert_eq!(results.len(), 5);
        assert_eq!(results[2].id, 2);
        assert_eq!(results[2].total_energy, 200.0);
    }

    #[test]
    fn test_parameter_sweep() {
        let friction_values = vec![0.5, 0.6, 0.7, 0.8, 0.9];

        let results = ParameterSweep::sweep_parameter(friction_values, |friction| {
            SimulationResult {
                id: 0,
                final_states: vec![],
                total_energy: *friction * 1000.0,
                num_collisions: 0,
                duration: 1.0,
                success: true,
            }
        });

        assert_eq!(results.len(), 5);
        assert!((results[2].1.total_energy - 700.0).abs() < 1.0);
    }
}
