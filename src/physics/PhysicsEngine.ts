/**
 * PhysicsEngine - Main Physics Simulation Engine
 * Orchestrates vehicle physics, collision detection, and simulation
 */

import { Vector2D } from './Vector2D';
import { CollisionDetector, CollisionResult, Polygon } from './CollisionDetector';
import { CollisionResolver, RigidBody, CollisionResponse } from './CollisionResolver';
import { FrictionModel, SurfaceType, TireCondition } from './FrictionModel';
import { SimulationRecorder, VehicleState, CollisionEvent } from './SimulationRecorder';

export interface PhysicsVehicle extends RigidBody {
  id: string;
  width: number;
  height: number;
  wheelbase: number;
  steeringAngle: number;
  throttle: number; // -1 to 1 (negative = brake)
  isBraking: boolean;
  surface: SurfaceType;
  tireCondition: TireCondition;
  vertices: Vector2D[]; // Vehicle shape vertices (local coordinates)
  acceleration?: Vector2D; // Optional acceleration for simulation
}

export interface SimulationConfig {
  gravity: number;
  timeStep: number;
  maxSubSteps: number;
  defaultSurface: SurfaceType;
  enableCollisions: boolean;
  enableFriction: boolean;
  recordHistory: boolean;
}

export interface SimulationState {
  time: number;
  running: boolean;
  paused: boolean;
  vehicles: Map<string, PhysicsVehicle>;
  collisions: CollisionEvent[];
}

export class PhysicsEngine {
  private config: SimulationConfig;
  private state: SimulationState;
  private recorder: SimulationRecorder | null = null;
  private accumulator: number = 0;

  constructor(config?: Partial<SimulationConfig>) {
    this.config = {
      gravity: 9.81,
      timeStep: 1 / 60, // 60 Hz
      maxSubSteps: 5,
      defaultSurface: SurfaceType.DRY_ASPHALT,
      enableCollisions: true,
      enableFriction: true,
      recordHistory: false,
      ...config
    };

    this.state = {
      time: 0,
      running: false,
      paused: false,
      vehicles: new Map(),
      collisions: []
    };
  }

  /**
   * Initialize simulation with vehicles
   */
  initialize(vehicles: PhysicsVehicle[]): void {
    this.state.vehicles.clear();
    this.state.time = 0;
    this.accumulator = 0;

    for (const vehicle of vehicles) {
      this.addVehicle(vehicle);
    }

    if (this.config.recordHistory) {
      this.recorder = new SimulationRecorder(
        `sim_${Date.now()}`,
        'Physics simulation',
        1 / this.config.timeStep
      );
      this.recorder.startRecording();
    }

    this.state.running = true;
    this.state.paused = false;
  }

  /**
   * Add vehicle to simulation
   */
  addVehicle(vehicle: PhysicsVehicle): void {
    // Calculate moment of inertia if not set
    if (vehicle.momentOfInertia === 0) {
      vehicle.momentOfInertia = CollisionResolver.calculateRectangleMomentOfInertia(
        vehicle.mass,
        vehicle.width,
        vehicle.height
      );
    }

    // Generate vertices if not provided
    if (!vehicle.vertices || vehicle.vertices.length === 0) {
      vehicle.vertices = this.generateVehicleVertices(vehicle.width, vehicle.height);
    }

    this.state.vehicles.set(vehicle.id, vehicle);
  }

  /**
   * Generate rectangular vehicle vertices (local coordinates)
   */
  private generateVehicleVertices(width: number, height: number): Vector2D[] {
    const halfWidth = width / 2;
    const halfHeight = height / 2;

    return [
      new Vector2D(-halfWidth, -halfHeight),
      new Vector2D(halfWidth, -halfHeight),
      new Vector2D(halfWidth, halfHeight),
      new Vector2D(-halfWidth, halfHeight)
    ];
  }

  /**
   * Remove vehicle from simulation
   */
  removeVehicle(vehicleId: string): void {
    this.state.vehicles.delete(vehicleId);
  }

  /**
   * Get vehicle by ID
   */
  getVehicle(vehicleId: string): PhysicsVehicle | undefined {
    return this.state.vehicles.get(vehicleId);
  }

