/**
 * AccidentReconstructor - Accident Reconstruction System
 * Works backwards from final positions to estimate initial conditions
 */

import { Vector2D } from './Vector2D';
import { FrictionModel, SurfaceType } from './FrictionModel';
import { SpeedCalculator, SpeedEstimate } from './SpeedCalculator';

export interface FinalState {
  vehicleId: string;
  position: Vector2D;
  velocity: Vector2D;
  rotation: number;
  damage: 'none' | 'minor' | 'moderate' | 'severe' | 'catastrophic';
  skidMarks?: {
    startPosition: Vector2D;
    endPosition: Vector2D;
    length: number;
  };
}

export interface EnvironmentConditions {
  surface: SurfaceType;
  weather: 'clear' | 'rain' | 'snow' | 'ice';
  visibility: 'good' | 'reduced' | 'poor';
  lighting: 'daylight' | 'dusk' | 'night' | 'night_lit';
  temperature: number; // Celsius
}

export interface ReconstructedScenario {
  id: string;
  confidence: number;
  initialStates: Map<string, InitialState>;
  collisionPoint: Vector2D;
  collisionTime: number;
  speedEstimates: Map<string, SpeedEstimate>;
  impactAngle: number;
  description: string;
  assumptions: string[];
  uncertainties: string[];
}

export interface InitialState {
  vehicleId: string;
  position: Vector2D;
  velocity: Vector2D;
  rotation: number;
  speed: number;
  heading: number; // degrees from north
}

export interface ReconstructionConstraints {
  knownSpeeds?: Map<string, number>;
  knownPositions?: Map<string, Vector2D>;
  witnessAccounts?: WitnessAccount[];
  physicalEvidence?: PhysicalEvidence[];
  timeConstraints?: { min: number; max: number };
}

export interface WitnessAccount {
  description: string;
  reliability: number; // 0-1
  estimatedSpeed?: number;
  estimatedDirection?: number;
  vehicleId?: string;
}

export interface PhysicalEvidence {
  type: 'skid' | 'debris' | 'gouge' | 'fluid' | 'damage';
  position: Vector2D;
  measurement?: number;
  description: string;
}

export class AccidentReconstructor {
  /**
   * Reconstruct accident from final states
   */
  static reconstruct(
    finalStates: FinalState[],
    environment: EnvironmentConditions,
    constraints?: ReconstructionConstraints
  ): ReconstructedScenario[] {
    const scenarios: ReconstructedScenario[] = [];

    // Generate multiple scenarios with different assumptions
    scenarios.push(this.reconstructPrimaryScenario(finalStates, environment, constraints));
    scenarios.push(this.reconstructAlternativeScenario1(finalStates, environment, constraints));
    scenarios.push(this.reconstructAlternativeScenario2(finalStates, environment, constraints));

    // Sort by confidence
    return scenarios.sort((a, b) => b.confidence - a.confidence);
  }

