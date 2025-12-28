/**
 * SpeedCalculator - Speed Estimation Methods
 * Implements various methods for estimating vehicle speeds in accident reconstruction
 */

import { Vector2D } from './Vector2D';
import { FrictionModel, SurfaceType } from './FrictionModel';

export interface SpeedEstimate {
  speed: number; // m/s
  speedMph: number;
  speedKph: number;
  method: string;
  confidence: number;
  range: { min: number; max: number };
}

export interface DamageAnalysis {
  crushDepth: number; // meters
  crushWidth: number; // meters
  vehicleMass: number; // kg
  vehicleStiffness: number; // N/m
}

export interface ThrowAnalysis {
  throwDistance: number; // meters
  throwAngle: number; // degrees
  height: number; // initial height above ground
}

export class SpeedCalculator {
  private static readonly GRAVITY = 9.81; // m/s²

  /**
   * Calculate speed from skid marks using drag factor method
   * v = √(2 * μ * g * d)
   */
  static fromSkidMarks(
    skidDistance: number,
    surface: SurfaceType,
    grade: number = 0
  ): SpeedEstimate {
    const dragFactor = FrictionModel.getDragFactor(surface);
    const analysis = FrictionModel.calculateSpeedFromSkidMarks(
      skidDistance,
      dragFactor,
      grade,
      this.GRAVITY
    );

    // Calculate uncertainty range (±10% for typical conditions)
    const uncertainty = 0.10;
    const range = {
      min: analysis.estimatedSpeed * (1 - uncertainty),
      max: analysis.estimatedSpeed * (1 + uncertainty)
    };

    return {
      speed: analysis.estimatedSpeed,
      speedMph: FrictionModel.mpsToMph(analysis.estimatedSpeed),
      speedKph: FrictionModel.mpsToKph(analysis.estimatedSpeed),
      method: 'Skid Mark Analysis',
      confidence: analysis.confidence,
      range
    };
  }

  /**
   * Calculate speed from crush damage (simplified crush energy method)
   * E = 0.5 * k * x²
   * KE = 0.5 * m * v²
   */
  static fromDamage(analysis: DamageAnalysis): SpeedEstimate {
    // Energy absorbed by crush
    const crushEnergy = 0.5 * analysis.vehicleStiffness *
      analysis.crushDepth * analysis.crushDepth;

    // Account for crush width (more width = more energy absorption)
    const totalCrushEnergy = crushEnergy * analysis.crushWidth;

    // Convert to velocity (assuming all kinetic energy absorbed)
    const speedSquared = (2 * totalCrushEnergy) / analysis.vehicleMass;
    const speed = Math.sqrt(Math.max(0, speedSquared));

    // Crush analysis has higher uncertainty (±20%)
    const uncertainty = 0.20;
    const confidence = 0.65; // Lower confidence than skid marks

    const range = {
      min: speed * (1 - uncertainty),
      max: speed * (1 + uncertainty)
    };

    return {
      speed,
      speedMph: FrictionModel.mpsToMph(speed),
      speedKph: FrictionModel.mpsToKph(speed),
      method: 'Crush Analysis',
      confidence,
      range
    };
  }

  /**
   * Calculate speed from throw distance (projectile motion)
   * Used for pedestrian/cyclist impacts
   */
  static fromThrowDistance(analysis: ThrowAnalysis): SpeedEstimate {
    // Convert angle to radians
    const angleRad = (analysis.throwAngle * Math.PI) / 180;

    // Time of flight: t = √(2h/g) for drop from height
    const timeOfFlight = Math.sqrt((2 * analysis.height) / this.GRAVITY);

    // Horizontal velocity: v = d / (t * cos(θ))
    const horizontalSpeed = analysis.throwDistance /
      (timeOfFlight * Math.cos(angleRad));

    // Total speed (accounting for vertical component)
    const speed = horizontalSpeed / Math.cos(angleRad);

    // Higher uncertainty for throw analysis (±25%)
    const uncertainty = 0.25;
    const confidence = 0.55;

    const range = {
      min: speed * (1 - uncertainty),
      max: speed * (1 + uncertainty)
    };

    return {
      speed,
      speedMph: FrictionModel.mpsToMph(speed),
      speedKph: FrictionModel.mpsToKph(speed),
      method: 'Throw Distance Analysis',
      confidence,
      range
    };
  }

