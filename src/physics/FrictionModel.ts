/**
 * FrictionModel - Surface Friction and Tire Modeling
 * Implements realistic friction coefficients and braking calculations
 */

import { Vector2D } from './Vector2D';

export enum SurfaceType {
  DRY_ASPHALT = 'dry_asphalt',
  WET_ASPHALT = 'wet_asphalt',
  ICE = 'ice',
  SNOW = 'snow',
  GRAVEL = 'gravel',
  DIRT = 'dirt',
  CONCRETE_DRY = 'concrete_dry',
  CONCRETE_WET = 'concrete_wet',
  GRASS = 'grass',
  SAND = 'sand'
}

export enum TireCondition {
  NEW = 'new',
  GOOD = 'good',
  WORN = 'worn',
  BALD = 'bald'
}

export interface FrictionCoefficients {
  static: number;  // μs - Static friction coefficient
  kinetic: number; // μk - Kinetic friction coefficient
}

export interface BrakingResult {
  stoppingDistance: number;
  stoppingTime: number;
  skidMarks: boolean;
  decelerationRate: number;
  finalSpeed: number;
}

export interface SkidAnalysis {
  estimatedSpeed: number;
  dragFactor: number;
  skidDistance: number;
  gradeFactor: number;
  confidence: number;
}

export class FrictionModel {
  /**
   * Get friction coefficients for surface type and tire condition
   */
  static getFrictionCoefficients(
    surface: SurfaceType,
    tireCondition: TireCondition = TireCondition.GOOD
  ): FrictionCoefficients {
    // Base coefficients for different surfaces
    const baseCoefficients: Record<SurfaceType, FrictionCoefficients> = {
      [SurfaceType.DRY_ASPHALT]: { static: 0.85, kinetic: 0.75 },
      [SurfaceType.WET_ASPHALT]: { static: 0.55, kinetic: 0.45 },
      [SurfaceType.ICE]: { static: 0.15, kinetic: 0.10 },
      [SurfaceType.SNOW]: { static: 0.30, kinetic: 0.25 },
      [SurfaceType.GRAVEL]: { static: 0.60, kinetic: 0.50 },
      [SurfaceType.DIRT]: { static: 0.55, kinetic: 0.45 },
      [SurfaceType.CONCRETE_DRY]: { static: 0.80, kinetic: 0.70 },
      [SurfaceType.CONCRETE_WET]: { static: 0.50, kinetic: 0.40 },
      [SurfaceType.GRASS]: { static: 0.40, kinetic: 0.35 },
      [SurfaceType.SAND]: { static: 0.35, kinetic: 0.30 }
    };

    // Tire condition modifiers
    const tireModifiers: Record<TireCondition, number> = {
      [TireCondition.NEW]: 1.10,
      [TireCondition.GOOD]: 1.00,
      [TireCondition.WORN]: 0.85,
      [TireCondition.BALD]: 0.70
    };

    const base = baseCoefficients[surface];
    const modifier = tireModifiers[tireCondition];

    return {
      static: base.static * modifier,
      kinetic: base.kinetic * modifier
    };
  }

  /**
   * Get drag factor (combined friction coefficient for calculations)
   * Commonly used in accident reconstruction
   */
  static getDragFactor(
    surface: SurfaceType,
    tireCondition: TireCondition = TireCondition.GOOD
  ): number {
    const coefficients = this.getFrictionCoefficients(surface, tireCondition);
    // Use kinetic friction as drag factor
    return coefficients.kinetic;
  }

  /**
   * Calculate braking force
   * F = μ * m * g
   */
  static calculateBrakingForce(
    mass: number,
    dragFactor: number,
    gravity: number = 9.81
  ): number {
    return dragFactor * mass * gravity;
  }

  /**
   * Calculate maximum braking deceleration
   * a = μ * g
   */
  static calculateMaxDeceleration(
    dragFactor: number,
    gravity: number = 9.81
  ): number {
    return dragFactor * gravity;
  }