  /**
   * Step simulation forward by deltaTime
   */
  step(deltaTime: number): void {
    if (!this.state.running || this.state.paused) {
      return;
    }

    this.accumulator += deltaTime;

    let subSteps = 0;
    while (this.accumulator >= this.config.timeStep && subSteps < this.config.maxSubSteps) {
      this.stepFixed(this.config.timeStep);
      this.accumulator -= this.config.timeStep;
      subSteps++;
    }
  }

  /**
   * Step simulation with fixed time step
   */
  private stepFixed(dt: number): void {
    this.state.collisions = [];

    // Update all vehicles
    this.state.vehicles.forEach(vehicle => {
      this.updateVehiclePhysics(vehicle, dt);
    });

    // Detect and resolve collisions
    if (this.config.enableCollisions) {
      this.detectAndResolveCollisions();
    }

    // Record frame
    if (this.recorder) {
      this.recordFrame(dt);
    }

    this.state.time += dt;
  }

  /**
   * Step simulation backward (for reverse playback)
   */
  stepBackward(deltaTime: number): void {
    // Negative time step
    this.step(-deltaTime);
  }

  /**
   * Update vehicle physics
   */
  private updateVehiclePhysics(vehicle: PhysicsVehicle, dt: number): void {
    // Calculate forces
    const forces = this.calculateVehicleForces(vehicle);

    // Apply forces (F = ma, a = F/m)
    const acceleration = forces.divide(vehicle.mass);
    vehicle.acceleration = acceleration;

    // Update velocity (Euler integration)
    vehicle.velocity = vehicle.velocity.add(acceleration.multiply(dt));

    // Apply friction if enabled
    if (this.config.enableFriction) {
      const dragFactor = FrictionModel.getDragFactor(vehicle.surface, vehicle.tireCondition);
      vehicle.velocity = FrictionModel.applyFriction(
        vehicle.velocity,
        vehicle.mass,
        dragFactor,
        dt,
        this.config.gravity
      );
    }

    // Update position
    vehicle.position = vehicle.position.add(vehicle.velocity.multiply(dt));

    // Update rotation
    vehicle.rotation += vehicle.angularVelocity * dt;

    // Normalize rotation to [0, 2π]
    vehicle.rotation = ((vehicle.rotation % (2 * Math.PI)) + 2 * Math.PI) % (2 * Math.PI);

    // Check if skidding
    const speed = vehicle.velocity.magnitude();
    vehicle.isBraking = vehicle.throttle < -0.1;

    // Skidding occurs when braking hard or sliding
    const dragFactor = FrictionModel.getDragFactor(vehicle.surface, vehicle.tireCondition);
    const maxGrip = dragFactor * this.config.gravity * vehicle.mass;
    const lateralForce = this.calculateLateralForce(vehicle);
    const isSkidding = lateralForce > maxGrip || (vehicle.isBraking && speed > 5);

    // Store skidding state (would be on vehicle object)
    (vehicle as any).isSkidding = isSkidding;
  }

  /**
   * Calculate forces acting on vehicle
   */
  private calculateVehicleForces(vehicle: PhysicsVehicle): Vector2D {
    let totalForce = Vector2D.zero();

    // Throttle force (along vehicle's forward direction)
    if (Math.abs(vehicle.throttle) > 0.01) {
      const forwardDir = Vector2D.fromAngle(vehicle.rotation);
      const throttleForce = 5000 * vehicle.throttle; // N (tunable)
      totalForce = totalForce.add(forwardDir.multiply(throttleForce));
    }

    // Braking force
    if (vehicle.isBraking) {
      const dragFactor = FrictionModel.getDragFactor(vehicle.surface, vehicle.tireCondition);
      const brakingForce = FrictionModel.calculateBrakingForce(
        vehicle.mass,
        dragFactor,
        this.config.gravity
      );
      const brakeDirection = vehicle.velocity.normalize().negate();
      totalForce = totalForce.add(brakeDirection.multiply(brakingForce));
    }

    // Steering force (simplified)
    if (Math.abs(vehicle.steeringAngle) > 0.01) {
      const speed = vehicle.velocity.magnitude();
      if (speed > 0.1) {
        const lateralForce = this.calculateSteeringForce(vehicle);
        totalForce = totalForce.add(lateralForce);
      }
    }

    return totalForce;
  }