  /**
   * Calculate speed from momentum analysis (two-vehicle collision)
   * Conservation of momentum: m1*v1 + m2*v2 = m1*v1' + m2*v2'
   */
  static fromMomentumAnalysis(
    vehicle1Mass: number,
    vehicle1PostVelocity: Vector2D,
    vehicle2Mass: number,
    vehicle2PostVelocity: Vector2D,
    vehicle2PreVelocity: Vector2D // Known or estimated
  ): SpeedEstimate {
    // Calculate total post-collision momentum
    const postMomentum1 = vehicle1PostVelocity.multiply(vehicle1Mass);
    const postMomentum2 = vehicle2PostVelocity.multiply(vehicle2Mass);
    const totalPostMomentum = postMomentum1.add(postMomentum2);

    // Calculate pre-collision momentum of vehicle 2
    const preMomentum2 = vehicle2PreVelocity.multiply(vehicle2Mass);

    // Solve for vehicle 1 pre-collision momentum
    const preMomentum1 = totalPostMomentum.subtract(preMomentum2);

    // Calculate vehicle 1 pre-collision velocity
    const preVelocity1 = preMomentum1.divide(vehicle1Mass);
    const speed = preVelocity1.magnitude();

    // Moderate uncertainty for momentum analysis (±15%)
    const uncertainty = 0.15;
    const confidence = 0.75;

    const range = {
      min: speed * (1 - uncertainty),
      max: speed * (1 + uncertainty)
    };

    return {
      speed,
      speedMph: FrictionModel.mpsToMph(speed),
      speedKph: FrictionModel.mpsToKph(speed),
      method: 'Momentum Analysis',
      confidence,
      range
    };
  }

  /**
   * Calculate speed from yaw marks (curved skid marks)
   * R = v² / (μ * g)
   * v = √(R * μ * g)
   */
  static fromYawMarks(
    yawRadius: number,
    surface: SurfaceType
  ): SpeedEstimate {
    const dragFactor = FrictionModel.getDragFactor(surface);

    // Critical speed formula for circular motion
    const speed = Math.sqrt(yawRadius * dragFactor * this.GRAVITY);

    const uncertainty = 0.15;
    const confidence = 0.70;

    const range = {
      min: speed * (1 - uncertainty),
      max: speed * (1 + uncertainty)
    };

    return {
      speed,
      speedMph: FrictionModel.mpsToMph(speed),
      speedKph: FrictionModel.mpsToKph(speed),
      method: 'Yaw Mark Analysis',
      confidence,
      range
    };
  }

  /**
   * Calculate speed from vault (vehicle going airborne)
   * v = √(R * g) where R is vault distance
   */
  static fromVault(
    vaultDistance: number,
    launchAngle: number = 15 // degrees
  ): SpeedEstimate {
    const angleRad = (launchAngle * Math.PI) / 180;

    // Range formula: R = v² * sin(2θ) / g
    // Solve for v: v = √(R * g / sin(2θ))
    const sin2Theta = Math.sin(2 * angleRad);

    if (sin2Theta === 0) {
      return this.createZeroEstimate('Vault Analysis');
    }

    const speedSquared = (vaultDistance * this.GRAVITY) / sin2Theta;
    const speed = Math.sqrt(Math.max(0, speedSquared));

    const uncertainty = 0.20;
    const confidence = 0.60;

    const range = {
      min: speed * (1 - uncertainty),
      max: speed * (1 + uncertainty)
    };

    return {
      speed,
      speedMph: FrictionModel.mpsToMph(speed),
      speedKph: FrictionModel.mpsToKph(speed),
      method: 'Vault Analysis',
      confidence,
      range
    };
  }

  /**
   * Calculate speed from flip/rollover
   * Based on energy required to flip vehicle
   */
  static fromRollover(
    vehicleMass: number,
    vehicleHeight: number, // center of gravity height
    vehicleWidth: number, // track width
    flipAngle: number = 90 // degrees of rotation achieved
  ): SpeedEstimate {
    const angleRad = (flipAngle * Math.PI) / 180;

    // Energy required to rotate vehicle
    // E = m * g * h * (1 - cos(θ))
    const energyRequired = vehicleMass * this.GRAVITY * vehicleHeight *
      (1 - Math.cos(angleRad));

    // Assume energy comes from lateral velocity
    // 0.5 * m * v² = E
    const speedSquared = (2 * energyRequired) / vehicleMass;
    const speed = Math.sqrt(Math.max(0, speedSquared));

    const uncertainty = 0.30;
    const confidence = 0.50; // Low confidence - many factors involved

    const range = {
      min: speed * (1 - uncertainty),
      max: speed * (1 + uncertainty)
    };

    return {
      speed,
      speedMph: FrictionModel.mpsToMph(speed),
      speedKph: FrictionModel.mpsToKph(speed),
      method: 'Rollover Analysis',
      confidence,
      range
    };
  }

