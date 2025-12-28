/**
 * Physics Engine Module - Complete Export
 * AccuScene Enterprise Accident Recreation Platform
 */

// Local imports for use in this file
import { Vector2D } from './Vector2D';
import { CollisionResolver, RigidBody } from './CollisionResolver';
import { CollisionDetector, AABB, Polygon } from './CollisionDetector';
import { FrictionModel, SurfaceType, TireCondition } from './FrictionModel';
import { SpeedCalculator } from './SpeedCalculator';

// Core Vector Mathematics
export { Vector2D } from './Vector2D';

// Collision Detection
export {
  CollisionDetector,
  AABB,
  Polygon,
  CollisionResult
} from './CollisionDetector';

// Collision Resolution
export {
  CollisionResolver,
  RigidBody,
  CollisionResponse
} from './CollisionResolver';

// Friction and Surface Modeling
export {
  FrictionModel,
  SurfaceType,
  TireCondition,
  FrictionCoefficients,
  BrakingResult,
  SkidAnalysis
} from './FrictionModel';

// Speed Calculation Methods
export {
  SpeedCalculator,
  SpeedEstimate,
  DamageAnalysis,
  ThrowAnalysis
} from './SpeedCalculator';

// Simulation Recording and Playback
export {
  SimulationRecorder,
  VehicleState,
  CollisionEvent,
  SimulationFrame,
  SimulationMetadata,
  PlaybackOptions,
  ExportOptions
} from './SimulationRecorder';

// Accident Reconstruction
export {
  AccidentReconstructor,
  FinalState,
  EnvironmentConditions,
  ReconstructedScenario,
  InitialState,
  ReconstructionConstraints,
  WitnessAccount,
  PhysicalEvidence
} from './AccidentReconstructor';

// Main Physics Engine
export {
  PhysicsEngine,
  PhysicsVehicle,
  SimulationConfig,
  SimulationState
} from './PhysicsEngine';

// Re-export commonly used types and utilities
export type {
  AABB as BoundingBox,
  Polygon as PhysicsPolygon,
  RigidBody as PhysicsBody
};

/**
 * Physics Engine Version
 */
export const VERSION = '1.0.0';

/**
 * Default simulation configuration
 */
export const DEFAULT_SIMULATION_CONFIG: Partial<import('./PhysicsEngine').SimulationConfig> = {
  gravity: 9.81,
  timeStep: 1 / 60,
  maxSubSteps: 5,
  defaultSurface: SurfaceType.DRY_ASPHALT,
  enableCollisions: true,
  enableFriction: true,
  recordHistory: true
};

/**
 * Common physical constants
 */
export const PHYSICS_CONSTANTS = {
  GRAVITY: 9.81, // m/s²
  AIR_DENSITY: 1.225, // kg/m³ at sea level
  SPEED_OF_SOUND: 343, // m/s at 20°C
  MPH_TO_MPS: 0.44704,
  MPS_TO_MPH: 2.23694,
  KPH_TO_MPS: 0.277778,
  MPS_TO_KPH: 3.6,
  FEET_TO_METERS: 0.3048,
  METERS_TO_FEET: 3.28084
} as const;

/**
 * Utility function to create a basic vehicle for simulation
 */
export function createVehicle(options: {
  id: string;
  position: { x: number; y: number };
  velocity?: { x: number; y: number };
  mass: number;
  width?: number;
  height?: number;
  rotation?: number;
  restitution?: number;
  friction?: number;
  surface?: SurfaceType;
  tireCondition?: TireCondition;
}): import('./PhysicsEngine').PhysicsVehicle {
  const pos = new Vector2D(options.position.x, options.position.y);
  const vel = options.velocity
    ? new Vector2D(options.velocity.x, options.velocity.y)
    : Vector2D.zero();

  const width = options.width || 2.0; // Default car width: 2m
  const height = options.height || 4.5; // Default car length: 4.5m

  return {
    id: options.id,
    position: pos,
    velocity: vel,
    mass: options.mass,
    rotation: options.rotation || 0,
    angularVelocity: 0,
    momentOfInertia: CollisionResolver.calculateRectangleMomentOfInertia(
      options.mass,
      width,
      height
    ),
    restitution: options.restitution || 0.3,
    friction: options.friction || 0.7,
    width,
    height,
    wheelbase: height * 0.6,
    steeringAngle: 0,
    throttle: 0,
    isBraking: false,
    surface: options.surface || SurfaceType.DRY_ASPHALT,
    tireCondition: options.tireCondition || TireCondition.GOOD,
    vertices: [],
    acceleration: Vector2D.zero()
  };
}

/**
 * Utility function to calculate stopping distance
 */
export function calculateStoppingDistance(
  speedMph: number,
  surface: SurfaceType = SurfaceType.DRY_ASPHALT,
  reactionTime: number = 1.5
): number {
  const speedMps = FrictionModel.mphToMps(speedMph);
  const dragFactor = FrictionModel.getDragFactor(surface);
  const result = FrictionModel.simulateBraking(speedMps, dragFactor, reactionTime);
  return result.stoppingDistance;
}

/**
 * Utility function to estimate speed from skid marks
 */
export function estimateSpeedFromSkidMarks(
  skidLengthMeters: number,
  surface: SurfaceType = SurfaceType.DRY_ASPHALT,
  grade: number = 0
): { mph: number; kph: number; confidence: number } {
  const estimate = SpeedCalculator.fromSkidMarks(skidLengthMeters, surface, grade);
  return {
    mph: estimate.speedMph,
    kph: estimate.speedKph,
    confidence: estimate.confidence
  };
}

/**
 * Utility function to create a simple two-vehicle collision scenario
 */
export function createCollisionScenario(
  vehicle1: {
    mass: number;
    speedMph: number;
    headingDegrees: number;
    position: { x: number; y: number };
  },
  vehicle2: {
    mass: number;
    speedMph: number;
    headingDegrees: number;
    position: { x: number; y: number };
  }
): { vehicle1: import('./PhysicsEngine').PhysicsVehicle; vehicle2: import('./PhysicsEngine').PhysicsVehicle } {
  const v1Speed = FrictionModel.mphToMps(vehicle1.speedMph);
  const v1Heading = (vehicle1.headingDegrees * Math.PI) / 180;
  const v1Velocity = Vector2D.fromAngle(v1Heading, v1Speed);

  const v2Speed = FrictionModel.mphToMps(vehicle2.speedMph);
  const v2Heading = (vehicle2.headingDegrees * Math.PI) / 180;
  const v2Velocity = Vector2D.fromAngle(v2Heading, v2Speed);

  return {
    vehicle1: createVehicle({
      id: 'vehicle1',
      position: vehicle1.position,
      velocity: v1Velocity.toObject(),
      mass: vehicle1.mass,
      rotation: v1Heading
    }),
    vehicle2: createVehicle({
      id: 'vehicle2',
      position: vehicle2.position,
      velocity: v2Velocity.toObject(),
      mass: vehicle2.mass,
      rotation: v2Heading
    })
  };
}
