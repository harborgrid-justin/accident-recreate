/**
 * Vehicle Physics Calculations
 * AccuScene Enterprise - Accident Recreation Platform
 */

import { VehicleWeightSpec } from './VehicleTypes';

/**
 * Constants for physics calculations
 */
export const PHYSICS_CONSTANTS = {
  GRAVITY: 32.174, // ft/s²
  FRICTION_DRY_ASPHALT: 0.7,
  FRICTION_WET_ASPHALT: 0.5,
  FRICTION_SNOW: 0.3,
  FRICTION_ICE: 0.15,
  FRICTION_GRAVEL: 0.6,
  AIR_DENSITY: 0.002378, // slugs/ft³
  DRAG_COEFFICIENT_SEDAN: 0.32,
  DRAG_COEFFICIENT_SUV: 0.35,
  DRAG_COEFFICIENT_TRUCK: 0.45,
};

export enum RoadCondition {
  DRY_ASPHALT = 'DRY_ASPHALT',
  WET_ASPHALT = 'WET_ASPHALT',
  SNOW = 'SNOW',
  ICE = 'ICE',
  GRAVEL = 'GRAVEL',
}

/**
 * Get friction coefficient for road condition
 */
export function getFrictionCoefficient(condition: RoadCondition): number {
  const coefficients: Record<RoadCondition, number> = {
    [RoadCondition.DRY_ASPHALT]: PHYSICS_CONSTANTS.FRICTION_DRY_ASPHALT,
    [RoadCondition.WET_ASPHALT]: PHYSICS_CONSTANTS.FRICTION_WET_ASPHALT,
    [RoadCondition.SNOW]: PHYSICS_CONSTANTS.FRICTION_SNOW,
    [RoadCondition.ICE]: PHYSICS_CONSTANTS.FRICTION_ICE,
    [RoadCondition.GRAVEL]: PHYSICS_CONSTANTS.FRICTION_GRAVEL,
  };
  return coefficients[condition];
}

/**
 * Calculate vehicle mass in slugs from weight in pounds
 */
export function calculateMass(weightPounds: number): number {
  return weightPounds / PHYSICS_CONSTANTS.GRAVITY;
}

/**
 * Calculate momentum (mass × velocity)
 * @param weightPounds - Vehicle weight in pounds
 * @param velocityMph - Velocity in miles per hour
 * @returns Momentum in slug-ft/s
 */
export function calculateMomentum(weightPounds: number, velocityMph: number): number {
  const mass = calculateMass(weightPounds);
  const velocityFps = mphToFps(velocityMph);
  return mass * velocityFps;
}

/**
 * Calculate kinetic energy (½ × mass × velocity²)
 * @param weightPounds - Vehicle weight in pounds
 * @param velocityMph - Velocity in miles per hour
 * @returns Kinetic energy in ft-lbs
 */
export function calculateKineticEnergy(weightPounds: number, velocityMph: number): number {
  const mass = calculateMass(weightPounds);
  const velocityFps = mphToFps(velocityMph);
  return 0.5 * mass * velocityFps * velocityFps;
}

/**
 * Calculate braking distance
 * @param velocityMph - Initial velocity in mph
 * @param roadCondition - Road surface condition
 * @param gradePercent - Road grade (positive for uphill, negative for downhill)
 * @param reactionTime - Driver reaction time in seconds (default 1.5s)
 * @returns Total stopping distance in feet
 */
export function calculateBrakingDistance(
  velocityMph: number,
  roadCondition: RoadCondition = RoadCondition.DRY_ASPHALT,
  gradePercent: number = 0,
  reactionTime: number = 1.5
): {
  reactionDistance: number;
  brakingDistance: number;
  totalDistance: number;
} {
  const velocityFps = mphToFps(velocityMph);
  const friction = getFrictionCoefficient(roadCondition);
  const grade = gradePercent / 100;

  // Reaction distance
  const reactionDistance = velocityFps * reactionTime;

  // Braking distance formula: d = v² / (2 × g × (μ ± grade))
  // + grade for uphill, - grade for downhill
  const effectiveFriction = friction + grade;
  const brakingDistance =
    (velocityFps * velocityFps) / (2 * PHYSICS_CONSTANTS.GRAVITY * effectiveFriction);

  const totalDistance = reactionDistance + brakingDistance;

  return {
    reactionDistance: Math.max(0, reactionDistance),
    brakingDistance: Math.max(0, brakingDistance),
    totalDistance: Math.max(0, totalDistance),
  };
}

