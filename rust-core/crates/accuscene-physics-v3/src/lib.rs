//! AccuScene Physics Engine v3.0
//!
//! A professional-grade physics engine for accident reconstruction featuring:
//!
//! # Features
//!
//! - **Rigid Body Dynamics**: Full 6-DOF dynamics with quaternion rotations
//! - **Advanced Collision Detection**: GJK/EPA narrow phase, sweep-and-prune broad phase
//! - **Deformable Bodies**: Finite Element Method (FEM) for crush analysis
//! - **Vehicle Physics**: Pacejka tire model, suspension, powertrain
//! - **Constraint Solvers**: Sequential Impulse and Projected Gauss-Seidel
//! - **Energy Analysis**: Kinetic, deformation, and dissipation tracking
//!
//! # Example
//!
//! ```rust,no_run
//! use accuscene_physics_v3::prelude::*;
//!
//! // Create a physics configuration
//! let config = PhysicsConfig::default();
//!
//! // Create rigid bodies
//! let mass_props = MassProperties::from_box(1500.0, Vector3::new(4.5, 1.8, 1.5));
//! let vehicle = RigidBody::new(0, mass_props);
//!
//! // Create collision shapes
//! let shape = CollisionShape::Box {
//!     half_extents: Vector3::new(2.25, 0.9, 0.75),
//! };
//! ```
//!
//! # Physics Equations
//!
//! ## Rigid Body Motion
//!
//! Linear motion:
//! ```text
//! F = m * a
//! v(t+dt) = v(t) + (F/m) * dt
//! x(t+dt) = x(t) + v(t+dt) * dt
//! ```
//!
//! Angular motion:
//! ```text
//! τ = I * α
//! ω(t+dt) = ω(t) + I^(-1) * τ * dt
//! q(t+dt) = q(t) + 0.5 * q(t) * ω * dt  (quaternion derivative)
//! ```
//!
//! ## Collision Response
//!
//! Impulse magnitude:
//! ```text
//! J = -(1 + e) * v_rel · n / (1/m_a + 1/m_b + (r_a × n)^T I_a^(-1) (r_a × n) + (r_b × n)^T I_b^(-1) (r_b × n))
//! ```
//!
//! ## Finite Element Method
//!
//! Deformation gradient:
//! ```text
//! F = ∂x/∂X
//! ```
//!
//! Green strain:
//! ```text
//! E = 0.5 * (F^T F - I)
//! ```
//!
//! Stress (linear elastic):
//! ```text
//! σ = λ * tr(ε) * I + 2μ * ε
//! ```
//!
//! ## Tire Model (Pacejka Magic Formula)
//!
//! ```text
//! F_x = D * sin(C * arctan(B*κ - E*(B*κ - arctan(B*κ))))
//! ```
//!
//! Where:
//! - κ = slip ratio
//! - B = stiffness factor
//! - C = shape factor
//! - D = peak value
//! - E = curvature factor

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod collision;
pub mod config;
pub mod deformable;
pub mod energy;
pub mod error;
pub mod rigid_body;
pub mod solver;
pub mod vehicle;

/// Prelude module for convenient imports.
pub mod prelude {
    pub use crate::collision::*;
    pub use crate::config::*;
    pub use crate::deformable::*;
    pub use crate::energy::*;
    pub use crate::error::*;
    pub use crate::rigid_body::*;
    pub use crate::solver::*;
    pub use crate::vehicle::*;

    pub use nalgebra::{Matrix3, Quaternion, UnitQuaternion, Vector3};
}

use collision::{BroadPhase, CollisionShape, NarrowPhase, AABB};
use config::PhysicsConfig;
use deformable::DeformableBody;
use energy::EnergyAnalysis;
use error::PhysicsResult;
use rigid_body::{constraints::ContactConstraint, RigidBody};
use solver::PhysicsSolver;
use vehicle::Vehicle;

/// Main physics world containing all simulation state.
pub struct PhysicsWorld {
    /// Configuration.
    config: PhysicsConfig,

    /// Rigid bodies.
    bodies: Vec<RigidBody>,

    /// Deformable bodies.
    deformable_bodies: Vec<DeformableBody>,

    /// Collision shapes (indexed by body ID).
    shapes: Vec<CollisionShape>,

    /// Vehicles.
    vehicles: Vec<Vehicle>,

    /// Broad phase collision detector.
    broad_phase: BroadPhase,

    /// Narrow phase collision detector.
    narrow_phase: NarrowPhase,

    /// Constraint solver.
    solver: PhysicsSolver,

    /// Current simulation time.
    time: f64,

    /// Gravity vector.
    gravity: nalgebra::Vector3<f64>,

    /// Energy analysis.
    energy_analysis: EnergyAnalysis,
}

impl PhysicsWorld {
    /// Creates a new physics world.
    pub fn new(config: PhysicsConfig) -> Self {
        let broad_phase = BroadPhase::new(config.collision.broad_phase);
        let narrow_phase = NarrowPhase::new();
        let solver = PhysicsSolver::new(config.solver.clone());

        Self {
            config: config.clone(),
            bodies: Vec::new(),
            deformable_bodies: Vec::new(),
            shapes: Vec::new(),
            vehicles: Vec::new(),
            broad_phase,
            narrow_phase,
            solver,
            time: 0.0,
            gravity: nalgebra::Vector3::new(0.0, 0.0, -9.81),
            energy_analysis: EnergyAnalysis::new(),
        }
    }