  /**
   * Calculate stopping distance with constant deceleration
   * d = v² / (2 * a)
   * where a = μ * g
   */
  static calculateStoppingDistance(
    initialSpeed: number, // m/s
    dragFactor: number,
    grade: number = 0, // road grade in percentage (positive = uphill, negative = downhill)
    gravity: number = 9.81
  ): number {
    // Adjust for grade: effective deceleration = μ*g ± g*sin(θ)
    const gradeRadians = Math.atan(grade / 100);
    const effectiveDeceleration = gravity * (dragFactor + Math.sin(gradeRadians));

    if (effectiveDeceleration <= 0) {
      return Infinity; // Cannot stop on steep downhill
    }

    return (initialSpeed * initialSpeed) / (2 * effectiveDeceleration);
  }

  /**
   * Calculate stopping time
   * t = v / a
   */
  static calculateStoppingTime(
    initialSpeed: number,
    dragFactor: number,
    grade: number = 0,
    gravity: number = 9.81
  ): number {
    const gradeRadians = Math.atan(grade / 100);
    const effectiveDeceleration = gravity * (dragFactor + Math.sin(gradeRadians));

    if (effectiveDeceleration <= 0) {
      return Infinity;
    }

    return initialSpeed / effectiveDeceleration;
  }

  /**
   * Simulate braking with reaction time
   */
  static simulateBraking(
    initialSpeed: number, // m/s
    dragFactor: number,
    reactionTime: number = 1.5, // seconds (typical human reaction time)
    grade: number = 0,
    gravity: number = 9.81
  ): BrakingResult {
    // Distance traveled during reaction time (no braking)
    const reactionDistance = initialSpeed * reactionTime;

    // Braking distance
    const brakingDistance = this.calculateStoppingDistance(
      initialSpeed,
      dragFactor,
      grade,
      gravity
    );

    // Total stopping distance
    const totalDistance = reactionDistance + brakingDistance;

    // Stopping time (excluding reaction time)
    const brakingTime = this.calculateStoppingTime(
      initialSpeed,
      dragFactor,
      grade,
      gravity
    );

    const totalTime = reactionTime + brakingTime;

    // Deceleration rate
    const deceleration = this.calculateMaxDeceleration(dragFactor, gravity);

    // Skid marks occur when wheels lock (kinetic friction)
    const skidMarks = true; // In locked-wheel braking

    return {
      stoppingDistance: totalDistance,
      stoppingTime: totalTime,
      skidMarks,
      decelerationRate: deceleration,
      finalSpeed: 0
    };
  }

  /**
   * Calculate speed from skid marks (critical speed formula)
   * v = √(2 * μ * g * d)
   * Standard formula used in accident reconstruction
   */
  static calculateSpeedFromSkidMarks(
    skidDistance: number, // meters
    dragFactor: number,
    grade: number = 0,
    gravity: number = 9.81
  ): SkidAnalysis {
    // Adjust for grade
    const gradeRadians = Math.atan(grade / 100);
    const effectiveDragFactor = dragFactor + Math.sin(gradeRadians);

    // Critical speed formula: v = √(2*μ*g*d)
    const estimatedSpeed = Math.sqrt(
      2 * effectiveDragFactor * gravity * skidDistance
    );

    // Confidence based on factors
    const confidence = this.calculateSkidConfidence(skidDistance, dragFactor);

    return {
      estimatedSpeed,
      dragFactor: effectiveDragFactor,
      skidDistance,
      gradeFactor: grade,
      confidence
    };
  }

  /**
   * Calculate confidence in skid mark analysis
   */
  private static calculateSkidConfidence(
    skidDistance: number,
    dragFactor: number
  ): number {
    let confidence = 1.0;

    // Reduce confidence for very short skids (measurement error)
    if (skidDistance < 3) {
      confidence *= 0.6;
    } else if (skidDistance < 10) {
      confidence *= 0.8;
    }

    // Reduce confidence for unusual drag factors
    if (dragFactor < 0.2 || dragFactor > 0.9) {
      confidence *= 0.7;
    }

    return Math.max(0, Math.min(1, confidence));
  }

