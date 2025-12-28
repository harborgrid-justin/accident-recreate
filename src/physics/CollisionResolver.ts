/**
 * CollisionResolver - Collision Response System
 * Implements elastic/inelastic collision response with momentum conservation
 */

import { Vector2D } from './Vector2D';
import { CollisionResult } from './CollisionDetector';

export interface RigidBody {
  position: Vector2D;
  velocity: Vector2D;
  mass: number;
  rotation: number;
  angularVelocity: number;
  momentOfInertia: number;
  restitution: number; // Coefficient of restitution (0 = inelastic, 1 = elastic)
  friction: number;
}

export interface CollisionResponse {
  body1: {
    velocity: Vector2D;
    angularVelocity: number;
    impulse: Vector2D;
  };
  body2: {
    velocity: Vector2D;
    angularVelocity: number;
    impulse: Vector2D;
  };
  relativeVelocity: number;
  impactSpeed: number;
  energyLoss: number;
}

export class CollisionResolver {
  /**
   * Resolve collision between two rigid bodies
   */
  static resolveCollision(
    body1: RigidBody,
    body2: RigidBody,
    collision: CollisionResult
  ): CollisionResponse {
    if (!collision.colliding) {
      return this.createEmptyResponse();
    }

    // Contact point relative to each body's center
    const r1 = collision.contactPoint.subtract(body1.position);
    const r2 = collision.contactPoint.subtract(body2.position);

    // Velocities at contact point
    const v1 = body1.velocity.add(new Vector2D(-r1.y, r1.x).multiply(body1.angularVelocity));
    const v2 = body2.velocity.add(new Vector2D(-r2.y, r2.x).multiply(body2.angularVelocity));

    // Relative velocity at contact point
    const relativeVelocity = v1.subtract(v2);
    const normalVelocity = relativeVelocity.dot(collision.normal);

    // Objects moving apart - no response needed
    if (normalVelocity > 0) {
      return this.createEmptyResponse();
    }

    // Calculate coefficient of restitution (combined)
    const restitution = Math.min(body1.restitution, body2.restitution);

    // Calculate impulse magnitude
    const r1CrossN = r1.cross(collision.normal);
    const r2CrossN = r2.cross(collision.normal);

    const inverseMass1 = body1.mass > 0 ? 1 / body1.mass : 0;
    const inverseMass2 = body2.mass > 0 ? 1 / body2.mass : 0;
    const inverseInertia1 = body1.momentOfInertia > 0 ? 1 / body1.momentOfInertia : 0;
    const inverseInertia2 = body2.momentOfInertia > 0 ? 1 / body2.momentOfInertia : 0;

    const denominator =
      inverseMass1 +
      inverseMass2 +
      r1CrossN * r1CrossN * inverseInertia1 +
      r2CrossN * r2CrossN * inverseInertia2;

    if (denominator === 0) {
      return this.createEmptyResponse();
    }

    // Calculate impulse with restitution
    const impulseMagnitude = -(1 + restitution) * normalVelocity / denominator;
    const impulse = collision.normal.multiply(impulseMagnitude);

    // Apply impulse to linear velocities
    const newVelocity1 = body1.velocity.add(impulse.multiply(inverseMass1));
    const newVelocity2 = body2.velocity.subtract(impulse.multiply(inverseMass2));

    // Apply impulse to angular velocities
    const newAngularVelocity1 = body1.angularVelocity + r1.cross(impulse) * inverseInertia1;
    const newAngularVelocity2 = body2.angularVelocity - r2.cross(impulse) * inverseInertia2;

    // Apply friction
    const frictionResponse = this.applyFriction(
      body1,
      body2,
      collision,
      relativeVelocity,
      impulse.magnitude()
    );

    // Calculate energy loss
    const kineticBefore = this.calculateKineticEnergy(body1) + this.calculateKineticEnergy(body2);

    const tempBody1 = { ...body1, velocity: newVelocity1.add(frictionResponse.body1.velocity), angularVelocity: newAngularVelocity1 };
    const tempBody2 = { ...body2, velocity: newVelocity2.add(frictionResponse.body2.velocity), angularVelocity: newAngularVelocity2 };

    const kineticAfter = this.calculateKineticEnergy(tempBody1) + this.calculateKineticEnergy(tempBody2);
    const energyLoss = kineticBefore - kineticAfter;

    return {
      body1: {
        velocity: newVelocity1.add(frictionResponse.body1.velocity),
        angularVelocity: newAngularVelocity1,
        impulse: impulse
      },
      body2: {
        velocity: newVelocity2.add(frictionResponse.body2.velocity),
        angularVelocity: newAngularVelocity2,
        impulse: impulse.negate()
      },
      relativeVelocity: Math.abs(normalVelocity),
      impactSpeed: relativeVelocity.magnitude(),
      energyLoss
    };
  }

  /**
   * Apply friction to collision response
   */
  private static applyFriction(
    body1: RigidBody,
    body2: RigidBody,
    collision: CollisionResult,
    relativeVelocity: Vector2D,
    normalImpulseMagnitude: number
  ): { body1: { velocity: Vector2D }; body2: { velocity: Vector2D } } {
    // Get tangent direction (perpendicular to normal)
    const tangent = collision.normal.perpendicular();
    const tangentVelocity = relativeVelocity.dot(tangent);

    if (Math.abs(tangentVelocity) < 0.001) {
      return {
        body1: { velocity: Vector2D.zero() },
        body2: { velocity: Vector2D.zero() }
      };
    }

    // Combined friction coefficient
    const friction = Math.sqrt(body1.friction * body2.friction);

    // Calculate friction impulse (Coulomb friction model)
    const frictionImpulseMagnitude = Math.min(
      Math.abs(tangentVelocity),
      friction * normalImpulseMagnitude
    );

    const frictionDirection = tangentVelocity > 0 ? tangent.negate() : tangent;
    const frictionImpulse = frictionDirection.multiply(frictionImpulseMagnitude);

    const inverseMass1 = body1.mass > 0 ? 1 / body1.mass : 0;
    const inverseMass2 = body2.mass > 0 ? 1 / body2.mass : 0;

    return {
      body1: { velocity: frictionImpulse.multiply(inverseMass1) },
      body2: { velocity: frictionImpulse.multiply(-inverseMass2) }
    };
  }