    /// Adds a rigid body to the world.
    pub fn add_body(&mut self, body: RigidBody, shape: CollisionShape) -> usize {
        let id = self.bodies.len();
        self.bodies.push(body);
        self.shapes.push(shape);
        id
    }

    /// Adds a deformable body to the world.
    pub fn add_deformable_body(&mut self, body: DeformableBody) -> usize {
        let id = self.deformable_bodies.len();
        self.deformable_bodies.push(body);
        id
    }

    /// Adds a vehicle to the world.
    pub fn add_vehicle(&mut self, vehicle: Vehicle) -> usize {
        let id = self.vehicles.len();
        self.vehicles.push(vehicle);
        id
    }

    /// Performs one physics simulation step.
    pub fn step(&mut self, dt: f64) -> PhysicsResult<()> {
        // Broad phase collision detection
        let aabbs: Vec<(usize, AABB)> = self
            .bodies
            .iter()
            .enumerate()
            .map(|(id, body)| {
                let shape = &self.shapes[id];
                let aabb = shape.compute_aabb(body.position, &body.orientation);
                (id, aabb)
            })
            .collect();

        let pairs = self.broad_phase.detect_pairs(&aabbs);

        // Narrow phase collision detection
        let mut contacts = Vec::new();

        for pair in pairs {
            let body_a = &self.bodies[pair.body_a];
            let body_b = &self.bodies[pair.body_b];
            let shape_a = &self.shapes[pair.body_a];
            let shape_b = &self.shapes[pair.body_b];

            // Check for actual collision
            if self.narrow_phase.gjk_intersect(
                shape_a,
                body_a.position,
                shape_b,
                body_b.position,
            ) {
                // Get contact information
                if let Ok(contact_point) = self.narrow_phase.epa_contact(
                    shape_a,
                    body_a.position,
                    shape_b,
                    body_b.position,
                ) {
                    let contact = ContactConstraint::new(
                        pair.body_a,
                        pair.body_b,
                        contact_point.point,
                        contact_point.point,
                        contact_point.normal,
                        contact_point.penetration,
                        self.config.collision.default_restitution,
                        self.config.collision.default_friction,
                    );

                    contacts.push(contact);
                }
            }
        }

        // Solve constraints
        self.solver.solve_constraints(&mut self.bodies, &mut contacts, dt)?;

        // Integrate motion
        for body in &mut self.bodies {
            rigid_body::dynamics::RigidBodyIntegrator::semi_implicit_euler(
                body,
                dt,
                self.gravity,
            );

            // Check sleep
            body.check_sleep(dt);
        }

        // Update deformable bodies
        for deformable_body in &mut self.deformable_bodies {
            let fem_solver = deformable::fem::FEMSolver::new();
            fem_solver.step(deformable_body, dt, self.gravity)?;
        }

        // Update energy analysis
        self.energy_analysis.analyze_rigid_bodies(&self.bodies);
        self.energy_analysis.analyze_deformation(&self.deformable_bodies);

        self.time += dt;

        Ok(())
    }

    /// Gets a reference to a rigid body.
    pub fn body(&self, id: usize) -> Option<&RigidBody> {
        self.bodies.get(id)
    }

    /// Gets a mutable reference to a rigid body.
    pub fn body_mut(&mut self, id: usize) -> Option<&mut RigidBody> {
        self.bodies.get_mut(id)
    }

    /// Gets current simulation time.
    pub fn time(&self) -> f64 {
        self.time
    }

    /// Gets energy analysis.
    pub fn energy_analysis(&self) -> &EnergyAnalysis {
        &self.energy_analysis
    }

    /// Sets gravity vector.
    pub fn set_gravity(&mut self, gravity: nalgebra::Vector3<f64>) {
        self.gravity = gravity;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::Vector3;
    use rigid_body::dynamics::MassProperties;

    #[test]
    fn test_physics_world_creation() {
        let config = PhysicsConfig::default();
        let world = PhysicsWorld::new(config);

        assert_eq!(world.time(), 0.0);
    }

    #[test]
    fn test_add_body() {
        let config = PhysicsConfig::default();
        let mut world = PhysicsWorld::new(config);

        let mass_props = MassProperties::from_sphere(1000.0, 1.0);
        let body = RigidBody::new(0, mass_props);
        let shape = CollisionShape::Sphere { radius: 1.0 };

        let id = world.add_body(body, shape);
        assert_eq!(id, 0);

        let retrieved_body = world.body(id);
        assert!(retrieved_body.is_some());
    }

    #[test]
    fn test_simulation_step() {
        let config = PhysicsConfig::default();
        let mut world = PhysicsWorld::new(config);

        let mass_props = MassProperties::from_sphere(1.0, 0.1);
        let mut body = RigidBody::new(0, mass_props);
        body.position = Vector3::new(0.0, 0.0, 10.0);

        let shape = CollisionShape::Sphere { radius: 0.1 };
        world.add_body(body, shape);

        let initial_height = world.body(0).unwrap().position.z;

        // Simulate falling under gravity
        world.step(0.1).unwrap();

        let final_height = world.body(0).unwrap().position.z;

        // Should have fallen
        assert!(final_height < initial_height);
    }
}