  /**
   * Calculate friction force vector
   */
  static calculateFrictionForce(
    velocity: Vector2D,
    normalForce: number,
    dragFactor: number
  ): Vector2D {
    const speed = velocity.magnitude();

    if (speed < 0.01) {
      return Vector2D.zero();
    }

    // Friction opposes motion
    const frictionDirection = velocity.normalize().negate();
    const frictionMagnitude = dragFactor * normalForce;

    return frictionDirection.multiply(frictionMagnitude);
  }

  /**
   * Apply friction to velocity (with time step)
   */
  static applyFriction(
    velocity: Vector2D,
    mass: number,
    dragFactor: number,
    deltaTime: number,
    gravity: number = 9.81
  ): Vector2D {
    const speed = velocity.magnitude();

    if (speed < 0.01) {
      return Vector2D.zero();
    }

    // Calculate deceleration due to friction
    const deceleration = dragFactor * gravity;
    const speedReduction = deceleration * deltaTime;

    // Don't overshoot - stop if speed reduction exceeds current speed
    if (speedReduction >= speed) {
      return Vector2D.zero();
    }

    const newSpeed = speed - speedReduction;
    return velocity.normalize().multiply(newSpeed);
  }

  /**
   * Calculate yaw marks (curved skid marks from rotation)
   */
  static calculateYawRadius(
    speed: number, // m/s
    dragFactor: number,
    gravity: number = 9.81
  ): number {
    // R = v² / (μ * g)
    return (speed * speed) / (dragFactor * gravity);
  }

  /**
   * Calculate critical speed for curve (maximum speed without skidding)
   */
  static calculateCriticalCurveSpeed(
    radius: number, // meters
    dragFactor: number,
    bankAngle: number = 0, // degrees
    gravity: number = 9.81
  ): number {
    // Convert bank angle to radians
    const bankRad = (bankAngle * Math.PI) / 180;

    // Critical speed: v = √(r * g * (μ + tan(θ)) / (1 - μ*tan(θ)))
    const tanBank = Math.tan(bankRad);
    const numerator = radius * gravity * (dragFactor + tanBank);
    const denominator = 1 - dragFactor * tanBank;

    if (denominator <= 0) {
      return Infinity; // Can navigate curve at any speed
    }

    return Math.sqrt(numerator / denominator);
  }

  /**
   * Calculate tire slip ratio
   * S = (v_wheel - v_vehicle) / v_vehicle
   */
  static calculateSlipRatio(
    wheelSpeed: number,
    vehicleSpeed: number
  ): number {
    if (vehicleSpeed === 0) {
      return wheelSpeed > 0 ? 1 : 0;
    }
    return (wheelSpeed - vehicleSpeed) / Math.abs(vehicleSpeed);
  }

  /**
   * Calculate friction coefficient based on slip ratio (simplified Pacejka model)
   */
  static calculateFrictionFromSlip(
    slipRatio: number,
    peakFriction: number
  ): number {
    const absSlip = Math.abs(slipRatio);

    // Simplified friction curve
    // Peak friction at ~15% slip, then decreases
    const optimalSlip = 0.15;

    if (absSlip <= optimalSlip) {
      // Linear increase to peak
      return peakFriction * (absSlip / optimalSlip);
    } else {
      // Gradual decrease after peak
      const decay = Math.exp(-(absSlip - optimalSlip) * 3);
      return peakFriction * (0.7 + 0.3 * decay);
    }
  }

  /**
   * Calculate braking efficiency (ABS vs locked wheels)
   */
  static calculateBrakingEfficiency(
    hasABS: boolean,
    surface: SurfaceType
  ): number {
    const coefficients = this.getFrictionCoefficients(surface);

    if (hasABS) {
      // ABS maintains optimal slip ratio, uses closer to static friction
      return (coefficients.static + coefficients.kinetic) / 2;
    } else {
      // Locked wheels use kinetic friction
      return coefficients.kinetic;
    }
  }

  /**
   * Convert speed units
   */
  static mpsToMph(mps: number): number {
    return mps * 2.23694;
  }

  static mpsToKph(mps: number): number {
    return mps * 3.6;
  }

  static mphToMps(mph: number): number {
    return mph / 2.23694;
  }

  static kphToMps(kph: number): number {
    return kph / 3.6;
  }
}