  /**
   * Calculate kinetic energy of a rigid body
   */
  static calculateKineticEnergy(body: RigidBody): number {
    const translationalEnergy = 0.5 * body.mass * body.velocity.magnitudeSquared();
    const rotationalEnergy = 0.5 * body.momentOfInertia * body.angularVelocity * body.angularVelocity;
    return translationalEnergy + rotationalEnergy;
  }

  /**
   * Calculate linear momentum of a rigid body
   */
  static calculateMomentum(body: RigidBody): Vector2D {
    return body.velocity.multiply(body.mass);
  }

  /**
   * Calculate angular momentum of a rigid body
   */
  static calculateAngularMomentum(body: RigidBody): number {
    return body.momentOfInertia * body.angularVelocity;
  }

  /**
   * Resolve elastic collision (1D simplification for head-on collisions)
   */
  static resolveElasticCollision1D(
    mass1: number,
    velocity1: number,
    mass2: number,
    velocity2: number
  ): { velocity1: number; velocity2: number } {
    const totalMass = mass1 + mass2;

    if (totalMass === 0) {
      return { velocity1: 0, velocity2: 0 };
    }

    const newVelocity1 =
      ((mass1 - mass2) * velocity1 + 2 * mass2 * velocity2) / totalMass;
    const newVelocity2 =
      ((mass2 - mass1) * velocity2 + 2 * mass1 * velocity1) / totalMass;

    return {
      velocity1: newVelocity1,
      velocity2: newVelocity2
    };
  }

  /**
   * Resolve perfectly inelastic collision (objects stick together)
   */
  static resolveInelasticCollision(
    body1: RigidBody,
    body2: RigidBody
  ): Vector2D {
    const totalMass = body1.mass + body2.mass;

    if (totalMass === 0) {
      return Vector2D.zero();
    }

    // Conservation of momentum: m1*v1 + m2*v2 = (m1+m2)*vf
    const momentum1 = body1.velocity.multiply(body1.mass);
    const momentum2 = body2.velocity.multiply(body2.mass);
    const totalMomentum = momentum1.add(momentum2);

    return totalMomentum.divide(totalMass);
  }

  /**
   * Calculate impact force (F = Δp/Δt)
   */
  static calculateImpactForce(
    impulse: Vector2D,
    collisionDuration: number = 0.01
  ): Vector2D {
    if (collisionDuration === 0) {
      return Vector2D.zero();
    }
    return impulse.divide(collisionDuration);
  }

  /**
   * Calculate momentum change
   */
  static calculateMomentumChange(
    initialVelocity: Vector2D,
    finalVelocity: Vector2D,
    mass: number
  ): Vector2D {
    const deltaV = finalVelocity.subtract(initialVelocity);
    return deltaV.multiply(mass);
  }

  /**
   * Separate penetrating bodies
   */
  static separateBodies(
    body1: RigidBody,
    body2: RigidBody,
    collision: CollisionResult,
    percent: number = 0.8, // Penetration correction percentage
    slop: number = 0.01 // Penetration allowance
  ): { position1: Vector2D; position2: Vector2D } {
    if (!collision.colliding || collision.penetrationDepth <= slop) {
      return {
        position1: body1.position,
        position2: body2.position
      };
    }

    const correction = collision.penetrationDepth - slop;
    const inverseMass1 = body1.mass > 0 ? 1 / body1.mass : 0;
    const inverseMass2 = body2.mass > 0 ? 1 / body2.mass : 0;
    const totalInverseMass = inverseMass1 + inverseMass2;

    if (totalInverseMass === 0) {
      return {
        position1: body1.position,
        position2: body2.position
      };
    }

    const correctionVector = collision.normal.multiply(
      (correction * percent) / totalInverseMass
    );

    return {
      position1: body1.position.subtract(correctionVector.multiply(inverseMass1)),
      position2: body2.position.add(correctionVector.multiply(inverseMass2))
    };
  }

  /**
   * Calculate coefficient of restitution from velocity change
   */
  static calculateRestitutionFromVelocities(
    approachVelocity: number,
    separationVelocity: number
  ): number {
    if (Math.abs(approachVelocity) < 0.001) {
      return 0;
    }
    return Math.abs(separationVelocity / approachVelocity);
  }

  /**
   * Create empty collision response
   */
  private static createEmptyResponse(): CollisionResponse {
    return {
      body1: {
        velocity: Vector2D.zero(),
        angularVelocity: 0,
        impulse: Vector2D.zero()
      },
      body2: {
        velocity: Vector2D.zero(),
        angularVelocity: 0,
        impulse: Vector2D.zero()
      },
      relativeVelocity: 0,
      impactSpeed: 0,
      energyLoss: 0
    };
  }

  /**
   * Calculate moment of inertia for a rectangle
   */
  static calculateRectangleMomentOfInertia(
    mass: number,
    width: number,
    height: number
  ): number {
    return (mass * (width * width + height * height)) / 12;
  }

  /**
   * Calculate moment of inertia for a circle
   */
  static calculateCircleMomentOfInertia(
    mass: number,
    radius: number
  ): number {
    return 0.5 * mass * radius * radius;
  }
}