  /**
   * Primary reconstruction scenario (most likely)
   */
  private static reconstructPrimaryScenario(
    finalStates: FinalState[],
    environment: EnvironmentConditions,
    constraints?: ReconstructionConstraints
  ): ReconstructedScenario {
    const assumptions: string[] = [];
    const uncertainties: string[] = [];
    const initialStates = new Map<string, InitialState>();
    const speedEstimates = new Map<string, SpeedEstimate>();

    // Estimate collision point (average of final positions)
    const collisionPoint = this.estimateCollisionPoint(finalStates);

    // Reconstruct each vehicle's initial state
    for (const finalState of finalStates) {
      // Calculate speed from skid marks if available
      let speedEstimate: SpeedEstimate;

      if (finalState.skidMarks) {
        speedEstimate = SpeedCalculator.fromSkidMarks(
          finalState.skidMarks.length,
          environment.surface,
          0 // Assume level road
        );
        assumptions.push(`${finalState.vehicleId}: Speed from skid marks`);
      } else {
        // Estimate from damage
        speedEstimate = SpeedCalculator.minimumSpeedForDamage(finalState.damage);
        assumptions.push(`${finalState.vehicleId}: Speed from damage level`);
        uncertainties.push(`${finalState.vehicleId}: No skid marks available`);
      }

      speedEstimates.set(finalState.vehicleId, speedEstimate);

      // Calculate initial position (work backwards from final position)
      const initialPosition = this.calculateInitialPosition(
        finalState,
        collisionPoint,
        speedEstimate.speed
      );

      // Calculate initial velocity vector
      const directionToCollision = collisionPoint.subtract(initialPosition).normalize();
      const initialVelocity = directionToCollision.multiply(speedEstimate.speed);

      initialStates.set(finalState.vehicleId, {
        vehicleId: finalState.vehicleId,
        position: initialPosition,
        velocity: initialVelocity,
        rotation: initialVelocity.angle(),
        speed: speedEstimate.speed,
        heading: this.vectorToHeading(initialVelocity)
      });
    }

    // Calculate impact angle
    const impactAngle = this.calculateImpactAngle(initialStates);

    // Calculate collision time (time to reach collision point)
    const collisionTime = this.estimateCollisionTime(initialStates, collisionPoint);

    // Calculate overall confidence
    const confidence = this.calculateScenarioConfidence(
      speedEstimates,
      assumptions,
      uncertainties,
      constraints
    );

    return {
      id: 'primary',
      confidence,
      initialStates,
      collisionPoint,
      collisionTime,
      speedEstimates,
      impactAngle,
      description: 'Primary reconstruction based on physical evidence',
      assumptions,
      uncertainties
    };
  }

  /**
   * Alternative scenario 1 (higher speed assumption)
   */
  private static reconstructAlternativeScenario1(
    finalStates: FinalState[],
    environment: EnvironmentConditions,
    constraints?: ReconstructionConstraints
  ): ReconstructedScenario {
    const scenario = this.reconstructPrimaryScenario(finalStates, environment, constraints);

    // Increase speeds by 20%
    scenario.initialStates.forEach((state, id) => {
      const newSpeed = state.speed * 1.2;
      const newVelocity = state.velocity.normalize().multiply(newSpeed);
      scenario.initialStates.set(id, {
        ...state,
        speed: newSpeed,
        velocity: newVelocity
      });
    });

    scenario.id = 'high_speed';
    scenario.confidence *= 0.7; // Lower confidence for alternative
    scenario.description = 'Alternative scenario with higher impact speeds';
    scenario.assumptions.push('Assumed higher pre-impact speeds (+20%)');

    return scenario;
  }

  /**
   * Alternative scenario 2 (late braking assumption)
   */
  private static reconstructAlternativeScenario2(
    finalStates: FinalState[],
    environment: EnvironmentConditions,
    constraints?: ReconstructionConstraints
  ): ReconstructedScenario {
    const scenario = this.reconstructPrimaryScenario(finalStates, environment, constraints);

    // Assume vehicles were traveling faster but braked hard
    scenario.initialStates.forEach((state, id) => {
      const brakingDistance = 20; // meters
      const dragFactor = FrictionModel.getDragFactor(environment.surface);
      const additionalSpeed = Math.sqrt(2 * dragFactor * 9.81 * brakingDistance);

      const newSpeed = state.speed + additionalSpeed;
      const newVelocity = state.velocity.normalize().multiply(newSpeed);

      // Move initial position further back
      const newPosition = state.position.subtract(
        state.velocity.normalize().multiply(brakingDistance)
      );

      scenario.initialStates.set(id, {
        ...state,
        speed: newSpeed,
        velocity: newVelocity,
        position: newPosition
      });
    });

    scenario.id = 'late_braking';
    scenario.confidence *= 0.6; // Lower confidence
    scenario.description = 'Alternative scenario with late braking before impact';
    scenario.assumptions.push('Assumed hard braking in final 20m before impact');

    return scenario;
  }