/**
 * Calculate speed from skid marks
 * @param skidLengthFeet - Length of skid marks in feet
 * @param roadCondition - Road surface condition
 * @param gradePercent - Road grade (positive for uphill, negative for downhill)
 * @returns Estimated speed in mph
 */
export function calculateSpeedFromSkidMarks(
  skidLengthFeet: number,
  roadCondition: RoadCondition = RoadCondition.DRY_ASPHALT,
  gradePercent: number = 0
): number {
  const friction = getFrictionCoefficient(roadCondition);
  const grade = gradePercent / 100;
  const effectiveFriction = friction + grade;

  // Rearranged braking distance formula: v = √(2 × g × μ × d)
  const velocityFps = Math.sqrt(
    2 * PHYSICS_CONSTANTS.GRAVITY * effectiveFriction * skidLengthFeet
  );

  return fpsToMph(velocityFps);
}

/**
 * Calculate impact speed from damage and conservation of energy
 * @param crushDepth - Average crush depth in inches
 * @param vehicleWeight - Vehicle weight in pounds
 * @param stiffnessCoefficient - Vehicle stiffness (default 150 lb/in)
 * @returns Estimated impact speed in mph
 */
export function calculateImpactSpeedFromDamage(
  crushDepth: number,
  vehicleWeight: number,
  stiffnessCoefficient: number = 150
): number {
  // Energy absorbed = ½ × k × d²
  // where k is stiffness and d is crush depth
  const energyAbsorbed = 0.5 * stiffnessCoefficient * crushDepth * crushDepth;

  // Convert to kinetic energy: KE = ½ × m × v²
  // Solving for v: v = √(2 × KE / m)
  const mass = calculateMass(vehicleWeight);
  const velocityFps = Math.sqrt((2 * energyAbsorbed) / mass);

  return fpsToMph(velocityFps);
}

/**
 * Calculate velocity change (delta-v) from collision
 * @param vehicle1Weight - First vehicle weight in pounds
 * @param vehicle1VelocityMph - First vehicle velocity in mph
 * @param vehicle2Weight - Second vehicle weight in pounds
 * @param vehicle2VelocityMph - Second vehicle velocity in mph
 * @param coefficientOfRestitution - Elasticity of collision (0=inelastic, 1=elastic)
 * @returns Delta-v for each vehicle in mph
 */
export function calculateDeltaV(
  vehicle1Weight: number,
  vehicle1VelocityMph: number,
  vehicle2Weight: number,
  vehicle2VelocityMph: number,
  coefficientOfRestitution: number = 0
): {
  vehicle1DeltaV: number;
  vehicle2DeltaV: number;
} {
  const m1 = calculateMass(vehicle1Weight);
  const m2 = calculateMass(vehicle2Weight);
  const v1 = mphToFps(vehicle1VelocityMph);
  const v2 = mphToFps(vehicle2VelocityMph);
  const e = coefficientOfRestitution;

  // Conservation of momentum and coefficient of restitution
  // v1' = ((m1 - e×m2)/(m1 + m2)) × v1 + ((m2 + e×m2)/(m1 + m2)) × v2
  // v2' = ((m1 + e×m1)/(m1 + m2)) × v1 + ((m2 - e×m1)/(m1 + m2)) × v2

  const totalMass = m1 + m2;

  const v1Final = ((m1 - e * m2) / totalMass) * v1 + ((m2 + e * m2) / totalMass) * v2;
  const v2Final = ((m1 + e * m1) / totalMass) * v1 + ((m2 - e * m1) / totalMass) * v2;

  const vehicle1DeltaV = Math.abs(v1Final - v1);
  const vehicle2DeltaV = Math.abs(v2Final - v2);

  return {
    vehicle1DeltaV: fpsToMph(vehicle1DeltaV),
    vehicle2DeltaV: fpsToMph(vehicle2DeltaV),
  };
}