  /**
   * Calculate steering force (simplified tire model)
   */
  private calculateSteeringForce(vehicle: PhysicsVehicle): Vector2D {
    const speed = vehicle.velocity.magnitude();
    const maxSteeringAngle = Math.PI / 6; // 30 degrees
    const steerAngle = Math.max(-maxSteeringAngle, Math.min(maxSteeringAngle, vehicle.steeringAngle));

    // Lateral force proportional to speed and steering angle
    const lateralDirection = Vector2D.fromAngle(vehicle.rotation + Math.PI / 2);
    const forceMagnitude = vehicle.mass * speed * Math.sin(steerAngle) * 2;

    return lateralDirection.multiply(forceMagnitude);
  }

  /**
   * Calculate lateral force on vehicle
   */
  private calculateLateralForce(vehicle: PhysicsVehicle): number {
    const forwardDir = Vector2D.fromAngle(vehicle.rotation);
    const lateralDir = forwardDir.perpendicular();
    return Math.abs(vehicle.velocity.dot(lateralDir)) * vehicle.mass;
  }

  /**
   * Detect and resolve all collisions
   */
  private detectAndResolveCollisions(): void {
    const vehicles = Array.from(this.state.vehicles.values());

    for (let i = 0; i < vehicles.length; i++) {
      for (let j = i + 1; j < vehicles.length; j++) {
        const v1 = vehicles[i];
        const v2 = vehicles[j];

        // Create polygons for collision detection
        const poly1: Polygon = {
          vertices: v1.vertices,
          position: v1.position,
          rotation: v1.rotation
        };

        const poly2: Polygon = {
          vertices: v2.vertices,
          position: v2.position,
          rotation: v2.rotation
        };

        // Detect collision
        const collision = CollisionDetector.testPolygonCollision(poly1, poly2);

        if (collision.colliding) {
          // Resolve collision
          const response = CollisionResolver.resolveCollision(v1, v2, collision);

          // Apply response
          v1.velocity = response.body1.velocity;
          v1.angularVelocity = response.body1.angularVelocity;
          v2.velocity = response.body2.velocity;
          v2.angularVelocity = response.body2.angularVelocity;

          // Separate bodies
          const separation = CollisionResolver.separateBodies(v1, v2, collision);
          v1.position = separation.position1;
          v2.position = separation.position2;

          // Record collision event
          this.state.collisions.push({
            time: this.state.time,
            vehicle1Id: v1.id,
            vehicle2Id: v2.id,
            contactPoint: collision.contactPoint,
            normal: collision.normal,
            impactSpeed: response.impactSpeed,
            penetrationDepth: collision.penetrationDepth,
            energyLoss: response.energyLoss
          });
        }
      }
    }
  }

  /**
   * Record current frame to recorder
   */
  private recordFrame(dt: number): void {
    if (!this.recorder) {
      return;
    }

    const vehicleStates = new Map<string, VehicleState>();

    this.state.vehicles.forEach((vehicle, id) => {
      vehicleStates.set(id, {
        id,
        position: vehicle.position.clone(),
        velocity: vehicle.velocity.clone(),
        rotation: vehicle.rotation,
        angularVelocity: vehicle.angularVelocity,
        acceleration: vehicle.acceleration.clone(),
        isSkidding: (vehicle as any).isSkidding || false,
        isBraking: vehicle.isBraking,
        steeringAngle: vehicle.steeringAngle
      });
    });

    this.recorder.recordFrame(
      this.state.time,
      dt,
      vehicleStates,
      this.state.collisions
    );
  }

  /**
   * Get simulation recorder
   */
  getRecorder(): SimulationRecorder | null {
    return this.recorder;
  }

  /**
   * Pause simulation
   */
  pause(): void {
    this.state.paused = true;
  }

  /**
   * Resume simulation
   */
  resume(): void {
    this.state.paused = false;
  }

  /**
   * Stop simulation
   */
  stop(): void {
    this.state.running = false;
    if (this.recorder) {
      this.recorder.stopRecording();
    }
  }