  /**
   * Estimate collision point from final positions
   */
  private static estimateCollisionPoint(finalStates: FinalState[]): Vector2D {
    if (finalStates.length === 0) {
      return Vector2D.zero();
    }

    // Use weighted average based on damage severity
    const weights: Record<string, number> = {
      none: 0.1,
      minor: 0.3,
      moderate: 0.6,
      severe: 0.9,
      catastrophic: 1.0
    };

    let totalWeight = 0;
    let weightedSum = Vector2D.zero();

    for (const state of finalStates) {
      const weight = weights[state.damage];
      const position = state.skidMarks?.startPosition || state.position;
      weightedSum = weightedSum.add(position.multiply(weight));
      totalWeight += weight;
    }

    return totalWeight > 0 ? weightedSum.divide(totalWeight) : finalStates[0].position;
  }

  /**
   * Calculate initial position working backwards from collision
   */
  private static calculateInitialPosition(
    finalState: FinalState,
    collisionPoint: Vector2D,
    speed: number
  ): Vector2D {
    // Estimate time from collision to final rest
    const dragFactor = 0.7; // Average
    const timeToStop = speed / (dragFactor * 9.81);

    // Distance traveled after collision
    const postCollisionDistance = speed * timeToStop / 2; // Average velocity

    // Direction from collision to final position
    const direction = finalState.position.subtract(collisionPoint);
    const distance = direction.magnitude();

    // Initial position is before collision point
    const preCollisionDistance = Math.max(10, speed * 2); // At least 2 seconds travel time

    // Calculate initial position opposite to post-collision movement
    const initialDirection = direction.magnitude() > 0
      ? direction.normalize().negate()
      : Vector2D.fromAngle(finalState.rotation);

    return collisionPoint.add(initialDirection.multiply(preCollisionDistance));
  }

  /**
   * Calculate impact angle between vehicles
   */
  private static calculateImpactAngle(initialStates: Map<string, InitialState>): number {
    const states = Array.from(initialStates.values());

    if (states.length < 2) {
      return 0;
    }

    const v1 = states[0].velocity.normalize();
    const v2 = states[1].velocity.normalize();

    const angle = v1.angleTo(v2);
    return (angle * 180) / Math.PI; // Convert to degrees
  }

  /**
   * Estimate time to collision
   */
  private static estimateCollisionTime(
    initialStates: Map<string, InitialState>,
    collisionPoint: Vector2D
  ): number {
    let totalTime = 0;
    let count = 0;

    initialStates.forEach(state => {
      const distance = state.position.distanceTo(collisionPoint);
      const time = state.speed > 0 ? distance / state.speed : 0;
      totalTime += time;
      count++;
    });

    return count > 0 ? totalTime / count : 0;
  }

  /**
   * Calculate scenario confidence score
   */
  private static calculateScenarioConfidence(
    speedEstimates: Map<string, SpeedEstimate>,
    assumptions: string[],
    uncertainties: string[],
    constraints?: ReconstructionConstraints
  ): number {
    let confidence = 1.0;

    // Reduce confidence based on speed estimate confidence
    let avgSpeedConfidence = 0;
    speedEstimates.forEach(estimate => {
      avgSpeedConfidence += estimate.confidence;
    });
    avgSpeedConfidence /= Math.max(1, speedEstimates.size);
    confidence *= avgSpeedConfidence;

    // Reduce confidence for each uncertainty
    confidence *= Math.pow(0.95, uncertainties.length);

    // Reduce confidence for each assumption
    confidence *= Math.pow(0.98, assumptions.length);

    // Increase confidence if constraints are satisfied
    if (constraints?.knownSpeeds && constraints.knownSpeeds.size > 0) {
      confidence *= 1.1;
    }

    if (constraints?.witnessAccounts && constraints.witnessAccounts.length > 0) {
      const avgReliability = constraints.witnessAccounts.reduce(
        (sum, w) => sum + w.reliability,
        0
      ) / constraints.witnessAccounts.length;
      confidence *= 0.9 + avgReliability * 0.2;
    }

    return Math.max(0, Math.min(1, confidence));
  }