/**
 * Calculate following distance needed for given speed
 * @param velocityMph - Vehicle speed in mph
 * @param reactionTime - Driver reaction time in seconds
 * @returns Recommended following distance in feet
 */
export function calculateFollowingDistance(
  velocityMph: number,
  reactionTime: number = 2.0
): number {
  // 2-second rule: maintain distance equal to 2 seconds of travel
  const velocityFps = mphToFps(velocityMph);
  return velocityFps * reactionTime;
}

/**
 * Calculate centripetal acceleration in a turn
 * @param velocityMph - Vehicle speed in mph
 * @param turnRadius - Radius of turn in feet
 * @returns Lateral acceleration in g's
 */
export function calculateLateralAcceleration(velocityMph: number, turnRadius: number): number {
  const velocityFps = mphToFps(velocityMph);
  const acceleration = (velocityFps * velocityFps) / turnRadius;
  return acceleration / PHYSICS_CONSTANTS.GRAVITY; // Convert to g's
}

/**
 * Calculate critical speed for rollover
 * @param trackWidth - Vehicle track width in feet
 * @param centerOfGravityHeight - Height of center of gravity in feet
 * @param turnRadius - Radius of turn in feet
 * @returns Critical speed in mph
 */
export function calculateRolloverSpeed(
  trackWidth: number,
  centerOfGravityHeight: number,
  turnRadius: number
): number {
  // Critical speed formula for rollover
  const velocityFps = Math.sqrt(
    (PHYSICS_CONSTANTS.GRAVITY * turnRadius * trackWidth) / (2 * centerOfGravityHeight)
  );
  return fpsToMph(velocityFps);
}

/**
 * Calculate time to collision
 * @param distance - Distance between vehicles in feet
 * @param velocity1Mph - First vehicle velocity in mph
 * @param velocity2Mph - Second vehicle velocity in mph (same direction = positive, opposite = negative)
 * @returns Time to collision in seconds
 */
export function calculateTimeToCollision(
  distance: number,
  velocity1Mph: number,
  velocity2Mph: number = 0
): number {
  const relativeVelocityFps = mphToFps(velocity1Mph - velocity2Mph);

  if (relativeVelocityFps <= 0) {
    return Infinity; // No collision if not approaching
  }

  return distance / relativeVelocityFps;
}

/**
 * Convert miles per hour to feet per second
 */
export function mphToFps(mph: number): number {
  return mph * 1.46667; // 1 mph = 1.46667 fps
}

/**
 * Convert feet per second to miles per hour
 */
export function fpsToMph(fps: number): number {
  return fps / 1.46667;
}

/**
 * Convert miles per hour to kilometers per hour
 */
export function mphToKph(mph: number): number {
  return mph * 1.60934;
}

/**
 * Convert kilometers per hour to miles per hour
 */
export function kphToMph(kph: number): number {
  return kph / 1.60934;
}

/**
 * Calculate weight distribution on axles
 * @param totalWeight - Total vehicle weight in pounds
 * @param distribution - Front/rear weight distribution
 * @returns Weight on front and rear axles
 */
export function calculateAxleWeights(
  totalWeight: number,
  distribution: { front: number; rear: number }
): { front: number; rear: number } {
  return {
    front: (totalWeight * distribution.front) / 100,
    rear: (totalWeight * distribution.rear) / 100,
  };
}

/**
 * Calculate acceleration from 0-60 mph time
 * @param zeroToSixtySeconds - Time to reach 60 mph in seconds
 * @returns Average acceleration in ft/s²
 */
export function calculateAcceleration(zeroToSixtySeconds: number): number {
  const finalVelocityFps = mphToFps(60);
  return finalVelocityFps / zeroToSixtySeconds;
}

/**
 * Estimate stopping distance on grade
 * @param velocityMph - Initial velocity in mph
 * @param gradePercent - Grade percentage (positive = uphill)
 * @param roadCondition - Road surface condition
 * @returns Estimated stopping distance in feet
 */
export function calculateStoppingDistanceOnGrade(
  velocityMph: number,
  gradePercent: number,
  roadCondition: RoadCondition = RoadCondition.DRY_ASPHALT
): number {
  const result = calculateBrakingDistance(velocityMph, roadCondition, gradePercent);
  return result.totalDistance;
}