  /**
   * Reset simulation
   */
  reset(): void {
    this.state.time = 0;
    this.accumulator = 0;
    this.state.collisions = [];
    this.state.running = false;
    this.state.paused = false;

    if (this.recorder) {
      this.recorder.clear();
    }
  }

  /**
   * Get current simulation state
   */
  getState(): SimulationState {
    return {
      ...this.state,
      vehicles: new Map(this.state.vehicles)
    };
  }

  /**
   * Get all vehicles
   */
  getVehicles(): PhysicsVehicle[] {
    return Array.from(this.state.vehicles.values());
  }

  /**
   * Get simulation time
   */
  getTime(): number {
    return this.state.time;
  }

  /**
   * Set vehicle velocity
   */
  setVehicleVelocity(vehicleId: string, velocity: Vector2D): void {
    const vehicle = this.state.vehicles.get(vehicleId);
    if (vehicle) {
      vehicle.velocity = velocity.clone();
    }
  }

  /**
   * Set vehicle position
   */
  setVehiclePosition(vehicleId: string, position: Vector2D): void {
    const vehicle = this.state.vehicles.get(vehicleId);
    if (vehicle) {
      vehicle.position = position.clone();
    }
  }

  /**
   * Set vehicle rotation
   */
  setVehicleRotation(vehicleId: string, rotation: number): void {
    const vehicle = this.state.vehicles.get(vehicleId);
    if (vehicle) {
      vehicle.rotation = rotation;
    }
  }

  /**
   * Apply impulse to vehicle
   */
  applyImpulse(vehicleId: string, impulse: Vector2D, point?: Vector2D): void {
    const vehicle = this.state.vehicles.get(vehicleId);
    if (!vehicle) {
      return;
    }

    // Apply linear impulse
    const deltaV = impulse.divide(vehicle.mass);
    vehicle.velocity = vehicle.velocity.add(deltaV);

    // Apply angular impulse if point specified
    if (point) {
      const r = point.subtract(vehicle.position);
      const torque = r.cross(impulse);
      const deltaOmega = torque / vehicle.momentOfInertia;
      vehicle.angularVelocity += deltaOmega;
    }
  }

  /**
   * Set vehicle throttle (-1 to 1)
   */
  setVehicleThrottle(vehicleId: string, throttle: number): void {
    const vehicle = this.state.vehicles.get(vehicleId);
    if (vehicle) {
      vehicle.throttle = Math.max(-1, Math.min(1, throttle));
      vehicle.isBraking = throttle < -0.1;
    }
  }

  /**
   * Set vehicle steering angle (radians)
   */
  setVehicleSteering(vehicleId: string, angle: number): void {
    const vehicle = this.state.vehicles.get(vehicleId);
    if (vehicle) {
      vehicle.steeringAngle = angle;
    }
  }

  /**
   * Get collision history
   */
  getCollisionHistory(): CollisionEvent[] {
    if (this.recorder) {
      return this.recorder.getAllCollisions();
    }
    return [];
  }

  /**
   * Export simulation data
   */
  exportSimulation(format: 'json' | 'csv' | 'timeline'): string {
    if (!this.recorder) {
      throw new Error('Recording not enabled');
    }

    return this.recorder.export({
      format,
      includeMetadata: true,
      includeCollisions: true,
      decimals: 3
    });
  }

  /**
   * Calculate total kinetic energy in system
   */
  getTotalKineticEnergy(): number {
    let total = 0;
    this.state.vehicles.forEach(vehicle => {
      total += CollisionResolver.calculateKineticEnergy(vehicle);
    });
    return total;
  }

  /**
   * Calculate total momentum in system
   */
  getTotalMomentum(): Vector2D {
    let total = Vector2D.zero();
    this.state.vehicles.forEach(vehicle => {
      total = total.add(CollisionResolver.calculateMomentum(vehicle));
    });
    return total;
  }

  /**
   * Update simulation configuration
   */
  updateConfig(config: Partial<SimulationConfig>): void {
    this.config = { ...this.config, ...config };
  }

  /**
   * Get simulation configuration
   */
  getConfig(): SimulationConfig {
    return { ...this.config };
  }
}