  /**
   * Combine multiple speed estimates with weighted average
   */
  static combineEstimates(estimates: SpeedEstimate[]): SpeedEstimate {
    if (estimates.length === 0) {
      return this.createZeroEstimate('No Data');
    }

    if (estimates.length === 1) {
      return estimates[0];
    }

    // Weight by confidence
    let totalWeight = 0;
    let weightedSum = 0;
    let minSpeed = Infinity;
    let maxSpeed = -Infinity;

    for (const estimate of estimates) {
      const weight = estimate.confidence;
      weightedSum += estimate.speed * weight;
      totalWeight += weight;
      minSpeed = Math.min(minSpeed, estimate.range.min);
      maxSpeed = Math.max(maxSpeed, estimate.range.max);
    }

    const averageSpeed = totalWeight > 0 ? weightedSum / totalWeight : 0;
    const averageConfidence = totalWeight / estimates.length;

    return {
      speed: averageSpeed,
      speedMph: FrictionModel.mpsToMph(averageSpeed),
      speedKph: FrictionModel.mpsToKph(averageSpeed),
      method: 'Combined Analysis',
      confidence: averageConfidence,
      range: { min: minSpeed, max: maxSpeed }
    };
  }

  /**
   * Calculate speed from impact deceleration (EDR data)
   * v = a * t
   */
  static fromDeceleration(
    peakDeceleration: number, // m/s² or g-force
    collisionDuration: number, // seconds
    isGForce: boolean = true
  ): SpeedEstimate {
    const deceleration = isGForce ? peakDeceleration * this.GRAVITY : peakDeceleration;

    // Estimate speed from deceleration pulse
    // Simplified: v = a * t (assuming constant deceleration)
    const speed = deceleration * collisionDuration;

    // High confidence if from EDR data
    const confidence = 0.85;
    const uncertainty = 0.10;

    const range = {
      min: speed * (1 - uncertainty),
      max: speed * (1 + uncertainty)
    };

    return {
      speed,
      speedMph: FrictionModel.mpsToMph(speed),
      speedKph: FrictionModel.mpsToKph(speed),
      method: 'Deceleration Analysis (EDR)',
      confidence,
      range
    };
  }

  /**
   * Calculate minimum speed to cause specific damage
   * Based on barrier equivalent speed (BES)
   */
  static minimumSpeedForDamage(
    damageLevel: 'none' | 'minor' | 'moderate' | 'severe' | 'catastrophic'
  ): SpeedEstimate {
    // Typical BES values for damage levels
    const damageThresholds: Record<string, number> = {
      none: 0,
      minor: FrictionModel.kphToMps(15), // ~15 kph
      moderate: FrictionModel.kphToMps(30), // ~30 kph
      severe: FrictionModel.kphToMps(50), // ~50 kph
      catastrophic: FrictionModel.kphToMps(70) // ~70 kph
    };

    const speed = damageThresholds[damageLevel] || 0;
    const uncertainty = 0.25;
    const confidence = 0.60;

    const range = {
      min: speed * (1 - uncertainty),
      max: speed * (1 + uncertainty)
    };

    return {
      speed,
      speedMph: FrictionModel.mpsToMph(speed),
      speedKph: FrictionModel.mpsToKph(speed),
      method: 'Damage Level Analysis',
      confidence,
      range
    };
  }

  /**
   * Create zero speed estimate
   */
  private static createZeroEstimate(method: string): SpeedEstimate {
    return {
      speed: 0,
      speedMph: 0,
      speedKph: 0,
      method,
      confidence: 0,
      range: { min: 0, max: 0 }
    };
  }

  /**
   * Format speed estimate for display
   */
  static formatEstimate(estimate: SpeedEstimate): string {
    return `${estimate.method}: ${estimate.speedMph.toFixed(1)} mph ` +
           `(${estimate.speedKph.toFixed(1)} kph) ` +
           `[Confidence: ${(estimate.confidence * 100).toFixed(0)}%]`;
  }
}