  /**
   * Convert velocity vector to heading (degrees from north)
   */
  private static vectorToHeading(velocity: Vector2D): number {
    // North is positive Y, East is positive X
    const angle = Math.atan2(velocity.x, velocity.y);
    let heading = (angle * 180) / Math.PI;

    // Normalize to 0-360
    while (heading < 0) heading += 360;
    while (heading >= 360) heading -= 360;

    return heading;
  }

  /**
   * Work backwards from final state to initial state
   */
  static reverseSimulate(
    finalState: FinalState,
    environment: EnvironmentConditions,
    timeStep: number = 0.016 // ~60 FPS
  ): InitialState {
    let position = finalState.position.clone();
    let velocity = finalState.velocity.clone();
    const dragFactor = FrictionModel.getDragFactor(environment.surface);

    // Simulate backwards in time
    const maxSteps = 1000;
    let step = 0;

    while (velocity.magnitude() < 20 && step < maxSteps) {
      // Reverse friction application
      const speed = velocity.magnitude();
      const acceleration = dragFactor * 9.81;
      const newSpeed = speed + acceleration * timeStep;

      velocity = velocity.magnitude() > 0
        ? velocity.normalize().multiply(newSpeed)
        : Vector2D.fromAngle(finalState.rotation, newSpeed);

      position = position.subtract(velocity.multiply(timeStep));
      step++;
    }

    return {
      vehicleId: finalState.vehicleId,
      position,
      velocity,
      rotation: velocity.angle(),
      speed: velocity.magnitude(),
      heading: this.vectorToHeading(velocity)
    };
  }

  /**
   * Generate confidence intervals for speed estimates
   */
  static generateConfidenceIntervals(
    scenario: ReconstructedScenario,
    confidenceLevel: number = 0.95
  ): Map<string, { lower: number; upper: number; mean: number }> {
    const intervals = new Map<string, { lower: number; upper: number; mean: number }>();

    scenario.speedEstimates.forEach((estimate, vehicleId) => {
      const mean = estimate.speed;
      const range = estimate.range.max - estimate.range.min;

      // Use confidence level to calculate interval
      const factor = confidenceLevel;
      const margin = (range / 2) * factor;

      intervals.set(vehicleId, {
        lower: Math.max(0, mean - margin),
        upper: mean + margin,
        mean
      });
    });

    return intervals;
  }

  /**
   * Validate scenario against physical constraints
   */
  static validateScenario(scenario: ReconstructedScenario): {
    valid: boolean;
    violations: string[];
  } {
    const violations: string[] = [];

    // Check for unrealistic speeds
    scenario.speedEstimates.forEach((estimate, vehicleId) => {
      const speedMph = FrictionModel.mpsToMph(estimate.speed);

      if (speedMph > 150) {
        violations.push(`${vehicleId}: Unrealistic speed (${speedMph.toFixed(0)} mph)`);
      }

      if (estimate.speed < 0) {
        violations.push(`${vehicleId}: Negative speed`);
      }
    });

    // Check for unrealistic impact angles
    if (scenario.impactAngle > 180) {
      violations.push(`Unrealistic impact angle: ${scenario.impactAngle.toFixed(0)}°`);
    }

    // Check for negative collision time
    if (scenario.collisionTime < 0) {
      violations.push('Negative collision time');
    }

    return {
      valid: violations.length === 0,
      violations
    };
  }

  /**
   * Compare scenarios and rank them
   */
  static compareScenarios(scenarios: ReconstructedScenario[]): ReconstructedScenario[] {
    return scenarios
      .map(scenario => ({
        ...scenario,
        validation: this.validateScenario(scenario)
      }))
      .sort((a, b) => {
        // Invalid scenarios ranked lower
        if (a.validation.valid !== b.validation.valid) {
          return a.validation.valid ? -1 : 1;
        }
        // Sort by confidence
        return b.confidence - a.confidence;
      });
  }
}
