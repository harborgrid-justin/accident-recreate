/**
 * AccuScene Enterprise - Operation Composition
 * v0.2.0
 *
 * Compose and optimize multiple operations
 */

import { Operation, OperationType } from '../types';

/**
 * Operation Composition Engine
 */
export class OperationComposer {
  /**
   * Compose two operations into one if possible
   */
  static compose(opA: Operation, opB: Operation): Operation | null {
    // Can only compose operations from same client on same target
    if (
      opA.clientId !== opB.clientId ||
      this.getOperationTarget(opA) !== this.getOperationTarget(opB)
    ) {
      return null;
    }

    // Compose based on operation types
    if (this.isObjectOperation(opA) && this.isObjectOperation(opB)) {
      return this.composeObjectOperations(opA, opB);
    }

    if (this.isAnnotationOperation(opA) && this.isAnnotationOperation(opB)) {
      return this.composeAnnotationOperations(opA, opB);
    }

    if (this.isMeasurementOperation(opA) && this.isMeasurementOperation(opB)) {
      return this.composeMeasurementOperations(opA, opB);
    }

    return null;
  }

  /**
   * Compose multiple operations into a minimal set
   */
  static composeMultiple(operations: Operation[]): Operation[] {
    if (operations.length === 0) {
      return [];
    }

    const composed: Operation[] = [];
    let current = operations[0];

    for (let i = 1; i < operations.length; i++) {
      const next = operations[i];
      const composedOp = this.compose(current, next);

      if (composedOp) {
        current = composedOp;
      } else {
        composed.push(current);
        current = next;
      }
    }

    composed.push(current);
    return composed;
  }

  /**
   * Get the target identifier of an operation
   */
  private static getOperationTarget(op: Operation): string {
    const data = op.data as any;

    if (data.objectId) return `object:${data.objectId}`;
    if (data.annotationId) return `annotation:${data.annotationId}`;
    if (data.measurementId) return `measurement:${data.measurementId}`;
    if (data.sceneId) return `scene:${data.sceneId}`;

    return `unknown:${op.id}`;
  }

  /**
   * Check if operation is an object operation
   */
  private static isObjectOperation(op: Operation): boolean {
    return op.type.toString().startsWith('object:');
  }

  /**
   * Check if operation is an annotation operation
   */
  private static isAnnotationOperation(op: Operation): boolean {
    return op.type.toString().startsWith('annotation:');
  }

  /**
   * Check if operation is a measurement operation
   */
  private static isMeasurementOperation(op: Operation): boolean {
    return op.type.toString().startsWith('measurement:');
  }

  /**
   * Compose object operations
   */
  private static composeObjectOperations(opA: Operation, opB: Operation): Operation | null {
    // Delete always wins
    if (opB.type === OperationType.OBJECT_DELETE) {
      return opB;
    }

    // Create followed by update
    if (
      opA.type === OperationType.OBJECT_CREATE &&
      opB.type === OperationType.OBJECT_UPDATE
    ) {
      return {
        ...opA,
        type: OperationType.OBJECT_CREATE,
        timestamp: opB.timestamp,
        vectorClock: opB.vectorClock,
        data: {
          ...(opA.data as any),
          ...(opB.data as any),
          properties: {
            ...(opA.data as any).properties,
            ...(opB.data as any).properties,
          },
        },
      };
    }

    // Update followed by update
    if (
      opA.type === OperationType.OBJECT_UPDATE &&
      opB.type === OperationType.OBJECT_UPDATE
    ) {
      return {
        ...opA,
        timestamp: opB.timestamp,
        vectorClock: opB.vectorClock,
        data: {
          ...(opA.data as any),
          ...(opB.data as any),
          properties: {
            ...(opA.data as any).properties,
            ...(opB.data as any).properties,
          },
        },
      };
    }

    // Move/Transform composition
    if (
      (opA.type === OperationType.OBJECT_MOVE || opA.type === OperationType.OBJECT_TRANSFORM) &&
      (opB.type === OperationType.OBJECT_MOVE || opB.type === OperationType.OBJECT_TRANSFORM)
    ) {
      return {
        ...opA,
        timestamp: opB.timestamp,
        vectorClock: opB.vectorClock,
        data: {
          ...(opA.data as any),
          ...(opB.data as any),
        },
      };
    }

    return null;
  }

  /**
   * Compose annotation operations
   */
  private static composeAnnotationOperations(opA: Operation, opB: Operation): Operation | null {
    // Delete always wins
    if (opB.type === OperationType.ANNOTATION_DELETE) {
      return opB;
    }

    // Create followed by update
    if (
      opA.type === OperationType.ANNOTATION_CREATE &&
      opB.type === OperationType.ANNOTATION_UPDATE
    ) {
      return {
        ...opA,
        type: OperationType.ANNOTATION_CREATE,
        timestamp: opB.timestamp,
        vectorClock: opB.vectorClock,
        data: {
          ...(opA.data as any),
          ...(opB.data as any),
        },
      };
    }

    // Update followed by update
    if (
      opA.type === OperationType.ANNOTATION_UPDATE &&
      opB.type === OperationType.ANNOTATION_UPDATE
    ) {
      return {
        ...opA,
        timestamp: opB.timestamp,
        vectorClock: opB.vectorClock,
        data: {
          ...(opA.data as any),
          ...(opB.data as any),
          style: {
            ...(opA.data as any).style,
            ...(opB.data as any).style,
          },
        },
      };
    }

    return null;
  }

  /**
   * Compose measurement operations
   */
  private static composeMeasurementOperations(opA: Operation, opB: Operation): Operation | null {
    // Delete always wins
    if (opB.type === OperationType.MEASUREMENT_DELETE) {
      return opB;
    }

    // Create followed by update
    if (
      opA.type === OperationType.MEASUREMENT_CREATE &&
      opB.type === OperationType.MEASUREMENT_UPDATE
    ) {
      return {
        ...opA,
        type: OperationType.MEASUREMENT_CREATE,
        timestamp: opB.timestamp,
        vectorClock: opB.vectorClock,
        data: {
          ...(opA.data as any),
          ...(opB.data as any),
        },
      };
    }

    // Update followed by update
    if (
      opA.type === OperationType.MEASUREMENT_UPDATE &&
      opB.type === OperationType.MEASUREMENT_UPDATE
    ) {
      return {
        ...opA,
        timestamp: opB.timestamp,
        vectorClock: opB.vectorClock,
        data: {
          ...(opA.data as any),
          ...(opB.data as any),
        },
      };
    }

    return null;
  }

  /**
   * Check if two operations can be composed
   */
  static canCompose(opA: Operation, opB: Operation): boolean {
    return this.compose(opA, opB) !== null;
  }

  /**
   * Optimize a list of operations by composing where possible
   */
  static optimize(operations: Operation[]): Operation[] {
    // Group by client and target
    const groups = new Map<string, Operation[]>();

    for (const op of operations) {
      const key = `${op.clientId}:${this.getOperationTarget(op)}`;
      if (!groups.has(key)) {
        groups.set(key, []);
      }
      groups.get(key)!.push(op);
    }

    // Compose within each group
    const optimized: Operation[] = [];

    for (const group of groups.values()) {
      group.sort((a, b) => a.timestamp.counter - b.timestamp.counter);
      optimized.push(...this.composeMultiple(group));
    }

    // Sort by timestamp
    return optimized.sort((a, b) => a.timestamp.counter - b.timestamp.counter);
  }
}
